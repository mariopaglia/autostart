use std::collections::HashSet;
use std::path::Path;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

use super::reporter::{lock, Reporter, SharedSession};
use crate::launcher::LaunchedApp;
use crate::models::{AppItem, ProcessNameMode};
use crate::process_tracker::{choose_process_name, expand_tracked, LaunchTrace, ProcessSample};
use crate::state::AppState;
use crate::{platform, processes};

pub const PROFILE_CHANGED_EVENT: &str = "profiles://changed";

const WATCH_INTERVAL: Duration = Duration::from_millis(500);
const LEARNING_WINDOW: Duration = Duration::from_secs(30);
const MINIMIZE_WINDOW: Duration = Duration::from_secs(10);

/// Follows an app after it is running: minimizes its first windows and learns which process
/// actually represents it, so launchers need no manual process name.
pub async fn watch(
    app: AppHandle,
    session: SharedSession,
    reporter: Reporter,
    item: AppItem,
    launched: LaunchedApp,
) {
    let learns = item.process_name_mode == ProcessNameMode::Auto;
    let observation = match (learns, item.start_minimized) {
        (true, _) => LEARNING_WINDOW,
        (false, true) => MINIMIZE_WINDOW,
        (false, false) => return,
    };

    let started = Instant::now();
    let mut tracked = launched.tracked.clone();
    let mut minimized = HashSet::new();
    let samples = loop {
        let samples = processes::samples();
        tracked = expand_tracked(&tracked, &samples);
        if item.start_minimized && started.elapsed() < MINIMIZE_WINDOW {
            platform::minimize_new_windows(&app_pids(&item, &tracked, &samples), &mut minimized);
        }
        if started.elapsed() >= observation {
            break samples;
        }
        tokio::time::sleep(WATCH_INTERVAL).await;
    };

    if !learns {
        return;
    }
    let install_dir = Path::new(&item.exe_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let trace = LaunchTrace {
        root_pid: launched.root_pid,
        tracked: &tracked,
        baseline: &launched.baseline,
        install_dir,
    };
    if let Some(process_name) = choose_process_name(&trace, &samples) {
        apply_learned_name(&app, &session, &reporter, &item.id, &process_name);
    }
}

fn app_pids(item: &AppItem, tracked: &HashSet<u32>, samples: &[ProcessSample]) -> HashSet<u32> {
    let wanted = processes::normalize_process_name(&item.process_name);
    samples
        .iter()
        .filter(|sample| {
            tracked.contains(&sample.pid)
                || processes::normalize_process_name(&sample.name) == wanted
        })
        .map(|sample| sample.pid)
        .collect()
}

fn apply_learned_name(
    app: &AppHandle,
    session: &SharedSession,
    reporter: &Reporter,
    item_id: &str,
    process_name: &str,
) {
    let profile_id = {
        let mut session = lock(session);
        if !session.learn_process_name(item_id, process_name) {
            return;
        }
        session.profile().id.clone()
    };
    reporter.process_name_learned(session, item_id, process_name);

    match app
        .state::<AppState>()
        .learn_process_name(&profile_id, item_id, process_name)
    {
        Ok(Some(profile)) => {
            if let Err(error) = app.emit(PROFILE_CHANGED_EVENT, profile) {
                log::warn!("failed to emit {PROFILE_CHANGED_EVENT}: {error}");
            }
        }
        Ok(None) => {}
        Err(error) => log::error!("could not save the learned process name: {error}"),
    }
}
