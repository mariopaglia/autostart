use tauri::{AppHandle, Manager, Window, WindowEvent};

use crate::launch_args::LaunchArgs;
use crate::models::Settings;

const MAIN_WINDOW: &str = "main";

pub fn show_main(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        log::error!("main window not found");
        return;
    };
    let shown = window
        .show()
        .and_then(|()| window.unminimize())
        .and_then(|()| window.set_focus());
    if let Err(error) = shown {
        log::warn!("could not show the main window: {error}");
    }
}

/// Closing the window only hides it: AutoStart keeps monitoring from the tray.
pub fn hide_on_close(window: &Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        if let Err(error) = window.hide() {
            log::warn!("could not hide the main window: {error}");
        }
    }
}

pub fn should_show_on_startup(settings: &Settings, args: &LaunchArgs) -> bool {
    if !settings.onboarding_completed || args.previous_instance_pid.is_some() {
        return true;
    }
    !settings.start_minimized && !args.minimized
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(onboarding_completed: bool, start_minimized: bool) -> Settings {
        Settings {
            onboarding_completed,
            start_minimized,
            ..Settings::default()
        }
    }

    #[test]
    fn onboarding_always_shows_the_window() {
        let args = LaunchArgs {
            minimized: true,
            ..LaunchArgs::default()
        };
        assert!(should_show_on_startup(&settings(false, true), &args));
    }

    #[test]
    fn start_minimized_keeps_the_window_hidden() {
        assert!(!should_show_on_startup(
            &settings(true, true),
            &LaunchArgs::default()
        ));
    }

    #[test]
    fn minimized_flag_keeps_the_window_hidden() {
        let args = LaunchArgs {
            minimized: true,
            ..LaunchArgs::default()
        };
        assert!(!should_show_on_startup(&settings(true, false), &args));
    }

    #[test]
    fn regular_start_shows_the_window() {
        assert!(should_show_on_startup(
            &settings(true, false),
            &LaunchArgs::default()
        ));
    }

    #[test]
    fn elevated_relaunch_shows_the_window() {
        let args = LaunchArgs {
            previous_instance_pid: Some(1),
            ..LaunchArgs::default()
        };
        assert!(should_show_on_startup(&settings(true, true), &args));
    }
}
