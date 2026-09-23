## Context

See proposal.md for motivation. Current state relevant to the approach:

- `monitor/mod.rs` polls every 2 s, watching only `settings.active_profile_id`'s single `trigger.process_name`. `monitor/state_machine.rs` is a pure function (`next(state, input)`) with `Idle`, `SimRunning { missed_ticks }`, `Closing`, `Paused`; two missed ticks end the session.
- The session (`monitor/session.rs`) is a frozen copy of the profile; `runner.rs` launches in two phases (SimConnect) and closes; `app_watch.rs` follows each launched app for up to 30 s to learn its process name and minimize windows.
- `Profile.enabled` already exists and is toggled from the top bar, but it only matters for the active profile. The tray submenu is a radio-like list of profiles that sets `activeProfileId`.
- The frontend is alive while the window is hidden (the window is hidden, never destroyed), and already receives `monitor://state`, `monitor://item-status` and `monitor://log`.
- Storage migrates raw JSON before deserializing (`read_migrated_or_recover`, used for `processNameMode`); imports are parsed by the frontend Zod `profileSchema`.

## Goals / Non-Goals

**Goals:**
- Keep the new session logic pure and unit-testable (state machine, profile selection, conflict resolution, crash verdict, notification mapping).
- Keep the Rust layer small: no new traits, one new Windows API surface (process exit codes).
- Existing installs upgrade without losing data or manual steps.

**Non-Goals:**
- Action buttons inside Windows notifications, or reacting to notification clicks.
- Running sessions for two profiles at the same time.
- Relaunching crashed apps during test sessions or in the first 30 s after an app's launch in automatic process-name mode (see Risks).
- Keeping more than the last session's timeline.

## Decisions

### D1. `Profile.triggers: Vec<Trigger>` instead of a list of names under one label
Each preset already carries its own label (MSFS 2020, MSFS 2024…), the top bar shows labels, and the SimConnect decision is per process. A `Vec<Trigger>` keeps `Trigger` unchanged and lets the UI render one chip per trigger. Alternative considered: `trigger.processNames: Vec<String>` with a single label — rejected because the label no longer describes the profile once it covers two simulators.

Validation (`validation.rs` and Zod): 1–5 triggers, each `.exe`, unique case-insensitively.

Migration: bump the profiles `SCHEMA_VERSION` to 2; the raw-JSON migration hook converts `trigger: {...}` into `triggers: [{...}]` (runs alongside `infer_process_name_modes`). Imports use `z.preprocess` in `profileSchema` for the same conversion, so exports from v0.1/v0.2 keep working.

### D2. One enabled profile per trigger, enforced in `AppState`
A pure function `profiles_to_disable(profiles, winner_id) -> Vec<String>` returns the enabled profiles sharing a trigger with the winner. `AppState::save_profile` (when the saved profile is enabled) and a new `AppState::set_profile_enabled(id, enabled)` apply it inside the same lock and persist once, so a conflict is never written. Both commands return `ProfilesUpdate { profiles: Vec<Profile>, disabled_profile_ids: Vec<String> }`; the frontend replaces its store and shows the notice. A profile saved for the first time (create, duplicate, import) never wins: if it is enabled and shares a trigger with an enabled profile, `save_profile` stores it disabled. This keeps "New profile" (which starts with the MSFS 2024 trigger) from silently switching off the pilot's current profile, and makes duplicate/import follow from the same rule without frontend logic.

Alternative considered: "first in the list wins" at detection time — rejected by the maintainer; it hides which profile will run.

Upgrade from v0.2: `Settings` drops `active_profile_id` (serde ignores the unknown field). Before deserializing settings, `Storage` reads the legacy `activeProfileId` from the raw JSON and `AppState::load` runs a pure `resolve_conflicts(profiles, legacy_active_id)` (in `trigger_conflicts.rs`, run on every load so the invariant holds even for hand-edited files) that keeps the legacy active profile (or the first in order) enabled in each conflicting group. The result is saved immediately.

`set_active_profile` (command, tray handler, settings store action) is removed.

