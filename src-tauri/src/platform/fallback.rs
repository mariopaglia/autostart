//! Development-only stand-ins so the app builds and the UI runs on non-Windows hosts.

use std::path::Path;
use std::process::Command;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, Signal, System};

use crate::error::{AppError, AppResult};

pub fn product_name(_exe_path: &Path) -> Option<String> {
    None
}

pub fn icon_png_base64(_exe_path: &Path) -> Option<String> {
    None
}

pub fn configure_launch(command: &mut Command, args: Option<&str>) {
    if let Some(args) = args {
        command.args(args.split_whitespace());
    }
}

pub fn launch_elevated(
    _exe_path: &Path,
    _args: Option<&str>,
    _working_dir: &Path,
) -> AppResult<()> {
    Err(AppError::Unsupported("run as administrator"))
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
