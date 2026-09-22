use std::time::Duration;

use tauri::AppHandle;

use super::reporter::{lock, Reporter, SharedSession};
use crate::closer::{self, CloseOutcome};
use crate::launcher::{self, LaunchOutcome};
use crate::models::{ItemRuntime, ItemStatus, TimelineKind};

pub async fn launch_items(app: AppHandle, session: SharedSession, reporter: Reporter) {
    let items = lock(&session).enabled_items();

    for item in items {
        tokio::time::sleep(Duration::from_millis(u64::from(item.delay_ms()))).await;
        reporter.item(&session, status(item.id(), ItemStatus::Launching));

        let outcome = launcher::launch_item(&app, &item).await;
        let (kind, message) = match &outcome {
            LaunchOutcome::Launched => (TimelineKind::Launched, String::new()),
            LaunchOutcome::AlreadyRunning => (TimelineKind::Skipped, String::new()),
            LaunchOutcome::Failed(error) => (TimelineKind::Error, error.to_string()),
        };
        reporter.timeline(&session, kind, Some(item.id()), message);
        reporter.item(&session, outcome.into_runtime(item.id()));
    }
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
        let (kind, message) = match &outcome {
            CloseOutcome::ClosedGracefully => (TimelineKind::ClosedGracefully, String::new()),
            CloseOutcome::ForceClosed => (TimelineKind::ForceClosed, String::new()),
            CloseOutcome::NotRunning => (TimelineKind::Skipped, "not running".to_owned()),
            CloseOutcome::Kept => (TimelineKind::Kept, String::new()),
            CloseOutcome::Failed(error) => (TimelineKind::Error, error.to_string()),
        };
        reporter.timeline(&session, kind, Some(&item_id), message);
        reporter.item(&session, outcome.into_runtime(&item_id));
    }
}

fn status(item_id: &str, status: ItemStatus) -> ItemRuntime {
    ItemRuntime {
        item_id: item_id.to_owned(),
        status,
        launched_by_app: false,
        message: None,
    }
}
