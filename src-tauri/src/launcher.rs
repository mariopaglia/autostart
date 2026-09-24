use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::{AppError, AppResult};
use crate::models::{AppItem, ItemRuntime, ItemStatus, LaunchItem, Trigger, UrlItem};
use crate::platform::LaunchOptions;
use crate::process_tracker::{any_tracked_alive, expand_tracked, ProcessSample};
use crate::{platform, processes};

const PROCESS_DETECTION_TIMEOUT: Duration = Duration::from_secs(10);
const PROCESS_DETECTION_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub enum LaunchOutcome {
    Launched,
    /// Only app items report their processes, which the watcher keeps following.
    AppLaunched(LaunchedApp),
    AlreadyRunning,
    Failed(AppError),
}

#[derive(Debug)]
pub struct LaunchedApp {
    pub root_pid: Option<u32>,
    pub tracked: HashSet<u32>,
    pub baseline: HashSet<u32>,
}

pub async fn launch_item(item: &LaunchItem) -> LaunchOutcome {
    let result = match item {
        LaunchItem::App(app_item) => launch_app(app_item).await,
        LaunchItem::Url(url_item) => open_url(url_item).await.map(|()| LaunchOutcome::Launched),
    };
    result.unwrap_or_else(LaunchOutcome::Failed)
}

async fn launch_app(item: &AppItem) -> AppResult<LaunchOutcome> {
    if processes::is_running(&item.process_name) {
        return Ok(LaunchOutcome::AlreadyRunning);
    }

    let exe_path = PathBuf::from(&item.exe_path);
    if !exe_path.is_file() {
        return Err(AppError::ExecutableNotFound(item.exe_path.clone()));
    }
    let working_dir = resolve_working_dir(item, &exe_path);
    let args = item.args.clone();
    let options = LaunchOptions {
        elevated: item.run_as_admin,
        minimized: item.start_minimized,
    };

    let baseline = processes::running_pids();
    // The UAC prompt blocks the calling thread until the user answers.
    let root_pid = tauri::async_runtime::spawn_blocking(move || {
        platform::launch(&exe_path, args.as_deref(), &working_dir, options)
    })
    .await
    .map_err(|error| launch_failed(item, error))??;

    let tracked = wait_for_process(&item.process_name, root_pid).await?;
    Ok(LaunchOutcome::AppLaunched(LaunchedApp {
        root_pid,
        tracked,
        baseline,
    }))
}

fn resolve_working_dir(item: &AppItem, exe_path: &Path) -> PathBuf {
    item.working_dir
        .as_deref()
        .map(PathBuf::from)
        .or_else(|| exe_path.parent().map(Path::to_path_buf))
        .unwrap_or_default()
}

/// A launcher may exit right after starting the real app, so any process it started counts.
async fn wait_for_process(process_name: &str, root_pid: Option<u32>) -> AppResult<HashSet<u32>> {
    let mut tracked: HashSet<u32> = root_pid.into_iter().collect();
    let attempts = PROCESS_DETECTION_TIMEOUT.as_millis() / PROCESS_DETECTION_INTERVAL.as_millis();
    for _ in 0..attempts {
        tokio::time::sleep(PROCESS_DETECTION_INTERVAL).await;
        let samples = processes::samples();
        tracked = expand_tracked(&tracked, &samples);
        if any_tracked_alive(&tracked, &samples) || has_process_named(&samples, process_name) {
            return Ok(tracked);
        }
    }
    Err(AppError::ProcessNotDetected(process_name.to_owned()))
}

fn has_process_named(samples: &[ProcessSample], process_name: &str) -> bool {
    let wanted = processes::normalize_process_name(process_name);
    samples
        .iter()
        .any(|sample| processes::normalize_process_name(&sample.name) == wanted)
}

async fn open_url(item: &UrlItem) -> AppResult<()> {
    let url = item.url.clone();
    tauri::async_runtime::spawn_blocking(move || platform::open_link(&url))
        .await
        .map_err(|error| AppError::LaunchFailed {
            name: item.name.clone(),
            reason: error.to_string(),
        })?
        .map_err(|error| match error {
            AppError::LaunchFailed { reason, .. } => AppError::LaunchFailed {
                name: item.name.clone(),
                reason,
            },
            other => other,
        })
}

