use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::error::{AppError, AppResult};

pub fn is_enabled(app: &AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or_else(|error| {
        log::warn!("could not read the startup entry: {error}");
        false
    })
}

pub fn apply(app: &AppHandle, enabled: bool) -> AppResult<()> {
    if is_enabled(app) == enabled {
        return Ok(());
    }
    let autolaunch = app.autolaunch();
    let result = if enabled {
        autolaunch.enable()
    } else {
        autolaunch.disable()
    };
    result.map_err(|error| AppError::Autostart(error.to_string()))
}