### D3. Monitor selects the profile at detection time
`poll()` refreshes `ProcessTable` once and then:
- with a session: `TriggerSeen` if any of the session profile's triggers is running, else `TriggerMissing`;
- without a session: iterates `AppState::profiles()` in sidebar order, picks the first enabled profile with a running trigger and keeps it in `pending_start: Option<(Profile, Trigger)>`, then feeds `TriggerSeen`/`TriggerMissing`.

`start_session` consumes `pending_start`. `Session` stores `started_by: Trigger`; `runner::launch_items` passes it to `simconnect::supports_simconnect` instead of the profile's trigger. The state machine's inputs stay the same, so profile selection is tested separately as a pure `first_running_profile(profiles, is_running) -> Option<(Profile, Trigger)>`.

### D4. Close delay as a counted state in the pure state machine
Add `State::ClosePending { remaining_ticks: u32 }`, where `remaining_ticks = ceil(close_delay_ms / POLL_INTERVAL)`. `next(state, input, close_delay_ticks)` gains the configured value as a parameter (read from settings on each poll, so changes apply immediately):

| From | Input | To | Action |
|---|---|---|---|
| `SimRunning` (2nd miss) | `TriggerMissing` | `ClosePending { n }` if n > 0, else `Closing` | `DelayClose` / `End` |
| `ClosePending` | `TriggerSeen` | `SimRunning { 0 }` | `Continue` |
| `ClosePending { 1 }` | `TriggerMissing` | `Closing` | `End` |
| `ClosePending { k }` | `TriggerMissing` | `ClosePending { k-1 }` | — |
| `ClosePending` | `CloseNow` | `Closing` | `End` |
| `ClosePending` | `KeepApps` | `Idle` | `Release` |
| `ClosePending` | `Pause` | `Paused` | `Discard` |

`DelayClose` records `CloseDelayed` (detail = seconds) and stamps `closes_at_ms` in the snapshot for the countdown; `Continue` records `SimulatorReturned`; `Release` records `KeptByUser` and finishes the session without closing. Ticks (±2 s precision) were chosen over wall-clock deadlines so the machine stays pure and needs no clock injection. The launch task is not aborted on `ClosePending`, so a SimConnect wait survives a quick simulator restart; it is aborted on `End`/`Release`/`Discard` as today.

New monitor commands `CloseNow` and `KeepApps`, exposed as Tauri commands and tray items. `MonitorState` gains `ClosePending`; `MonitorSnapshot` gains `closes_at_ms: Option<u64>`.

### D5. Crash detection by exit code, relaunch owned by a per-item watcher
Distinguishing a crash from the user closing the app is the core problem. Windows keeps a process's exit code while someone holds a handle to it; a normal exit returns 0, crashes return exception codes (e.g. `0xC0000005`) and Task Manager kills return 1. So:

