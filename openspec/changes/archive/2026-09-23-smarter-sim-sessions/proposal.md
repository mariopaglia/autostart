## Why

AutoStart still depends on the pilot doing the right thing at the right time: they must switch the active profile before opening a different simulator, duplicate profiles to share apps between MSFS 2020 and 2024, and watch the window to notice failures. When the simulator crashes to desktop, every app closes within about 4 seconds (vPilot drops off VATSIM, Volanta stops tracking), and an app that crashes mid-flight stays closed. These gaps break the app's core promise: open the simulator and everything just works.

## What Changes

- **BREAKING** The "active profile" concept is removed. The monitor watches **every enabled profile**; the first profile whose trigger appears starts the session. Only one enabled profile may use a given trigger process: enabling a profile (from the UI or the tray) disables the enabled profiles that share one of its triggers, with a notice. Existing installs are migrated: the previously active profile wins any conflict.
- **BREAKING** A profile has a **list of triggers** (one or more processes, e.g. `FlightSimulator.exe` and `FlightSimulator2024.exe`) instead of a single trigger. The session lasts while any of them is running. Profiles saved or exported by earlier versions are converted on load/import.
- New **close delay** after the simulator exits (`closeDelayMs`, default 60 s, 0 disables it). During the delay the monitor is in a new `closePending` state; if a trigger of the session profile reappears, the session continues without closing or relaunching anything. The user can **close now** or **keep apps open** from the main window and the tray.
- New **native Windows notifications** (can be turned off in Settings) for launch failures, SimConnect timeout, the close countdown, crashed apps being relaunched and relaunch giving up.
- New per-app option **"Reopen if it crashes"** (`restartOnCrash`, default off): during a real session, when every process of the item exits abnormally (non-zero exit code), AutoStart relaunches it, up to 3 times per session. A normal exit (exit code 0, e.g. the user closed the app) is respected.
- Timeline gains entries for the close delay, the simulator returning, apps kept by the user, crashes and relaunches.

## Capabilities

### New Capabilities

- `desktop-notifications`: native Windows notifications for events that need the pilot's attention while AutoStart is in the tray, and the setting that turns them off.

### Modified Capabilities

- `process-monitor`: detection across all enabled profiles and all of a profile's triggers; new `closePending` state with the close delay, "close now" and "keep apps open"; session snapshot no longer tied to an active profile; new status and state in the interface events.
- `profile-management`: `trigger` becomes `triggers`; the "Active profile" requirement is replaced by watched (enabled) profiles with one enabled profile per trigger; `restartOnCrash` on app items; migration of saved and imported profiles.
- `launch-orchestration`: SimConnect wait decided by the trigger that started the session; relaunch of crashed items; manual tests blocked during `closePending` and without crash relaunch.
- `app-settings`: `activeProfileId` removed; `closeDelayMs` and `showNotifications` added.
- `desktop-integration`: tray menu toggles watched profiles and offers "close now"/"keep apps open" during the delay; tooltip and icon no longer refer to an active profile.
- `app-interface`: sidebar without "set active"; multi-trigger selector; close-delay banner in the top bar; "Reopen if it crashes" option and badge; new settings fields.
- `activity-log`: new timeline entry kinds.

## Impact

- **Rust**: `models.rs` (`Profile.triggers`, `AppItem.restart_on_crash`, `Settings` fields, `MonitorState::ClosePending`, `ItemStatus::Restarting`, new `TimelineKind`s, close deadline in the snapshot), `monitor/state_machine.rs` (close delay), `monitor/mod.rs` (multi-profile polling, new commands), `monitor/runner.rs` + new crash watcher, `storage.rs`/`state.rs` (migrations, one-enabled-per-trigger rule), `validation.rs`, `tray.rs`, `simconnect.rs`, `platform/windows/` (process exit codes) and `platform/fallback.rs`.
- **Frontend**: Zod schemas (with conversion of old imports), `TriggerSelector` (multiple triggers), `ProfileList`, `TopBar` (close-delay banner), `AppItemForm`/`ItemCard`, `SettingsScreen`, stores (settings no longer holds the active profile), notification dispatch from monitor events, pt-BR/en translations.
- **Dependency**: `tauri-plugin-notification` (Rust + JS) and its capability permission.
- **Bindings** regenerated.
- **Docs**: README (profiles, close delay, notifications, reopen on crash) and CHANGELOG.
- **Ordering dependency**: this change modifies requirements created by `bootstrap-autostart-app`, `improve-item-launching` and `quick-add-apps`, which must be archived before this one.
