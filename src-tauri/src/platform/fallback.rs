//! Development-only stand-ins so the app builds and the UI runs on non-Windows hosts.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, Signal, System};

use super::{LaunchOptions, ShortcutTarget};
use crate::error::{AppError, AppResult};

pub fn product_name(_exe_path: &Path) -> Option<String> {
    None
}

pub fn icon_png_base64(_exe_path: &Path) -> Option<String> {
    None
}

/// Windows shortcuts cannot be resolved elsewhere, so dropped `.lnk` files are rejected.
pub struct ShortcutResolver;

impl ShortcutResolver {
    pub fn new() -> Option<Self> {
        Some(Self)
    }

    pub fn resolve(&self, _shortcut: &Path) -> Option<ShortcutTarget> {
        None
    }
}

pub fn shortcut_folders() -> Vec<PathBuf> {
    Vec::new()
}

pub fn pids_with_visible_windows() -> HashSet<u32> {
    HashSet::new()
}

pub fn launch(
    exe_path: &Path,
    args: Option<&str>,
    working_dir: &Path,
    options: LaunchOptions,
) -> AppResult<Option<u32>> {
    if options.elevated {
        return Err(AppError::Unsupported("run as administrator"));
    }
    Command::new(exe_path)
        .current_dir(working_dir)
        .args(args.unwrap_or_default().split_whitespace())
        .spawn()
        .map(|child| Some(child.id()))
        .map_err(|error| AppError::LaunchFailed {
            name: exe_path.display().to_string(),
            reason: error.to_string(),
        })
}

pub fn minimize_new_windows(_pids: &HashSet<u32>, _already_minimized: &mut HashSet<isize>) {}

/// Lets the SimConnect launch phase run end to end on development hosts.
pub fn is_simconnect_available() -> bool {
    true
}

/// Exit codes of foreign processes are not read here, so nothing ever looks like a crash.
#[derive(Default)]
pub struct ExitWatcher {
    pids: HashSet<u32>,
}

impl ExitWatcher {
    pub fn watch(&mut self, pid: u32) {
        self.pids.insert(pid);
    }

    pub fn exit_codes(&self) -> Vec<Option<u32>> {
        let running = crate::processes::running_pids();
        self.pids
            .iter()
            .map(|pid| if running.contains(pid) { None } else { Some(0) })
            .collect()
    }
}

pub fn is_elevated() -> bool {
    false
}

pub fn request_close(pid: u32) -> usize {
    with_process(pid, |process| {
        process.kill_with(Signal::Term).unwrap_or(false)
    })
    .map_or(0, usize::from)
}

pub fn terminate(pid: u32, process_name: &str) -> AppResult<()> {
    match with_process(pid, |process| process.kill()) {
        Some(false) => Err(AppError::AccessDenied(process_name.to_owned())),
        _ => Ok(()),
    }
}

fn with_process<T>(pid: u32, action: impl FnOnce(&sysinfo::Process) -> T) -> Option<T> {
    let pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing(),
    );
    system.process(pid).map(action)
}
