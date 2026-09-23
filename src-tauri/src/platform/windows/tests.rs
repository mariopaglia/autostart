use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

use ::windows::core::{Interface, HSTRING};
use ::windows::Win32::System::Com::{CoCreateInstance, IPersistFile, CLSCTX_INPROC_SERVER};
use ::windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
use ::windows::Win32::UI::WindowsAndMessaging::IsIconic;

use super::top_level_windows::top_level_windows;
use super::*;
use crate::app_discovery;
use crate::models::{DropRejection, DroppedCandidate};
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

/// Requires COM on the current thread, e.g. a live `ShortcutResolver`.
fn create_shortcut(shortcut: &Path, target: &Path, args: &str, working_dir: &Path) {
    // SAFETY: the caller keeps COM initialized on this thread; every string outlives its call.
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).unwrap();
        link.SetPath(&HSTRING::from(target.as_os_str())).unwrap();
        link.SetArguments(&HSTRING::from(args)).unwrap();
        link.SetWorkingDirectory(&HSTRING::from(working_dir.as_os_str()))
            .unwrap();
        link.cast::<IPersistFile>()
            .unwrap()
            .Save(&HSTRING::from(shortcut.as_os_str()), true)
            .unwrap();
    }
}

fn same_path(left: &str, right: &Path) -> bool {
    left.eq_ignore_ascii_case(&right.display().to_string())
}

#[test]
fn shortcut_resolver_reads_target_arguments_and_working_folder() {
    let resolver = ShortcutResolver::new().expect("COM is available");
    let dir = tempfile::tempdir().unwrap();
    let shortcut = dir.path().join("Notepad.lnk");
    create_shortcut(&shortcut, &notepad(), "--test", &system32());

    let target = resolver.resolve(&shortcut).expect("shortcut resolves");

    // The shell may normalize the path's casing when it stores the shortcut.
    assert!(same_path(
        &target.target_path.display().to_string(),
        &notepad()
    ));
    assert_eq!(target.args.as_deref(), Some("--test"));
    let working_dir = target.working_dir.expect("working folder kept");
    assert!(same_path(&working_dir.display().to_string(), &system32()));
    assert_eq!(resolver.resolve(&dir.path().join("Missing.lnk")), None);
}

#[test]
fn shortcut_folders_include_the_start_menu() {
    let folders = shortcut_folders();

    assert!(folders.len() >= 2);
    assert!(folders
        .iter()
        .any(|folder| folder.ends_with(r"Start Menu\Programs")));
}

#[test]
fn visible_windows_include_an_open_app() {
    let process = LaunchedProcess::start(LaunchOptions::default());

    assert!(wait_until(
        || pids_with_visible_windows().contains(&process.0)
    ));
}

#[test]
fn open_apps_exclude_apps_from_the_windows_folder() {
    let process = LaunchedProcess::start(LaunchOptions::default());
    assert!(wait_until(
        || pids_with_visible_windows().contains(&process.0)
    ));

    let open = app_discovery::list_open_apps();

    assert!(!open
        .iter()
        .any(|candidate| same_path(&candidate.exe_path, &notepad())));
    assert!(!open
        .iter()
        .any(|candidate| candidate.process_name.eq_ignore_ascii_case("svchost.exe")));
}

#[test]
fn installed_apps_are_listed_quickly_without_uninstallers() {
    let started = Instant::now();
    let installed = app_discovery::list_installed_apps();
    let elapsed = started.elapsed();

    eprintln!(
        "list_installed_apps: {} apps in {elapsed:?}",
        installed.len()
    );
    assert!(elapsed < Duration::from_secs(20), "took {elapsed:?}");
    assert!(installed
        .iter()
        .all(|candidate| !candidate.name.to_lowercase().contains("uninstall")));
}

#[test]
fn dropped_shortcut_executable_and_web_shortcut_are_resolved() {
    let _com = ShortcutResolver::new().expect("COM is available");
    let dir = tempfile::tempdir().unwrap();
    let shortcut = dir.path().join("My Editor.lnk");
    create_shortcut(&shortcut, &notepad(), "", &system32());
    let web = dir.path().join("SimBrief.url");
    std::fs::write(
        &web,
        "[InternetShortcut]\r\nURL=https://www.simbrief.com\r\n",
    )
    .unwrap();
    let document = dir.path().join("notes.txt");
    std::fs::write(&document, "hello").unwrap();

    let resolution =
        app_discovery::resolve_dropped_paths(&[shortcut, notepad(), web, document.clone()]);

    let [DroppedCandidate::App(from_shortcut), DroppedCandidate::App(from_exe), DroppedCandidate::Url(url)] =
        resolution.candidates.as_slice()
    else {
        panic!("unexpected candidates: {:?}", resolution.candidates);
    };
    assert_eq!(from_shortcut.name, "My Editor");
    assert!(same_path(&from_shortcut.exe_path, &notepad()));
    assert!(from_shortcut.icon_base64.is_some());
    assert!(same_path(&from_exe.exe_path, &notepad()));
    assert!(from_exe.process_name.eq_ignore_ascii_case("notepad.exe"));
    assert_eq!(url.url, "https://www.simbrief.com");
    assert_eq!(resolution.rejected.len(), 1);
    assert_eq!(resolution.rejected[0].path, document.display().to_string());
    assert_eq!(
        resolution.rejected[0].reason,
        DropRejection::UnsupportedFile
    );
}

/// A `cmd.exe` that stays alive for about a second, then exits with `exit_code`.
fn spawn_cmd_exiting_with(exit_code: u32) -> u32 {
    std::process::Command::new(system32().join("cmd.exe"))
        .args([
            "/c",
            &format!("ping -n 2 127.0.0.1 >nul & exit /b {exit_code}"),
        ])
        .spawn()
        .unwrap()
        .id()
}

fn watch_until_exited(pid: u32) -> Option<u32> {
    let mut watcher = ExitWatcher::default();
    watcher.watch(pid);
    assert_eq!(
        watcher.exit_codes().len(),
        1,
        "the process should be watchable"
    );
    assert!(wait_until(|| watcher.exit_codes() != [None]));
    watcher.exit_codes()[0]
}

#[test]
fn exit_watcher_reads_normal_and_failing_exit_codes() {
    let normal = spawn_cmd_exiting_with(0);
    let failing = spawn_cmd_exiting_with(3);

    assert_eq!(watch_until_exited(normal), Some(0));
    assert_eq!(watch_until_exited(failing), Some(3));
}

#[test]
fn exit_watcher_sees_a_killed_process_as_failing() {
    let process = LaunchedProcess::start(LaunchOptions::default());
    let mut watcher = ExitWatcher::default();
    watcher.watch(process.0);
    assert_eq!(watcher.exit_codes(), [None]);

    terminate(process.0, "notepad.exe").unwrap();

    assert!(wait_until(|| watcher.exit_codes() != [None]));
    assert_ne!(watcher.exit_codes(), [Some(0)]);
}
