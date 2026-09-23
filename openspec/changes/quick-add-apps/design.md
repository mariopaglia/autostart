## Context

Today "Add app" opens `ItemFormDialog` → `AppItemForm`, whose only source is the native dialog (`@tauri-apps/plugin-dialog`) followed by the `inspect_exe` command (`exe_inspection.rs` → `platform::product_name` / `platform::icon_png_base64`). Relevant building blocks already exist:

- `processes::list_running()` (sysinfo, name + exe path) used by `TriggerSelector` with shadcn's `Command` list.
- `platform/windows/top_level_windows.rs` enumerates top-level windows for a set of PIDs.
- `ElevationBanner` already asks `commands.isElevated()`.
- The `windows` crate is on 0.62 with `Win32_UI_Shell` enabled; no COM feature yet.
- The window is a single WebView2 webview; Tauri's `dragDropEnabled` is on by default, so OS file drops arrive as webview drag-drop events with absolute paths (the HTML5 `dataTransfer` path API is not available). `dnd-kit` uses pointer sensors, so card reordering is unaffected.

See proposal.md for motivation and the specs for behavior.

## Goals / Non-Goals

**Goals:**
- Keep all filtering, deduplication, `.url` parsing, drop classification and suggestion matching in pure Rust functions, unit-tested on every host.
- Keep the Windows layer thin: resolve a shortcut, list shortcut folders, list PIDs with visible windows.
- One review path in the UI: every source ends in the same pre-filled `AppItemForm` (except multi-file drops).

