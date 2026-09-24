use tauri::image::Image;
use tauri::menu::{
    CheckMenuItem, IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::models::{Language, MonitorSnapshot, MonitorState, Profile};
use crate::monitor::{MonitorHandle, PROFILE_CHANGED_EVENT};
use crate::state::AppState;
use crate::window;

const TRAY_ID: &str = "main";
const PROFILE_ID_PREFIX: &str = "profile:";
const FLIGHT_ID_PREFIX: &str = "flight:";
const CANCEL_START_ID: &str = "cancel-start";
const TOGGLE_PAUSE_ID: &str = "toggle-pause";
const CLOSE_NOW_ID: &str = "close-now";
const KEEP_APPS_ID: &str = "keep-apps";
const OPEN_ID: &str = "open";
const QUIT_ID: &str = "quit";

const IDLE_ICON: &[u8] = include_bytes!("../icons/tray-idle.png");
const RUNNING_ICON: &[u8] = include_bytes!("../icons/tray-running.png");
const PAUSED_ICON: &[u8] = include_bytes!("../icons/tray-paused.png");

// The tray menu exists before the webview loads, so its few strings live here instead of i18n.
struct TrayLabels {
    start_flight: &'static str,
    cancel_start: &'static str,
    profiles: &'static str,
    pause: &'static str,
    resume: &'static str,
    close_now: &'static str,
    keep_apps: &'static str,
    open: &'static str,
    quit: &'static str,
    watching_one: &'static str,
    watching_many: &'static str,
    idle: &'static str,
    sim_starting: &'static str,
    sim_running: &'static str,
    close_pending: &'static str,
    closing: &'static str,
    paused: &'static str,
}

const PT_BR_LABELS: TrayLabels = TrayLabels {
    start_flight: "Iniciar voo",
    cancel_start: "Cancelar início do simulador",
    profiles: "Perfis",
    pause: "Pausar monitoramento",
    resume: "Retomar monitoramento",
    close_now: "Fechar apps agora",
    keep_apps: "Manter apps abertos",
    open: "Abrir AutoStart",
    quit: "Sair",
    watching_one: "1 perfil monitorado",
    watching_many: "{count} perfis monitorados",
    idle: "Aguardando simulador",
    sim_starting: "Iniciando simulador",
    sim_running: "Simulador em execução",
    close_pending: "Fechando apps em breve",
    closing: "Fechando apps",
    paused: "Pausado",
};

const EN_LABELS: TrayLabels = TrayLabels {
    start_flight: "Start flight",
    cancel_start: "Cancel simulator start",
    profiles: "Profiles",
    pause: "Pause monitoring",
    resume: "Resume monitoring",
    close_now: "Close apps now",
    keep_apps: "Keep apps open",
    open: "Open AutoStart",
    quit: "Quit",
    watching_one: "Watching 1 profile",
    watching_many: "Watching {count} profiles",
    idle: "Waiting for simulator",
    sim_starting: "Starting simulator",
    sim_running: "Simulator running",
    close_pending: "Closing apps soon",
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
    snapshot: MonitorSnapshot,
    profiles: Vec<Profile>,
}

impl TrayView {
    fn current(app: &AppHandle) -> Self {
        let app_state = app.state::<AppState>();
        let snapshot = app
            .try_state::<MonitorHandle>()
            .map(|monitor| monitor.snapshot())
            .unwrap_or_default();
        Self {
            labels: labels(app_state.settings().language),
            snapshot,
            profiles: app_state.profiles(),
        }
    }

    fn icon(&self) -> tauri::Result<Image<'static>> {
        let bytes = match self.snapshot.state {
            MonitorState::Idle => IDLE_ICON,
            MonitorState::SimStarting
            | MonitorState::SimRunning
            | MonitorState::ClosePending
            | MonitorState::Closing => RUNNING_ICON,
            MonitorState::Paused => PAUSED_ICON,
        };
        Image::from_bytes(bytes)
    }

    fn tooltip(&self) -> String {
        let session_profile = self
            .profiles
            .iter()
            .find(|profile| Some(&profile.id) == self.snapshot.session_profile_id.as_ref())
            .filter(|_| has_session(self.snapshot.state) && !self.snapshot.is_test_session);
        let subject = match session_profile {
            Some(profile) => profile.name.clone(),
            None => watching_text(
                self.labels,
                self.profiles.iter().filter(|p| p.enabled).count(),
            ),
        };
        tooltip_text(self.labels, &subject, self.snapshot.state)
    }

    fn menu(&self, app: &AppHandle) -> tauri::Result<Menu<Wry>> {
        let profile_items = self
            .profiles
            .iter()
            .map(|profile| {
                CheckMenuItem::with_id(
                    app,
                    format!("{PROFILE_ID_PREFIX}{}", profile.id),
                    &profile.name,
                    true,
                    profile.enabled,
                    None::<&str>,
                )
            })
            .collect::<tauri::Result<Vec<_>>>()?;
        let profile_refs: Vec<&dyn IsMenuItem<Wry>> = profile_items
            .iter()
            .map(|item| item as &dyn IsMenuItem<Wry>)
            .collect();
        let profiles = Submenu::with_items(app, self.labels.profiles, true, &profile_refs)?;

        let pause_label = if self.snapshot.state == MonitorState::Paused {
            self.labels.resume
        } else {
            self.labels.pause
        };
        let toggle_pause =
            MenuItem::with_id(app, TOGGLE_PAUSE_ID, pause_label, true, None::<&str>)?;
        let open = MenuItem::with_id(app, OPEN_ID, self.labels.open, true, None::<&str>)?;
        let quit = MenuItem::with_id(app, QUIT_ID, self.labels.quit, true, None::<&str>)?;
        let separator = PredefinedMenuItem::separator(app)?;
        let close_now =
            MenuItem::with_id(app, CLOSE_NOW_ID, self.labels.close_now, true, None::<&str>)?;
        let keep_apps =
            MenuItem::with_id(app, KEEP_APPS_ID, self.labels.keep_apps, true, None::<&str>)?;

        let flight_items = flight_starts(&self.profiles)
            .into_iter()
            .map(|start| MenuItem::with_id(app, start.menu_id, start.label, true, None::<&str>))
            .collect::<tauri::Result<Vec<_>>>()?;
        let flight_refs: Vec<&dyn IsMenuItem<Wry>> = flight_items
            .iter()
            .map(|item| item as &dyn IsMenuItem<Wry>)
            .collect();
        let start_flight = Submenu::with_items(
            app,
            self.labels.start_flight,
            self.snapshot.state == MonitorState::Idle,
            &flight_refs,
        )?;
        let cancel_start = MenuItem::with_id(
            app,
            CANCEL_START_ID,
            self.labels.cancel_start,
            true,
            None::<&str>,
        )?;

        let mut entries: Vec<&dyn IsMenuItem<Wry>> = Vec::new();
        if !flight_items.is_empty() {
            entries.push(&start_flight);
        }
        entries.extend([&profiles as &dyn IsMenuItem<Wry>, &separator]);
        if self.snapshot.state == MonitorState::SimStarting {
            entries.extend([&cancel_start as &dyn IsMenuItem<Wry>, &separator]);
        }
        if self.snapshot.state == MonitorState::ClosePending {
            entries.extend([&close_now as &dyn IsMenuItem<Wry>, &keep_apps, &separator]);
        }
        entries.extend([
            &toggle_pause as &dyn IsMenuItem<Wry>,
            &open,
            &separator,
            &quit,
        ]);
        Menu::with_items(app, &entries)
    }
}

