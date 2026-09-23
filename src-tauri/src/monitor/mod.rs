mod app_watch;
mod crash_watch;
mod detection;
mod reporter;
mod runner;
mod session;
mod state_machine;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Manager};
use tokio::sync::{mpsc, oneshot};
use tokio::time::MissedTickBehavior;

use crate::error::{AppError, AppResult};
use crate::models::{MonitorSnapshot, Profile, SessionLog, TimelineKind, Trigger};
use crate::processes::ProcessTable;
use crate::state::AppState;
pub use app_watch::PROFILE_CHANGED_EVENT;
use reporter::{lock, Reporter, SharedSession};
use session::{now_ms, Session};
use state_machine::{Input, SessionAction, State};

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const COMMAND_BUFFER: usize = 32;

enum MonitorCommand {
    Pause,
    Resume,
    CloseNow,
    KeepApps,
    ClosingFinished,
    TestLaunch {
        profile_id: String,
        reply: oneshot::Sender<AppResult<()>>,
    },
    TestClose {
        profile_id: String,
        reply: oneshot::Sender<AppResult<()>>,
    },
}

/// Cheap, cloneable entry point used by commands and the tray; the monitor itself runs in its own task.
#[derive(Clone)]
pub struct MonitorHandle {
    commands: mpsc::Sender<MonitorCommand>,
    reporter: Reporter,
}

impl MonitorHandle {
    pub fn spawn(app: AppHandle) -> Self {
        let last_session_log = app.state::<AppState>().load_last_session();
        let reporter = Reporter::new(app.clone(), last_session_log);
        let (commands, receiver) = mpsc::channel(COMMAND_BUFFER);

        let monitor = Monitor {
            app,
            commands: commands.clone(),
            reporter: reporter.clone(),
            state: State::default(),
            processes: ProcessTable::snapshot(),
            session: None,
            pending_start: None,
            launch_task: None,
            test_session: None,
            test_task: None,
        };
        tauri::async_runtime::spawn(monitor.run(receiver));

        Self { commands, reporter }
    }

    pub fn snapshot(&self) -> MonitorSnapshot {
        self.reporter.snapshot()
    }

    pub fn session_log(&self) -> Option<SessionLog> {
        self.reporter.session_log()
    }

    pub async fn pause(&self) {
        self.send(MonitorCommand::Pause).await;
    }

    pub async fn resume(&self) {
        self.send(MonitorCommand::Resume).await;
    }

    pub async fn close_now(&self) {
        self.send(MonitorCommand::CloseNow).await;
    }

    pub async fn keep_apps_open(&self) {
        self.send(MonitorCommand::KeepApps).await;
    }

    pub async fn test_launch(&self, profile_id: String) -> AppResult<()> {
        let (reply, response) = oneshot::channel();
        self.send(MonitorCommand::TestLaunch { profile_id, reply })
            .await;
        response.await.unwrap_or(Err(AppError::SessionInProgress))
    }

    pub async fn test_close(&self, profile_id: String) -> AppResult<()> {
        let (reply, response) = oneshot::channel();
        self.send(MonitorCommand::TestClose { profile_id, reply })
            .await;
        response.await.unwrap_or(Err(AppError::SessionInProgress))
    }

    async fn send(&self, command: MonitorCommand) {
        if self.commands.send(command).await.is_err() {
            log::error!("monitor task is not running");
        }
    }
}

struct Monitor {
    app: AppHandle,
    commands: mpsc::Sender<MonitorCommand>,
    reporter: Reporter,
    state: State,
    processes: ProcessTable,
    session: Option<SharedSession>,
    pending_start: Option<(Profile, Trigger)>,
    launch_task: Option<JoinHandle<()>>,
    test_session: Option<SharedSession>,
    test_task: Option<JoinHandle<()>>,
}

