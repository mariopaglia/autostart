use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::error::{AppError, AppResult};
use crate::models::{
    AppCandidate, DropResolution, ExeInfo, MonitorSnapshot, ProcessInfo, Profile, SessionLog,
    Settings,
};
use crate::monitor::MonitorHandle;
use crate::platform::LaunchOptions;
use crate::state::AppState;
use crate::{
    app_discovery, exe_inspection, launch_args, platform, processes, system_autostart, tray,
};

#[tauri::command]
pub fn get_profiles(state: State<'_, AppState>) -> Vec<Profile> {
    state.profiles()
}

#[tauri::command]
pub fn save_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    profile: Profile,
) -> AppResult<Profile> {
    let saved = state.save_profile(profile)?;
    tray::refresh(&app);
    Ok(saved)
}

#[tauri::command]
pub fn delete_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
) -> AppResult<Settings> {
    let settings = state.delete_profile(&profile_id)?;
    tray::refresh(&app);
    Ok(settings)
}

#[tauri::command]
pub fn set_active_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
) -> AppResult<Settings> {
    let settings = state.set_active_profile(&profile_id)?;
    tray::refresh(&app);
    Ok(settings)
}

/// The startup entry can be removed outside AutoStart (e.g. Task Manager), so the system wins.
#[tauri::command]
pub fn get_settings(app: AppHandle, state: State<'_, AppState>) -> AppResult<Settings> {
    state.sync_start_with_windows(system_autostart::is_enabled(&app))
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> AppResult<Settings> {
    let saved = state.save_settings(settings)?;
    tray::refresh(&app);
    if let Err(error) = system_autostart::apply(&app, saved.start_with_windows) {
        state.sync_start_with_windows(system_autostart::is_enabled(&app))?;
        return Err(error);
    }
    Ok(saved)
}

#[tauri::command]
pub fn inspect_exe(path: PathBuf) -> AppResult<ExeInfo> {
    exe_inspection::inspect_exe(&path)
}

#[tauri::command]
pub fn find_missing_executables(paths: Vec<String>) -> Vec<String> {
    paths
        .into_iter()
        .filter(|path| !std::path::Path::new(path).is_file())
        .collect()
}

#[tauri::command]
pub fn is_elevated() -> bool {
    platform::is_elevated()
}

/// Returns only when the UAC prompt was refused; on success this instance exits.
#[tauri::command]
pub async fn relaunch_as_admin(app: AppHandle) -> AppResult<()> {
    let exe_path = std::env::current_exe()?;
    let working_dir = exe_path.parent().map(PathBuf::from).unwrap_or_default();
    let args = launch_args::wait_for_pid_args(std::process::id());

    let options = LaunchOptions {
        elevated: true,
        minimized: false,
    };
    tauri::async_runtime::spawn_blocking(move || {
        platform::launch(&exe_path, Some(&args), &working_dir, options)
    })
    .await??;
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub fn open_log_dir(app: AppHandle) -> AppResult<()> {
    let log_dir = app.path().app_log_dir()?;
    std::fs::create_dir_all(&log_dir)?;
    app.opener()
        .open_path(log_dir.to_string_lossy(), None::<&str>)
        .map_err(|error| AppError::Io(std::io::Error::other(error.to_string())))
}

#[tauri::command]
pub async fn list_running_processes() -> AppResult<Vec<ProcessInfo>> {
    Ok(tauri::async_runtime::spawn_blocking(processes::list_running).await?)
}

#[tauri::command]
pub async fn list_installed_apps() -> AppResult<Vec<AppCandidate>> {
    Ok(tauri::async_runtime::spawn_blocking(app_discovery::list_installed_apps).await?)
}

#[tauri::command]
pub async fn list_open_apps() -> AppResult<Vec<AppCandidate>> {
    Ok(tauri::async_runtime::spawn_blocking(app_discovery::list_open_apps).await?)
}

#[tauri::command]
pub async fn resolve_dropped_paths(paths: Vec<PathBuf>) -> AppResult<DropResolution> {
    Ok(
        tauri::async_runtime::spawn_blocking(move || app_discovery::resolve_dropped_paths(&paths))
            .await?,
    )
}

/// Returns the raw JSON so the frontend can validate it with Zod and report field-level errors.
#[tauri::command]
pub fn import_profile(path: PathBuf) -> AppResult<serde_json::Value> {
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

#[tauri::command]
pub fn export_profile(
    state: State<'_, AppState>,
    profile_id: String,
    path: PathBuf,
) -> AppResult<()> {
    let profile = state.profile(&profile_id)?;
    std::fs::write(path, serde_json::to_vec_pretty(&profile)?)?;
    Ok(())
}

#[tauri::command]
pub fn get_monitor_state(monitor: State<'_, MonitorHandle>) -> MonitorSnapshot {
    monitor.snapshot()
}

#[tauri::command]
pub fn get_session_log(monitor: State<'_, MonitorHandle>) -> Option<SessionLog> {
    monitor.session_log()
}

#[tauri::command]
pub async fn pause_monitor(monitor: State<'_, MonitorHandle>) -> AppResult<()> {
    monitor.pause().await;
    Ok(())
}

#[tauri::command]
pub async fn resume_monitor(monitor: State<'_, MonitorHandle>) -> AppResult<()> {
    monitor.resume().await;
    Ok(())
}

/// Starts in the background; progress arrives through the monitor events.
#[tauri::command]
pub async fn test_launch(monitor: State<'_, MonitorHandle>, profile_id: String) -> AppResult<()> {
    monitor.test_launch(profile_id).await
}

/// Without a previous test launch every app item is treated as launched by AutoStart;
/// the UI asks for confirmation before calling it in that case.
#[tauri::command]
pub async fn test_close(monitor: State<'_, MonitorHandle>, profile_id: String) -> AppResult<()> {
    monitor.test_close(profile_id).await
}
