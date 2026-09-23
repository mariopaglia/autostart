use std::collections::HashSet;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, Instant};

use ::windows::Win32::UI::WindowsAndMessaging::IsIconic;

use super::top_level_windows::top_level_windows;
use super::*;
use crate::platform::LaunchOptions;
use crate::processes;

const TIMEOUT: Duration = Duration::from_secs(15);

fn system32() -> PathBuf {
    let root = std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into());
    PathBuf::from(root).join("System32")
}

fn notepad() -> PathBuf {
    system32().join("notepad.exe")
}

fn wait_until(mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + TIMEOUT;
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        sleep(Duration::from_millis(200));
    }
    false
}

fn has_minimized_window(pid: u32) -> bool {
    top_level_windows(&HashSet::from([pid]))
        .into_iter()
        // SAFETY: querying the state of a window handle has no memory-safety requirements.
        .any(|window| unsafe { IsIconic(window) }.as_bool())
}

fn is_alive(pid: u32) -> bool {
    processes::running_pids().contains(&pid)
}

/// Terminates the launched process even when an assertion fails, so no window outlives the test.
struct LaunchedProcess(u32);

impl LaunchedProcess {
    fn start(options: LaunchOptions) -> Self {
        let pid = launch(&notepad(), None, &system32(), options)
            .unwrap()
            .expect("notepad should start a new process");
        let process = Self(pid);
        assert!(
            wait_until(|| !top_level_windows(&HashSet::from([pid])).is_empty()),
            "notepad should open a window"
        );
        process
    }
}

impl Drop for LaunchedProcess {
    fn drop(&mut self) {
        if is_alive(self.0) {
            let _ = terminate(self.0, "notepad.exe");
        }
    }
}

#[test]
fn launched_process_is_seen_by_the_process_monitor() {
    let process = LaunchedProcess::start(LaunchOptions::default());

    assert!(is_alive(process.0));
    assert!(processes::pids_by_name("Notepad.exe").contains(&process.0));
}

#[test]
fn request_close_ends_a_process_gracefully() {
    let process = LaunchedProcess::start(LaunchOptions::default());

    assert!(request_close(process.0) > 0);
    assert!(wait_until(|| !is_alive(process.0)));
}

#[test]
fn terminate_ends_a_process() {
    let process = LaunchedProcess::start(LaunchOptions::default());

    terminate(process.0, "notepad.exe").unwrap();
    assert!(wait_until(|| !is_alive(process.0)));
}

#[test]
fn minimized_launch_opens_a_minimized_window() {
    let process = LaunchedProcess::start(LaunchOptions {
        minimized: true,
        ..LaunchOptions::default()
    });

    assert!(wait_until(|| has_minimized_window(process.0)));
}

#[test]
fn minimize_new_windows_minimizes_visible_windows() {
    let process = LaunchedProcess::start(LaunchOptions::default());
    let pids = HashSet::from([process.0]);
    let mut already_minimized = HashSet::new();

    assert!(wait_until(|| {
        minimize_new_windows(&pids, &mut already_minimized);
        !already_minimized.is_empty()
    }));
    assert!(wait_until(|| has_minimized_window(process.0)));
}

#[test]
fn exe_info_reads_product_name_and_icon() {
    let name = product_name(&notepad()).expect("notepad has version info");
    assert!(!name.is_empty());

    let icon = icon_png_base64(&notepad()).expect("notepad has an icon");
    assert!(
        icon.starts_with("iVBORw0KGgo"),
        "icon should be a base64 PNG"
    );
}

#[test]
fn simconnect_is_unavailable_without_a_simulator() {
    assert!(!is_simconnect_available());
}