- `platform::windows::process_exit` exposes `ExitWatcher` holding `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` handles and `exit_codes() -> Vec<Option<u32>>` via `GetExitCodeProcess` (`STILL_ACTIVE` = still running). `PROCESS_QUERY_LIMITED_INFORMATION` also works on elevated processes of the same user. The fallback returns `None` codes (never a crash), so macOS development never relaunches.
- Pure `crash_verdict(exit_codes) -> Option<Verdict>`: `None` while any process runs; `Crashed` if any code is non-zero; `ClosedNormally` otherwise.
- New `monitor/crash_watch.rs`: one task per `app` item with `restart_on_crash` in a real session, started when the item becomes `running` — at the end of `app_watch::watch` for launched items (so it sees the learned process name, avoiding a launcher's exit 0 being read as "closed by the user"), and right away for pre-existing (`skipped`) items. Every 2 s it adds new PIDs matching the item's current `process_name` to the `ExitWatcher`, then evaluates the verdict. On `Crashed`: status `restarting`, `Crashed` timeline entry, sleep 5 s, and if relaunch is still allowed, `launcher::launch_item` and `Relaunched`; after the 3rd relaunch, the 4th crash yields `error` with `AppError::KeptCrashing`.
- `Session` gains `relaunch_allowed: bool`, set by the monitor on entering/leaving `SimRunning`, and the watcher exits once the session is finished. A flag was chosen over tracking and aborting `JoinHandle`s because the watcher must also stop mid-sleep and the session is already shared.

Alternatives considered: treating any disappearance as a crash (reopens apps the user closed on purpose); Windows Error Reporting events (complex, not every crash reports).

### D6. Notifications dispatched from the frontend
`tauri-plugin-notification` is added, but notifications are sent from the webview: a `useSessionNotifications` hook listens to `monitor://log` and calls `sendNotification` using i18n strings. A pure `notificationFor(entry, context) -> { title, body } | null` maps timeline entries (`Error` with item name and message, SimConnect error, `CloseDelayed`, `Relaunched`, `Error` with `KeptCrashing`) and returns `null` for test sessions or when `showNotifications` is off. This keeps every user-facing string in `pt-BR.json`/`en.json` (the tray is the only Rust-side exception and stays that way) and reuses events that already exist.

Alternatives considered: Rust-side notifications with a label table like the tray (duplicates translations); `tauri-winrt-notification` with action buttons for "keep apps open" (COM activation and more Windows code for a Rust-light codebase). The countdown actions live in the top bar and tray instead, and the notification text points to them.

### D7. Settings and UI
- `Settings`: remove `active_profile_id`, add `close_delay_ms` (default 60 000, 0–600 000) and `show_notifications` (default true). The settings screen edits the delay in seconds.
- Sidebar: drop the "Active" badge and "Set active" action; show a muted "Disabled" label (exists) and a running indicator on the session's profile (`snapshot.sessionProfileId`). The main screen displays the previously selected profile via `ui-store` (it used `activeProfileId` as the initial selection; it now falls back to the first profile).
- `TriggerSelector` becomes a chip list plus the existing popover used as "Add trigger"; removing is disabled on the last chip.
- `AppShell`: while `closePending`, a `CloseCountdown` banner above the current screen (so it is visible from Logs and Settings too), computed from `closesAtMs` with a 1 s timer, with "Close now" and "Keep apps open".
- Tray: `CheckMenuItem` checked = `enabled`; clicking calls `set_profile_enabled(id, !enabled)` and emits `profiles://changed` with the updated list. Tooltip: session profile name during a session, otherwise "Watching N profiles".

## Risks / Trade-offs

- [After upgrading, profiles that were never active start being watched (e.g. an old test profile with a common process as trigger)] → Migration only disables conflicting profiles; the CHANGELOG and README explain that every enabled profile is now watched, and the sidebar shows which are enabled.
- [Downgrading to v0.2 cannot read `triggers` and would quarantine the profiles file] → The updater only moves forward; the risk is limited to manual reinstalls of an old installer. Documented in the CHANGELOG.
- [Apps that exit with a non-zero code on a normal close would be reopened] → The option is off by default and per item; the 3-relaunch cap bounds the damage; the hint in the form explains the rule.
- [A crash in the first 30 s of an automatic-mode item is not caught] → Acceptable: the watch needs the learned process name; crashes that early are usually startup failures, which relaunching would not fix.
- [Crash detection depends on opening each PID's handle while it is alive (2 s sampling)] → A process that starts and dies within 2 s is missed; its sibling PIDs still decide the verdict, and a missed case only means no relaunch.
- [Windows toasts show a generic app identity when running from `pnpm tauri dev`] → Only affects development; the installed build uses the bundle identifier.
- [Close-delay precision is ±2 s] → Irrelevant for a 60 s delay; the UI countdown uses `closesAtMs`.
- [Launch task keeps running during `closePending`] → Only items still in the launch phases can open while the simulator is gone; `End` aborts it when the delay expires.

## Migration Plan

1. Ship in one release (minor bump, since saved data changes). On first start, storage converts `trigger` → `triggers`, resolves legacy conflicts from `activeProfileId`, fills the new settings with defaults and saves both files.
2. Rollback: publish a fixed version rather than reverting; v0.2 cannot read the migrated profiles file (see Risks).
