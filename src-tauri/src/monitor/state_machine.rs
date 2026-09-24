use crate::models::MonitorState;

/// Consecutive polls without the trigger before the session ends; absorbs brief restarts.
const MISSED_TICKS_TO_CLOSE: u8 = 2;
/// Five minutes of 2-second polls for a simulator started with "Start flight" to show up.
pub const SIMULATOR_START_TICKS: u32 = 150;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Idle,
    /// A flight started from AutoStart, waiting for the chosen simulator to show up.
    SimStarting {
        remaining_ticks: u32,
    },
    SimRunning {
        missed_ticks: u8,
    },
    /// Waiting before closing, so a simulator restarted after a crash keeps its apps.
    ClosePending {
        remaining_ticks: u32,
    },
    Closing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    StartFlight,
    /// The simulator could not be started, or the user cancelled the start.
    SimulatorFailed,
    TriggerSeen,
    TriggerMissing,
    ClosingFinished,
    CloseNow,
    KeepApps,
    Pause,
    Resume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionAction {
    Start,
    SimulatorArrived,
    /// The simulator never showed up; the session closes like after a simulator exit.
    StartFailed,
    DelayClose,
    Continue,
    End,
    Release,
    Discard,
}

/// `close_delay_ticks` is the configured close delay in polls; 0 closes right away.
pub fn next(state: State, input: Input, close_delay_ticks: u32) -> (State, Option<SessionAction>) {
    match (state, input) {
        (State::Paused, Input::Resume) => (State::Idle, None),
        (State::Paused, _) => (State::Paused, None),
        (
            State::SimStarting { .. } | State::SimRunning { .. } | State::ClosePending { .. },
            Input::Pause,
        ) => (State::Paused, Some(SessionAction::Discard)),
        (_, Input::Pause) => (State::Paused, None),

        (State::Idle, Input::StartFlight) => (
            State::SimStarting {
                remaining_ticks: SIMULATOR_START_TICKS,
            },
            Some(SessionAction::Start),
        ),
        (State::SimStarting { .. }, Input::TriggerSeen) => (
            State::SimRunning { missed_ticks: 0 },
            Some(SessionAction::SimulatorArrived),
        ),
        (State::SimStarting { remaining_ticks }, Input::TriggerMissing) if remaining_ticks > 1 => (
            State::SimStarting {
                remaining_ticks: remaining_ticks - 1,
            },
            None,
        ),
        (State::SimStarting { .. }, Input::TriggerMissing | Input::SimulatorFailed) => (
            closing_state(close_delay_ticks),
            Some(SessionAction::StartFailed),
        ),

        (State::Idle, Input::TriggerSeen) => (
            State::SimRunning { missed_ticks: 0 },
            Some(SessionAction::Start),
        ),
        (State::SimRunning { .. }, Input::TriggerSeen) => {
            (State::SimRunning { missed_ticks: 0 }, None)
        }
        (State::SimRunning { missed_ticks }, Input::TriggerMissing) => {
            let missed_ticks = missed_ticks.saturating_add(1);
            if missed_ticks < MISSED_TICKS_TO_CLOSE {
                (State::SimRunning { missed_ticks }, None)
            } else if close_delay_ticks == 0 {
                (State::Closing, Some(SessionAction::End))
            } else {
                (
                    State::ClosePending {
                        remaining_ticks: close_delay_ticks,
                    },
                    Some(SessionAction::DelayClose),
                )
            }
        }

        (State::ClosePending { .. }, Input::TriggerSeen) => (
            State::SimRunning { missed_ticks: 0 },
            Some(SessionAction::Continue),
        ),
        (State::ClosePending { remaining_ticks }, Input::TriggerMissing) => {
            if remaining_ticks <= 1 {
                (State::Closing, Some(SessionAction::End))
            } else {
                (
                    State::ClosePending {
                        remaining_ticks: remaining_ticks - 1,
                    },
                    None,
                )
            }
        }
        (State::ClosePending { .. }, Input::CloseNow) => (State::Closing, Some(SessionAction::End)),
        (State::ClosePending { .. }, Input::KeepApps) => {
            (State::Idle, Some(SessionAction::Release))
        }

        (State::Closing, Input::ClosingFinished) => (State::Idle, None),

        (current, _) => (current, None),
    }
}

