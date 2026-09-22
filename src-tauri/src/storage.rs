use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::AppResult;
use crate::models::{LaunchItem, Profile, SessionLog, Settings, Trigger, UrlItem};
use crate::processes::normalize_process_name;

const SCHEMA_VERSION: u32 = 1;
const PROFILES_FILE: &str = "profiles.json";
const SETTINGS_FILE: &str = "settings.json";
const LAST_SESSION_FILE: &str = "last-session.json";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfilesFile {
    schema_version: u32,
    profiles: Vec<Profile>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsFile {
    schema_version: u32,
    #[serde(flatten)]
    settings: Settings,
}

pub struct Storage {
    dir: PathBuf,
}

impl Storage {
    pub fn new(dir: PathBuf) -> AppResult<Self> {
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    pub fn load_profiles(&self) -> AppResult<Vec<Profile>> {
        let profiles = self
            .read_migrated_or_recover::<ProfilesFile>(PROFILES_FILE, infer_process_name_modes)?
            .map(|file| file.profiles)
            .unwrap_or_default();

        if profiles.is_empty() {
            return Ok(vec![example_profile()]);
        }
        Ok(profiles)
    }

    pub fn save_profiles(&self, profiles: &[Profile]) -> AppResult<()> {
        let file = ProfilesFile {
            schema_version: SCHEMA_VERSION,
            profiles: profiles.to_vec(),
        };
        self.write_atomic(PROFILES_FILE, &file)
    }

    pub fn load_settings(&self) -> AppResult<Settings> {
        Ok(self
            .read_or_recover::<SettingsFile>(SETTINGS_FILE)?
            .map(|file| file.settings)
            .unwrap_or_default())
    }

    pub fn save_settings(&self, settings: &Settings) -> AppResult<()> {
        let file = SettingsFile {
            schema_version: SCHEMA_VERSION,
            settings: settings.clone(),
        };
        self.write_atomic(SETTINGS_FILE, &file)
    }

    pub fn load_last_session(&self) -> AppResult<Option<SessionLog>> {
        self.read_or_recover(LAST_SESSION_FILE)
    }

    pub fn save_last_session(&self, log: &SessionLog) -> AppResult<()> {
        self.write_atomic(LAST_SESSION_FILE, log)
    }

    fn read_or_recover<T: DeserializeOwned>(&self, file_name: &str) -> AppResult<Option<T>> {
        self.read_migrated_or_recover(file_name, |_| {})
    }

    fn read_migrated_or_recover<T: DeserializeOwned>(
        &self,
        file_name: &str,
        migrate: impl FnOnce(&mut Value),
    ) -> AppResult<Option<T>> {
        let path = self.dir.join(file_name);
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };

        let parsed = serde_json::from_str::<Value>(&content).and_then(|mut value| {
            migrate(&mut value);
            serde_json::from_value(value)
        });
        match parsed {
            Ok(value) => Ok(Some(value)),
            Err(error) => {
                let quarantined = quarantine(&path)?;
                log::error!(
                    "{} is corrupt ({error}); moved to {}",
                    path.display(),
                    quarantined.display()
                );
                Ok(None)
            }
        }
    }

    // Writing to a temp file and renaming guarantees the previous file survives a crash mid-write.
    fn write_atomic<T: Serialize>(&self, file_name: &str, value: &T) -> AppResult<()> {
        let path = self.dir.join(file_name);
        let temp_path = self.dir.join(format!("{file_name}.tmp"));

        let mut temp_file = fs::File::create(&temp_path)?;
        temp_file.write_all(&serde_json::to_vec_pretty(value)?)?;
        temp_file.sync_all()?;
        drop(temp_file);

        fs::rename(&temp_path, &path)?;
        Ok(())
    }
}

fn quarantine(path: &Path) -> AppResult<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let mut quarantined = path.as_os_str().to_owned();
    quarantined.push(format!(".corrupt-{timestamp}"));
    let quarantined = PathBuf::from(quarantined);
    fs::rename(path, &quarantined)?;
    Ok(quarantined)
}

/// Profiles saved before `processNameMode` existed: a process name that differs from the
/// executable was typed by the user, so learning must not overwrite it.
fn infer_process_name_modes(raw: &mut Value) {
    let Some(profiles) = raw.get_mut("profiles").and_then(Value::as_array_mut) else {
        return;
    };
    let items = profiles
        .iter_mut()
        .filter_map(|profile| profile.get_mut("items")?.as_array_mut())
        .flatten()
        .filter_map(Value::as_object_mut)
        .filter(|item| {
            item.get("type").and_then(Value::as_str) == Some("app")
                && !item.contains_key("processNameMode")
        });

    for item in items {
        let text = |key: &str| item.get(key).and_then(Value::as_str).unwrap_or_default();
        let executable = text("exePath")
            .rsplit(['\\', '/'])
            .next()
            .unwrap_or_default();
        let typed_by_user =
            normalize_process_name(text("processName")) != normalize_process_name(executable);
        let mode = if typed_by_user { "manual" } else { "auto" };
        item.insert("processNameMode".into(), Value::from(mode));
    }
}

