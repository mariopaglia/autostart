use crate::models::SessionLog;

pub const MAX_REAL_SESSIONS: usize = 20;

/// Newest first. Only the latest test is kept, so repeated tests never push flights out.
pub fn record(history: &mut Vec<SessionLog>, log: SessionLog) {
    if log.is_test {
        history.retain(|session| !session.is_test);
    }
    history.insert(0, log);

    let mut real_sessions = 0;
    history.retain(|session| {
        if session.is_test {
            return true;
        }
        real_sessions += 1;
        real_sessions <= MAX_REAL_SESSIONS
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(started_at_ms: u64, is_test: bool) -> SessionLog {
        SessionLog {
            profile_id: "profile".into(),
            profile_name: "MSFS".into(),
            is_test,
            started_at_ms,
            ended_at_ms: Some(started_at_ms + 1),
            entries: Vec::new(),
        }
    }

    fn starts(history: &[SessionLog]) -> Vec<u64> {
        history
            .iter()
            .map(|session| session.started_at_ms)
            .collect()
    }

    #[test]
    fn newest_session_goes_first() {
        let mut history = Vec::new();
        record(&mut history, session(1, false));
        record(&mut history, session(2, false));

        assert_eq!(starts(&history), [2, 1]);
    }

    #[test]
    fn a_new_test_replaces_the_previous_test() {
        let mut history = Vec::new();
        record(&mut history, session(1, false));
        record(&mut history, session(2, true));
        record(&mut history, session(3, true));

        assert_eq!(starts(&history), [3, 1]);
    }

    #[test]
    fn a_real_session_keeps_the_latest_test() {
        let mut history = Vec::new();
        record(&mut history, session(1, true));
        record(&mut history, session(2, false));

        assert_eq!(starts(&history), [2, 1]);
    }

    #[test]
    fn keeps_only_the_most_recent_real_sessions() {
        let mut history = Vec::new();
        record(&mut history, session(0, true));
        for started_at_ms in 1..=21 {
            record(&mut history, session(started_at_ms, false));
        }

        let real: Vec<u64> = history
            .iter()
            .filter(|session| !session.is_test)
            .map(|session| session.started_at_ms)
            .collect();
        assert_eq!(real.len(), MAX_REAL_SESSIONS);
        assert_eq!(real.first(), Some(&21));
        assert_eq!(real.last(), Some(&2));
        assert!(history.iter().any(|session| session.is_test));
    }
}
