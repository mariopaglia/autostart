use std::collections::HashSet;

use crate::error::{AppError, AppResult};
use crate::models::{
    LaunchItem, Profile, Settings, MAX_DELAY_MS, MAX_GRACEFUL_TIMEOUT_MS, MIN_GRACEFUL_TIMEOUT_MS,
};

const MAX_NAME_LENGTH: usize = 80;

pub fn validate_profile(profile: &Profile) -> AppResult<()> {
    require_uuid("id", &profile.id)?;
    require_name("name", &profile.name)?;
    require_executable_name("trigger.processName", &profile.trigger.process_name)?;
    require_name("trigger.label", &profile.trigger.label)?;

    let mut seen_ids = HashSet::new();
    for (index, item) in profile.items.iter().enumerate() {
        if !seen_ids.insert(item.id()) {
            return Err(invalid(&format!("items.{index}.id"), "is duplicated"));
        }
        validate_item(index, item)?;
    }
    Ok(())
}

fn validate_item(index: usize, item: &LaunchItem) -> AppResult<()> {
    let field = |name: &str| format!("items.{index}.{name}");
    require_uuid(&field("id"), item.id())?;
    require_name(&field("name"), item.name())?;
    if item.delay_ms() > MAX_DELAY_MS {
        return Err(invalid(&field("delayMs"), "exceeds the maximum delay"));
    }

    match item {
        LaunchItem::App(app) => {
            if app.exe_path.trim().is_empty() {
                return Err(invalid(&field("exePath"), "is required"));
            }
            require_executable_name(&field("processName"), &app.process_name)
        }
        LaunchItem::Url(url) => require_web_url(&field("url"), &url.url),
    }
}

pub fn validate_settings(settings: &Settings, profiles: &[Profile]) -> AppResult<()> {
    if !(MIN_GRACEFUL_TIMEOUT_MS..=MAX_GRACEFUL_TIMEOUT_MS).contains(&settings.graceful_timeout_ms)
    {
        return Err(invalid("gracefulTimeoutMs", "is out of range"));
    }
    if let Some(active_id) = &settings.active_profile_id {
        if !profiles.iter().any(|profile| &profile.id == active_id) {
            return Err(AppError::ProfileNotFound(active_id.clone()));
        }
    }
    Ok(())
}

fn require_uuid(field: &str, value: &str) -> AppResult<()> {
    uuid::Uuid::parse_str(value)
        .map(drop)
        .map_err(|_| invalid(field, "is not a valid UUID"))
}

fn require_name(field: &str, value: &str) -> AppResult<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > MAX_NAME_LENGTH {
        return Err(invalid(field, "must have between 1 and 80 characters"));
    }
    Ok(())
}

fn require_executable_name(field: &str, value: &str) -> AppResult<()> {
    let has_invalid_chars = value.chars().any(|c| r#"\/:*?"<>|"#.contains(c));
    let is_exe = value.to_lowercase().ends_with(".exe") && value.len() > ".exe".len();
    if has_invalid_chars || !is_exe {
        return Err(invalid(field, "must be an executable name like App.exe"));
    }
    Ok(())
}

fn require_web_url(field: &str, value: &str) -> AppResult<()> {
    match tauri::Url::parse(value) {
        Ok(url) if matches!(url.scheme(), "http" | "https") => Ok(()),
        _ => Err(invalid(field, "must be an http or https URL")),
    }
}

fn invalid(field: &str, reason: &str) -> AppError {
    AppError::Validation(format!("{field} {reason}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Trigger, UrlItem};
    use crate::storage::example_profile;

    fn profile_with_url(url: &str) -> Profile {
        Profile {
            items: vec![LaunchItem::Url(UrlItem {
                id: uuid::Uuid::new_v4().to_string(),
                name: "Site".into(),
                url: url.into(),
                delay_ms: 0,
                enabled: true,
            })],
            ..example_profile()
        }
    }

    #[test]
    fn example_profile_is_valid() {
        assert!(validate_profile(&example_profile()).is_ok());
    }

    #[test]
    fn rejects_blank_name_and_non_exe_trigger() {
        let blank_name = Profile {
            name: "  ".into(),
            ..example_profile()
        };
        let bad_trigger = Profile {
            trigger: Trigger {
                process_name: "FlightSimulator".into(),
                label: "MSFS".into(),
            },
            ..example_profile()
        };

        assert!(validate_profile(&blank_name).is_err());
        assert!(validate_profile(&bad_trigger).is_err());
    }

    #[test]
    fn accepts_only_web_urls() {
        assert!(validate_profile(&profile_with_url("https://simbrief.com")).is_ok());
        assert!(validate_profile(&profile_with_url("ftp://example.com")).is_err());
        assert!(validate_profile(&profile_with_url("not a url")).is_err());
    }

    #[test]
    fn rejects_out_of_range_timeout_and_unknown_active_profile() {
        let profiles = vec![example_profile()];
        let short_timeout = Settings {
            graceful_timeout_ms: 200,
            ..Settings::default()
        };
        let unknown_active = Settings {
            active_profile_id: Some(uuid::Uuid::new_v4().to_string()),
            ..Settings::default()
        };

        assert!(validate_settings(&short_timeout, &profiles).is_err());
        assert!(validate_settings(&unknown_active, &profiles).is_err());
    }
}
