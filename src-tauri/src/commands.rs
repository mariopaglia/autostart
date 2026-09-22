use std::path::PathBuf;

use tauri::State;

use crate::error::AppResult;
use crate::exe_inspection;
use crate::models::{ExeInfo, MonitorSnapshot, ProcessInfo, Profile, SessionLog, Settings};
use crate::monitor::MonitorHandle;
use crate::state::AppState;
use crate::{platform, processes};

#[tauri::command]
pub fn get_profiles(state: State<'_, AppState>) -> Vec<Profile> {
    state.profiles()
}

#[tauri::command]
pub fn save_profile(state: State<'_, AppState>, profile: Profile) -> AppResult<Profile> {
    state.save_profile(profile)
}

#[tauri::command]
pub fn delete_profile(state: State<'_, AppState>, profile_id: String) -> AppResult<Settings> {
    state.delete_profile(&profile_id)
}

#[tauri::command]
pub fn set_active_profile(state: State<'_, AppState>, profile_id: String) -> AppResult<Settings> {
    state.set_active_profile(&profile_id)
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings()
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> AppResult<Settings> {
    state.save_settings(settings)
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

#[tauri::command]
pub async fn list_running_processes() -> AppResult<Vec<ProcessInfo>> {
    Ok(tauri::async_runtime::spawn_blocking(processes::list_running).await?)
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
