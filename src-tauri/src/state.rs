use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::error::{AppError, AppResult};
use crate::models::{LaunchItem, ProcessNameMode, Profile, ProfilesUpdate, SessionLog, Settings};
use crate::session_history;
use crate::storage::Storage;
use crate::trigger_conflicts::{disable, profiles_to_disable, resolve_conflicts, shares_trigger};
use crate::validation::{validate_profile, validate_settings};

struct AppData {
    profiles: Vec<Profile>,
    settings: Settings,
    session_history: Vec<SessionLog>,
}

pub struct AppState {
    storage: Storage,
    data: Mutex<AppData>,
}

impl AppState {
    pub fn load(storage: Storage) -> AppResult<Self> {
        let mut profiles = storage.load_profiles()?;
        let settings = storage.load_settings()?;
        let legacy_active_id = storage.legacy_active_profile_id();
        let disabled = resolve_conflicts(&mut profiles, legacy_active_id.as_deref());
        if !disabled.is_empty() {
            log::info!("disabled {} profile(s) sharing a trigger", disabled.len());
        }

        storage.save_profiles(&profiles)?;
        storage.save_settings(&settings)?;
        let session_history = storage.load_session_history().unwrap_or_else(|error| {
            log::warn!("could not read the session history: {error}");
            Vec::new()
        });

        Ok(Self {
            storage,
            data: Mutex::new(AppData {
                profiles,
                settings,
                session_history,
            }),
        })
    }

    pub fn profiles(&self) -> Vec<Profile> {
        self.data().profiles.clone()
    }

    pub fn profile(&self, id: &str) -> AppResult<Profile> {
        self.data()
            .profiles
            .iter()
            .find(|profile| profile.id == id)
            .cloned()
            .ok_or_else(|| AppError::ProfileNotFound(id.to_owned()))
    }

    /// A new profile never switches off an existing one; saving an existing enabled profile
    /// disables the other enabled profiles that watch one of its triggers.
    pub fn save_profile(&self, mut profile: Profile) -> AppResult<ProfilesUpdate> {
        validate_profile(&profile)?;
        let profile_id = profile.id.clone();
        let mut data = self.data();

        match data
            .profiles
            .iter_mut()
            .find(|existing| existing.id == profile.id)
        {
            Some(existing) => *existing = profile,
            None => {
                profile.enabled &= !data
                    .profiles
                    .iter()
                    .any(|existing| existing.enabled && shares_trigger(existing, &profile));
                data.profiles.push(profile);
            }
        }
        self.save_with_winner(&mut data, &profile_id)
    }

    pub fn set_profile_enabled(&self, id: &str, enabled: bool) -> AppResult<ProfilesUpdate> {
        let mut data = self.data();
        let profile = data
            .profiles
            .iter_mut()
            .find(|profile| profile.id == id)
            .ok_or_else(|| AppError::ProfileNotFound(id.to_owned()))?;
        profile.enabled = enabled;
        self.save_with_winner(&mut data, id)
    }

    fn save_with_winner(&self, data: &mut AppData, winner_id: &str) -> AppResult<ProfilesUpdate> {
        let disabled_profile_ids = profiles_to_disable(&data.profiles, winner_id);
        disable(&mut data.profiles, &disabled_profile_ids);
        self.storage.save_profiles(&data.profiles)?;
        Ok(ProfilesUpdate {
            profiles: data.profiles.clone(),
            disabled_profile_ids,
        })
    }

    /// Returns the updated profile, or `None` when the item was removed, switched to manual
    /// mode or already had that name while the app was being observed.
    pub fn learn_process_name(
        &self,
        profile_id: &str,
        item_id: &str,
        process_name: &str,
    ) -> AppResult<Option<Profile>> {
        let mut data = self.data();
        let Some(profile) = data
            .profiles
            .iter_mut()
            .find(|profile| profile.id == profile_id)
        else {
            return Ok(None);
        };
        let Some(item) = profile.items.iter_mut().find_map(|item| match item {
            LaunchItem::App(app) if app.id == item_id => Some(app),
            _ => None,
        }) else {
            return Ok(None);
        };
        if item.process_name_mode != ProcessNameMode::Auto || item.process_name == process_name {
            return Ok(None);
        }

        item.process_name = process_name.to_owned();
        let updated = profile.clone();
        self.storage.save_profiles(&data.profiles)?;
        Ok(Some(updated))
    }

    pub fn delete_profile(&self, id: &str) -> AppResult<Vec<Profile>> {
        let mut data = self.data();
        if !data.profiles.iter().any(|profile| profile.id == id) {
            return Err(AppError::ProfileNotFound(id.to_owned()));
        }
        if data.profiles.len() == 1 {
            return Err(AppError::LastProfile);
        }

        data.profiles.retain(|profile| profile.id != id);
        self.storage.save_profiles(&data.profiles)?;
        Ok(data.profiles.clone())
    }

