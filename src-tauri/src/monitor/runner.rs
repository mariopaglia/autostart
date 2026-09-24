use std::sync::Arc;
use std::time::Duration;

use tauri::AppHandle;
use tokio::sync::{mpsc, Notify};

use super::reporter::{lock, Reporter, SharedSession};
use super::session::StartMode;
use super::MonitorCommand;
use super::{app_watch, crash_watch};
use crate::closer::{self, CloseOutcome};
use crate::error::AppError;
use crate::launcher::{self, LaunchOutcome, SimulatorStart};
use crate::models::{ItemRuntime, ItemStatus, LaunchItem, TimelineKind, Trigger};
use crate::simconnect;

/// Signals from the monitor and back to it while a flight started from AutoStart waits for
/// the simulator.
pub struct FlightLink {
    pub simulator_arrived: Arc<Notify>,
    pub monitor: mpsc::Sender<MonitorCommand>,
}

pub async fn launch_items(
    app: AppHandle,
    session: SharedSession,
    reporter: Reporter,
    flight: FlightLink,
) {
    let (items, supports_simconnect, mode) = {
        let session = lock(&session);
        (
            session.enabled_items(),
            session.supports_simconnect(),
            session.mode().clone(),
        )
    };
    let is_test = mode == StartMode::Test;
    let phases = split_launch_phases(items, matches!(mode, StartMode::Flight(_)));

    if let StartMode::Flight(trigger) = &mode {
        for item in &phases.before_simulator {
            launch_one(&app, &session, &reporter, item).await;
        }
        if !start_simulator(&session, &reporter, trigger, &flight).await {
            return;
        }
    }
    for item in &phases.immediate {
        launch_one(&app, &session, &reporter, item).await;
    }
    if phases.deferred.is_empty()
        || !await_simconnect(
            &session,
            &reporter,
            &phases.deferred,
            supports_simconnect,
            is_test,
        )
        .await
    {
        return;
    }
    for item in &phases.deferred {
        launch_one(&app, &session, &reporter, item).await;
    }
}

#[derive(Debug, Default)]
struct LaunchPhases {
    before_simulator: Vec<LaunchItem>,
    immediate: Vec<LaunchItem>,
    deferred: Vec<LaunchItem>,
}

/// Each phase keeps the list order. Only flights started from AutoStart open items before
/// the simulator; items that need SimConnect always go last.
fn split_launch_phases(items: Vec<LaunchItem>, is_flight: bool) -> LaunchPhases {
    let mut phases = LaunchPhases::default();
    for item in items {
        match &item {
            LaunchItem::App(app) if is_flight && app.launch_before_simulator => {
                phases.before_simulator.push(item);
            }
            LaunchItem::App(app) if app.wait_for_sim_connect => phases.deferred.push(item),
            _ => phases.immediate.push(item),
        }
    }
    phases
}

/// Returns whether the simulator showed up and the remaining items should be launched.
async fn start_simulator(
    session: &SharedSession,
    reporter: &Reporter,
    trigger: &Trigger,
    flight: &FlightLink,
) -> bool {
    match launcher::start_simulator(trigger).await {
        Ok(SimulatorStart::Started) => reporter.timeline_detail(
            session,
            TimelineKind::SimulatorStarted,
            None,
            trigger.label.clone(),
        ),
        Ok(SimulatorStart::AlreadyRunning) => {}
        Err(error) => {
            reporter.simulator_not_started(session, &trigger.label, Some(&error));
            if flight
                .monitor
                .send(MonitorCommand::SimulatorFailed)
                .await
                .is_err()
            {
                log::error!("monitor task is not running");
            }
            return false;
        }
    }
    flight.simulator_arrived.notified().await;
    true
}

/// Returns whether the deferred items should be launched.
async fn await_simconnect(
    session: &SharedSession,
    reporter: &Reporter,
    deferred: &[LaunchItem],
    supports_simconnect: bool,
    is_test: bool,
) -> bool {
    if !supports_simconnect {
        return true;
    }
    if is_test {
        reporter.timeline(session, TimelineKind::SimConnectWaitSkipped, None, None);
        return true;
    }

    for item in deferred {
        reporter.item(session, status(item.id(), ItemStatus::WaitingSimConnect));
    }
    if simconnect::wait_until_available(simconnect::WAIT_TIMEOUT).await {
        reporter.timeline(session, TimelineKind::SimConnectReady, None, None);
        return true;
    }

    let error = AppError::SimConnectUnavailable;
    for item in deferred {
        reporter.timeline(session, TimelineKind::Error, Some(item.id()), Some(&error));
        reporter.item(
            session,
            ItemRuntime {
                error: Some((&error).into()),
                ..status(item.id(), ItemStatus::Error)
            },
        );
    }
    false
}

