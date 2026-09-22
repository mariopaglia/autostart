use std::time::Duration;

use tokio::task::JoinSet;

use crate::error::{AppError, AppResult};
use crate::models::{AppItem, ItemRuntime, ItemStatus, OnClose};
use crate::{platform, processes};

const EXIT_POLL_INTERVAL: Duration = Duration::from_millis(250);
const GLOBAL_TIMEOUT_MARGIN: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct CloseTarget {
    pub item_id: String,
    pub process_name: String,
    pub on_close: OnClose,
}

impl CloseTarget {
    pub fn from_app_item(item: &AppItem) -> Self {
        Self {
            item_id: item.id.clone(),
            process_name: item.process_name.clone(),
            on_close: item.on_close,
        }
    }
}

#[derive(Debug)]
pub enum CloseOutcome {
    ClosedGracefully,
    ForceClosed,
    NotRunning,
    Kept,
    Failed(AppError),
}

/// Closes every target in parallel; the whole batch never exceeds the graceful timeout plus a margin.
pub async fn close_all(
    targets: Vec<CloseTarget>,
    graceful_timeout: Duration,
) -> Vec<(String, CloseOutcome)> {
    let pending = targets.clone();
    let mut tasks = JoinSet::new();
    for target in targets {
        tasks.spawn(async move {
            let outcome = close_target(&target, graceful_timeout).await;
            (target.item_id, outcome)
        });
    }

    let mut results = Vec::with_capacity(pending.len());
    let deadline = tokio::time::sleep(graceful_timeout + GLOBAL_TIMEOUT_MARGIN);
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            joined = tasks.join_next() => match joined {
                Some(Ok(result)) => results.push(result),
                Some(Err(error)) => log::error!("close task panicked: {error}"),
                None => break,
            },
            () = &mut deadline => {
                tasks.abort_all();
                break;
            }
        }
    }

    for target in pending {
        if !results.iter().any(|(id, _)| *id == target.item_id) {
            let timeout = AppError::CloseTimedOut(target.process_name);
            results.push((target.item_id, CloseOutcome::Failed(timeout)));
        }
    }
    results
}

async fn close_target(target: &CloseTarget, graceful_timeout: Duration) -> CloseOutcome {
    if target.on_close == OnClose::Keep {
        return CloseOutcome::Kept;
    }
    let pids = processes::pids_by_name(&target.process_name);
    if pids.is_empty() {
        return CloseOutcome::NotRunning;
    }

    let result = match target.on_close {
        OnClose::Force => {
            terminate_all(&pids, &target.process_name).map(|()| CloseOutcome::ForceClosed)
        }
        _ => close_gracefully(target, &pids, graceful_timeout).await,
    };
    result.unwrap_or_else(CloseOutcome::Failed)
}

async fn close_gracefully(
    target: &CloseTarget,
    pids: &[u32],
    graceful_timeout: Duration,
) -> AppResult<CloseOutcome> {
    for &pid in pids {
        platform::request_close(pid);
    }

    let attempts = graceful_timeout.as_millis() / EXIT_POLL_INTERVAL.as_millis();
    for _ in 0..attempts {
        tokio::time::sleep(EXIT_POLL_INTERVAL).await;
        if still_running(pids, &target.process_name).is_empty() {
            return Ok(CloseOutcome::ClosedGracefully);
        }
    }

    terminate_all(
        &still_running(pids, &target.process_name),
        &target.process_name,
    )?;
    Ok(CloseOutcome::ForceClosed)
}

fn still_running(pids: &[u32], process_name: &str) -> Vec<u32> {
    let alive = processes::pids_by_name(process_name);
    pids.iter()
        .copied()
        .filter(|pid| alive.contains(pid))
        .collect()
}

fn terminate_all(pids: &[u32], process_name: &str) -> AppResult<()> {
    pids.iter()
        .try_for_each(|&pid| platform::terminate(pid, process_name))
}

impl CloseOutcome {
    pub fn into_runtime(self, item_id: &str) -> ItemRuntime {
        let (status, message) = match self {
            Self::ClosedGracefully | Self::ForceClosed | Self::NotRunning => {
                (ItemStatus::Closed, None)
            }
            Self::Kept => (ItemStatus::Skipped, None),
            Self::Failed(error) => (ItemStatus::Error, Some(error.to_string())),
        };
        ItemRuntime {
            item_id: item_id.to_owned(),
            status,
            launched_by_app: false,
            message,
        }
    }
}