fn closing_state(close_delay_ticks: u32) -> State {
    if close_delay_ticks == 0 {
        State::Closing
    } else {
        State::ClosePending {
            remaining_ticks: close_delay_ticks,
        }
    }
}

impl State {
    pub fn public(self) -> MonitorState {
        match self {
            Self::Idle => MonitorState::Idle,
            Self::SimStarting { .. } => MonitorState::SimStarting,
            Self::SimRunning { .. } => MonitorState::SimRunning,
            Self::ClosePending { .. } => MonitorState::ClosePending,
            Self::Closing => MonitorState::Closing,
            Self::Paused => MonitorState::Paused,
        }
    }

    pub fn has_session(self) -> bool {
        matches!(
            self,
            Self::SimStarting { .. }
                | Self::SimRunning { .. }
                | Self::ClosePending { .. }
                | Self::Closing
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NO_DELAY: u32 = 0;
    const DELAY: u32 = 3;

    fn step(state: State, input: Input) -> (State, Option<SessionAction>) {
        next(state, input, NO_DELAY)
    }

    fn run_with_delay(delay: u32, inputs: &[Input]) -> (State, Vec<SessionAction>) {
        inputs
            .iter()
            .fold((State::Idle, Vec::new()), |(state, mut actions), &input| {
                let (next_state, action) = next(state, input, delay);
                actions.extend(action);
                (next_state, actions)
            })
    }

    fn run(inputs: &[Input]) -> (State, Vec<SessionAction>) {
        run_with_delay(NO_DELAY, inputs)
    }

    const SIM_EXITED: [Input; 3] = [
        Input::TriggerSeen,
        Input::TriggerMissing,
        Input::TriggerMissing,
    ];

    #[test]
    fn simulator_exit_with_delay_waits_before_closing() {
        let (state, actions) = run_with_delay(DELAY, &SIM_EXITED);

        assert_eq!(state, State::ClosePending { remaining_ticks: 3 });
        assert_eq!(state.public(), MonitorState::ClosePending);
        assert!(state.has_session());
        assert_eq!(actions, [SessionAction::Start, SessionAction::DelayClose]);
    }

    #[test]
    fn delay_expires_after_its_ticks_and_ends_the_session() {
        let inputs = [SIM_EXITED.as_slice(), &[Input::TriggerMissing; 3]].concat();
        let (state, actions) = run_with_delay(DELAY, &inputs);

        assert_eq!(state, State::Closing);
        assert_eq!(
            actions,
            [
                SessionAction::Start,
                SessionAction::DelayClose,
                SessionAction::End
            ]
        );
    }

    #[test]
    fn simulator_back_during_delay_continues_the_session() {
        let inputs = [
            SIM_EXITED.as_slice(),
            &[Input::TriggerMissing, Input::TriggerSeen],
        ]
        .concat();
        let (state, actions) = run_with_delay(DELAY, &inputs);

        assert_eq!(state, State::SimRunning { missed_ticks: 0 });
        assert_eq!(
            actions,
            [
                SessionAction::Start,
                SessionAction::DelayClose,
                SessionAction::Continue
            ]
        );
    }

    #[test]
    fn close_now_ends_the_delay_immediately() {
        let inputs = [SIM_EXITED.as_slice(), &[Input::CloseNow]].concat();
        let (state, actions) = run_with_delay(DELAY, &inputs);

        assert_eq!(state, State::Closing);
        assert_eq!(actions.last(), Some(&SessionAction::End));
    }

    #[test]
    fn keep_apps_releases_the_session_without_closing() {
        let inputs = [SIM_EXITED.as_slice(), &[Input::KeepApps]].concat();
        let (state, actions) = run_with_delay(DELAY, &inputs);

        assert_eq!(state, State::Idle);
        assert_eq!(actions.last(), Some(&SessionAction::Release));
    }

    #[test]
    fn pausing_during_delay_discards_the_session() {
        let inputs = [SIM_EXITED.as_slice(), &[Input::Pause]].concat();
        let (state, actions) = run_with_delay(DELAY, &inputs);

        assert_eq!(state, State::Paused);
        assert_eq!(actions.last(), Some(&SessionAction::Discard));
    }

    #[test]
    fn close_now_and_keep_apps_are_ignored_outside_the_delay() {
        let running = State::SimRunning { missed_ticks: 0 };

        assert_eq!(step(running, Input::CloseNow), (running, None));
        assert_eq!(step(State::Idle, Input::KeepApps), (State::Idle, None));
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
        let (state, action) = step(state, Input::TriggerSeen);
        assert_eq!((state, action), (State::Closing, None));

        let (state, _) = step(state, Input::ClosingFinished);
        let (state, action) = step(state, Input::TriggerSeen);

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
        let (state, action) = step(State::default(), Input::TriggerSeen);

        assert_eq!(state.public(), MonitorState::SimRunning);
        assert_eq!(action, Some(SessionAction::Start));
    }

    #[test]
    fn closing_finished_while_paused_stays_paused() {
        let (state, action) = step(State::Paused, Input::ClosingFinished);

        assert_eq!((state, action), (State::Paused, None));
    }

    const STARTING: State = State::SimStarting {
        remaining_ticks: SIMULATOR_START_TICKS,
    };

    #[test]
    fn start_flight_waits_for_the_simulator() {
        let (state, action) = step(State::Idle, Input::StartFlight);

        assert_eq!(state, STARTING);
        assert_eq!(state.public(), MonitorState::SimStarting);
        assert!(state.has_session());
        assert_eq!(action, Some(SessionAction::Start));
    }

    #[test]
    fn start_flight_is_ignored_outside_idle() {
        for state in [
            State::SimRunning { missed_ticks: 0 },
            State::Closing,
            State::Paused,
            STARTING,
        ] {
            assert_eq!(step(state, Input::StartFlight), (state, None));
        }
    }

    #[test]
    fn simulator_showing_up_runs_the_session() {
        let (state, action) = step(STARTING, Input::TriggerSeen);

        assert_eq!(state, State::SimRunning { missed_ticks: 0 });
        assert_eq!(action, Some(SessionAction::SimulatorArrived));
    }

    #[test]
    fn missing_simulator_counts_down_while_starting() {
        let (state, action) = step(STARTING, Input::TriggerMissing);

        assert_eq!(
            state,
            State::SimStarting {
                remaining_ticks: SIMULATOR_START_TICKS - 1
            }
        );
        assert_eq!(action, None);
    }

    #[test]
    fn simulator_that_never_shows_up_follows_the_close_delay() {
        let last_tick = State::SimStarting { remaining_ticks: 1 };

        assert_eq!(
            next(last_tick, Input::TriggerMissing, DELAY),
            (
                State::ClosePending {
                    remaining_ticks: DELAY
                },
                Some(SessionAction::StartFailed)
            )
        );
        assert_eq!(
            next(last_tick, Input::TriggerMissing, NO_DELAY),
            (State::Closing, Some(SessionAction::StartFailed))
        );
    }

    #[test]
    fn failed_or_cancelled_start_follows_the_close_delay_right_away() {
        assert_eq!(
            next(STARTING, Input::SimulatorFailed, DELAY),
            (
                State::ClosePending {
                    remaining_ticks: DELAY
                },
                Some(SessionAction::StartFailed)
            )
        );
        assert_eq!(
            next(STARTING, Input::SimulatorFailed, NO_DELAY),
            (State::Closing, Some(SessionAction::StartFailed))
        );
    }

    #[test]
    fn late_simulator_during_the_close_delay_continues_the_session() {
        let (pending, _) = next(STARTING, Input::SimulatorFailed, DELAY);

        assert_eq!(
            next(pending, Input::TriggerSeen, DELAY),
            (
                State::SimRunning { missed_ticks: 0 },
                Some(SessionAction::Continue)
            )
        );
    }

    #[test]
    fn pausing_while_starting_discards_the_session() {
        assert_eq!(
            step(STARTING, Input::Pause),
            (State::Paused, Some(SessionAction::Discard))
        );
    }

    #[test]
    fn simulator_failure_is_ignored_outside_starting() {
        let running = State::SimRunning { missed_ticks: 0 };

        assert_eq!(step(running, Input::SimulatorFailed), (running, None));
        assert_eq!(
            step(State::Idle, Input::SimulatorFailed),
            (State::Idle, None)
        );
    }
}
