use std::time::Duration;

use crate::models::{Profile, Trigger};

/// The first enabled profile, in sidebar order, with one of its triggers running.
pub fn first_running_profile(
    profiles: &[Profile],
    is_running: impl Fn(&str) -> bool,
) -> Option<(Profile, Trigger)> {
    profiles
        .iter()
        .filter(|profile| profile.enabled)
        .find_map(|profile| {
            profile
                .triggers
                .iter()
                .find(|trigger| is_running(&trigger.process_name))
                .map(|trigger| (profile.clone(), trigger.clone()))
        })
}

pub fn any_trigger_running(profile: &Profile, is_running: impl Fn(&str) -> bool) -> bool {
    profile
        .triggers
        .iter()
        .any(|trigger| is_running(&trigger.process_name))
}

/// While a flight starts, only the simulator the pilot picked counts as its arrival.
pub fn session_simulator_running(
    profile: &Profile,
    awaited: Option<&Trigger>,
    is_running: impl Fn(&str) -> bool,
) -> bool {
    match awaited {
        Some(trigger) => is_running(&trigger.process_name),
        None => any_trigger_running(profile, is_running),
    }
}

pub fn close_delay_ticks(close_delay_ms: u32, poll_interval: Duration) -> u32 {
    let poll_ms = u32::try_from(poll_interval.as_millis()).unwrap_or(u32::MAX);
    close_delay_ms.div_ceil(poll_ms.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::example_profile;

    fn profile(name: &str, triggers: &[&str], enabled: bool) -> Profile {
        Profile {
            name: name.into(),
            triggers: triggers
                .iter()
                .map(|process_name| Trigger {
                    process_name: (*process_name).into(),
                    label: (*process_name).into(),
                    launch_target: None,
                })
                .collect(),
            enabled,
            ..example_profile()
        }
    }

    fn running(names: &'static [&'static str]) -> impl Fn(&str) -> bool {
        move |name| names.contains(&name)
    }

    fn selected(profiles: &[Profile], names: &'static [&'static str]) -> Option<(String, String)> {
        first_running_profile(profiles, running(names))
            .map(|(profile, trigger)| (profile.name, trigger.process_name))
    }

    #[test]
    fn picks_the_profile_of_the_running_simulator() {
        let profiles = [
            profile("MSFS", &["FlightSimulator2024.exe"], true),
            profile("X-Plane", &["X-Plane.exe"], true),
        ];

        assert_eq!(
            selected(&profiles, &["X-Plane.exe"]),
            Some(("X-Plane".into(), "X-Plane.exe".into()))
        );
        assert_eq!(selected(&profiles, &[]), None);
    }

    #[test]
    fn ignores_disabled_profiles() {
        let profiles = [profile("X-Plane", &["X-Plane.exe"], false)];

        assert_eq!(selected(&profiles, &["X-Plane.exe"]), None);
    }

    #[test]
    fn matches_any_trigger_and_remembers_which_one() {
        let profiles = [profile(
            "MSFS",
            &["FlightSimulator.exe", "FlightSimulator2024.exe"],
            true,
        )];

        assert_eq!(
            selected(&profiles, &["FlightSimulator.exe"]),
            Some(("MSFS".into(), "FlightSimulator.exe".into()))
        );
        assert!(any_trigger_running(
            &profiles[0],
            running(&["FlightSimulator2024.exe"])
        ));
    }

    #[test]
    fn two_simulators_at_once_pick_the_higher_profile() {
        let profiles = [
            profile("X-Plane", &["X-Plane.exe"], true),
            profile("MSFS", &["FlightSimulator2024.exe"], true),
        ];

        let names: &[&str] = &["FlightSimulator2024.exe", "X-Plane.exe"];
        let picked = first_running_profile(&profiles, |name| names.contains(&name));
        assert_eq!(
            picked.map(|(profile, _)| profile.name).as_deref(),
            Some("X-Plane")
        );
    }

    #[test]
    fn a_starting_flight_waits_only_for_the_chosen_simulator() {
        let both = profile(
            "MSFS",
            &["FlightSimulator.exe", "FlightSimulator2024.exe"],
            true,
        );
        let msfs_2024 = both.triggers[1].clone();
        let only_2020 = running(&["FlightSimulator.exe"]);

        assert!(!session_simulator_running(
            &both,
            Some(&msfs_2024),
            &only_2020
        ));
        assert!(session_simulator_running(&both, None, &only_2020));
        assert!(session_simulator_running(
            &both,
            Some(&msfs_2024),
            running(&["FlightSimulator2024.exe"])
        ));
    }

    #[test]
    fn close_delay_rounds_up_to_whole_polls() {
        let poll = Duration::from_secs(2);

        assert_eq!(close_delay_ticks(0, poll), 0);
        assert_eq!(close_delay_ticks(1_000, poll), 1);
        assert_eq!(close_delay_ticks(60_000, poll), 30);
    }
}
