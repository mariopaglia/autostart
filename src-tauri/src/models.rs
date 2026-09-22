use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::ErrorPayload;

pub const DEFAULT_DELAY_MS: u32 = 800;
pub const MAX_DELAY_MS: u32 = 60_000;
pub const DEFAULT_GRACEFUL_TIMEOUT_MS: u32 = 5_000;
pub const MIN_GRACEFUL_TIMEOUT_MS: u32 = 1_000;
pub const MAX_GRACEFUL_TIMEOUT_MS: u32 = 60_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Trigger {
    pub process_name: String,
    pub label: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum OnClose {
    #[default]
    Graceful,
    Force,
    Keep,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppItem {
    pub id: String,
    pub name: String,
    pub exe_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub args: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub working_dir: Option<String>,
    pub process_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub icon_base64: Option<String>,
    #[serde(default = "default_delay_ms")]
    pub delay_ms: u32,
    #[serde(default)]
    pub run_as_admin: bool,
    #[serde(default)]
    pub on_close: OnClose,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UrlItem {
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(default = "default_delay_ms")]
    pub delay_ms: u32,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "lowercase")]
#[ts(export)]
pub enum LaunchItem {
    App(AppItem),
    Url(UrlItem),
}

impl LaunchItem {
    pub fn id(&self) -> &str {
        match self {
            Self::App(item) => &item.id,
            Self::Url(item) => &item.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::App(item) => &item.name,
            Self::Url(item) => &item.name,
        }
    }

    pub fn delay_ms(&self) -> u32 {
        match self {
            Self::App(item) => item.delay_ms,
            Self::Url(item) => item.delay_ms,
        }
    }

    pub fn is_enabled(&self) -> bool {
        match self {
            Self::App(item) => item.enabled,
            Self::Url(item) => item.enabled,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub trigger: Trigger,
    #[serde(default)]
    pub items: Vec<LaunchItem>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Theme {
    System,
    Light,
    #[default]
    Dark,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Language {
    #[default]
    #[serde(rename = "pt-BR")]
    PtBr,
    #[serde(rename = "en")]
    En,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct Settings {
    pub active_profile_id: Option<String>,
    pub start_with_windows: bool,
    pub start_minimized: bool,
    pub graceful_timeout_ms: u32,
    pub close_only_if_launched_by_app: bool,
    pub theme: Theme,
    pub language: Language,
    pub onboarding_completed: bool,
    pub check_updates_on_startup: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            active_profile_id: None,
            start_with_windows: false,
            start_minimized: true,
            graceful_timeout_ms: DEFAULT_GRACEFUL_TIMEOUT_MS,
            close_only_if_launched_by_app: true,
            theme: Theme::default(),
            language: Language::default(),
            onboarding_completed: false,
            check_updates_on_startup: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ItemStatus {
    Pending,
    Launching,
    Running,
    Skipped,
    Closing,
    Closed,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ItemRuntime {
    pub item_id: String,
    pub status: ItemStatus,
    pub launched_by_app: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum MonitorState {
    #[default]
    Idle,
    SimRunning,
    Closing,
    Paused,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MonitorSnapshot {
    pub state: MonitorState,
    pub session_profile_id: Option<String>,
    pub is_test_session: bool,
    pub items: Vec<ItemRuntime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TimelineKind {
    SessionStarted,
    Launched,
    Skipped,
    NotRunning,
    ClosedGracefully,
    ForceClosed,
    Kept,
    Error,
    SessionEnded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TimelineEntry {
    #[ts(type = "number")]
    pub timestamp_ms: u64,
    pub kind: TimelineKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub item_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub item_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SessionLog {
    pub profile_id: String,
    pub profile_name: String,
    pub is_test: bool,
    #[ts(type = "number")]
    pub started_at_ms: u64,
    #[ts(type = "number | null")]
    pub ended_at_ms: Option<u64>,
    pub entries: Vec<TimelineEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExeInfo {
    pub process_name: String,
    pub product_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub icon_base64: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProcessInfo {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub exe_path: Option<String>,
}

fn default_delay_ms() -> u32 {
    DEFAULT_DELAY_MS
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_item_uses_type_tag_and_camel_case() {
        let json = r#"{"type":"app","id":"1","name":"Volanta","exePath":"C:\\Volanta.exe","processName":"Volanta.exe"}"#;
        let item: LaunchItem = serde_json::from_str(json).expect("valid app item");
        let LaunchItem::App(app) = item else {
            panic!("expected app item");
        };
        assert_eq!(app.delay_ms, DEFAULT_DELAY_MS);
        assert_eq!(app.on_close, OnClose::Graceful);
        assert!(app.enabled);
        assert!(!app.run_as_admin);
    }

    #[test]
    fn settings_fill_missing_fields_with_defaults() {
        let settings: Settings =
            serde_json::from_str(r#"{"language":"en"}"#).expect("valid settings");
        assert_eq!(settings.language, Language::En);
        assert_eq!(settings.graceful_timeout_ms, DEFAULT_GRACEFUL_TIMEOUT_MS);
        assert!(settings.close_only_if_launched_by_app);
    }
}
