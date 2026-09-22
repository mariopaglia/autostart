use std::path::PathBuf;
use std::time::Duration;

use tauri::{AppHandle, State};

use crate::closer::{self, CloseTarget};
use crate::error::AppResult;
use crate::exe_inspection;
use crate::launcher;
use crate::models::{ExeInfo, ItemRuntime, LaunchItem, ProcessInfo, Profile, Settings};
use crate::state::{AppState, TestLaunch};
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
pub async fn test_launch(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
) -> AppResult<Vec<ItemRuntime>> {
    let profile = state.profile(&profile_id)?;
    let mut results = Vec::new();

    for item in profile.items.iter().filter(|item| item.is_enabled()) {
        tokio::time::sleep(Duration::from_millis(u64::from(item.delay_ms()))).await;
        let runtime = launcher::launch_item(&app, item)
            .await
            .into_runtime(item.id());
        log::info!("test launch: {} -> {:?}", item.name(), runtime.status);
        results.push(runtime);
    }

    state.record_test_launch(TestLaunch {
        profile_id,
        launched_item_ids: results
            .iter()
            .filter(|runtime| runtime.launched_by_app)
            .map(|runtime| runtime.item_id.clone())
            .collect(),
    });
    Ok(results)
}

/// Without a previous test launch every app item is treated as launched by AutoStart;
/// the UI asks for confirmation before calling it in that case.
#[tauri::command]
pub async fn test_close(
    state: State<'_, AppState>,
    profile_id: String,
) -> AppResult<Vec<ItemRuntime>> {
    let profile = state.profile(&profile_id)?;
    let settings = state.settings();
    let launched_ids = state
        .take_test_launch(&profile_id)
        .filter(|_| settings.close_only_if_launched_by_app)
        .map(|test_launch| test_launch.launched_item_ids);

    let targets = profile
        .items
        .iter()
        .filter_map(|item| match item {
            LaunchItem::App(app_item) => Some(app_item),
            LaunchItem::Url(_) => None,
        })
        .filter(|app_item| {
            launched_ids
                .as_ref()
                .is_none_or(|ids| ids.contains(&app_item.id))
        })
        .map(CloseTarget::from_app_item)
        .collect();

    let timeout = Duration::from_millis(u64::from(settings.graceful_timeout_ms));
    let results = closer::close_all(targets, timeout)
        .await
        .into_iter()
        .map(|(item_id, outcome)| {
            log::info!("test close: {item_id} -> {outcome:?}");
            outcome.into_runtime(&item_id)
        })
        .collect();
    Ok(results)
}
