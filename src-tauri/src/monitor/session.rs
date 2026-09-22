use std::time::{SystemTime, UNIX_EPOCH};

use crate::closer::CloseTarget;
use crate::error::AppError;
use crate::models::{
    ItemRuntime, ItemStatus, LaunchItem, ProcessNameMode, Profile, SessionLog, TimelineEntry,
    TimelineKind,
};

/// One run of a profile, real or test: a frozen copy of the profile, per-item runtime and timeline.
pub struct Session {
    profile: Profile,
    items: Vec<ItemRuntime>,
    log: SessionLog,
}

impl Session {
    pub fn start(profile: Profile, is_test: bool) -> Self {
        let items = profile
            .items
            .iter()
            .filter(|item| item.is_enabled())
            .map(|item| ItemRuntime {
                item_id: item.id().to_owned(),
                status: ItemStatus::Pending,
                launched_by_app: false,
                error: None,
            })
            .collect();
        let log = SessionLog {
            profile_id: profile.id.clone(),
            profile_name: profile.name.clone(),
            is_test,
            started_at_ms: now_ms(),
            ended_at_ms: None,
            entries: Vec::new(),
        };

        let mut session = Self {
            profile,
            items,
            log,
        };
        session.record(TimelineKind::SessionStarted, None, None);
        session
    }

    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    pub fn is_test(&self) -> bool {
        self.log.is_test
    }

    pub fn items(&self) -> &[ItemRuntime] {
        &self.items
    }

    pub fn log(&self) -> &SessionLog {
        &self.log
    }

    pub fn enabled_items(&self) -> Vec<LaunchItem> {
        self.profile
            .items
            .iter()
            .filter(|item| item.is_enabled())
            .cloned()
            .collect()
    }

    pub fn update(&mut self, runtime: ItemRuntime) {
        if let Some(existing) = self
            .items
            .iter_mut()
            .find(|existing| existing.item_id == runtime.item_id)
        {
            // Closing never forgets that AutoStart launched the item.
            let launched_by_app = existing.launched_by_app || runtime.launched_by_app;
            *existing = ItemRuntime {
                launched_by_app,
                ..runtime
            };
        }
    }

    pub fn record(
        &mut self,
        kind: TimelineKind,
        item_id: Option<&str>,
        error: Option<&AppError>,
    ) -> TimelineEntry {
        self.push_entry(kind, item_id, error, None)
    }

    pub fn record_detail(
        &mut self,
        kind: TimelineKind,
        item_id: &str,
        detail: String,
    ) -> TimelineEntry {
        self.push_entry(kind, Some(item_id), None, Some(detail))
    }

    /// Returns whether the name changed; items in manual mode keep what the user typed.
    pub fn learn_process_name(&mut self, item_id: &str, process_name: &str) -> bool {
        let Some(item) = self.profile.items.iter_mut().find_map(|item| match item {
            LaunchItem::App(app) if app.id == item_id => Some(app),
            _ => None,
        }) else {
            return false;
        };
        if item.process_name_mode != ProcessNameMode::Auto || item.process_name == process_name {
            return false;
        }
        item.process_name = process_name.to_owned();
        true
    }

    fn push_entry(
        &mut self,
        kind: TimelineKind,
        item_id: Option<&str>,
        error: Option<&AppError>,
        detail: Option<String>,
    ) -> TimelineEntry {
        let entry = TimelineEntry {
            timestamp_ms: now_ms(),
            kind,
            item_id: item_id.map(str::to_owned),
            item_name: item_id.and_then(|id| self.item_name(id)),
            error: error.map(Into::into),
            detail,
        };
        self.log.entries.push(entry.clone());
        entry
    }

    pub fn finish(&mut self) -> TimelineEntry {
        let entry = self.record(TimelineKind::SessionEnded, None, None);
        self.log.ended_at_ms = Some(entry.timestamp_ms);
        entry
    }

    /// Used by "test close" without a previous "test launch": the user confirmed that every
    /// running app of the profile should be treated as launched by AutoStart.
    pub fn assume_all_launched(&mut self) {
        for runtime in &mut self.items {
            runtime.launched_by_app = true;
        }
    }

    pub fn close_targets(&self, close_only_if_launched_by_app: bool) -> Vec<CloseTarget> {
        self.profile
            .items
            .iter()
            .filter_map(|item| match item {
                LaunchItem::App(app_item) => Some(app_item),
                LaunchItem::Url(_) => None,
            })
            .filter(|app_item| {
                self.runtime(&app_item.id).is_some_and(|runtime| {
                    runtime.launched_by_app || !close_only_if_launched_by_app
                })
            })
            .map(CloseTarget::from_app_item)
            .collect()
    }

    fn runtime(&self, item_id: &str) -> Option<&ItemRuntime> {
        self.items.iter().find(|runtime| runtime.item_id == item_id)
    }

    fn item_name(&self, item_id: &str) -> Option<String> {
        self.profile
            .items
            .iter()
            .find(|item| item.id() == item_id)
            .map(|item| item.name().to_owned())
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AppItem, OnClose, ProcessNameMode, Trigger, UrlItem};