**Non-Goals:**
- Caching discovery results across picker openings or persisting them.
- Reading the registry `Uninstall` keys (they rarely point to the app's main `.exe`; Start Menu shortcuts do).
- Store/UWP apps, `steam://` shortcuts (see proposal non-goals).

## Decisions

### D1. Start Menu and desktop shortcuts as the installed-apps source
Walk `FOLDERID_CommonPrograms`, `FOLDERID_Programs`, `FOLDERID_PublicDesktop` and `FOLDERID_Desktop` (via `SHGetKnownFolderPath`) recursively for `*.lnk`, resolve each, keep `.exe` targets that exist.
- *Alternative*: registry `HKLM/HKCU\...\Uninstall` — gives display names but `DisplayIcon`/`InstallLocation` often point to uninstallers or folders; would need heuristics to find the main exe. Rejected.
- *Alternative*: scanning `Program Files` — slow and full of helper executables. Rejected.

Filters (pure, `app_discovery.rs`): uninstall patterns on target file name (`unins*.exe`, `uninstall*.exe`, `*uninst*.exe`) and shortcut name (contains "uninstall", case-insensitive); target under `%SystemRoot%`; dedupe key = lowercase target path + args, first occurrence wins (Start Menu is scanned before desktop so its names win). Recursion depth is capped (e.g. 4) to avoid pathological trees.

### D2. Shortcut resolution through `IShellLinkW`
`platform::resolve_shortcut(path) -> Option<ShortcutTarget { exe_path, args, working_dir }>` uses `CoCreateInstance(ShellLink)` + `IPersistFile::Load` + `GetPath`/`GetArguments`/`GetWorkingDirectory`, without `Resolve` (it may show UI or search the disk). COM is initialized per call site with `CoInitializeEx(COINIT_APARTMENTTHREADED)` on the `spawn_blocking` thread and balanced with `CoUninitialize` through a small guard. MSI "advertised" shortcuts return an empty path and are skipped. Requires the `Win32_System_Com` feature.
- *Alternative*: parsing the `.lnk` binary format in Rust (e.g. `lnk` crate). Rejected: new dependency, and it does not expand environment-variable paths the way the shell does.

`.url` files are INI text; `parse_internet_shortcut(content) -> Option<String>` reads `URL=` under `[InternetShortcut]` in pure Rust, so it also works on macOS.

### D3. Open apps = processes owning a visible, unowned top-level window
Add `platform::pids_with_visible_windows() -> HashSet<u32>` next to `top_level_windows`, using `EnumWindows` + `IsWindowVisible` + `GetWindow(GW_OWNER)` is null + not `WS_EX_TOOLWINDOW`. Then `processes::samples()` supplies exe paths; entries without a path (protected processes when not elevated), our own PID and `%SystemRoot%` executables are dropped; dedupe by lowercase exe path. The `process_name` is the running name, which is exactly what the monitor matches.
- *Alternative*: every process from `list_running()` — would flood the list with `svchost.exe` and helpers. Rejected.

### D4. One `AppCandidate` model, built in Rust
```rust
pub struct AppCandidate {
    pub name: String,
    pub exe_path: String,
    pub args: Option<String>,
    pub working_dir: Option<String>,
    pub process_name: String,
    pub icon_base64: Option<String>,
    pub source: CandidateSource, // Installed | Open | Dropped
    pub suggested: bool,
}
```
Drops return `DropResolution { candidates: Vec<DroppedCandidate>, rejected: Vec<RejectedDrop> }`, where `DroppedCandidate` is tagged by `type` (`app` → `AppCandidate`, `url` → `{ name, url }`) and `RejectedDrop { path, reason }` uses a `DropRejection` enum (`unsupportedFile`, `executableNotFound`, `unsupportedUrl`, `unsupportedShortcut`) translated by the frontend. All derive `TS` and get Zod schemas in `src/schemas/app-candidate.ts`.

Commands (all `async` + `spawn_blocking`, like `list_running_processes`): `list_installed_apps`, `list_open_apps`, `resolve_dropped_paths(paths)`. Names and icons reuse `exe_inspection` helpers (`platform::product_name`, `platform::icon_png_base64`); for installed apps the shortcut name is used, so version info is only read for open and bare `.exe` candidates.

### D5. Curated flight sim suggestions as a static Rust list
`FLIGHT_SIM_SUGGESTIONS: &[&str]` of lowercase fragments matched against exe file name and display name (e.g. `navigraph`, `simbrief`, `littlenavmap`, `volanta`, `vpilot`, `fsuipc`, `spad.next`, `couatl`, `gsx`, `fs2crew`, `pilot2atc`, `beyondatc`, `sayintentions`, `fenix`, `pmdg`, `a32nx`/`flybywire`, `aerosoft`, `simtoolkitpro`, `navigraph simlink`). Kept in Rust so the pure filter tests cover it; extending it is a one-line change.
- *Alternative*: remote list. Rejected: network dependency and privacy expectations for a tray app.

### D6. Frontend flow
- `AppPickerDialog` (tabs `installed`/`open` with a shared `CommandInput`; one `Command` list per tab; each tab fetches when first shown and on every picker opening). Entry rendering in `AppCandidateRow`. "Browse for file…" is a footer button.
- Pure mapping in `src/lib/app-candidates.ts`: `candidateToAppForm(candidate): AppFormInput` (merged over the form's defaults, `processNameMode: "auto"`), `candidateToItem(candidate): LaunchItem` for multi-drop (default options, new UUID), `urlCandidateToItem`, `isAlreadyInProfile(exePath, profile)` (case-insensitive path compare). `pickExecutable(t)` (native dialog + `inspectExe` → `AppFormInput`) is extracted from `AppItemForm.chooseExecutable` so both the form and the picker use it.
- `ItemFormTarget` gains an optional `initial` on the create variant (`{ mode: "create"; type; initial?: AppFormInput | UrlFormInput }`); the tabs are skipped when `initial` is present. `AppItemForm` receives `initial` and `existingExePaths` (for the duplicate warning, excluding the item being edited).
- `useFileDrop({ enabled, onDrop })` hook wraps `getCurrentWebview().onDragDropEvent` (enter/over → show overlay, leave → hide, drop → `commands.resolveDroppedPaths`). It lives behind a typed wrapper in `src/lib/tauri.ts` (`windowEvents.onFileDrop`) to respect the "no direct Tauri calls in components" rule. `enabled` is false while the picker or form is open; other Radix dialogs (sidebar rename/confirm, onboarding) are detected with `document.querySelector('[role="dialog"], [role="alertdialog"]')` at drop time, so no global dialog registry is needed.
- `DropOverlay` is a full-window absolutely positioned layer in `MainScreen`, `pointer-events-none`, with a reduced-motion-safe fade.
- The elevated state for the empty-state hint reuses a shared `useIsElevated` hook extracted from `ElevationBanner` into `src/hooks/use-is-elevated.ts`.

### D7. Multi-drop appends directly
Several candidates are appended with defaults (`enabled: true`, default delay, `onClose: "graceful"`, `processNameMode: "auto"`), because opening N forms in a row is worse UX than editing a card afterwards. A single candidate still opens the form so the user sees what will be saved.

## Risks / Trade-offs

- [Icon extraction for 100–300 Start Menu apps may take a second or more] → runs in `spawn_blocking` with a loading state in the picker; if measured on CI (Windows runner timing in the integration test) to exceed ~3 s, fall back to returning names first and fetching icons lazily per visible row via the existing `inspect_exe`.
- [When AutoStart runs elevated, UIPI blocks OLE drag and drop from Explorer; `ChangeWindowMessageFilterEx` does not help for WebView2 OLE drops] → documented limitation; the empty state explains it (spec) and the picker covers the same need.
- [Shortcut resolution without `IShellLink::Resolve` returns stale paths for moved apps] → such shortcuts fail the "target exists" check and are skipped (installed list) or rejected with `executableNotFound` (drop).
- [COM initialization on a thread that already has a different apartment returns `RPC_E_CHANGED_MODE`] → treat that result as "already initialized" and skip the matching `CoUninitialize`; `spawn_blocking` threads normally have no apartment.
- [Some apps are only a launcher shortcut (e.g. `Update.exe --processStart App.exe` for Squirrel apps)] → the shortcut's arguments are preserved in the candidate, and the automatic process-name learning from `improve-item-launching` corrects the process name after the first launch.
- [macOS dev host cannot exercise discovery] → pure logic is covered by unit tests everywhere; Windows behavior by integration tests in `platform/windows/tests.rs` on CI.

## Migration Plan

No data migration: no profile or settings format change. Rollback is reverting the release. `bootstrap-autostart-app` and `improve-item-launching` must be archived before this change is archived (its `app-interface` delta modifies their requirements).