async fn launch_one(
    app: &AppHandle,
    session: &SharedSession,
    reporter: &Reporter,
    item: &LaunchItem,
) {
    tokio::time::sleep(Duration::from_millis(u64::from(item.delay_ms()))).await;
    reporter.item(session, status(item.id(), ItemStatus::Launching));

    let watches_crashes = matches!(item, LaunchItem::App(app_item) if app_item.restart_on_crash)
        && !lock(session).is_test();
    let outcome = match (launcher::launch_item(item).await, item) {
        (LaunchOutcome::AppLaunched(launched), LaunchItem::App(app_item)) => {
            let (app, session, reporter, app_item) = (
                app.clone(),
                session.clone(),
                reporter.clone(),
                app_item.clone(),
            );
            tauri::async_runtime::spawn(async move {
                // Crash watching needs the learned process name, so it starts after learning.
                app_watch::watch(
                    app.clone(),
                    session.clone(),
                    reporter.clone(),
                    app_item.clone(),
                    launched,
                )
                .await;
                if watches_crashes {
                    crash_watch::watch(session, reporter, app_item.id).await;
                }
            });
            LaunchOutcome::Launched
        }
        (LaunchOutcome::AlreadyRunning, _) if watches_crashes => {
            tauri::async_runtime::spawn(crash_watch::watch(
                session.clone(),
                reporter.clone(),
                item.id().to_owned(),
            ));
            LaunchOutcome::AlreadyRunning
        }
        (outcome, _) => outcome,
    };
    let (kind, error) = match &outcome {
        LaunchOutcome::Launched | LaunchOutcome::AppLaunched(_) => (TimelineKind::Launched, None),
        LaunchOutcome::AlreadyRunning => (TimelineKind::Skipped, None),
        LaunchOutcome::Failed(error) => (TimelineKind::Error, Some(error)),
    };
    reporter.timeline(session, kind, Some(item.id()), error);
    reporter.item(session, outcome.into_runtime(item.id()));
}

pub async fn close_items(
    session: SharedSession,
    reporter: Reporter,
    close_only_if_launched_by_app: bool,
    graceful_timeout: Duration,
) {
    let targets = lock(&session).close_targets(close_only_if_launched_by_app);
    for target in &targets {
        reporter.item(&session, status(&target.item_id, ItemStatus::Closing));
    }

    for (item_id, outcome) in closer::close_all(targets, graceful_timeout).await {
        let (kind, error) = match &outcome {
            CloseOutcome::ClosedGracefully => (TimelineKind::ClosedGracefully, None),
            CloseOutcome::ForceClosed => (TimelineKind::ForceClosed, None),
            CloseOutcome::NotRunning => (TimelineKind::NotRunning, None),
            CloseOutcome::Kept => (TimelineKind::Kept, None),
            CloseOutcome::Failed(error) => (TimelineKind::Error, Some(error)),
        };
        reporter.timeline(&session, kind, Some(&item_id), error);
        reporter.item(&session, outcome.into_runtime(&item_id));
    }
}

fn status(item_id: &str, status: ItemStatus) -> ItemRuntime {
    ItemRuntime {
        item_id: item_id.to_owned(),
        status,
        launched_by_app: false,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AppItem, OnClose, ProcessNameMode, UrlItem};

    fn app(id: &str, wait_for_sim_connect: bool, launch_before_simulator: bool) -> LaunchItem {
        LaunchItem::App(AppItem {
            id: id.into(),
            name: id.into(),
            exe_path: format!("C:\\{id}.exe"),
            args: None,
            working_dir: None,
            process_name: format!("{id}.exe"),
            process_name_mode: ProcessNameMode::Auto,
            icon_base64: None,
            delay_ms: 0,
            run_as_admin: false,
            start_minimized: false,
            wait_for_sim_connect,
            restart_on_crash: false,
            launch_before_simulator,
            only_for_triggers: Vec::new(),
            on_close: OnClose::Graceful,
            enabled: true,
        })
    }

    fn url(id: &str) -> LaunchItem {
        LaunchItem::Url(UrlItem {
            id: id.into(),
            name: id.into(),
            url: "https://simbrief.com".into(),
            delay_ms: 0,
            only_for_triggers: Vec::new(),
            enabled: true,
        })
    }

    fn ids(items: &[LaunchItem]) -> Vec<&str> {
        items.iter().map(LaunchItem::id).collect()
    }

    #[test]
    fn simconnect_items_go_last_keeping_their_order() {
        let phases = split_launch_phases(
            vec![
                app("a", false, false),
                app("b", true, false),
                url("c"),
                app("d", true, false),
            ],
            false,
        );

        assert_eq!(ids(&phases.immediate), ["a", "c"]);
        assert_eq!(ids(&phases.deferred), ["b", "d"]);
    }

    #[test]
    fn without_simconnect_items_there_is_a_single_phase() {
        let phases = split_launch_phases(vec![app("a", false, false), url("b")], false);

        assert_eq!(ids(&phases.immediate), ["a", "b"]);
        assert!(phases.deferred.is_empty());
    }

    #[test]
    fn a_flight_opens_the_marked_items_before_the_simulator() {
        let items = vec![
            app("a", false, false),
            app("b", false, true),
            url("c"),
            app("d", true, false),
        ];

        let phases = split_launch_phases(items, true);

        assert_eq!(ids(&phases.before_simulator), ["b"]);
        assert_eq!(ids(&phases.immediate), ["a", "c"]);
        assert_eq!(ids(&phases.deferred), ["d"]);
    }

    #[test]
    fn a_detected_simulator_opens_the_marked_items_in_list_order() {
        let phases =
            split_launch_phases(vec![app("a", false, false), app("b", false, true)], false);

        assert!(phases.before_simulator.is_empty());
        assert_eq!(ids(&phases.immediate), ["a", "b"]);
    }
}
