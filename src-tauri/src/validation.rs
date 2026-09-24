use std::collections::HashSet;

use crate::error::{AppError, AppResult};
use crate::models::{
    LaunchItem, Profile, Settings, Trigger, MAX_CLOSE_DELAY_MS, MAX_DELAY_MS,
    MAX_GRACEFUL_TIMEOUT_MS, MAX_TRIGGERS, MIN_GRACEFUL_TIMEOUT_MS,
};
use crate::processes::normalize_process_name;

const MAX_NAME_LENGTH: usize = 80;

pub fn validate_profile(profile: &Profile) -> AppResult<()> {
    require_uuid("id", &profile.id)?;
    require_name("name", &profile.name)?;
    validate_triggers(&profile.triggers)?;

    let mut seen_ids = HashSet::new();
    for (index, item) in profile.items.iter().enumerate() {
        if !seen_ids.insert(item.id()) {
            return Err(invalid(&format!("items.{index}.id"), "is duplicated"));
        }
        validate_item(index, item)?;
        validate_only_for_triggers(index, item, &profile.triggers)?;
    }
    Ok(())
}

fn validate_triggers(triggers: &[Trigger]) -> AppResult<()> {
    if triggers.is_empty() || triggers.len() > MAX_TRIGGERS {
        return Err(invalid("triggers", "must have between 1 and 5 entries"));
    }
    let mut seen_names = HashSet::new();
    for (index, trigger) in triggers.iter().enumerate() {
        require_executable_name(
            &format!("triggers.{index}.processName"),
            &trigger.process_name,
        )?;
        require_name(&format!("triggers.{index}.label"), &trigger.label)?;
        if !seen_names.insert(normalize_process_name(&trigger.process_name)) {
            return Err(invalid(
                &format!("triggers.{index}.processName"),
                "is duplicated",
            ));
        }
        if let Some(target) = &trigger.launch_target {
            require_launch_target(&format!("triggers.{index}.launchTarget"), target)?;
        }
    }
    Ok(())
}

fn validate_only_for_triggers(
    index: usize,
    item: &LaunchItem,
    triggers: &[Trigger],
) -> AppResult<()> {
    let field = format!("items.{index}.onlyForTriggers");
    let profile_triggers: HashSet<String> = triggers
        .iter()
        .map(|trigger| normalize_process_name(&trigger.process_name))
        .collect();
    let mut seen = HashSet::new();
    for process_name in item.only_for_triggers() {
        let normalized = normalize_process_name(process_name);
        if !profile_triggers.contains(&normalized) {
            return Err(invalid(&field, "must list triggers of the profile"));
        }
        if !seen.insert(normalized) {
            return Err(invalid(&field, "is duplicated"));
        }
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
            if app.launch_before_simulator && app.wait_for_sim_connect {
                return Err(invalid(
                    &field("launchBeforeSimulator"),
                    "cannot be combined with waitForSimConnect",
                ));
            }
            require_executable_name(&field("processName"), &app.process_name)
        }
        LaunchItem::Url(url) => require_item_url(&field("url"), &url.url),
    }
}