pub fn example_profile() -> Profile {
    Profile {
        id: uuid::Uuid::new_v4().to_string(),
        name: "MSFS 2024".into(),
        trigger: Trigger {
            process_name: "FlightSimulator2024.exe".into(),
            label: "MSFS 2024".into(),
        },
        items: vec![LaunchItem::Url(UrlItem {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Navigraph Charts (web)".into(),
            url: "https://charts.navigraph.com".into(),
            delay_ms: 0,
            enabled: false,
        })],
        enabled: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Language, ProcessNameMode, DEFAULT_GRACEFUL_TIMEOUT_MS};

    fn storage() -> (tempfile::TempDir, Storage) {
        let dir = tempfile::tempdir().expect("temp dir");
        let storage = Storage::new(dir.path().to_path_buf()).expect("storage");
        (dir, storage)
    }

    #[test]
    fn missing_files_yield_example_profile_and_default_settings() {
        let (_dir, storage) = storage();

        let profiles = storage.load_profiles().expect("profiles");
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].trigger.process_name, "FlightSimulator2024.exe");
        assert_eq!(
            storage.load_settings().expect("settings"),
            Settings::default()
        );
    }

    #[test]
    fn profiles_and_settings_round_trip() {
        let (_dir, storage) = storage();
        let profiles = vec![example_profile(), example_profile()];
        let settings = Settings {
            language: Language::En,
            ..Settings::default()
        };

        storage.save_profiles(&profiles).expect("save profiles");
        storage.save_settings(&settings).expect("save settings");

        assert_eq!(storage.load_profiles().expect("profiles"), profiles);
        assert_eq!(storage.load_settings().expect("settings"), settings);
    }

    #[test]
    fn settings_missing_new_fields_use_defaults() {
        let (dir, storage) = storage();
        fs::write(
            dir.path().join(SETTINGS_FILE),
            r#"{"schemaVersion":1,"language":"en"}"#,
        )
        .expect("write");

        let settings = storage.load_settings().expect("settings");
        assert_eq!(settings.language, Language::En);
        assert_eq!(settings.graceful_timeout_ms, DEFAULT_GRACEFUL_TIMEOUT_MS);
    }

    #[test]
    fn corrupt_file_is_quarantined_and_replaced_by_example() {
        let (dir, storage) = storage();
        fs::write(dir.path().join(PROFILES_FILE), "{ not json").expect("write");

        let profiles = storage.load_profiles().expect("profiles");

        assert_eq!(profiles.len(), 1);
        assert!(!dir.path().join(PROFILES_FILE).exists());
        let quarantined = fs::read_dir(dir.path())
            .expect("read dir")
            .filter_map(Result::ok)
            .any(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("profiles.json.corrupt-")
            });
        assert!(quarantined);
    }

    #[test]
    fn atomic_write_leaves_no_temp_file() {
        let (dir, storage) = storage();
        storage.save_settings(&Settings::default()).expect("save");

        assert!(dir.path().join(SETTINGS_FILE).exists());
        assert!(!dir.path().join("settings.json.tmp").exists());
    }

    #[test]
    fn items_from_v010_keep_user_typed_process_names_as_manual() {
        let (dir, storage) = storage();
        let item = |id: &str, exe: &str, process: &str| {
            format!(
                r#"{{"type":"app","id":"{id}","name":"App","exePath":"{exe}","processName":"{process}"}}"#
            )
        };
        let items = [
            item(
                "00000000-0000-4000-8000-000000000001",
                r"C:\\Apps\\Spad.exe",
                "SPAD.exe",
            ),
            item(
                "00000000-0000-4000-8000-000000000002",
                r"C:\\Apps\\Launcher.exe",
                "Volanta.exe",
            ),
        ]
        .join(",");
        fs::write(
            dir.path().join(PROFILES_FILE),
            format!(
                r#"{{"schemaVersion":1,"profiles":[{{"id":"00000000-0000-4000-8000-000000000000","name":"MSFS","trigger":{{"processName":"FlightSimulator2024.exe","label":"MSFS 2024"}},"items":[{items}]}}]}}"#
            ),
        )
        .expect("write");

        let profiles = storage.load_profiles().expect("profiles");
        let modes: Vec<ProcessNameMode> = profiles[0]
            .items
            .iter()
            .filter_map(|item| match item {
                LaunchItem::App(app) => Some(app.process_name_mode),
                LaunchItem::Url(_) => None,
            })
            .collect();

        assert_eq!(modes, [ProcessNameMode::Auto, ProcessNameMode::Manual]);
    }
}
