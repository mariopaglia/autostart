use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessSample {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub exe_path: Option<PathBuf>,
    pub start_time: u64,
}

/// What the launcher knows about one launch while it decides which process is the real app.
pub struct LaunchTrace<'a> {
    pub root_pid: Option<u32>,
    pub tracked: &'a HashSet<u32>,
    pub baseline: &'a HashSet<u32>,
    pub install_dir: &'a Path,
}

/// The set only grows, so grandchildren stay tracked after an intermediate launcher exits.
pub fn expand_tracked(tracked: &HashSet<u32>, samples: &[ProcessSample]) -> HashSet<u32> {
    let mut expanded = tracked.clone();
    loop {
        let before = expanded.len();
        for sample in samples {
            if sample
                .parent_pid
                .is_some_and(|parent| expanded.contains(&parent))
            {
                expanded.insert(sample.pid);
            }
        }
        if expanded.len() == before {
            return expanded;
        }
    }
}

pub fn any_tracked_alive(tracked: &HashSet<u32>, samples: &[ProcessSample]) -> bool {
    samples.iter().any(|sample| tracked.contains(&sample.pid))
}

pub fn choose_process_name(trace: &LaunchTrace, samples: &[ProcessSample]) -> Option<String> {
    if let Some(root) = trace
        .root_pid
        .and_then(|pid| samples.iter().find(|sample| sample.pid == pid))
    {
        return Some(root.name.clone());
    }

    let descendant = oldest(
        samples
            .iter()
            .filter(|sample| trace.tracked.contains(&sample.pid)),
    );
    let fallback = || {
        oldest(samples.iter().filter(|sample| {
            !trace.baseline.contains(&sample.pid)
                && sample
                    .exe_path
                    .as_deref()
                    .is_some_and(|exe| is_inside(exe, trace.install_dir))
        }))
    };
    descendant
        .or_else(fallback)
        .map(|sample| sample.name.clone())
}

fn oldest<'a>(candidates: impl Iterator<Item = &'a ProcessSample>) -> Option<&'a ProcessSample> {
    candidates.min_by(|left, right| {
        left.start_time
            .cmp(&right.start_time)
            .then_with(|| left.name.cmp(&right.name))
    })
}

// Windows paths are case-insensitive, and the executable path reported by the OS may differ in case.
fn is_inside(path: &Path, dir: &Path) -> bool {
    let lowercase = |value: &Path| PathBuf::from(value.to_string_lossy().to_lowercase());
    lowercase(path).starts_with(lowercase(dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    const INSTALL_DIR: &str = "/apps/volanta";

    fn sample(pid: u32, parent_pid: Option<u32>, name: &str, start_time: u64) -> ProcessSample {
        ProcessSample {
            pid,
            parent_pid,
            name: name.into(),
            exe_path: Some(PathBuf::from(format!("/elsewhere/{name}"))),
            start_time,
        }
    }

    fn in_install_dir(mut sample: ProcessSample) -> ProcessSample {
        sample.exe_path = Some(PathBuf::from(INSTALL_DIR).join("BIN").join(&sample.name));
        sample
    }

    fn choose(
        root_pid: u32,
        baseline: &[u32],
        history: &[&[ProcessSample]],
        now: &[ProcessSample],
    ) -> Option<String> {
        let mut tracked = HashSet::from([root_pid]);
        for samples in history {
            tracked = expand_tracked(&tracked, samples);
        }
        let baseline = baseline.iter().copied().collect();
        let trace = LaunchTrace {
            root_pid: Some(root_pid),
            tracked: &tracked,
            baseline: &baseline,
            install_dir: Path::new(INSTALL_DIR),
        };
        choose_process_name(&trace, now)
    }

    #[test]
    fn keeps_the_root_process_while_it_runs() {
        let now = [
            sample(10, Some(1), "Spad.exe", 5),
            sample(11, Some(10), "Helper.exe", 6),
        ];
        assert_eq!(choose(10, &[], &[&now], &now), Some("Spad.exe".into()));
    }

    #[test]
    fn learns_the_child_when_the_launcher_exits() {
        let during = [
            sample(10, Some(1), "Launcher.exe", 5),
            sample(11, Some(10), "Volanta.exe", 6),
        ];
        let now = [sample(11, Some(10), "Volanta.exe", 6)];
        assert_eq!(
            choose(10, &[], &[&during], &now),
            Some("Volanta.exe".into())
        );
    }

    #[test]
    fn keeps_tracking_a_grandchild_after_the_intermediate_process_exits() {
        let first = [
            sample(10, Some(1), "Launcher.exe", 5),
            sample(11, Some(10), "Updater.exe", 6),
        ];
        let second = [
            sample(11, Some(10), "Updater.exe", 6),
            sample(12, Some(11), "App.exe", 7),
        ];
        let now = [sample(12, Some(11), "App.exe", 7)];
        assert_eq!(
            choose(10, &[], &[&first, &second], &now),
            Some("App.exe".into())
        );
    }

    #[test]
    fn falls_back_to_a_new_process_in_the_install_dir() {
        let now = [
            in_install_dir(sample(20, Some(2), "Volanta.exe", 8)),
            sample(21, Some(2), "Other.exe", 7),
        ];
        assert_eq!(choose(10, &[], &[&now], &now), Some("Volanta.exe".into()));
    }

    #[test]
    fn ignores_install_dir_processes_that_existed_before_launch() {
        let now = [in_install_dir(sample(20, Some(2), "Volanta.exe", 1))];
        assert_eq!(choose(10, &[20], &[&now], &now), None);
    }

    #[test]
    fn prefers_the_oldest_descendant_and_breaks_ties_by_name() {
        let during = [
            sample(10, Some(1), "Launcher.exe", 5),
            sample(11, Some(10), "Zeta.exe", 6),
            sample(12, Some(10), "Alpha.exe", 6),
            sample(13, Some(10), "Early.exe", 3),
        ];
        let now = &during[1..];
        assert_eq!(choose(10, &[], &[&during], now), Some("Early.exe".into()));
        let tied = &during[1..3];
        assert_eq!(choose(10, &[], &[&during], tied), Some("Alpha.exe".into()));
    }

    #[test]
    fn returns_none_without_candidates() {
        let now = [sample(30, Some(1), "Unrelated.exe", 9)];
        assert_eq!(choose(10, &[], &[&now], &now), None);
    }

    #[test]
    fn detects_tracked_processes_still_alive() {
        let tracked = HashSet::from([10]);
        assert!(any_tracked_alive(&tracked, &[sample(10, None, "A.exe", 1)]));
        assert!(!any_tracked_alive(
            &tracked,
            &[sample(11, None, "B.exe", 1)]
        ));
    }
}
