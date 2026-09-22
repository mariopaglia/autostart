mod closer;
mod commands;
mod error;
mod exe_inspection;
mod launcher;
mod models;
mod platform;
mod processes;
mod state;
mod storage;
mod validation;

use tauri::Manager;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

use crate::state::AppState;
use crate::storage::Storage;

const MAX_LOG_FILE_BYTES: u128 = 5 * 1024 * 1024;
const KEPT_LOG_FILES: usize = 5;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        .plugin(log_plugin())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let storage = Storage::new(app.path().app_data_dir()?)?;
            app.manage(AppState::load(storage)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_profiles,
            commands::save_profile,
            commands::delete_profile,
            commands::set_active_profile,
            commands::get_settings,
            commands::save_settings,
            commands::inspect_exe,
            commands::list_running_processes,
            commands::is_elevated,
            commands::import_profile,
            commands::export_profile,
            commands::test_launch,
            commands::test_close,
        ])
        .run(tauri::generate_context!());

    if let Err(error) = result {
        log::error!("AutoStart terminated with an error: {error}");
        std::process::exit(1);
    }
}

fn log_plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri_plugin_log::Builder::new()
        .targets([
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::LogDir {
                file_name: Some("autostart".into()),
            }),
        ])
        .rotation_strategy(RotationStrategy::KeepSome(KEPT_LOG_FILES))
        .max_file_size(MAX_LOG_FILE_BYTES)
        .level(log::LevelFilter::Info)
        .build()
}
