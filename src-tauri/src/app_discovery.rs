use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::exe_inspection;
use crate::models::{
    AppCandidate, CandidateSource, DropRejection, DropResolution, DroppedCandidate, RejectedDrop,
    UrlCandidate,
};
use crate::platform::{self, ShortcutTarget};
use crate::process_tracker::ProcessSample;
use crate::processes;

const MAX_SHORTCUT_DEPTH: usize = 4;

const UNINSTALL_WORDS: &[&str] = &["uninstall", "desinstal", "deinstall", "désinstall"];

/// Matched against names reduced to lowercase letters and digits, so "Little Navmap" and
/// `littlenavmap.exe` both hit `littlenavmap`.
const FLIGHT_SIM_SUGGESTIONS: &[&str] = &[
    "navigraph",
    "simbrief",
    "littlenavmap",
    "volanta",
    "vpilot",
    "xpilot",
    "ivao",
    "fsuipc",
    "spadnext",
    "couatl",
    "gsx",
    "fs2crew",
    "pilot2atc",
    "beyondatc",
    "sayintentions",
    "simtoolkitpro",
    "flybywire",
    "fenix",
    "pmdg",
    "aerosoft",
    "chaseplane",
    "fsrealistic",
    "simshaker",
    "selfloadingcargo",
    "addonslinker",
];

pub fn list_installed_apps() -> Vec<AppCandidate> {
    let Some(resolver) = platform::ShortcutResolver::new() else {
        return Vec::new();
    };
    let shortcuts: Vec<PathBuf> = platform::shortcut_folders()
        .iter()
        .flat_map(|folder| find_shortcuts(folder, MAX_SHORTCUT_DEPTH))
        .collect();
    installed_candidates(
        &shortcuts,
        |shortcut| resolver.resolve(shortcut),
        windows_dir().as_deref(),
    )
}

pub fn list_open_apps() -> Vec<AppCandidate> {
    open_candidates(
        &processes::samples(),
        &platform::pids_with_visible_windows(),
        std::process::id(),
        windows_dir().as_deref(),
    )
}

pub fn resolve_dropped_paths(paths: &[PathBuf]) -> DropResolution {
    let resolver = platform::ShortcutResolver::new();
    let resolve = |shortcut: &Path| resolver.as_ref()?.resolve(shortcut);
    let mut resolution = DropResolution::default();
    for path in paths {
        match classify_dropped_path(path, resolve) {
            Ok(candidate) => resolution.candidates.push(candidate),
            Err(reason) => resolution.rejected.push(RejectedDrop {
                path: path.display().to_string(),
                reason,
            }),
        }
    }
    resolution
}

fn installed_candidates(
    shortcuts: &[PathBuf],
    resolve: impl Fn(&Path) -> Option<ShortcutTarget>,
    windows_dir: Option<&Path>,
) -> Vec<AppCandidate> {
    let candidates = shortcuts
        .iter()
        .filter(|shortcut| !is_uninstaller_name(&file_stem(shortcut)))
        .filter_map(|shortcut| {
            let target = resolve(shortcut)?;
            if is_uninstaller_exe(&target.target_path)
                || is_under_dir(&target.target_path, windows_dir)
            {
                return None;
            }
            app_from_shortcut(shortcut, target, CandidateSource::Installed).ok()
        })
        .collect();
    finalize(candidates)
}

fn open_candidates(
    samples: &[ProcessSample],
    visible_pids: &HashSet<u32>,
    own_pid: u32,
    windows_dir: Option<&Path>,
) -> Vec<AppCandidate> {
    let candidates = samples
        .iter()
        .filter(|sample| sample.pid != own_pid && visible_pids.contains(&sample.pid))
        .filter_map(|sample| {
            let exe_path = sample.exe_path.as_deref()?;
            if is_under_dir(exe_path, windows_dir) {
                return None;
            }
            let mut candidate =
                app_candidate(exe_path, None, None, None, CandidateSource::Open).ok()?;
            candidate.process_name = sample.name.clone();
            Some(candidate)
        })
        .collect();
    finalize(candidates)
}