fn has_session(state: MonitorState) -> bool {
    matches!(
        state,
        MonitorState::SimStarting
            | MonitorState::SimRunning
            | MonitorState::ClosePending
            | MonitorState::Closing
    )
}

#[derive(Debug, PartialEq, Eq)]
struct FlightStart {
    menu_id: String,
    label: String,
}

/// One entry per simulator that AutoStart knows how to start, in sidebar order.
fn flight_starts(profiles: &[Profile]) -> Vec<FlightStart> {
    profiles
        .iter()
        .flat_map(|profile| {
            profile
                .triggers
                .iter()
                .filter(|trigger| trigger.launch_target.is_some())
                .map(move |trigger| FlightStart {
                    menu_id: format!("{FLIGHT_ID_PREFIX}{}:{}", profile.id, trigger.process_name),
                    label: format!("{} — {}", profile.name, trigger.label),
                })
        })
        .collect()
}

fn watching_text(labels: &TrayLabels, count: usize) -> String {
    if count == 1 {
        return labels.watching_one.to_owned();
    }
    labels.watching_many.replace("{count}", &count.to_string())
}

fn tooltip_text(labels: &TrayLabels, subject: &str, state: MonitorState) -> String {
    let state_label = match state {
        MonitorState::Idle => labels.idle,
        MonitorState::SimStarting => labels.sim_starting,
        MonitorState::SimRunning => labels.sim_running,
        MonitorState::ClosePending => labels.close_pending,
        MonitorState::Closing => labels.closing,
        MonitorState::Paused => labels.paused,
    };
    format!("AutoStart · {subject} · {state_label}")
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
        CLOSE_NOW_ID => end_close_delay(app, true),
        KEEP_APPS_ID => end_close_delay(app, false),
        CANCEL_START_ID => cancel_start(app),
        id => {
            if let Some(profile_id) = id.strip_prefix(PROFILE_ID_PREFIX) {
                toggle_profile(app, profile_id);
            } else if let Some((profile_id, process_name)) = id
                .strip_prefix(FLIGHT_ID_PREFIX)
                .and_then(|rest| rest.split_once(':'))
            {
                start_flight(app, profile_id.to_owned(), process_name.to_owned());
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

fn start_flight(app: &AppHandle, profile_id: String, trigger_process_name: String) {
    let monitor = app.state::<MonitorHandle>().inner().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = monitor.start_flight(profile_id, trigger_process_name).await {
            log::error!("could not start the flight from the tray: {error}");
        }
    });
}

fn cancel_start(app: &AppHandle) {
    let monitor = app.state::<MonitorHandle>().inner().clone();
    tauri::async_runtime::spawn(async move {
        monitor.cancel_flight_start().await;
    });
}

fn end_close_delay(app: &AppHandle, close_now: bool) {
    let monitor = app.state::<MonitorHandle>().inner().clone();
    tauri::async_runtime::spawn(async move {
        if close_now {
            monitor.close_now().await;
        } else {
            monitor.keep_apps_open().await;
        }
    });
}

fn toggle_profile(app: &AppHandle, profile_id: &str) {
    let state = app.state::<AppState>();
    let Ok(profile) = state.profile(profile_id) else {
        return;
    };
    match state.set_profile_enabled(profile_id, !profile.enabled) {
        Ok(update) => {
            let changed = update.profiles.iter().filter(|updated| {
                updated.id == profile_id || update.disabled_profile_ids.contains(&updated.id)
            });
            for updated in changed {
                if let Err(error) = app.emit(PROFILE_CHANGED_EVENT, updated) {
                    log::warn!("failed to emit {PROFILE_CHANGED_EVENT}: {error}");
                }
            }
        }
        Err(error) => log::error!("could not toggle the profile from the tray: {error}"),
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
    fn watching_text_counts_profiles_in_both_languages() {
        assert_eq!(watching_text(&PT_BR_LABELS, 1), "1 perfil monitorado");
        assert_eq!(watching_text(&PT_BR_LABELS, 2), "2 perfis monitorados");
        assert_eq!(watching_text(&EN_LABELS, 1), "Watching 1 profile");
        assert_eq!(watching_text(&EN_LABELS, 0), "Watching 0 profiles");
    }

    #[test]
    fn tooltip_shows_close_pending() {
        assert_eq!(
            tooltip_text(&EN_LABELS, "MSFS 2024", MonitorState::ClosePending),
            "AutoStart · MSFS 2024 · Closing apps soon"
        );
    }

    #[test]
    fn tooltip_shows_the_simulator_starting() {
        assert_eq!(
            tooltip_text(&EN_LABELS, "MSFS", MonitorState::SimStarting),
            "AutoStart · MSFS · Starting simulator"
        );
        assert!(has_session(MonitorState::SimStarting));
    }

    #[test]
    fn flight_starts_list_only_simulators_with_a_start_target() {
        let mut msfs = crate::storage::example_profile();
        msfs.name = "MSFS".into();
        msfs.triggers[0].launch_target = Some("steam://rungameid/2537590".into());
        let mut xplane = crate::storage::example_profile();
        xplane.triggers[0].process_name = "X-Plane.exe".into();

        let starts = flight_starts(&[msfs.clone(), xplane]);

        assert_eq!(
            starts,
            [FlightStart {
                menu_id: format!("flight:{}:FlightSimulator2024.exe", msfs.id),
                label: "MSFS — MSFS 2024".into(),
            }]
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
