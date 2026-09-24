use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::ErrorPayload;
use crate::processes::normalize_process_name;

pub const DEFAULT_DELAY_MS: u32 = 800;
pub const MAX_DELAY_MS: u32 = 60_000;
pub const DEFAULT_GRACEFUL_TIMEOUT_MS: u32 = 5_000;
pub const MIN_GRACEFUL_TIMEOUT_MS: u32 = 1_000;
pub const MAX_GRACEFUL_TIMEOUT_MS: u32 = 60_000;
pub const DEFAULT_CLOSE_DELAY_MS: u32 = 60_000;
pub const MAX_CLOSE_DELAY_MS: u32 = 600_000;
pub const MAX_TRIGGERS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Trigger {
    pub process_name: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub launch_target: Option<String>,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ProcessNameMode {
    #[default]
    Auto,
    Manual,
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
    #[serde(default)]
    pub process_name_mode: ProcessNameMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub icon_base64: Option<String>,
    #[serde(default = "default_delay_ms")]
    pub delay_ms: u32,
    #[serde(default)]
    pub run_as_admin: bool,
    #[serde(default)]
    pub start_minimized: bool,
    #[serde(default)]
    pub wait_for_sim_connect: bool,
    #[serde(default)]
    pub restart_on_crash: bool,
    #[serde(default)]
    pub launch_before_simulator: bool,
    #[serde(default)]
    pub only_for_triggers: Vec<String>,
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
    #[serde(default)]
    pub only_for_triggers: Vec<String>,
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

    pub fn only_for_triggers(&self) -> &[String] {
        match self {
            Self::App(item) => &item.only_for_triggers,
            Self::Url(item) => &item.only_for_triggers,
        }
    }

    /// An item without restrictions applies to every trigger of its profile.
    pub fn applies_to(&self, trigger_process_name: &str) -> bool {
        let wanted = normalize_process_name(trigger_process_name);
        let only_for = self.only_for_triggers();
        only_for.is_empty()
            || only_for
                .iter()
                .any(|process_name| normalize_process_name(process_name) == wanted)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub triggers: Vec<Trigger>,
    #[serde(default)]
    pub items: Vec<LaunchItem>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProfilesUpdate {
    pub profiles: Vec<Profile>,
    pub disabled_profile_ids: Vec<String>,
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
    pub start_with_windows: bool,
    pub start_minimized: bool,
    pub graceful_timeout_ms: u32,
    pub close_delay_ms: u32,
    pub close_only_if_launched_by_app: bool,
    pub show_notifications: bool,
    pub theme: Theme,
    pub language: Language,
    pub onboarding_completed: bool,
    pub check_updates_on_startup: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            start_with_windows: false,
            start_minimized: true,
            graceful_timeout_ms: DEFAULT_GRACEFUL_TIMEOUT_MS,
            close_delay_ms: DEFAULT_CLOSE_DELAY_MS,
            close_only_if_launched_by_app: true,
            show_notifications: true,
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
    #[serde(rename = "waitingSimConnect")]
    WaitingSimConnect,
    Launching,
    Running,
    Restarting,
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
    SimStarting,
    SimRunning,
    ClosePending,
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
    #[ts(type = "number | null")]
    pub closes_at_ms: Option<u64>,
    /// Label of the simulator being started while the state is `simStarting`.
    pub starting_simulator: Option<String>,
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
    ProcessNameLearned,
    SimConnectReady,
    SimConnectWaitSkipped,
    CloseDelayed,
    SimulatorReturned,
    SimulatorStarted,
    SimulatorNotStarted,
    KeptByUser,
    Crashed,
    Relaunched,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub detail: Option<String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum CandidateSource {
    Installed,
    Open,
    Dropped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppCandidate {
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
    pub source: CandidateSource,
    pub suggested: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UrlCandidate {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "lowercase")]
#[ts(export)]
pub enum DroppedCandidate {
    App(AppCandidate),
    Url(UrlCandidate),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DropRejection {
    UnsupportedFile,
    ExecutableNotFound,
    UnsupportedUrl,
    UnsupportedShortcut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RejectedDrop {
    pub path: String,
    pub reason: DropRejection,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DropResolution {
    pub candidates: Vec<DroppedCandidate>,
    pub rejected: Vec<RejectedDrop>,
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
        assert_eq!(app.process_name_mode, ProcessNameMode::Auto);
        assert!(!app.start_minimized);
        assert!(!app.wait_for_sim_connect);
        assert!(!app.restart_on_crash);
        assert!(!app.launch_before_simulator);
        assert!(app.only_for_triggers.is_empty());
    }

    #[test]
    fn trigger_without_launch_target_keeps_its_json_shape() {
        let json = r#"{"processName":"X-Plane.exe","label":"X-Plane 12"}"#;
        let trigger: Trigger = serde_json::from_str(json).expect("valid trigger");

        assert_eq!(trigger.launch_target, None);
        assert_eq!(serde_json::to_string(&trigger).expect("serializable"), json);
    }

    #[test]
    fn item_applies_to_its_listed_triggers_ignoring_case() {
        let json = r#"{"type":"url","id":"1","name":"Site","url":"https://a.com","onlyForTriggers":["FlightSimulator2024.exe"]}"#;
        let item: LaunchItem = serde_json::from_str(json).expect("valid url item");

        assert!(item.applies_to("flightsimulator2024.EXE"));
        assert!(!item.applies_to("FlightSimulator.exe"));
    }

    #[test]
    fn unrestricted_item_applies_to_every_trigger() {
        let json = r#"{"type":"url","id":"1","name":"Site","url":"https://a.com"}"#;
        let item: LaunchItem = serde_json::from_str(json).expect("valid url item");

        assert!(item.applies_to("X-Plane.exe"));
    }

    #[test]
    fn settings_fill_missing_fields_with_defaults() {
        let settings: Settings =
            serde_json::from_str(r#"{"language":"en"}"#).expect("valid settings");
        assert_eq!(settings.language, Language::En);
        assert_eq!(settings.graceful_timeout_ms, DEFAULT_GRACEFUL_TIMEOUT_MS);
        assert!(settings.close_only_if_launched_by_app);
        assert_eq!(settings.close_delay_ms, DEFAULT_CLOSE_DELAY_MS);
        assert!(settings.show_notifications);
    }

    #[test]
    fn new_states_and_statuses_use_camel_case() {
        assert_eq!(
            serde_json::to_value(MonitorState::ClosePending).expect("serializable"),
            "closePending"
        );
        assert_eq!(
            serde_json::to_value(ItemStatus::Restarting).expect("serializable"),
            "restarting"
        );
        assert_eq!(
            serde_json::to_value(TimelineKind::KeptByUser).expect("serializable"),
            "keptByUser"
        );
    }

    #[test]
    fn dropped_candidate_uses_type_tag_and_camel_case() {
        let candidate = DroppedCandidate::Url(UrlCandidate {
            name: "SimBrief".into(),
            url: "https://www.simbrief.com".into(),
        });
        let json = serde_json::to_value(&candidate).expect("serializable");
        assert_eq!(
            json,
            serde_json::json!({ "type": "url", "name": "SimBrief", "url": "https://www.simbrief.com" })
        );

        let rejected = RejectedDrop {
            path: "C:\\a.pdf".into(),
            reason: DropRejection::ExecutableNotFound,
        };
        let json = serde_json::to_value(&rejected).expect("serializable");
        assert_eq!(json["reason"], "executableNotFound");
    }
}