    fn app(name: &str, on_close: OnClose, enabled: bool) -> LaunchItem {
        LaunchItem::App(AppItem {
            id: name.into(),
            name: name.into(),
            exe_path: format!("C:\\{name}.exe"),
            args: None,
            working_dir: None,
            process_name: format!("{name}.exe"),
            process_name_mode: ProcessNameMode::Auto,
            icon_base64: None,
            delay_ms: 0,
            run_as_admin: false,
            start_minimized: false,
            wait_for_sim_connect: false,
            on_close,
            enabled,
        })
    }

    fn profile() -> Profile {
        Profile {
            id: "profile".into(),
            name: "MSFS".into(),
            trigger: Trigger {
                process_name: "FlightSimulator2024.exe".into(),
                label: "MSFS 2024".into(),
            },
            items: vec![
                app("launched", OnClose::Graceful, true),
                app("preexisting", OnClose::Graceful, true),
                app("kept", OnClose::Keep, true),
                app("disabled", OnClose::Force, false),
                LaunchItem::Url(UrlItem {
                    id: "site".into(),
                    name: "Site".into(),
                    url: "https://example.com".into(),
                    delay_ms: 0,
                    enabled: true,
                }),
            ],
            enabled: true,
        }
    }

    fn runtime(item_id: &str, status: ItemStatus, launched_by_app: bool) -> ItemRuntime {
        ItemRuntime {
            item_id: item_id.into(),
            status,
            launched_by_app,
            error: None,
        }
    }

    fn session_after_launch() -> Session {
        let mut session = Session::start(profile(), false);
        session.update(runtime("launched", ItemStatus::Running, true));
        session.update(runtime("preexisting", ItemStatus::Skipped, false));
        session.update(runtime("kept", ItemStatus::Running, true));
        session.update(runtime("site", ItemStatus::Running, true));
        session
    }

    fn target_ids(targets: &[CloseTarget]) -> Vec<&str> {
        targets
            .iter()
            .map(|target| target.item_id.as_str())
            .collect()
    }

    #[test]
    fn start_tracks_only_enabled_items_as_pending() {
        let session = Session::start(profile(), false);

        let ids: Vec<&str> = session.items().iter().map(|r| r.item_id.as_str()).collect();
        assert_eq!(ids, ["launched", "preexisting", "kept", "site"]);
        assert!(session
            .items()
            .iter()
            .all(|r| r.status == ItemStatus::Pending));
        assert_eq!(session.log().entries[0].kind, TimelineKind::SessionStarted);
    }

    #[test]
    fn close_only_launched_skips_preexisting_urls_and_disabled_items() {
        let targets = session_after_launch().close_targets(true);

        assert_eq!(target_ids(&targets), ["launched", "kept"]);
    }

    #[test]
    fn closing_everything_still_ignores_urls_and_disabled_items() {
        let targets = session_after_launch().close_targets(false);

        assert_eq!(target_ids(&targets), ["launched", "preexisting", "kept"]);
    }

    #[test]
    fn closing_status_keeps_launched_flag() {
        let mut session = session_after_launch();
        session.update(runtime("launched", ItemStatus::Closing, false));

        assert_eq!(
            target_ids(&session.close_targets(true)),
            ["launched", "kept"]
        );
    }

    #[test]
    fn assume_all_launched_targets_every_enabled_app() {
        let mut session = Session::start(profile(), true);
        session.assume_all_launched();

        assert_eq!(
            target_ids(&session.close_targets(true)),
            ["launched", "preexisting", "kept"]
        );
    }

    #[test]
    fn timeline_resolves_item_names_and_finish_sets_end_time() {
        let mut session = Session::start(profile(), false);
        let entry = session.record(TimelineKind::Launched, Some("launched"), None);
        session.finish();

        assert_eq!(entry.item_name.as_deref(), Some("launched"));
        assert!(session.log().ended_at_ms.is_some());
        assert_eq!(
            session.log().entries.last().map(|e| e.kind),
            Some(TimelineKind::SessionEnded)
        );
    }

    #[test]
    fn learned_name_is_used_to_close_the_session() {
        let mut session = Session::start(profile(), false);
        session.assume_all_launched();

        assert!(session.learn_process_name("launched", "Volanta.exe"));
        let target = session
            .close_targets(true)
            .into_iter()
            .find(|target| target.item_id == "launched")
            .expect("launched item is a close target");
        assert_eq!(target.process_name, "Volanta.exe");
    }

    #[test]
    fn learning_keeps_manual_names_and_ignores_unchanged_ones() {
        let mut manual = profile();
        if let Some(LaunchItem::App(app)) = manual.items.first_mut() {
            app.process_name_mode = ProcessNameMode::Manual;
        }
        let mut session = Session::start(manual, false);

        assert!(!session.learn_process_name("launched", "Volanta.exe"));
        assert!(!session.learn_process_name("preexisting", "preexisting.exe"));
        assert!(!session.learn_process_name("missing", "Volanta.exe"));
    }
}