#[derive(Debug, PartialEq, Eq)]
pub enum SimulatorStart {
    Started,
    AlreadyRunning,
}

/// The simulator is never tracked like an item: AutoStart starts it but never closes it.
pub async fn start_simulator(trigger: &Trigger) -> AppResult<SimulatorStart> {
    if processes::is_running(&trigger.process_name) {
        return Ok(SimulatorStart::AlreadyRunning);
    }
    let target = trigger
        .launch_target
        .clone()
        .ok_or_else(|| AppError::Validation(format!("{} has no launch target", trigger.label)))?;
    tauri::async_runtime::spawn_blocking(move || open_launch_target(&target))
        .await
        .map_err(|error| AppError::LaunchFailed {
            name: trigger.label.clone(),
            reason: error.to_string(),
        })??;
    Ok(SimulatorStart::Started)
}

fn open_launch_target(target: &str) -> AppResult<()> {
    if !target.to_lowercase().ends_with(".exe") {
        return platform::open_link(target);
    }
    let exe_path = PathBuf::from(target);
    if !exe_path.is_file() {
        return Err(AppError::ExecutableNotFound(target.to_owned()));
    }
    let working_dir = exe_path.parent().map(Path::to_path_buf).unwrap_or_default();
    platform::launch(&exe_path, None, &working_dir, LaunchOptions::default()).map(drop)
}

fn launch_failed(item: &AppItem, error: impl std::fmt::Display) -> AppError {
    AppError::LaunchFailed {
        name: item.name.clone(),
        reason: error.to_string(),
    }
}

impl LaunchOutcome {
    pub fn into_runtime(self, item_id: &str) -> ItemRuntime {
        let (status, launched_by_app, error) = match self {
            Self::Launched | Self::AppLaunched(_) => (ItemStatus::Running, true, None),
            Self::AlreadyRunning => (ItemStatus::Skipped, false, None),
            Self::Failed(error) => (ItemStatus::Error, false, Some((&error).into())),
        };
        ItemRuntime {
            item_id: item_id.to_owned(),
            status,
            launched_by_app,
            error,
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;
    use crate::process_tracker::{choose_process_name, LaunchTrace};

    fn trigger_for(process_name: &str, launch_target: &Path) -> Trigger {
        Trigger {
            process_name: process_name.into(),
            label: "Sim".into(),
            launch_target: Some(launch_target.display().to_string()),
        }
    }

    #[tokio::test]
    async fn running_simulator_is_not_started_again() {
        let running = processes::list_running()
            .into_iter()
            .next()
            .expect("some process is running");
        let trigger = trigger_for(&running.name, Path::new("/missing/Sim.exe"));

        assert_eq!(
            start_simulator(&trigger).await.expect("skipped"),
            SimulatorStart::AlreadyRunning
        );
    }

    #[tokio::test]
    async fn missing_simulator_executable_is_reported() {
        let trigger = trigger_for("NotRunningSim.exe", Path::new("/missing/Sim.exe"));

        assert!(matches!(
            start_simulator(&trigger).await,
            Err(AppError::ExecutableNotFound(_))
        ));
    }

    #[tokio::test]
    async fn learns_the_child_of_a_launcher_that_exits() {
        let dir = tempfile::tempdir().expect("temp dir");
        let launcher = dir.path().join("launcher.sh");
        std::fs::write(&launcher, "#!/bin/sh\nsleep 6 &\nsleep 1\n").expect("write script");
        std::fs::set_permissions(&launcher, std::fs::Permissions::from_mode(0o755))
            .expect("make executable");

        let baseline = processes::running_pids();
        let root_pid = platform::launch(&launcher, None, dir.path(), LaunchOptions::default())
            .expect("launch");
        let mut tracked = wait_for_process("launcher.sh", root_pid)
            .await
            .expect("confirmed");

        tokio::time::sleep(Duration::from_secs(2)).await;
        let samples = processes::samples();
        tracked = expand_tracked(&tracked, &samples);
        let trace = LaunchTrace {
            root_pid,
            tracked: &tracked,
            baseline: &baseline,
            install_dir: dir.path(),
        };

        assert_eq!(
            choose_process_name(&trace, &samples).as_deref(),
            Some("sleep")
        );
    }
}
