use tauri::image::Image;
use tauri::menu::{
    CheckMenuItem, IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::models::{Language, MonitorState, Profile, Settings};
use crate::monitor::MonitorHandle;
use crate::state::AppState;
use crate::window;

pub const SETTINGS_CHANGED_EVENT: &str = "settings://changed";

const TRAY_ID: &str = "main";
const PROFILE_ID_PREFIX: &str = "profile:";
const TOGGLE_PAUSE_ID: &str = "toggle-pause";
const OPEN_ID: &str = "open";
const QUIT_ID: &str = "quit";

const IDLE_ICON: &[u8] = include_bytes!("../icons/tray-idle.png");
const RUNNING_ICON: &[u8] = include_bytes!("../icons/tray-running.png");
const PAUSED_ICON: &[u8] = include_bytes!("../icons/tray-paused.png");

// The tray menu exists before the webview loads, so its few strings live here instead of i18n.
struct TrayLabels {
    profiles: &'static str,
    pause: &'static str,
    resume: &'static str,
    open: &'static str,
    quit: &'static str,
    no_profile: &'static str,
    idle: &'static str,
    sim_running: &'static str,
    closing: &'static str,
    paused: &'static str,
}

const PT_BR_LABELS: TrayLabels = TrayLabels {
    profiles: "Perfis",
    pause: "Pausar monitoramento",
    resume: "Retomar monitoramento",
    open: "Abrir AutoStart",
    quit: "Sair",
    no_profile: "Nenhum perfil ativo",
    idle: "Aguardando simulador",
    sim_running: "Simulador em execução",
    closing: "Fechando apps",
    paused: "Pausado",
};

const EN_LABELS: TrayLabels = TrayLabels {
    profiles: "Profiles",
    pause: "Pause monitoring",
    resume: "Resume monitoring",
    open: "Open AutoStart",
    quit: "Quit",
    no_profile: "No active profile",
    idle: "Waiting for simulator",
    sim_running: "Simulator running",
    closing: "Closing apps",
    paused: "Paused",
};

fn labels(language: Language) -> &'static TrayLabels {
    match language {
        Language::PtBr => &PT_BR_LABELS,
        Language::En => &EN_LABELS,
    }
}

struct TrayView {
    labels: &'static TrayLabels,
    state: MonitorState,
    profiles: Vec<Profile>,
    settings: Settings,
}

impl TrayView {
    fn current(app: &AppHandle) -> Self {
        let app_state = app.state::<AppState>();
        let settings = app_state.settings();
        let state = app
            .try_state::<MonitorHandle>()
            .map(|monitor| monitor.snapshot().state)
            .unwrap_or_default();
        Self {
            labels: labels(settings.language),
            state,
            profiles: app_state.profiles(),
            settings,
        }
    }

