use std::time::Duration;

use super::reporter::{lock, Reporter, SharedSession};
use crate::error::AppError;
use crate::launcher::{self, LaunchOutcome};
use crate::models::{AppItem, ItemRuntime, ItemStatus, LaunchItem, TimelineKind};
use crate::platform::ExitWatcher;
use crate::processes;

const CHECK_INTERVAL: Duration = Duration::from_secs(2);
const RELAUNCH_DELAY: Duration = Duration::from_secs(5);
const MAX_RELAUNCHES: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Crashed { exit_code: u32 },
    ClosedNormally,
}

/// Decides only once every watched process has exited: a non-zero exit code means a crash
/// (or a kill), while exit code 0 means the app was closed on purpose.
pub fn crash_verdict(exit_codes: &[Option<u32>]) -> Option<Verdict> {
    if exit_codes.is_empty() || exit_codes.iter().any(Option::is_none) {
        return None;
    }
    let failing = exit_codes.iter().flatten().find(|&&code| code != 0);
    Some(
        failing.map_or(Verdict::ClosedNormally, |&exit_code| Verdict::Crashed {
            exit_code,
        }),
    )
}

pub fn may_relaunch(relaunches: u32) -> bool {
    relaunches < MAX_RELAUNCHES
}

/// Follows an item with `restart_on_crash` for the rest of a real session and reopens it
/// after a crash while the simulator is running.
pub async fn watch(session: SharedSession, reporter: Reporter, item_id: String) {
    let mut relaunches = 0;
    loop {
        let Some(Verdict::Crashed { exit_code }) = wait_for_exit(&session, &item_id).await else {
            return;
        };
        if !lock(&session).relaunch_allowed() {
            return;
        }
        reporter.timeline_detail(
            &session,
            TimelineKind::Crashed,
            Some(&item_id),
            format!("{exit_code:#x}"),
        );
        if !may_relaunch(relaunches) {
            give_up(&session, &reporter, &item_id);
            return;
        }

        reporter.item(&session, runtime(&item_id, ItemStatus::Restarting, false));
        tokio::time::sleep(RELAUNCH_DELAY).await;
        let Some(item) = relaunchable_item(&session, &item_id) else {
            return;
        };
        if !relaunch(&session, &reporter, item).await {
            return;
        }
        relaunches += 1;
    }
}

/// Returns `None` when the session ends or the item is no longer part of it.
async fn wait_for_exit(session: &SharedSession, item_id: &str) -> Option<Verdict> {
    let mut watcher = ExitWatcher::default();
    loop {
        let process_name = {
            let session = lock(session);
            if session.is_finished() {
                return None;
            }
            session.app_item(item_id)?.process_name
        };
        // Watching every current process first keeps a freshly started one from being missed.
        for pid in processes::pids_by_name(&process_name) {
            watcher.watch(pid);
        }
        if let Some(verdict) = crash_verdict(&watcher.exit_codes()) {
            return Some(verdict);
        }
        tokio::time::sleep(CHECK_INTERVAL).await;
    }
}

fn relaunchable_item(session: &SharedSession, item_id: &str) -> Option<AppItem> {
    let session = lock(session);
    session
        .relaunch_allowed()
        .then(|| session.app_item(item_id))
        .flatten()
}

/// Returns whether the item is running again and should keep being watched.
async fn relaunch(session: &SharedSession, reporter: &Reporter, item: AppItem) -> bool {
    match launcher::launch_item(&LaunchItem::App(item.clone())).await {
        LaunchOutcome::Launched | LaunchOutcome::AppLaunched(_) => {
            reporter.timeline(session, TimelineKind::Relaunched, Some(&item.id), None);
            reporter.item(session, runtime(&item.id, ItemStatus::Running, true));
            true
        }
        // The pilot reopened it during the delay.
        LaunchOutcome::AlreadyRunning => {
            reporter.item(session, runtime(&item.id, ItemStatus::Running, false));
            true
        }
        LaunchOutcome::Failed(error) => {
            reporter.timeline(session, TimelineKind::Error, Some(&item.id), Some(&error));
            reporter.item(
                session,
                ItemRuntime {
                    error: Some((&error).into()),
                    ..runtime(&item.id, ItemStatus::Error, false)
                },
            );
            false
        }
    }
}

fn give_up(session: &SharedSession, reporter: &Reporter, item_id: &str) {
    let name = lock(session)
        .app_item(item_id)
        .map(|item| item.name)
        .unwrap_or_default();
    let error = AppError::KeptCrashing(name);
    reporter.timeline(session, TimelineKind::Error, Some(item_id), Some(&error));
    reporter.item(
        session,
        ItemRuntime {
            error: Some((&error).into()),
            ..runtime(item_id, ItemStatus::Error, false)
        },
    );
}

fn runtime(item_id: &str, status: ItemStatus, launched_by_app: bool) -> ItemRuntime {
    ItemRuntime {
        item_id: item_id.to_owned(),
        status,
        launched_by_app,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_verdict_while_a_process_runs_or_none_was_seen() {
        assert_eq!(crash_verdict(&[]), None);
        assert_eq!(crash_verdict(&[None]), None);
        assert_eq!(crash_verdict(&[Some(0xC000_0005), None]), None);
    }

    #[test]
    fn exit_code_zero_is_a_normal_close() {
        assert_eq!(
            crash_verdict(&[Some(0), Some(0)]),
            Some(Verdict::ClosedNormally)
        );
    }

    #[test]
    fn any_failing_exit_code_is_a_crash() {
        assert_eq!(
            crash_verdict(&[Some(0), Some(0xC000_0005)]),
            Some(Verdict::Crashed {
                exit_code: 0xC000_0005
            })
        );
        assert_eq!(
            crash_verdict(&[Some(1)]),
            Some(Verdict::Crashed { exit_code: 1 })
        );
    }

    #[test]
    fn relaunches_stop_after_the_third() {
        assert!(may_relaunch(0));
        assert!(may_relaunch(2));
        assert!(!may_relaunch(3));
    }
}
