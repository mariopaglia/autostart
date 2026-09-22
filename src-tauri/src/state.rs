use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::error::{AppError, AppResult};
use crate::models::{LaunchItem, ProcessNameMode, Profile, SessionLog, Settings};
use crate::storage::Storage;
use crate::validation::{validate_profile, validate_settings};

struct AppData {
    profiles: Vec<Profile>,
    settings: Settings,
}

pub struct AppState {
    storage: Storage,
    data: Mutex<AppData>,
}

impl AppState {
    pub fn load(storage: Storage) -> AppResult<Self> {
        let profiles = storage.load_profiles()?;
        let mut settings = storage.load_settings()?;

        let active_is_valid = settings
            .active_profile_id
            .as_ref()
            .is_some_and(|id| profiles.iter().any(|profile| &profile.id == id));
        if !active_is_valid {
            settings.active_profile_id = profiles.first().map(|profile| profile.id.clone());
        }

        storage.save_profiles(&profiles)?;
        storage.save_settings(&settings)?;

        Ok(Self {
            storage,
            data: Mutex::new(AppData { profiles, settings }),
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

    pub fn save_profile(&self, profile: Profile) -> AppResult<Profile> {
        validate_profile(&profile)?;
        let mut data = self.data();

        match data
            .profiles
            .iter_mut()
            .find(|existing| existing.id == profile.id)
        {
            Some(existing) => *existing = profile.clone(),
            None => data.profiles.push(profile.clone()),
        }
        self.storage.save_profiles(&data.profiles)?;
        Ok(profile)
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

    pub fn delete_profile(&self, id: &str) -> AppResult<Settings> {
        let mut data = self.data();
        if !data.profiles.iter().any(|profile| profile.id == id) {
            return Err(AppError::ProfileNotFound(id.to_owned()));
        }
        if data.profiles.len() == 1 {
            return Err(AppError::LastProfile);
        }

        data.profiles.retain(|profile| profile.id != id);
        if data.settings.active_profile_id.as_deref() == Some(id) {
            data.settings.active_profile_id =
                data.profiles.first().map(|profile| profile.id.clone());
            self.storage.save_settings(&data.settings)?;
        }
        self.storage.save_profiles(&data.profiles)?;
        Ok(data.settings.clone())
    }

    pub fn settings(&self) -> Settings {
        self.data().settings.clone()
    }

    pub fn save_settings(&self, settings: Settings) -> AppResult<Settings> {
        let mut data = self.data();
        validate_settings(&settings, &data.profiles)?;
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

    pub fn set_active_profile(&self, id: &str) -> AppResult<Settings> {
        let settings = Settings {
            active_profile_id: Some(id.to_owned()),
            ..self.settings()
        };
        self.save_settings(settings)
    }

    pub fn load_last_session(&self) -> Option<SessionLog> {
        self.storage.load_last_session().unwrap_or_else(|error| {
            log::warn!("could not read the last session: {error}");
            None
        })
    }

    pub fn save_last_session(&self, log: &SessionLog) {
        if let Err(error) = self.storage.save_last_session(log) {
            log::error!("could not save the last session: {error}");
        }
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
    use crate::storage::example_profile;

    fn state() -> (tempfile::TempDir, AppState) {
        let dir = tempfile::tempdir().expect("temp dir");
        let storage = Storage::new(dir.path().to_path_buf()).expect("storage");
        (dir, AppState::load(storage).expect("state"))
    }

    #[test]
    fn first_load_activates_example_profile() {
        let (_dir, state) = state();
        let profiles = state.profiles();

        assert_eq!(
            state.settings().active_profile_id,
            Some(profiles[0].id.clone())
        );
    }

    #[test]
    fn deleting_active_profile_activates_first_remaining() {
        let (_dir, state) = state();
        let original = state.profiles()[0].clone();
        let second = state.save_profile(example_profile()).expect("save");

        let settings = state.delete_profile(&original.id).expect("delete");

        assert_eq!(settings.active_profile_id, Some(second.id));
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
            on_close: crate::models::OnClose::Graceful,
            enabled: true,
        })];
        profile
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
        let profile = state
            .save_profile(profile_with_app(ProcessNameMode::Auto))
            .expect("save");
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
        let profile = state
            .save_profile(profile_with_app(ProcessNameMode::Manual))
            .expect("save");
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