    pub fn settings(&self) -> Settings {
        self.data().settings.clone()
    }

    pub fn save_settings(&self, settings: Settings) -> AppResult<Settings> {
        let mut data = self.data();
        validate_settings(&settings)?;
        self.storage.save_settings(&settings)?;
        data.settings = settings.clone();
        Ok(settings)
    }

    pub fn sync_start_with_windows(&self, registered: bool) -> AppResult<Settings> {
        let settings = self.settings();
        if settings.start_with_windows == registered {
            return Ok(settings);
        }
        self.save_settings(Settings {
            start_with_windows: registered,
            ..settings
        })
    }

    pub fn session_history(&self) -> Vec<SessionLog> {
        self.data().session_history.clone()
    }

    pub fn record_session(&self, log: SessionLog) {
        let mut data = self.data();
        session_history::record(&mut data.session_history, log);
        if let Err(error) = self.storage.save_session_history(&data.session_history) {
            log::error!("could not save the session history: {error}");
        }
    }

    pub fn clear_session_history(&self) -> AppResult<()> {
        let mut data = self.data();
        self.storage.save_session_history(&[])?;
        data.session_history.clear();
        Ok(())
    }

    fn data(&self) -> MutexGuard<'_, AppData> {
        lock(&self.data)
    }
}