pub fn validate_settings(settings: &Settings) -> AppResult<()> {
    if !(MIN_GRACEFUL_TIMEOUT_MS..=MAX_GRACEFUL_TIMEOUT_MS).contains(&settings.graceful_timeout_ms)
    {
        return Err(invalid("gracefulTimeoutMs", "is out of range"));
    }
    if settings.close_delay_ms > MAX_CLOSE_DELAY_MS {
        return Err(invalid("closeDelayMs", "is out of range"));
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

fn require_item_url(field: &str, value: &str) -> AppResult<()> {
    if is_supported_item_url(value) {
        return Ok(());
    }
    Err(invalid(
        field,
        "must be an http(s) URL or a Steam launch link",
    ))
}

/// Profiles are shared between pilots, so only schemes that cannot run arbitrary protocol
/// handlers are accepted.
pub fn is_supported_item_url(value: &str) -> bool {
    tauri::Url::parse(value).is_ok_and(|url| matches!(url.scheme(), "http" | "https"))
        || is_steam_launch_link(value)
}

fn require_launch_target(field: &str, value: &str) -> AppResult<()> {
    if is_executable_path(value) || is_steam_launch_link(value) || is_store_app(value) {
        return Ok(());
    }
    Err(invalid(
        field,
        "must be an absolute .exe path, a Steam launch link or shell:AppsFolder\\<app id>",
    ))
}

// Checked by hand too, so Windows paths validate the same way while developing on macOS.
fn is_executable_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    let has_drive =
        bytes.len() > 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'\\';
    (has_drive || std::path::Path::new(value).is_absolute())
        && value.to_lowercase().ends_with(".exe")
}

/// Microsoft Store apps have no regular executable and start through their AppUserModelID.
fn is_store_app(value: &str) -> bool {
    let Some(app_id) = value.strip_prefix(r"shell:AppsFolder\") else {
        return false;
    };
    app_id.contains('!')
        && !app_id
            .chars()
            .any(|c| c.is_whitespace() || r#"\/:*?"<>|"#.contains(c))
}

pub fn is_steam_launch_link(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("steam://") else {
        return false;
    };
    let Some((action, app_id)) = rest.split_once('/') else {
        return false;
    };
    matches!(action, "rungameid" | "run")
        && !app_id.is_empty()
        && app_id.chars().all(|c| c.is_ascii_digit())
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
                only_for_triggers: Vec::new(),
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
            triggers: vec![trigger("FlightSimulator")],
            ..example_profile()
        };

        assert!(validate_profile(&blank_name).is_err());
        assert!(validate_profile(&bad_trigger).is_err());
    }

    fn trigger(process_name: &str) -> Trigger {
        Trigger {
            process_name: process_name.into(),
            label: "Sim".into(),
            launch_target: None,
        }
    }

    fn profile_with_triggers(process_names: &[&str]) -> Profile {
        Profile {
            triggers: process_names.iter().map(|name| trigger(name)).collect(),
            ..example_profile()
        }
    }

    #[test]
    fn accepts_up_to_five_distinct_triggers() {
        let both_msfs = profile_with_triggers(&["FlightSimulator.exe", "FlightSimulator2024.exe"]);
        let five = profile_with_triggers(&["A.exe", "B.exe", "C.exe", "D.exe", "E.exe"]);

        assert!(validate_profile(&both_msfs).is_ok());
        assert!(validate_profile(&five).is_ok());
    }

    #[test]
    fn rejects_missing_excess_and_repeated_triggers() {
        let none = profile_with_triggers(&[]);
        let six = profile_with_triggers(&["A.exe", "B.exe", "C.exe", "D.exe", "E.exe", "F.exe"]);
        let repeated =
            profile_with_triggers(&["FlightSimulator2024.exe", "flightsimulator2024.EXE"]);

        assert!(validate_profile(&none).is_err());
        assert!(validate_profile(&six).is_err());
        assert!(validate_profile(&repeated).is_err());
    }

    #[test]
    fn accepts_web_urls_and_steam_launch_links_only() {
        let accepted = [
            "https://simbrief.com",
            "http://localhost:8080/map",
            "steam://rungameid/1234560",
            "steam://run/1250410",
        ];
        let rejected = [
            "ftp://example.com",
            "not a url",
            "steam://uninstall/123",
            "steam://rungameid/",
            "steam://rungameid/12a",
            "steam://rungameid/1/extra",
            "ms-msdt:/id PCWDiagnostic",
            "file:///C:/Windows/notepad.exe",
            "search-ms:query=x",
        ];

        for url in accepted {
            assert!(validate_profile(&profile_with_url(url)).is_ok(), "{url}");
        }
        for url in rejected {
            assert!(validate_profile(&profile_with_url(url)).is_err(), "{url}");
        }
    }

    fn profile_with_launch_target(target: &str) -> Profile {
        let mut profile = example_profile();
        profile.triggers[0].launch_target = Some(target.into());
        profile
    }

    #[test]
    fn accepts_executables_steam_links_and_store_apps_as_launch_targets() {
        let accepted = [
            r"D:\X-Plane 12\X-Plane.exe",
            r"C:\Games\msfs.EXE",
            "steam://rungameid/2537590",
            r"shell:AppsFolder\Microsoft.Limitless_8wekyb3d8bbwe!App",
        ];
        let rejected = [
            r"X-Plane 12\X-Plane.exe",
            "X-Plane.exe",
            r"C:\Games\readme.txt",
            "ms-msdt:/id PCWDiagnostic",
            "https://example.com/setup.exe",
            "steam://uninstall/2537590",
            r"shell:AppsFolder\Microsoft.Limitless_8wekyb3d8bbwe",
            r"shell:AppsFolder\..\evil!App",
            r"shell:AppsFolder\My App!App",
        ];

        for target in accepted {
            assert!(
                validate_profile(&profile_with_launch_target(target)).is_ok(),
                "{target}"
            );
        }
        for target in rejected {
            assert!(
                validate_profile(&profile_with_launch_target(target)).is_err(),
                "{target}"
            );
        }
    }

    fn profile_restricting_item_to(only_for: &[&str]) -> Profile {
        let mut profile =
            profile_with_triggers(&["FlightSimulator.exe", "FlightSimulator2024.exe"]);
        profile.items = profile_with_url("https://simbrief.com").items;
        if let Some(LaunchItem::Url(item)) = profile.items.first_mut() {
            item.only_for_triggers = only_for.iter().map(|name| (*name).into()).collect();
        }
        profile
    }

    #[test]
    fn item_triggers_must_belong_to_the_profile_without_repeats() {
        assert!(validate_profile(&profile_restricting_item_to(&[])).is_ok());
        assert!(
            validate_profile(&profile_restricting_item_to(&["flightsimulator2024.EXE"])).is_ok()
        );
        assert!(validate_profile(&profile_restricting_item_to(&["X-Plane.exe"])).is_err());
        assert!(validate_profile(&profile_restricting_item_to(&[
            "FlightSimulator.exe",
            "flightsimulator.exe"
        ]))
        .is_err());
    }

    #[test]
    fn opening_before_the_simulator_excludes_waiting_for_simconnect() {
        let mut profile = example_profile();
        profile.items = vec![LaunchItem::App(crate::models::AppItem {
            id: uuid::Uuid::new_v4().to_string(),
            name: "TrackIR".into(),
            exe_path: r"C:\TrackIR\TrackIR5.exe".into(),
            args: None,
            working_dir: None,
            process_name: "TrackIR5.exe".into(),
            process_name_mode: crate::models::ProcessNameMode::Auto,
            icon_base64: None,
            delay_ms: 0,
            run_as_admin: false,
            start_minimized: false,
            wait_for_sim_connect: false,
            restart_on_crash: false,
            launch_before_simulator: true,
            only_for_triggers: Vec::new(),
            on_close: crate::models::OnClose::Graceful,
            enabled: true,
        })];
        assert!(validate_profile(&profile).is_ok());

        if let Some(LaunchItem::App(item)) = profile.items.first_mut() {
            item.wait_for_sim_connect = true;
        }
        assert!(validate_profile(&profile).is_err());
    }

    #[test]
    fn rejects_out_of_range_timeout_and_close_delay() {
        let short_timeout = Settings {
            graceful_timeout_ms: 200,
            ..Settings::default()
        };
        let long_delay = Settings {
            close_delay_ms: MAX_CLOSE_DELAY_MS + 1,
            ..Settings::default()
        };
        let no_delay = Settings {
            close_delay_ms: 0,
            ..Settings::default()
        };

        assert!(validate_settings(&short_timeout).is_err());
        assert!(validate_settings(&long_delay).is_err());
        assert!(validate_settings(&no_delay).is_ok());
    }
}