impl Monitor {
    async fn run(mut self, mut receiver: mpsc::Receiver<MonitorCommand>) {
        let mut interval = tokio::time::interval(POLL_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = interval.tick() => self.poll(),
                command = receiver.recv() => match command {
                    Some(command) => self.handle(command),
                    None => break,
                },
            }
        }
    }

    fn poll(&mut self) {
        if self.state == State::Paused {
            return;
        }
        self.processes.refresh();
        let is_running = |name: &str| self.processes.is_running(name);

        // A running session keeps watching its own profile even if the profiles change meanwhile.
        let seen = match &self.session {
            Some(session) => detection::any_trigger_running(lock(session).profile(), is_running),
            None => {
                let profiles = self.app.state::<AppState>().profiles();
                self.pending_start = detection::first_running_profile(&profiles, is_running);
                self.pending_start.is_some()
            }
        };
        self.apply(if seen {
            Input::TriggerSeen
        } else {
            Input::TriggerMissing
        });
    }

    fn handle(&mut self, command: MonitorCommand) {
        match command {
            MonitorCommand::Pause => self.apply(Input::Pause),
            MonitorCommand::Resume => self.apply(Input::Resume),
            MonitorCommand::CloseNow => self.apply(Input::CloseNow),
            MonitorCommand::KeepApps => self.apply(Input::KeepApps),
            MonitorCommand::ClosingFinished => {
                self.session = None;
                self.apply(Input::ClosingFinished);
            }
            MonitorCommand::TestLaunch { profile_id, reply } => {
                let _ = reply.send(self.start_test_launch(&profile_id));
            }
            MonitorCommand::TestClose { profile_id, reply } => {
                let _ = reply.send(self.start_test_close(&profile_id));
            }
        }
    }

    fn apply(&mut self, input: Input) {
        let (next_state, action) = state_machine::next(self.state, input, self.close_delay_ticks());
        let previous = std::mem::replace(&mut self.state, next_state);
        if previous.public() != next_state.public() {
            self.reporter.set_state(next_state.public());
        }

        match action {
            Some(SessionAction::Start) => self.start_session(),
            Some(SessionAction::DelayClose) => self.delay_close(),
            Some(SessionAction::Continue) => self.continue_session(),
            Some(SessionAction::End) => self.end_session(),
            Some(SessionAction::Release) => self.release_session(),
            Some(SessionAction::Discard) => self.discard_session(),
            None => {}
        }
    }

    fn start_session(&mut self) {
        abort(self.test_task.take());
        self.test_session = None;

        let Some((profile, trigger)) = self.pending_start.take() else {
            self.state = State::Idle;
            self.reporter.set_state(self.state.public());
            return;
        };
        let session = self.attach(Session::start(profile, Some(trigger), false));
        self.launch_task = Some(self.spawn_launch(session.clone()));
        self.session = Some(session);
    }

    fn delay_close(&mut self) {
        let Some(session) = &self.session else {
            return;
        };
        lock(session).set_relaunch_allowed(false);
        let delay_ms = self.app.state::<AppState>().settings().close_delay_ms;
        self.reporter
            .close_delayed(session, now_ms() + u64::from(delay_ms), delay_ms / 1000);
    }

    fn continue_session(&mut self) {
        let Some(session) = &self.session else {
            return;
        };
        lock(session).set_relaunch_allowed(true);
        self.reporter
            .timeline(session, TimelineKind::SimulatorReturned, None, None);
    }

    fn release_session(&mut self) {
        abort(self.launch_task.take());
        if let Some(session) = self.session.take() {
            self.reporter
                .timeline(&session, TimelineKind::KeptByUser, None, None);
            self.reporter.finish(&session);
        }
    }

    fn end_session(&mut self) {
        abort(self.launch_task.take());
        let Some(session) = self.session.clone() else {
            return;
        };
        lock(&session).set_relaunch_allowed(false);

        let (close_only, timeout) = self.close_settings();
        let reporter = self.reporter.clone();
        let commands = self.commands.clone();
        tauri::async_runtime::spawn(async move {
            runner::close_items(session.clone(), reporter.clone(), close_only, timeout).await;
            reporter.finish(&session);
            let _ = commands.send(MonitorCommand::ClosingFinished).await;
        });
    }

    fn discard_session(&mut self) {
        abort(self.launch_task.take());
        if let Some(session) = self.session.take() {
            self.reporter.finish(&session);
        }
    }

    fn start_test_launch(&mut self, profile_id: &str) -> AppResult<()> {
        self.ensure_no_session()?;
        abort(self.test_task.take());

        let session = self.attach(Session::start(self.profile(profile_id)?, None, true));
        let launch = self.spawn_launch(session.clone());
        let reporter = self.reporter.clone();
        let finished_session = session.clone();
        self.test_task = Some(tauri::async_runtime::spawn(async move {
            let _ = launch.await;
            reporter.finish(&finished_session);
        }));
        self.test_session = Some(session);
        Ok(())
    }

    fn start_test_close(&mut self, profile_id: &str) -> AppResult<()> {
        self.ensure_no_session()?;
        abort(self.test_task.take());

        let previous_test = self
            .test_session
            .take()
            .filter(|session| lock(session).profile().id == profile_id);
        let session = match previous_test {
            Some(session) => {
                self.reporter.attach(&session);
                session
            }
            None => {
                let mut session = Session::start(self.profile(profile_id)?, None, true);
                session.assume_all_launched();
                self.attach(session)
            }
        };

        let (close_only, timeout) = self.close_settings();
        let reporter = self.reporter.clone();
        self.test_task = Some(tauri::async_runtime::spawn(async move {
            runner::close_items(session.clone(), reporter.clone(), close_only, timeout).await;
            reporter.finish(&session);
        }));
        Ok(())
    }

    fn ensure_no_session(&self) -> AppResult<()> {
        if self.state.has_session() {
            return Err(AppError::SessionInProgress);
        }
        Ok(())
    }

    fn attach(&self, session: Session) -> SharedSession {
        let session = Arc::new(Mutex::new(session));
        self.reporter.attach(&session);
        session
    }

    fn spawn_launch(&self, session: SharedSession) -> JoinHandle<()> {
        tauri::async_runtime::spawn(runner::launch_items(
            self.app.clone(),
            session,
            self.reporter.clone(),
        ))
    }

    fn close_settings(&self) -> (bool, Duration) {
        let settings = self.app.state::<AppState>().settings();
        (
            settings.close_only_if_launched_by_app,
            Duration::from_millis(u64::from(settings.graceful_timeout_ms)),
        )
    }

    fn close_delay_ticks(&self) -> u32 {
        let delay_ms = self.app.state::<AppState>().settings().close_delay_ms;
        detection::close_delay_ticks(delay_ms, POLL_INTERVAL)
    }

    fn profile(&self, profile_id: &str) -> AppResult<Profile> {
        self.app.state::<AppState>().profile(profile_id)
    }
}

fn abort(task: Option<JoinHandle<()>>) {
    if let Some(task) = task {
        task.abort();
    }
}