// A panic while holding the lock cannot leave AppData half-written (every mutation is a single
// assignment after validation), so recovering the guard is safe and keeps the app responsive.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Trigger;
    use crate::storage::example_profile;

    fn state() -> (tempfile::TempDir, AppState) {
        let dir = tempfile::tempdir().expect("temp dir");
        let storage = Storage::new(dir.path().to_path_buf()).expect("storage");
        (dir, AppState::load(storage).expect("state"))
    }

    fn xplane_profile() -> Profile {
        Profile {
            name: "X-Plane".into(),
            triggers: vec![Trigger {
                process_name: "X-Plane.exe".into(),
                label: "X-Plane 12".into(),
                launch_target: None,
            }],
            ..example_profile()
        }
    }

    fn enabled_names(update: &ProfilesUpdate) -> Vec<&str> {
        update
            .profiles
            .iter()
            .filter(|profile| profile.enabled)
            .map(|profile| profile.name.as_str())
            .collect()
    }

    #[test]
    fn first_load_enables_example_profile() {
        let (_dir, state) = state();

        assert!(state.profiles()[0].enabled);
    }

    #[test]
    fn new_profile_for_a_watched_simulator_is_created_disabled() {
        let (_dir, state) = state();
        let copy = Profile {
            name: "MSFS copy".into(),
            ..example_profile()
        };

        let update = state.save_profile(copy).expect("save");

        assert_eq!(enabled_names(&update), ["MSFS 2024"]);
        assert!(update.disabled_profile_ids.is_empty());
    }

    #[test]
    fn new_profile_for_another_simulator_stays_enabled() {
        let (_dir, state) = state();

        let update = state.save_profile(xplane_profile()).expect("save");

        assert_eq!(enabled_names(&update), ["MSFS 2024", "X-Plane"]);
    }

    #[test]
    fn enabling_a_profile_disables_the_one_sharing_its_trigger() {
        let (_dir, state) = state();
        let original = state.profiles()[0].clone();
        let offline = Profile {
            name: "MSFS Offline".into(),
            ..example_profile()
        };
        let offline_id = offline.id.clone();
        state.save_profile(offline).expect("save");

        let update = state
            .set_profile_enabled(&offline_id, true)
            .expect("enable");

        assert_eq!(enabled_names(&update), ["MSFS Offline"]);
        assert_eq!(update.disabled_profile_ids, [original.id]);
    }

    #[test]
    fn adding_a_trigger_to_an_enabled_profile_disables_the_conflicting_one() {
        let (_dir, state) = state();
        let msfs = state.profiles()[0].clone();
        let xplane = state.save_profile(xplane_profile()).expect("save").profiles[1].clone();

        let mut widened = msfs.clone();
        widened.triggers.push(xplane.triggers[0].clone());
        let update = state.save_profile(widened).expect("save");

        assert_eq!(enabled_names(&update), ["MSFS 2024"]);
        assert_eq!(update.disabled_profile_ids, [xplane.id]);
    }

    #[test]
    fn deleting_a_profile_keeps_the_others_as_they_were() {
        let (_dir, state) = state();
        let original = state.profiles()[0].clone();
        state.save_profile(xplane_profile()).expect("save");

        let profiles = state.delete_profile(&original.id).expect("delete");

        assert_eq!(profiles.len(), 1);
        assert!(profiles[0].enabled);
    }

    #[test]
    fn load_disables_profiles_sharing_a_trigger_with_the_legacy_active_one() {
        let dir = tempfile::tempdir().expect("temp dir");
        let storage = Storage::new(dir.path().to_path_buf()).expect("storage");
        let online = Profile {
            name: "Online".into(),
            ..example_profile()
        };
        let offline = Profile {
            name: "Offline".into(),
            ..example_profile()
        };
        storage
            .save_profiles(&[offline.clone(), online.clone()])
            .expect("save");
        std::fs::write(
            dir.path().join("settings.json"),
            format!(r#"{{"schemaVersion":1,"activeProfileId":"{}"}}"#, online.id),
        )
        .expect("write");

        let state = AppState::load(storage).expect("load");

        let enabled: Vec<String> = state
            .profiles()
            .into_iter()
            .filter(|profile| profile.enabled)
            .map(|profile| profile.name)
            .collect();
        assert_eq!(enabled, ["Online"]);
    }

    fn finished_session(is_test: bool) -> SessionLog {
        SessionLog {
            profile_id: "profile".into(),
            profile_name: "MSFS".into(),
            is_test,
            started_at_ms: 1,
            ended_at_ms: Some(2),
            entries: Vec::new(),
        }
    }

    #[test]
    fn recorded_sessions_survive_a_restart_until_cleared() {
        let (dir, state) = state();
        state.record_session(finished_session(false));
        state.record_session(finished_session(true));

        let reload = || {
            AppState::load(Storage::new(dir.path().to_path_buf()).expect("storage"))
                .expect("reload")
        };
        assert_eq!(reload().session_history().len(), 2);

        state.clear_session_history().expect("clear");
        assert!(state.session_history().is_empty());
        assert!(reload().session_history().is_empty());
    }

    #[test]
    fn last_profile_cannot_be_deleted() {
        let (_dir, state) = state();
        let only = state.profiles()[0].id.clone();

        assert!(matches!(
            state.delete_profile(&only),
            Err(AppError::LastProfile)
        ));
    }

    fn profile_with_app(mode: ProcessNameMode) -> Profile {
        let mut profile = example_profile();
        profile.items = vec![LaunchItem::App(crate::models::AppItem {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Volanta".into(),
            exe_path: "C:\\Apps\\Volanta\\Launcher.exe".into(),
            args: None,
            working_dir: None,
            process_name: "Launcher.exe".into(),
            process_name_mode: mode,
            icon_base64: None,
            delay_ms: 0,
            run_as_admin: false,
            start_minimized: false,
            wait_for_sim_connect: false,
            restart_on_crash: false,
            on_close: crate::models::OnClose::Graceful,
            enabled: true,
            launch_before_simulator: false,
            only_for_triggers: Vec::new(),
        })];
        profile
    }

    fn saved(state: &AppState, profile: Profile) -> Profile {
        let id = profile.id.clone();
        state.save_profile(profile).expect("save");
        state.profile(&id).expect("profile")
    }

    fn first_process_name(profile: &Profile) -> &str {
        match profile.items.first() {
            Some(LaunchItem::App(app)) => &app.process_name,
            _ => "",
        }
    }

    #[test]
    fn learned_name_is_persisted_for_auto_items() {
        let (dir, state) = state();
        let profile = saved(&state, profile_with_app(ProcessNameMode::Auto));
        let item_id = profile.items[0].id().to_owned();

        let updated = state
            .learn_process_name(&profile.id, &item_id, "Volanta.exe")
            .expect("learn")
            .expect("profile changed");

        assert_eq!(first_process_name(&updated), "Volanta.exe");
        let reloaded = AppState::load(Storage::new(dir.path().to_path_buf()).expect("storage"))
            .expect("reload")
            .profile(&profile.id)
            .expect("profile");
        assert_eq!(first_process_name(&reloaded), "Volanta.exe");
    }

    #[test]
    fn learning_ignores_manual_items_and_unknown_ids() {
        let (_dir, state) = state();
        let profile = saved(&state, profile_with_app(ProcessNameMode::Manual));
        let item_id = profile.items[0].id().to_owned();

        assert!(state
            .learn_process_name(&profile.id, &item_id, "Volanta.exe")
            .expect("learn")
            .is_none());
        assert!(state
            .learn_process_name(&profile.id, "missing", "Volanta.exe")
            .expect("learn")
            .is_none());
        assert_eq!(
            first_process_name(&state.profile(&profile.id).expect("profile")),
            "Launcher.exe"
        );
    }
}
