use std::time::Duration;

use tauri::AppHandle;

use super::app_watch;
use super::reporter::{lock, Reporter, SharedSession};
use crate::closer::{self, CloseOutcome};
use crate::error::AppError;
use crate::launcher::{self, LaunchOutcome};
use crate::models::{ItemRuntime, ItemStatus, LaunchItem, TimelineKind};
use crate::simconnect;

pub async fn launch_items(app: AppHandle, session: SharedSession, reporter: Reporter) {
    let (items, trigger, is_test) = {
        let session = lock(&session);
        (
            session.enabled_items(),
            session.profile().trigger.process_name.clone(),
            session.is_test(),
        )
    };
    let (immediate, deferred) = split_launch_phases(items);

    for item in &immediate {
        launch_one(&app, &session, &reporter, item).await;
    }
    if deferred.is_empty()
        || !await_simconnect(&session, &reporter, &deferred, &trigger, is_test).await
    {
        return;
    }
    for item in &deferred {
        launch_one(&app, &session, &reporter, item).await;
    }
}

/// Items that need SimConnect move to a second phase; each phase keeps the list order.
fn split_launch_phases(items: Vec<LaunchItem>) -> (Vec<LaunchItem>, Vec<LaunchItem>) {
    items
        .into_iter()
        .partition(|item| !matches!(item, LaunchItem::App(app) if app.wait_for_sim_connect))
}

/// Returns whether the deferred items should be launched.
async fn await_simconnect(
    session: &SharedSession,
    reporter: &Reporter,
    deferred: &[LaunchItem],
    trigger: &str,
    is_test: bool,
) -> bool {
    if !simconnect::supports_simconnect(trigger) {
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

    let outcome = match (launcher::launch_item(app, item).await, item) {
        (LaunchOutcome::AppLaunched(launched), LaunchItem::App(app_item)) => {
            tauri::async_runtime::spawn(app_watch::watch(
                app.clone(),
                session.clone(),
                reporter.clone(),
                app_item.clone(),
                launched,
            ));
            LaunchOutcome::Launched
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

    fn app(id: &str, wait_for_sim_connect: bool) -> LaunchItem {
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
            enabled: true,
        })
    }

    fn ids(items: &[LaunchItem]) -> Vec<&str> {
        items.iter().map(LaunchItem::id).collect()
    }

    #[test]
    fn simconnect_items_go_last_keeping_their_order() {
        let (immediate, deferred) = split_launch_phases(vec![
            app("a", false),
            app("b", true),
            url("c"),
            app("d", true),
        ]);

        assert_eq!(ids(&immediate), ["a", "c"]);
        assert_eq!(ids(&deferred), ["b", "d"]);
    }

    #[test]
    fn without_simconnect_items_there_is_a_single_phase() {
        let (immediate, deferred) = split_launch_phases(vec![app("a", false), url("b")]);

        assert_eq!(ids(&immediate), ["a", "b"]);
        assert!(deferred.is_empty());
    }
}
