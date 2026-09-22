use std::collections::BTreeMap;
use std::ffi::OsStr;

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use crate::models::ProcessInfo;

/// Case-insensitive and extension-agnostic, so `FlightSimulator2024.exe` matches
/// `flightsimulator2024.exe` on Windows and `TextEdit.exe` matches `TextEdit` on macOS dev hosts.
pub fn normalize_process_name(name: &str) -> String {
    let lowercase = name.trim().to_lowercase();
    match lowercase.strip_suffix(".exe") {
        Some(stem) => stem.to_owned(),
        None => lowercase,
    }
}

pub struct ProcessTable {
    system: System,
}

impl ProcessTable {
    pub fn snapshot() -> Self {
        let mut table = Self {
            system: System::new(),
        };
        table.refresh();
        table
    }

    pub fn refresh(&mut self) {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing(),
        );
    }

    pub fn pids_by_name(&self, process_name: &str) -> Vec<u32> {
        let wanted = normalize_process_name(process_name);
        self.system
            .processes()
            .values()
            .filter(|process| matches_name(process.name(), &wanted))
            .map(|process| process.pid().as_u32())
            .collect()
    }

    pub fn is_running(&self, process_name: &str) -> bool {
        !self.pids_by_name(process_name).is_empty()
    }
}

pub fn is_running(process_name: &str) -> bool {
    ProcessTable::snapshot().is_running(process_name)
}

pub fn pids_by_name(process_name: &str) -> Vec<u32> {
    ProcessTable::snapshot().pids_by_name(process_name)
}

pub fn list_running() -> Vec<ProcessInfo> {
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
    );

    let mut unique: BTreeMap<String, ProcessInfo> = BTreeMap::new();
    for process in system.processes().values() {
        let name = process.name().to_string_lossy().into_owned();
        if name.is_empty() {
            continue;
        }
        unique
            .entry(name.to_lowercase())
            .or_insert_with(|| ProcessInfo {
                name,
                exe_path: process.exe().map(|path| path.display().to_string()),
            });
    }
    unique.into_values().collect()
}

fn matches_name(candidate: &OsStr, normalized_wanted: &str) -> bool {
    normalize_process_name(&candidate.to_string_lossy()) == normalized_wanted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_ignores_case_and_exe_extension() {
        assert_eq!(
            normalize_process_name("FlightSimulator2024.exe"),
            "flightsimulator2024"
        );
        assert_eq!(
            normalize_process_name("flightsimulator2024.EXE"),
            "flightsimulator2024"
        );
        assert_eq!(normalize_process_name("  TextEdit "), "textedit");
    }

    #[test]
    fn normalization_keeps_inner_dots() {
        assert_eq!(normalize_process_name("SPAD.neXt.exe"), "spad.next");
    }

    #[test]
    fn listing_is_sorted_and_unique() {
        let processes = list_running();
        let keys: Vec<String> = processes.iter().map(|p| p.name.to_lowercase()).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(keys, sorted);
    }
}
