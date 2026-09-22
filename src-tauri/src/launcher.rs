use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use tauri::{AppHandle, Runtime};
use tauri_plugin_opener::OpenerExt;

use crate::error::{AppError, AppResult};
use crate::models::{AppItem, ItemRuntime, ItemStatus, LaunchItem, UrlItem};
use crate::{platform, processes};

const PROCESS_DETECTION_TIMEOUT: Duration = Duration::from_secs(10);
const PROCESS_DETECTION_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub enum LaunchOutcome {
    Launched,
    AlreadyRunning,
    Failed(AppError),
}

pub async fn launch_item<R: Runtime>(app: &AppHandle<R>, item: &LaunchItem) -> LaunchOutcome {
    let result = match item {
        LaunchItem::App(app_item) => launch_app(app_item).await,
        LaunchItem::Url(url_item) => open_url(app, url_item).map(|()| LaunchOutcome::Launched),
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

    if item.run_as_admin {
        // The UAC prompt blocks the calling thread until the user answers.
        let exe = exe_path.clone();
        tauri::async_runtime::spawn_blocking(move || {
            platform::launch_elevated(&exe, args.as_deref(), &working_dir)
        })
        .await
        .map_err(|error| launch_failed(item, error))??;
    } else {
        spawn_detached(&exe_path, args.as_deref(), &working_dir)
            .map_err(|error| launch_failed(item, error))?;
    }

    wait_for_process(&item.process_name).await?;
    Ok(LaunchOutcome::Launched)
}

fn spawn_detached(exe_path: &Path, args: Option<&str>, working_dir: &Path) -> std::io::Result<()> {
    let mut command = Command::new(exe_path);
    command.current_dir(working_dir);
    platform::configure_launch(&mut command, args);
    command.spawn().map(drop)
}

fn resolve_working_dir(item: &AppItem, exe_path: &Path) -> PathBuf {
    item.working_dir
        .as_deref()
        .map(PathBuf::from)
        .or_else(|| exe_path.parent().map(Path::to_path_buf))
        .unwrap_or_default()
}

async fn wait_for_process(process_name: &str) -> AppResult<()> {
    let attempts = PROCESS_DETECTION_TIMEOUT.as_millis() / PROCESS_DETECTION_INTERVAL.as_millis();
    for _ in 0..attempts {
        tokio::time::sleep(PROCESS_DETECTION_INTERVAL).await;
        if processes::is_running(process_name) {
            return Ok(());
        }
    }
    Err(AppError::ProcessNotDetected(process_name.to_owned()))
}

fn open_url<R: Runtime>(app: &AppHandle<R>, item: &UrlItem) -> AppResult<()> {
    app.opener()
        .open_url(&item.url, None::<&str>)
        .map_err(|error| AppError::LaunchFailed {
            name: item.name.clone(),
            reason: error.to_string(),
        })
}

fn launch_failed(item: &AppItem, error: impl std::fmt::Display) -> AppError {
    AppError::LaunchFailed {
        name: item.name.clone(),
        reason: error.to_string(),
    }
}

impl LaunchOutcome {
    pub fn into_runtime(self, item_id: &str) -> ItemRuntime {
        let (status, launched_by_app, message) = match self {
            Self::Launched => (ItemStatus::Running, true, None),
            Self::AlreadyRunning => (ItemStatus::Skipped, false, None),
            Self::Failed(error) => (ItemStatus::Error, false, Some(error.to_string())),
        };
        ItemRuntime {
            item_id: item_id.to_owned(),
            status,
            launched_by_app,
            message,
        }
    }
}