    fn icon(&self) -> tauri::Result<Image<'static>> {
        let bytes = match self.state {
            MonitorState::Idle => IDLE_ICON,
            MonitorState::SimRunning | MonitorState::Closing => RUNNING_ICON,
            MonitorState::Paused => PAUSED_ICON,
        };
        Image::from_bytes(bytes)
    }

    fn tooltip(&self) -> String {
        let profile_name = self
            .profiles
            .iter()
            .find(|profile| Some(&profile.id) == self.settings.active_profile_id.as_ref())
            .map_or(self.labels.no_profile, |profile| profile.name.as_str());
        tooltip_text(self.labels, profile_name, self.state)
    }

    fn menu(&self, app: &AppHandle) -> tauri::Result<Menu<Wry>> {
        let profile_items = self
            .profiles
            .iter()
            .map(|profile| {
                let is_active = Some(&profile.id) == self.settings.active_profile_id.as_ref();
                CheckMenuItem::with_id(
                    app,
                    format!("{PROFILE_ID_PREFIX}{}", profile.id),
                    &profile.name,
                    true,
                    is_active,
                    None::<&str>,
                )
            })
            .collect::<tauri::Result<Vec<_>>>()?;
        let profile_refs: Vec<&dyn IsMenuItem<Wry>> = profile_items
            .iter()
            .map(|item| item as &dyn IsMenuItem<Wry>)
            .collect();
        let profiles = Submenu::with_items(app, self.labels.profiles, true, &profile_refs)?;

        let pause_label = if self.state == MonitorState::Paused {
            self.labels.resume
        } else {
            self.labels.pause
        };
        let toggle_pause =
            MenuItem::with_id(app, TOGGLE_PAUSE_ID, pause_label, true, None::<&str>)?;
        let open = MenuItem::with_id(app, OPEN_ID, self.labels.open, true, None::<&str>)?;
        let quit = MenuItem::with_id(app, QUIT_ID, self.labels.quit, true, None::<&str>)?;

        Menu::with_items(
            app,
            &[
                &profiles,
                &PredefinedMenuItem::separator(app)?,
                &toggle_pause,
                &open,
                &PredefinedMenuItem::separator(app)?,
                &quit,
            ],
        )
    }
}

fn tooltip_text(labels: &TrayLabels, profile_name: &str, state: MonitorState) -> String {
    let state_label = match state {
        MonitorState::Idle => labels.idle,
        MonitorState::SimRunning => labels.sim_running,
        MonitorState::Closing => labels.closing,
        MonitorState::Paused => labels.paused,
    };
    format!("AutoStart · {profile_name} · {state_label}")
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let view = TrayView::current(app);
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(view.icon()?)
        .tooltip(view.tooltip())
        .menu(&view.menu(app)?)
        .show_menu_on_left_click(false)
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                window::show_main(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

/// Rebuilding the whole menu is simpler than patching items and it only has a handful of entries.
pub fn refresh(app: &AppHandle) {
    if let Err(error) = try_refresh(app) {
        log::warn!("could not refresh the tray: {error}");
    }
}

fn try_refresh(app: &AppHandle) -> tauri::Result<()> {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return Ok(());
    };
    let view = TrayView::current(app);
    tray.set_icon(Some(view.icon()?))?;
    tray.set_tooltip(Some(view.tooltip()))?;
    tray.set_menu(Some(view.menu(app)?))
}

fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        OPEN_ID => window::show_main(app),
        QUIT_ID => app.exit(0),
        TOGGLE_PAUSE_ID => toggle_pause(app),
        id => {
            if let Some(profile_id) = id.strip_prefix(PROFILE_ID_PREFIX) {
                activate_profile(app, profile_id);
            }
        }
    }
}

fn toggle_pause(app: &AppHandle) {
    let monitor = app.state::<MonitorHandle>().inner().clone();
    tauri::async_runtime::spawn(async move {
        if monitor.snapshot().state == MonitorState::Paused {
            monitor.resume().await;
        } else {
            monitor.pause().await;
        }
    });
}

fn activate_profile(app: &AppHandle, profile_id: &str) {
    match app.state::<AppState>().set_active_profile(profile_id) {
        Ok(settings) => {
            if let Err(error) = app.emit(SETTINGS_CHANGED_EVENT, settings) {
                log::warn!("failed to emit {SETTINGS_CHANGED_EVENT}: {error}");
            }
        }
        Err(error) => log::error!("could not activate profile from the tray: {error}"),
    }
    refresh(app);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltip_names_profile_and_state() {
        assert_eq!(
            tooltip_text(&PT_BR_LABELS, "MSFS 2024", MonitorState::SimRunning),
            "AutoStart · MSFS 2024 · Simulador em execução"
        );
    }

    #[test]
    fn tooltip_follows_language() {
        assert_eq!(
            tooltip_text(labels(Language::En), "X-Plane 12", MonitorState::Paused),
            "AutoStart · X-Plane 12 · Paused"
        );
    }
}
