use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::error::{AppError, AppResult};
use crate::models::{Profile, Settings};
use crate::storage::Storage;
use crate::validation::{validate_profile, validate_settings};

struct AppData {
    profiles: Vec<Profile>,
    settings: Settings,
}

#[derive(Debug, Clone)]
pub struct TestLaunch {
    pub profile_id: String,
    pub launched_item_ids: Vec<String>,
}

pub struct AppState {
    storage: Storage,
    data: Mutex<AppData>,
    last_test_launch: Mutex<Option<TestLaunch>>,
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
            last_test_launch: Mutex::new(None),
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

    pub fn set_active_profile(&self, id: &str) -> AppResult<Settings> {
        let settings = Settings {
            active_profile_id: Some(id.to_owned()),
            ..self.settings()
        };
        self.save_settings(settings)
    }

    pub fn record_test_launch(&self, test_launch: TestLaunch) {
        *lock(&self.last_test_launch) = Some(test_launch);
    }

    pub fn take_test_launch(&self, profile_id: &str) -> Option<TestLaunch> {
        let mut last = lock(&self.last_test_launch);
        match last.as_ref() {
            Some(test_launch) if test_launch.profile_id == profile_id => last.take(),
            _ => None,
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

    #[test]
    fn test_launch_is_consumed_only_by_its_profile() {
        let (_dir, state) = state();
        state.record_test_launch(TestLaunch {
            profile_id: "a".into(),
            launched_item_ids: vec!["1".into()],
        });

        assert!(state.take_test_launch("b").is_none());
        assert!(state.take_test_launch("a").is_some());
        assert!(state.take_test_launch("a").is_none());
    }
}
