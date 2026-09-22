use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use tauri::{AppHandle, Emitter, Manager};

use super::session::Session;
use crate::models::{ItemRuntime, MonitorSnapshot, MonitorState, SessionLog, TimelineKind};
use crate::state::AppState;

pub const STATE_EVENT: &str = "monitor://state";
pub const ITEM_STATUS_EVENT: &str = "monitor://item-status";
pub const LOG_EVENT: &str = "monitor://log";

pub type SharedSession = Arc<Mutex<Session>>;

#[derive(Default)]
struct Published {
    snapshot: MonitorSnapshot,
    session_log: Option<SessionLog>,
}

/// Single place that mutates what the UI sees: the snapshot, the session log and the events.
#[derive(Clone)]
pub struct Reporter {
    app: AppHandle,
    published: Arc<Mutex<Published>>,
}

impl Reporter {
    pub fn new(app: AppHandle, last_session_log: Option<SessionLog>) -> Self {
        let published = Published {
            snapshot: MonitorSnapshot::default(),
            session_log: last_session_log,
        };
        Self {
            app,
            published: Arc::new(Mutex::new(published)),
        }
    }

    pub fn snapshot(&self) -> MonitorSnapshot {
        self.published().snapshot.clone()
    }

    pub fn session_log(&self) -> Option<SessionLog> {
        self.published().session_log.clone()
    }

    pub fn set_state(&self, state: MonitorState) {
        let snapshot = {
            let mut published = self.published();
            published.snapshot.state = state;
            published.snapshot.clone()
        };
        self.emit(STATE_EVENT, snapshot);
    }

    pub fn attach(&self, session: &SharedSession) {
        let (snapshot, log) = {
            let session = lock(session);
            let mut published = self.published();
            published.snapshot.session_profile_id = Some(session.profile().id.clone());
            published.snapshot.is_test_session = session.is_test();
            published.snapshot.items = session.items().to_vec();
            published.session_log = Some(session.log().clone());
            (
                published.snapshot.clone(),
                session.log().entries.last().cloned(),
            )
        };
        self.emit(STATE_EVENT, snapshot);
        if let Some(entry) = log {
            self.emit(LOG_EVENT, entry);
        }
    }

    pub fn item(&self, session: &SharedSession, runtime: ItemRuntime) {
        let snapshot = {
            let mut session = lock(session);
            session.update(runtime.clone());
            let mut published = self.published();
            published.snapshot.items = session.items().to_vec();
            published.snapshot.clone()
        };
        self.emit(ITEM_STATUS_EVENT, runtime);
        self.emit(STATE_EVENT, snapshot);
    }

    pub fn timeline(
        &self,
        session: &SharedSession,
        kind: TimelineKind,
        item_id: Option<&str>,
        message: String,
    ) {
        let entry = {
            let mut session = lock(session);
            let entry = session.record(kind, item_id, message);
            self.published().session_log = Some(session.log().clone());
            entry
        };
        log::info!(
            "{kind:?} {} {}",
            entry.item_name.as_deref().unwrap_or(""),
            entry.message
        );
        self.emit(LOG_EVENT, entry);
    }

    pub fn finish(&self, session: &SharedSession) {
        let (entry, log) = {
            let mut session = lock(session);
            let entry = session.finish();
            let log = session.log().clone();
            self.published().session_log = Some(log.clone());
            (entry, log)
        };
        self.emit(LOG_EVENT, entry);
        self.app.state::<AppState>().save_last_session(&log);
    }

    fn emit<T: serde::Serialize + Clone>(&self, event: &str, payload: T) {
        if let Err(error) = self.app.emit(event, payload) {
            log::warn!("failed to emit {event}: {error}");
        }
    }

    fn published(&self) -> MutexGuard<'_, Published> {
        lock(&self.published)
    }
}

// Every critical section above is a plain field assignment, so a poisoned lock holds consistent data.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
