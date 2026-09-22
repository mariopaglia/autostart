mod app_watch;
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
use crate::models::{MonitorSnapshot, Profile, SessionLog};
use crate::processes::ProcessTable;
use crate::state::AppState;
use reporter::{lock, Reporter, SharedSession};
use session::Session;
use state_machine::{Input, SessionAction, State};

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const COMMAND_BUFFER: usize = 32;

enum MonitorCommand {
    Pause,
    Resume,
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
        let Some(trigger) = self.watched_trigger() else {
            return;
        };
        self.processes.refresh();
        let input = if self.processes.is_running(&trigger) {
            Input::TriggerSeen
        } else {
            Input::TriggerMissing
        };
        self.apply(input);
    }

    /// A running session keeps watching its own trigger even if the active profile changes.
    fn watched_trigger(&self) -> Option<String> {
        if let Some(session) = &self.session {
            return Some(lock(session).profile().trigger.process_name.clone());
        }
        let profile = self.active_profile()?;
        profile.enabled.then_some(profile.trigger.process_name)
    }

    fn handle(&mut self, command: MonitorCommand) {
        match command {
            MonitorCommand::Pause => self.apply(Input::Pause),
            MonitorCommand::Resume => self.apply(Input::Resume),
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
        let (next_state, action) = state_machine::next(self.state, input);
        let previous = std::mem::replace(&mut self.state, next_state);
        if previous.public() != next_state.public() {
            self.reporter.set_state(next_state.public());
        }

        match action {
            Some(SessionAction::Start) => self.start_session(),
            Some(SessionAction::End) => self.end_session(),
            Some(SessionAction::Discard) => self.discard_session(),
            None => {}
        }
    }

    fn start_session(&mut self) {
        abort(self.test_task.take());
        self.test_session = None;

        let Some(profile) = self.active_profile() else {
            self.state = State::Idle;
            self.reporter.set_state(self.state.public());
            return;
        };
        let session = self.attach(Session::start(profile, false));
        self.launch_task = Some(self.spawn_launch(session.clone()));
        self.session = Some(session);
    }

    fn end_session(&mut self) {
        abort(self.launch_task.take());
        let Some(session) = self.session.clone() else {
            return;
        };

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

        let session = self.attach(Session::start(self.profile(profile_id)?, true));
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
                let mut session = Session::start(self.profile(profile_id)?, true);
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

    fn active_profile(&self) -> Option<Profile> {
        let state = self.app.state::<AppState>();
        let active_id = state.settings().active_profile_id?;
        state.profile(&active_id).ok()
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