fn classify_dropped_path(
    path: &Path,
    resolve: impl Fn(&Path) -> Option<ShortcutTarget>,
) -> Result<DroppedCandidate, DropRejection> {
    match lowercase_extension(path).as_deref() {
        Some("exe") => app_candidate(path, None, None, None, CandidateSource::Dropped)
            .map(DroppedCandidate::App),
        Some("lnk") => {
            let target = resolve(path).ok_or(DropRejection::UnsupportedShortcut)?;
            app_from_shortcut(path, target, CandidateSource::Dropped).map(DroppedCandidate::App)
        }
        Some("url") => url_from_internet_shortcut(path).map(DroppedCandidate::Url),
        _ => Err(DropRejection::UnsupportedFile),
    }
}

fn app_from_shortcut(
    shortcut: &Path,
    target: ShortcutTarget,
    source: CandidateSource,
) -> Result<AppCandidate, DropRejection> {
    if lowercase_extension(&target.target_path).as_deref() != Some("exe") {
        return Err(DropRejection::UnsupportedShortcut);
    }
    app_candidate(
        &target.target_path,
        Some(file_stem(shortcut)),
        target.args,
        target.working_dir,
        source,
    )
}

fn app_candidate(
    exe_path: &Path,
    shortcut_name: Option<String>,
    args: Option<String>,
    working_dir: Option<PathBuf>,
    source: CandidateSource,
) -> Result<AppCandidate, DropRejection> {
    let info = exe_inspection::inspect_exe(exe_path).map_err(|error| match error {
        AppError::ExecutableNotFound(_) => DropRejection::ExecutableNotFound,
        _ => DropRejection::UnsupportedFile,
    })?;
    let name = shortcut_name
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(info.product_name);
    Ok(AppCandidate {
        suggested: is_flight_sim_suggestion(&name, exe_path),
        name,
        exe_path: exe_path.display().to_string(),
        args: args.filter(|args| !args.trim().is_empty()),
        working_dir: working_dir.map(|dir| dir.display().to_string()),
        process_name: info.process_name,
        icon_base64: info.icon_base64,
        source,
    })
}

fn url_from_internet_shortcut(path: &Path) -> Result<UrlCandidate, DropRejection> {
    let content = std::fs::read_to_string(path).map_err(|_| DropRejection::UnsupportedFile)?;
    let url = parse_internet_shortcut(&content).ok_or(DropRejection::UnsupportedUrl)?;
    if !is_web_url(&url) {
        return Err(DropRejection::UnsupportedUrl);
    }
    Ok(UrlCandidate {
        name: file_stem(path),
        url,
    })
}

fn parse_internet_shortcut(content: &str) -> Option<String> {
    let mut in_section = false;
    for line in content.lines().map(str::trim) {
        if line.starts_with('[') {
            in_section = line.eq_ignore_ascii_case("[InternetShortcut]");
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if in_section && key.trim().eq_ignore_ascii_case("url") && !value.trim().is_empty() {
            return Some(value.trim().to_owned());
        }
    }
    None
}

fn is_web_url(value: &str) -> bool {
    tauri::Url::parse(value).is_ok_and(|url| matches!(url.scheme(), "http" | "https"))
}

fn find_shortcuts(folder: &Path, depth: usize) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut shortcuts = Vec::new();
    for path in entries.flatten().map(|entry| entry.path()) {
        if path.is_dir() {
            if depth > 0 {
                shortcuts.extend(find_shortcuts(&path, depth - 1));
            }
        } else if lowercase_extension(&path).as_deref() == Some("lnk") {
            shortcuts.push(path);
        }
    }
    shortcuts
}

fn finalize(candidates: Vec<AppCandidate>) -> Vec<AppCandidate> {
    let mut candidates = dedupe_candidates(candidates);
    sort_by_name(&mut candidates);
    candidates
}

/// The first occurrence wins, so callers list their preferred source first.
fn dedupe_candidates(candidates: Vec<AppCandidate>) -> Vec<AppCandidate> {
    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|candidate| {
            seen.insert((
                candidate.exe_path.to_lowercase(),
                candidate.args.clone().unwrap_or_default(),
            ))
        })
        .collect()
}

fn sort_by_name(candidates: &mut [AppCandidate]) {
    candidates.sort_by_cached_key(|candidate| candidate.name.to_lowercase());
}

fn is_uninstaller_name(name: &str) -> bool {
    let name = name.to_lowercase();
    UNINSTALL_WORDS.iter().any(|word| name.contains(word))
}

