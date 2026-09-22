use std::time::Duration;

use tauri::AppHandle;

use super::app_watch;
use super::reporter::{lock, Reporter, SharedSession};
use crate::closer::{self, CloseOutcome};
use crate::launcher::{self, LaunchOutcome};
use crate::models::{ItemRuntime, ItemStatus, LaunchItem, TimelineKind};

pub async fn launch_items(app: AppHandle, session: SharedSession, reporter: Reporter) {
    let items = lock(&session).enabled_items();

    for item in items {
        launch_one(&app, &session, &reporter, &item).await;
    }
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
