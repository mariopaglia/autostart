use crate::models::MonitorState;

/// Consecutive polls without the trigger before the session ends; absorbs brief restarts.
const MISSED_TICKS_TO_CLOSE: u8 = 2;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Idle,
    SimRunning {
        missed_ticks: u8,
    },
    Closing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    TriggerSeen,
    TriggerMissing,
    ClosingFinished,
    Pause,
    Resume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionAction {
    Start,
    End,
    Discard,
}

pub fn next(state: State, input: Input) -> (State, Option<SessionAction>) {
    match (state, input) {
        (State::Paused, Input::Resume) => (State::Idle, None),
        (State::Paused, _) => (State::Paused, None),
        (State::SimRunning { .. }, Input::Pause) => (State::Paused, Some(SessionAction::Discard)),
        (_, Input::Pause) => (State::Paused, None),

        (State::Idle, Input::TriggerSeen) => (
            State::SimRunning { missed_ticks: 0 },
            Some(SessionAction::Start),
        ),
        (State::SimRunning { .. }, Input::TriggerSeen) => {
            (State::SimRunning { missed_ticks: 0 }, None)
        }
        (State::SimRunning { missed_ticks }, Input::TriggerMissing) => {
            let missed_ticks = missed_ticks.saturating_add(1);
            if missed_ticks >= MISSED_TICKS_TO_CLOSE {
                (State::Closing, Some(SessionAction::End))
            } else {
                (State::SimRunning { missed_ticks }, None)
            }
        }
        (State::Closing, Input::ClosingFinished) => (State::Idle, None),

        (current, _) => (current, None),
    }
}

impl State {
    pub fn public(self) -> MonitorState {
        match self {
            Self::Idle => MonitorState::Idle,
            Self::SimRunning { .. } => MonitorState::SimRunning,
            Self::Closing => MonitorState::Closing,
            Self::Paused => MonitorState::Paused,
        }
    }

    pub fn has_session(self) -> bool {
        matches!(self, Self::SimRunning { .. } | Self::Closing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(inputs: &[Input]) -> (State, Vec<SessionAction>) {
        inputs
            .iter()
            .fold((State::Idle, Vec::new()), |(state, mut actions), &input| {
                let (next_state, action) = next(state, input);
                actions.extend(action);
                (next_state, actions)
            })
    }

    #[test]
    fn trigger_start_begins_session() {
        let (state, actions) = run(&[Input::TriggerMissing, Input::TriggerSeen]);

        assert_eq!(state, State::SimRunning { missed_ticks: 0 });
        assert_eq!(actions, vec![SessionAction::Start]);
    }

    #[test]
    fn two_consecutive_missing_polls_end_session() {
        let (state, actions) = run(&[
            Input::TriggerSeen,
            Input::TriggerMissing,
            Input::TriggerMissing,
        ]);

        assert_eq!(state, State::Closing);
        assert_eq!(actions, vec![SessionAction::Start, SessionAction::End]);
    }

    #[test]
    fn momentary_disappearance_keeps_session() {
        let (state, actions) = run(&[
            Input::TriggerSeen,
            Input::TriggerMissing,
            Input::TriggerSeen,
            Input::TriggerMissing,
        ]);

        assert_eq!(state, State::SimRunning { missed_ticks: 1 });
        assert_eq!(actions, vec![SessionAction::Start]);
    }

    #[test]
    fn trigger_back_during_closing_restarts_only_after_closing_finishes() {
        let closing = [
            Input::TriggerSeen,
            Input::TriggerMissing,
            Input::TriggerMissing,
        ];
        let (state, _) = run(&closing);
        let (state, action) = next(state, Input::TriggerSeen);
        assert_eq!((state, action), (State::Closing, None));

        let (state, _) = next(state, Input::ClosingFinished);
        let (state, action) = next(state, Input::TriggerSeen);

        assert_eq!(state, State::SimRunning { missed_ticks: 0 });
        assert_eq!(action, Some(SessionAction::Start));
    }

    #[test]
    fn pausing_during_session_discards_it_and_ignores_trigger() {
        let (state, actions) = run(&[
            Input::TriggerSeen,
            Input::Pause,
            Input::TriggerMissing,
            Input::TriggerMissing,
        ]);

        assert_eq!(state, State::Paused);
        assert_eq!(actions, vec![SessionAction::Start, SessionAction::Discard]);
    }

    #[test]
    fn resuming_with_simulator_open_starts_new_session() {
        let (state, actions) = run(&[Input::Pause, Input::Resume, Input::TriggerSeen]);

        assert_eq!(state, State::SimRunning { missed_ticks: 0 });
        assert_eq!(actions, vec![SessionAction::Start]);
    }

    #[test]
    fn app_started_with_simulator_already_open_starts_session_on_first_poll() {
        let (state, action) = next(State::default(), Input::TriggerSeen);

        assert_eq!(state.public(), MonitorState::SimRunning);
        assert_eq!(action, Some(SessionAction::Start));
    }

    #[test]
    fn closing_finished_while_paused_stays_paused() {
        let (state, action) = next(State::Paused, Input::ClosingFinished);

        assert_eq!((state, action), (State::Paused, None));
    }
}
