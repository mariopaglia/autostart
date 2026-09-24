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
use tokio::sync::{mpsc, oneshot, Notify};
use tokio::time::MissedTickBehavior;

use crate::error::{AppError, AppResult};
use crate::models::{MonitorSnapshot, Profile, SessionLog, TimelineKind};
use crate::processes::{normalize_process_name, ProcessTable};
use crate::state::AppState;
pub use app_watch::PROFILE_CHANGED_EVENT;
use reporter::{lock, Reporter, SharedSession};
use session::{now_ms, Session, StartMode};
use state_machine::{Input, SessionAction, State};

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const COMMAND_BUFFER: usize = 32;

enum MonitorCommand {
    StartFlight {
        profile_id: String,
        trigger_process_name: String,
        reply: oneshot::Sender<AppResult<()>>,
    },
    CancelStart,
    SimulatorFailed,
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
        let last_session_log = app.state::<AppState>().session_history().into_iter().next();
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
            simulator_arrived: Arc::new(Notify::new()),
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

    pub async fn start_flight(
        &self,
        profile_id: String,
        trigger_process_name: String,
    ) -> AppResult<()> {
        let (reply, response) = oneshot::channel();
        self.send(MonitorCommand::StartFlight {
            profile_id,
            trigger_process_name,
            reply,
        })
        .await;
        response.await.unwrap_or(Err(AppError::SessionInProgress))
    }

    pub async fn cancel_flight_start(&self) {
        self.send(MonitorCommand::CancelStart).await;
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
    pending_start: Option<(Profile, StartMode)>,
    simulator_arrived: Arc<Notify>,
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
            Some(session) => {
                let session = lock(session);
                let starting = matches!(self.state, State::SimStarting { .. });
                let awaited = session.mode().trigger().filter(|_| starting);
                detection::session_simulator_running(session.profile(), awaited, is_running)
            }
            None => {
                let profiles = self.app.state::<AppState>().profiles();
                self.pending_start = detection::first_running_profile(&profiles, is_running)
                    .map(|(profile, trigger)| (profile, StartMode::Detected(trigger)));
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
            MonitorCommand::StartFlight {
                profile_id,
                trigger_process_name,
                reply,
            } => {
                let _ = reply.send(self.start_flight(&profile_id, &trigger_process_name));
            }
            MonitorCommand::CancelStart | MonitorCommand::SimulatorFailed => {
                self.apply(Input::SimulatorFailed);
            }
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
            Some(SessionAction::SimulatorArrived) => self.simulator_arrived.notify_one(),
            Some(SessionAction::StartFailed) => self.start_failed(input),
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

        let Some((profile, mode)) = self.pending_start.take() else {
            self.state = State::Idle;
            self.reporter.set_state(self.state.public());
            return;
        };
        let session = self.attach(Session::start(profile, mode));
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

    /// Timing out records it here; a failed start or a cancel was already reported.
    fn start_failed(&mut self, input: Input) {
        if input == Input::TriggerMissing {
            if let Some(session) = &self.session {
                let label = lock(session)
                    .mode()
                    .trigger()
                    .map(|trigger| trigger.label.clone())
                    .unwrap_or_default();
                self.reporter.simulator_not_started(session, &label, None);
            }
        }
        if self.state == State::Closing {
            self.end_session();
        } else {
            self.delay_close();
        }
    }

    fn continue_session(&mut self) {
        // A simulator that shows up late, during the close delay, still lets a flight go on.
        self.simulator_arrived.notify_one();
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

    fn start_flight(&mut self, profile_id: &str, trigger_process_name: &str) -> AppResult<()> {
        if self.state != State::Idle {
            return Err(AppError::SessionInProgress);
        }
        let profile = self.profile(profile_id)?;
        let trigger = profile
            .triggers
            .iter()
            .find(|trigger| {
                normalize_process_name(&trigger.process_name)
                    == normalize_process_name(trigger_process_name)
            })
            .filter(|trigger| trigger.launch_target.is_some())
            .cloned()
            .ok_or_else(|| {
                AppError::Validation(format!("{trigger_process_name} has no launch target"))
            })?;

        abort(self.test_task.take());
        self.test_session = None;
        self.pending_start = Some((profile, StartMode::Flight(trigger)));
        self.apply(Input::StartFlight);
        Ok(())
    }

    fn start_test_launch(&mut self, profile_id: &str) -> AppResult<()> {
        self.ensure_no_session()?;
        abort(self.test_task.take());

        let session = self.attach(Session::start(self.profile(profile_id)?, StartMode::Test));
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
                let mut session = Session::start(self.profile(profile_id)?, StartMode::Test);
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

    fn spawn_launch(&mut self, session: SharedSession) -> JoinHandle<()> {
        self.simulator_arrived = Arc::new(Notify::new());
        let flight = runner::FlightLink {
            simulator_arrived: self.simulator_arrived.clone(),
            monitor: self.commands.clone(),
        };
        tauri::async_runtime::spawn(runner::launch_items(
            self.app.clone(),
            session,
            self.reporter.clone(),
            flight,
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
