mod closer;
mod commands;
mod error;
mod exe_inspection;
mod launch_args;
mod launcher;
mod models;
mod monitor;
mod platform;
mod processes;
mod state;
mod storage;
mod system_autostart;
mod tray;
mod validation;
mod window;

use tauri::{App, Manager};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

use crate::launch_args::LaunchArgs;
use crate::monitor::MonitorHandle;
use crate::state::AppState;
use crate::storage::Storage;

const MAX_LOG_FILE_BYTES: u128 = 5 * 1024 * 1024;
const KEPT_LOG_FILES: usize = 5;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args = LaunchArgs::from_env();
    if let Some(pid) = args.previous_instance_pid {
        launch_args::wait_for_previous_instance(pid);
    }

    let result = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            window::show_main(app);
        }))
        .plugin(log_plugin())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![launch_args::MINIMIZED_FLAG]),
        ))
        .setup(move |app| setup(app, args))
        .on_window_event(window::hide_on_close)
        .invoke_handler(tauri::generate_handler![
            commands::get_profiles,
            commands::save_profile,
            commands::delete_profile,
            commands::set_active_profile,
            commands::get_settings,
            commands::save_settings,
            commands::inspect_exe,
            commands::find_missing_executables,
            commands::list_running_processes,
            commands::is_elevated,
            commands::relaunch_as_admin,
            commands::open_log_dir,
            commands::import_profile,
            commands::export_profile,
            commands::get_monitor_state,
            commands::get_session_log,
            commands::pause_monitor,
            commands::resume_monitor,
            commands::test_launch,
            commands::test_close,
        ])
        .run(tauri::generate_context!());

    if let Err(error) = result {
        log::error!("AutoStart terminated with an error: {error}");
        std::process::exit(1);
    }
}

fn setup(app: &mut App, args: LaunchArgs) -> Result<(), Box<dyn std::error::Error>> {
    let storage = Storage::new(app.path().app_data_dir()?)?;
    let state = AppState::load(storage)?;
    let settings = state.sync_start_with_windows(system_autostart::is_enabled(app.handle()))?;
    app.manage(state);
    app.manage(MonitorHandle::spawn(app.handle().clone()));
    tray::create(app.handle())?;

    if window::should_show_on_startup(&settings, &args) {
        window::show_main(app.handle());
    }
    Ok(())
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