fn is_uninstaller_exe(exe_path: &Path) -> bool {
    let name = file_stem(exe_path).to_lowercase();
    name.starts_with("unins") || name.contains("uninst") || is_uninstaller_name(&name)
}

fn is_under_dir(path: &Path, dir: Option<&Path>) -> bool {
    let Some(dir) = dir else {
        return false;
    };
    let path = path.to_string_lossy().to_lowercase().replace('/', "\\");
    let dir = dir.to_string_lossy().to_lowercase().replace('/', "\\");
    let dir = dir.trim_end_matches('\\');
    path.strip_prefix(dir)
        .is_some_and(|rest| rest.starts_with('\\'))
}

fn is_flight_sim_suggestion(name: &str, exe_path: &Path) -> bool {
    let haystacks = [compact(name), compact(&file_stem(exe_path))];
    FLIGHT_SIM_SUGGESTIONS
        .iter()
        .any(|fragment| haystacks.iter().any(|haystack| haystack.contains(fragment)))
}

fn compact(text: &str) -> String {
    text.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn windows_dir() -> Option<PathBuf> {
    std::env::var_os("SystemRoot").map(PathBuf::from)
}

fn lowercase_extension(path: &Path) -> Option<String> {
    path.extension()
        .map(|extension| extension.to_string_lossy().to_lowercase())
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_exe(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::create_dir_all(dir).expect("create dir");
        std::fs::write(&path, b"not a real binary").expect("write exe");
        path
    }

    fn target(path: &Path) -> ShortcutTarget {
        ShortcutTarget {
            target_path: path.to_path_buf(),
            args: None,
            working_dir: None,
        }
    }

    fn candidate(name: &str, exe_path: &str) -> AppCandidate {
        AppCandidate {
            name: name.into(),
            exe_path: exe_path.into(),
            args: None,
            working_dir: None,
            process_name: "App.exe".into(),
            icon_base64: None,
            source: CandidateSource::Installed,
            suggested: false,
        }
    }

    fn no_shortcuts(_: &Path) -> Option<ShortcutTarget> {
        None
    }

    #[test]
    fn uninstallers_are_recognized_by_shortcut_and_executable_name() {
        assert!(is_uninstaller_name("Uninstall Little Navmap"));
        assert!(is_uninstaller_name("Desinstalar App"));
        assert!(!is_uninstaller_name("Little Navmap"));
        assert!(is_uninstaller_exe(Path::new("unins000.exe")));
        assert!(is_uninstaller_exe(Path::new("Uninstall.exe")));
        assert!(!is_uninstaller_exe(Path::new("Volanta.exe")));
    }

    #[test]
    fn windows_folder_check_ignores_case_and_requires_a_separator() {
        let windows = Some(Path::new(r"C:\Windows"));
        assert!(is_under_dir(
            Path::new(r"c:\windows\System32\notepad.exe"),
            windows
        ));
        assert!(!is_under_dir(Path::new(r"C:\WindowsApps\app.exe"), windows));
        assert!(!is_under_dir(Path::new(r"C:\Tools\app.exe"), windows));
        assert!(!is_under_dir(Path::new(r"C:\Windows\app.exe"), None));
    }

    #[test]
    fn duplicates_keep_the_first_occurrence_ignoring_path_case() {
        let mut with_args = candidate("App (safe mode)", r"C:\App\App.exe");
        with_args.args = Some("--safe".into());
        let candidates = vec![
            candidate("App", r"C:\App\App.exe"),
            candidate("App on desktop", r"c:\app\APP.exe"),
            with_args,
        ];

        let names: Vec<String> = dedupe_candidates(candidates)
            .into_iter()
            .map(|candidate| candidate.name)
            .collect();

        assert_eq!(names, ["App", "App (safe mode)"]);
    }

    #[test]
    fn sorting_ignores_case() {
        let mut candidates = vec![
            candidate("volanta", "b"),
            candidate("Little Navmap", "a"),
            candidate("FSUIPC", "c"),
        ];
        sort_by_name(&mut candidates);
        let names: Vec<&str> = candidates.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["FSUIPC", "Little Navmap", "volanta"]);
    }

    #[test]
    fn flight_sim_tools_are_suggested() {
        assert!(is_flight_sim_suggestion(
            "Navigraph Charts",
            Path::new(r"C:\Navigraph\Charts.exe")
        ));
        assert!(is_flight_sim_suggestion(
            "Little Navmap",
            Path::new(r"C:\Tools\app.exe")
        ));
        assert!(is_flight_sim_suggestion(
            "Tool",
            Path::new(r"C:\Tools\littlenavmap.exe")
        ));
        assert!(!is_flight_sim_suggestion(
            "Notepad",
            Path::new(r"C:\Tools\notepad.exe")
        ));
    }

    #[test]
    fn internet_shortcut_url_is_read_from_its_section() {
        let content = "[DEFAULT]\r\nBASEURL=https://ignored\r\n[InternetShortcut]\r\nIconIndex=0\r\nURL=https://www.simbrief.com\r\n";
        assert_eq!(
            parse_internet_shortcut(content).as_deref(),
            Some("https://www.simbrief.com")
        );
        assert_eq!(parse_internet_shortcut("URL=https://no.section"), None);
        assert_eq!(parse_internet_shortcut("[InternetShortcut]\nURL=\n"), None);
    }

    #[test]
    fn dropped_executable_becomes_an_app_candidate() {
        let dir = tempfile::tempdir().expect("temp dir");
        let exe = fake_exe(dir.path(), "vPilot.exe");

        let dropped = classify_dropped_path(&exe, no_shortcuts).expect("accepted");

        let DroppedCandidate::App(app) = dropped else {
            panic!("expected app candidate");
        };
        assert_eq!(app.name, "vPilot");
        assert_eq!(app.process_name, "vPilot.exe");
        assert_eq!(app.source, CandidateSource::Dropped);
        assert!(app.suggested);
    }

    #[test]
    fn dropped_shortcut_keeps_its_name_arguments_and_working_folder() {
        let dir = tempfile::tempdir().expect("temp dir");
        let exe = fake_exe(&dir.path().join("App"), "App.exe");
        let shortcut = dir.path().join("Navigraph Charts.lnk");
        let resolve = |_: &Path| {
            Some(ShortcutTarget {
                target_path: exe.clone(),
                args: Some("--minimized".into()),
                working_dir: Some(dir.path().join("App")),
            })
        };

        let DroppedCandidate::App(app) =
            classify_dropped_path(&shortcut, resolve).expect("accepted")
        else {
            panic!("expected app candidate");
        };

        assert_eq!(app.name, "Navigraph Charts");
        assert_eq!(app.exe_path, exe.display().to_string());
        assert_eq!(app.args.as_deref(), Some("--minimized"));
        assert_eq!(
            app.working_dir,
            Some(dir.path().join("App").display().to_string())
        );
    }

    #[test]
    fn dropped_web_shortcut_becomes_a_url_candidate() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("SimBrief.url");
        std::fs::write(&path, "[InternetShortcut]\nURL=https://www.simbrief.com\n")
            .expect("write url");

        let dropped = classify_dropped_path(&path, no_shortcuts).expect("accepted");

        assert_eq!(
            dropped,
            DroppedCandidate::Url(UrlCandidate {
                name: "SimBrief".into(),
                url: "https://www.simbrief.com".into(),
            })
        );
    }

    #[test]
    fn unsupported_drops_are_rejected_with_a_reason() {
        let dir = tempfile::tempdir().expect("temp dir");
        let pdf = dir.path().join("manual.pdf");
        std::fs::write(&pdf, b"%PDF").expect("write pdf");
        let steam = dir.path().join("Game.url");
        std::fs::write(&steam, "[InternetShortcut]\nURL=steam://rungameid/123\n")
            .expect("write url");
        let folder_shortcut = dir.path().join("Folder.lnk");
        let to_folder = |_: &Path| Some(target(dir.path()));
        let broken_shortcut = dir.path().join("Broken.lnk");
        let to_missing = |_: &Path| Some(target(&dir.path().join("Missing.exe")));

        assert_eq!(
            classify_dropped_path(&pdf, no_shortcuts),
            Err(DropRejection::UnsupportedFile)
        );
        assert_eq!(
            classify_dropped_path(dir.path(), no_shortcuts),
            Err(DropRejection::UnsupportedFile)
        );
        assert_eq!(
            classify_dropped_path(&steam, no_shortcuts),
            Err(DropRejection::UnsupportedUrl)
        );
        assert_eq!(
            classify_dropped_path(&folder_shortcut, to_folder),
            Err(DropRejection::UnsupportedShortcut)
        );
        assert_eq!(
            classify_dropped_path(&folder_shortcut, no_shortcuts),
            Err(DropRejection::UnsupportedShortcut)
        );
        assert_eq!(
            classify_dropped_path(&broken_shortcut, to_missing),
            Err(DropRejection::ExecutableNotFound)
        );
        assert_eq!(
            classify_dropped_path(&dir.path().join("Gone.exe"), no_shortcuts),
            Err(DropRejection::ExecutableNotFound)
        );
    }

    #[test]
    fn shortcut_search_descends_into_subfolders_up_to_the_depth_limit() {
        let dir = tempfile::tempdir().expect("temp dir");
        let nested = dir.path().join("Vendor").join("App");
        std::fs::create_dir_all(&nested).expect("create dirs");
        std::fs::write(dir.path().join("Top.lnk"), b"").expect("write");
        std::fs::write(nested.join("Nested.lnk"), b"").expect("write");
        std::fs::write(nested.join("readme.txt"), b"").expect("write");

        let mut all: Vec<String> = find_shortcuts(dir.path(), 4)
            .iter()
            .map(|path| file_stem(path))
            .collect();
        all.sort();
        let shallow: Vec<String> = find_shortcuts(dir.path(), 1)
            .iter()
            .map(|path| file_stem(path))
            .collect();

        assert_eq!(all, ["Nested", "Top"]);
        assert_eq!(shallow, ["Top"]);
    }

    #[test]
    fn installed_list_filters_uninstallers_system_apps_and_duplicates() {
        let dir = tempfile::tempdir().expect("temp dir");
        let app = fake_exe(&dir.path().join("Tools"), "littlenavmap.exe");
        let uninstaller = fake_exe(&dir.path().join("Tools"), "unins000.exe");
        let system = fake_exe(&dir.path().join("Windows"), "notepad.exe");
        let start_menu = dir.path().join("Little Navmap.lnk");
        let desktop = dir.path().join("LNM on desktop.lnk");
        let uninstall_shortcut = dir.path().join("Uninstall Little Navmap.lnk");
        let uninstaller_shortcut = dir.path().join("Remove.lnk");
        let notepad_shortcut = dir.path().join("Notepad.lnk");
        let shortcuts = [
            start_menu.clone(),
            desktop.clone(),
            uninstall_shortcut.clone(),
            uninstaller_shortcut.clone(),
            notepad_shortcut.clone(),
        ];
        let resolve = |shortcut: &Path| {
            let exe = if shortcut == start_menu
                || shortcut == desktop
                || shortcut == uninstall_shortcut
            {
                &app
            } else if shortcut == uninstaller_shortcut {
                &uninstaller
            } else {
                &system
            };
            Some(target(exe))
        };

        let installed =
            installed_candidates(&shortcuts, resolve, Some(&dir.path().join("Windows")));

        assert_eq!(installed.len(), 1);
        assert_eq!(installed[0].name, "Little Navmap");
        assert_eq!(installed[0].source, CandidateSource::Installed);
        assert!(installed[0].suggested);
    }

    #[test]
    fn open_list_keeps_visible_apps_once_with_their_running_name() {
        let dir = tempfile::tempdir().expect("temp dir");
        let app = fake_exe(&dir.path().join("Tools"), "Volanta.exe");
        let system = fake_exe(&dir.path().join("Windows"), "explorer.exe");
        let sample = |pid: u32, name: &str, exe_path: Option<&Path>| ProcessSample {
            pid,
            parent_pid: None,
            name: name.into(),
            exe_path: exe_path.map(Path::to_path_buf),
            start_time: 0,
        };
        let samples = [
            sample(1, "Volanta.exe", Some(&app)),
            sample(2, "Volanta.exe", Some(&app)),
            sample(3, "explorer.exe", Some(&system)),
            sample(4, "protected.exe", None),
            sample(5, "background.exe", Some(&app)),
            sample(6, "autostart.exe", Some(&app)),
        ];
        let visible = HashSet::from([1, 2, 3, 4, 6]);

        let open = open_candidates(&samples, &visible, 6, Some(&dir.path().join("Windows")));

        assert_eq!(open.len(), 1);
        assert_eq!(open[0].name, "Volanta");
        assert_eq!(open[0].process_name, "Volanta.exe");
        assert_eq!(open[0].source, CandidateSource::Open);
    }
}
