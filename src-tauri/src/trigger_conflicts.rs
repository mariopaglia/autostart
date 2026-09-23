use std::collections::HashSet;

use crate::models::Profile;
use crate::processes::normalize_process_name;

fn trigger_names(profile: &Profile) -> HashSet<String> {
    profile
        .triggers
        .iter()
        .map(|trigger| normalize_process_name(&trigger.process_name))
        .collect()
}

pub fn shares_trigger(first: &Profile, second: &Profile) -> bool {
    !trigger_names(first).is_disjoint(&trigger_names(second))
}

/// Ids of the other enabled profiles that would watch one of the winner's triggers.
pub fn profiles_to_disable(profiles: &[Profile], winner_id: &str) -> Vec<String> {
    let Some(winner) = profiles
        .iter()
        .find(|profile| profile.id == winner_id && profile.enabled)
    else {
        return Vec::new();
    };
    profiles
        .iter()
        .filter(|profile| profile.id != winner_id && profile.enabled)
        .filter(|profile| shares_trigger(winner, profile))
        .map(|profile| profile.id.clone())
        .collect()
}

pub fn disable(profiles: &mut [Profile], ids: &[String]) {
    for profile in profiles
        .iter_mut()
        .filter(|profile| ids.contains(&profile.id))
    {
        profile.enabled = false;
    }
}

/// Keeps at most one enabled profile per trigger: the legacy active profile first, then the
/// sidebar order. Returns the ids it disabled.
pub fn resolve_conflicts(profiles: &mut [Profile], preferred_id: Option<&str>) -> Vec<String> {
    let mut order: Vec<usize> = (0..profiles.len()).collect();
    order.sort_by_key(|&index| Some(profiles[index].id.as_str()) != preferred_id);

    let mut watched = HashSet::new();
    let mut disabled = Vec::new();
    for index in order {
        let profile = &mut profiles[index];
        if !profile.enabled {
            continue;
        }
        let names = trigger_names(profile);
        if names.is_disjoint(&watched) {
            watched.extend(names);
        } else {
            profile.enabled = false;
            disabled.push(profile.id.clone());
        }
    }
    disabled
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Trigger;
    use crate::storage::example_profile;

    fn profile(name: &str, triggers: &[&str], enabled: bool) -> Profile {
        Profile {
            id: name.into(),
            name: name.into(),
            triggers: triggers
                .iter()
                .map(|process_name| Trigger {
                    process_name: (*process_name).into(),
                    label: (*process_name).into(),
                })
                .collect(),
            enabled,
            ..example_profile()
        }
    }

    fn enabled_ids(profiles: &[Profile]) -> Vec<&str> {
        profiles
            .iter()
            .filter(|profile| profile.enabled)
            .map(|profile| profile.id.as_str())
            .collect()
    }

    #[test]
    fn upgrade_keeps_the_legacy_active_profile_of_each_simulator() {
        let mut profiles = vec![
            profile("offline", &["FlightSimulator2024.exe"], true),
            profile("online", &["FlightSimulator2024.exe"], true),
            profile("xplane", &["X-Plane.exe"], true),
        ];

        let disabled = resolve_conflicts(&mut profiles, Some("online"));

        assert_eq!(disabled, ["offline"]);
        assert_eq!(enabled_ids(&profiles), ["online", "xplane"]);
    }

    #[test]
    fn upgrade_falls_back_to_sidebar_order_outside_the_active_group() {
        let mut profiles = vec![
            profile("xplane", &["X-Plane.exe"], true),
            profile("first", &["flightsimulator2024.EXE"], true),
            profile("second", &["FlightSimulator2024.exe"], true),
            profile("off", &["FlightSimulator2024.exe"], false),
        ];

        let disabled = resolve_conflicts(&mut profiles, Some("xplane"));

        assert_eq!(disabled, ["second"]);
        assert_eq!(enabled_ids(&profiles), ["xplane", "first"]);
    }

    #[test]
    fn different_simulators_are_untouched() {
        let mut profiles = vec![
            profile(
                "msfs",
                &["FlightSimulator.exe", "FlightSimulator2024.exe"],
                true,
            ),
            profile("xplane", &["X-Plane.exe"], true),
        ];

        assert!(resolve_conflicts(&mut profiles, None).is_empty());
    }

    #[test]
    fn enabling_a_profile_disables_every_enabled_profile_sharing_a_trigger() {
        let profiles = vec![
            profile(
                "both",
                &["FlightSimulator.exe", "FlightSimulator2024.exe"],
                true,
            ),
            profile("msfs2020", &["FlightSimulator.exe"], true),
            profile("msfs2024", &["FlightSimulator2024.exe"], true),
            profile("disabled", &["FlightSimulator.exe"], false),
            profile("xplane", &["X-Plane.exe"], true),
        ];

        assert_eq!(
            profiles_to_disable(&profiles, "both"),
            ["msfs2020", "msfs2024"]
        );
    }

    #[test]
    fn a_disabled_winner_disables_nothing() {
        let profiles = vec![
            profile("off", &["X-Plane.exe"], false),
            profile("on", &["X-Plane.exe"], true),
        ];

        assert!(profiles_to_disable(&profiles, "off").is_empty());
    }
}
