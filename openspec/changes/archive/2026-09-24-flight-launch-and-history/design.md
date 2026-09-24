## Context

See proposal.md for motivation and the delta specs for the required behavior. The relevant parts of today's code:

- **Monitor** (`monitor/mod.rs`): a single task that polls every 2 s and runs a pure state machine (`monitor/state_machine.rs`: `Idle`, `SimRunning`, `ClosePending`, `Closing`, `Paused`). Sessions start only on `Input::TriggerSeen`, using `pending_start: Option<(Profile, Trigger)>`. Commands arrive through `MonitorCommand` over an mpsc channel.
- **Session** (`monitor/session.rs`): a frozen copy of the profile. `Session::start` creates an `ItemRuntime` for every enabled item. `close_targets` only considers items that have a runtime, and `enabled_items` feeds `runner::launch_items`, which splits the items into an immediate phase and a SimConnect phase.
- **Reporter** (`monitor/reporter.rs`): holds the published snapshot plus the single `session_log`, and calls `AppState::save_last_session` when a session finishes. Storage writes `last-session.json` atomically (`storage.rs`).
- **Launching** (`launcher.rs`): app items go through `platform::launch` (`ShellExecuteExW` on Windows, which also accepts URIs and `shell:` paths). URL items go through `tauri-plugin-opener`'s Rust API. Validation (`validation.rs`) and the Zod schemas (`src/schemas/profile.ts`) both allow only http/https.
- **Tray** (`tray.rs`): the menu is rebuilt on every refresh from the profiles and the snapshot, with hard-coded pt-BR and en labels.

## Goals / Non-Goals

**Goals:**
- Keep the state machine pure and unit-testable. The new state and inputs are covered by table-style tests like the existing ones.
- Keep today's behavior for profiles that use none of the new fields. There is no schema version bump: every new field has a serde default.
- Keep the Rust changes small and flat, with no new traits and no new crates.

**Non-Goals:**
- Listing Steam games or Microsoft Store apps in the app picker. Drag and drop of Steam `.url` shortcuts covers Steam items.
- Closing things opened through Steam links, or closing the simulator.
- Per-profile close settings, global hotkeys, and a "Start flight" that picks the profile automatically.
- Any scheme beyond http, https and the two Steam launch link forms.

## Decisions

### 1. Session history as one capped JSON file

A new file, `session-history.json`, holds `{ schemaVersion, sessions: SessionLog[] }`, newest first. A pure `record(history, log)` in a new `session_history.rs` module drops any earlier test session when `log.is_test`, inserts at the front and keeps at most 20 real sessions. Clearing is just emptying the list.

On the first `load`, when the history file is missing and `last-session.json` exists, the history is seeded with that one session and the old file is deleted after the new one is written. `Reporter::finish` calls `AppState::record_session` instead of `save_last_session`.

The commands are `get_session_history` (stored sessions only) and `clear_session_history`. The session in progress keeps coming from `get_session_log` and the `monitor://log` events, and the frontend puts it in front of the stored list.

A session in the selector is keyed by `startedAtMs` plus `profileId`. That avoids adding an `id` to `SessionLog`, which would need a migration for the old file.

*Alternatives:* one file per session in a `sessions/` folder (more I/O and more cleanup code, and at ~100 entries per session one file stays small); SQLite (a new dependency for 21 records).

### 2. Steam links: a fixed allow-list shared by Rust and Zod

A single rule defines what a URL item accepts: `http` or `https` (as today), or a URL whose scheme is `steam`, whose host is `rungameid` or `run`, and whose path is only digits. Rust has a single `is_supported_item_url` in `validation.rs`, used both by `validate_item` and by `app_discovery::url_from_internet_shortcut` in place of `is_web_url`. Zod has a matching `itemUrlSchema` in `src/schemas/profile.ts`, with unit tests on both sides using the same table of accepted and rejected values.

URL items go through a new `platform::open_link` (`ShellExecuteExW` on Windows, `open` on macOS) instead of `tauri-plugin-opener`. That gives web addresses, Steam links and the simulator's start targets one code path, which a Windows integration test covers with a scheme registered only for the test, and it drops the `AppHandle` parameter from `launcher::launch_item`.

*Why an allow-list:* profiles are imported from files shared in the community. Accepting any scheme would let an imported profile run protocol handlers like `ms-msdt:` or `search-ms:`. Two Steam link forms cover the real need.

### 3. Items for specific simulators: `onlyForTriggers: Vec<String>`

The field lives on both `AppItem` and `UrlItem` (serde default empty, `skip_serializing_if = "Vec::is_empty"`), and `LaunchItem::applies_to(trigger_process_name)` compares with `normalize_process_name`.

The filter is applied in exactly one place: `Session::start` keeps only the items that apply when `started_by` is `Some`. Everything downstream (item runtimes, `enabled_items`, `close_targets`, crash watch) works off the session's items, so a filtered-out item gets no status, no launch and no close. Test sessions (`started_by = None`) keep every enabled item.

Validation rejects entries that are not triggers of the same profile, as well as repeats. The frontend's `removeTrigger` also strips the removed process name from every item's `onlyForTriggers`, so saving never fails because of a trigger the user just removed.

*Alternative:* store trigger indexes. They are fragile when triggers are reordered or removed, and process names are already the triggers' identity.

### 4. Start flight: an on-demand session plus a `SimStarting` state

**Model.** `Trigger` gains `launch_target: Option<String>`. Validation accepts only one of three forms:
- an absolute path ending in `.exe`;
- a Steam launch link (the same rule as decision 2);
- `shell:AppsFolder\<AUMID>`, where the AUMID has no path separators or spaces and contains `!`.

`AppItem` gains `launch_before_simulator: bool`, and validation rejects it together with `wait_for_sim_connect`.

**State machine.** It gets a new `State::SimStarting { remaining_ticks }` (public `MonitorState::SimStarting`) and new inputs:
- `StartFlight`: from `Idle` goes to `SimStarting` with `remaining_ticks` = 5 min / 2 s = 150, and action `Start`;
- `SimulatorFailed` (start target failed, or the user cancelled): goes to `ClosePending`/`Closing` with `DelayClose`/`End`, depending on the close delay.

Existing inputs get new transitions from `SimStarting`:
- `TriggerSeen` goes to `SimRunning`, with a new action `SimulatorArrived`;
- `TriggerMissing` counts down, and at zero behaves like `SimulatorFailed`;
- `Pause` goes to `Paused` with `Discard`.

`has_session()` includes `SimStarting`.

**Monitor.** It gets a new `MonitorCommand::StartFlight { profile_id, trigger_process_name, reply }` and `MonitorCommand::CancelStart`. `start_flight` checks that the state is `Idle` (otherwise `AppError::SessionInProgress`) and that the trigger has a `launch_target`. It then sets `pending_start` to the chosen profile and trigger and applies `Input::StartFlight`. `Session::start` gets a `StartMode` (`Detected`, `Flight`, `Test`), replacing the `is_test` flag, so the runner knows whether to run the before-simulator phase.

While a session is in `SimStarting`, `poll` checks only the chosen trigger (`Session::started_by`), not every trigger of the profile. A different simulator of the same profile opening by chance must not count as arrival.

**Runner.** `launch_items` becomes explicit phases:
1. if `StartMode::Flight`: launch the `launch_before_simulator` items, then start the simulator (`launcher::start_simulator`), then await a `tokio::sync::Notify` that the monitor signals on `SimulatorArrived`;
2. the immediate items (minus phase 1);
3. the SimConnect items.

`start_simulator` skips the launch when the trigger's process is already running. Otherwise it calls `platform::launch(target, None, working_dir, LaunchOptions::default())`, where `working_dir` is the exe's folder, or empty for URIs, since `ShellExecuteExW` handles `steam://` and `shell:AppsFolder\...` natively. On failure, the runner records `TimelineKind::Error` (with no item) and sends `MonitorCommand::SimulatorFailed` to the monitor, which applies the input.

The timeline gets `SimulatorStarted` and `SimulatorNotStarted`. The start target does not become an item, so it is never closed.

**Snapshot.** `MonitorSnapshot` gains `starting_simulator: Option<String>` (the trigger label) for the top bar and the tray tooltip.

*Alternatives considered:*
- **Only launch the simulator and let detection start the session.** Items opened before the simulator would then be "pre-existing" and would never be closed, which defeats the purpose.
- **Launch every item before the simulator.** Some tools misbehave or fail to attach when the simulator isn't there yet. A per-item flag keeps today's order for everything else.
- **Make the simulator a list item.** That mixes the trigger into items and complicates closing (the simulator must never be closed).

### 5. Presets for the start target

`src/lib/trigger-presets.ts` gains `LAUNCH_TARGET_PRESETS` keyed by process name:

| Simulator | Steam | Microsoft Store |
| --- | --- | --- |
| MSFS 2020 | `steam://rungameid/1250410` | `shell:AppsFolder\Microsoft.FlightSimulator_8wekyb3d8bbwe!App` |
| MSFS 2024 | `steam://rungameid/2537590` | `shell:AppsFolder\Microsoft.Limitless_8wekyb3d8bbwe!App` |

X-Plane and custom triggers offer only "Choose executable…" and the typed field. The presets live only in the frontend, because the backend validates the form, not the catalog.

### 6. UI placement

- **TopBar.** A "Start flight" button (lucide `Plane`), shown when any trigger of the displayed profile has a target. With one such trigger it is a plain button, with more it is a `DropdownMenu`. In `simStarting` it becomes "Cancel", and the status badge reads "Starting <label>".
- **Trigger chips.** Each chip in `TriggerSelector` opens a `Popover` with the start target editor. A chip with a target shows a small `Rocket` mark.
- **Item forms.** `AppItemForm` gets a "Open before the simulator" `SwitchField` with a hint, wired to turn off "Wait for SimConnect" and vice versa. Both forms get an "Only with" multi-select built from the profile's triggers (toggle chips, no selection = all), shown only with 2+ triggers. This needs a new `triggers` prop in place of the current `simConnectSupported` boolean, which becomes derived.
- **Logs screen.** A `Select` with the session list (profile, date, "Test" badge, error count), plus a "Clear history" button behind `ConfirmDialog`. History goes in a new `logs-store.ts` (the logs domain), which refetches when a `sessionEnded` entry arrives.
- **Tray.** A "Start flight" submenu listing "<profile> — <label>" for every trigger with a target, disabled unless `idle`, and "Cancel start" while `simStarting`. Tray labels gain the pt-BR and en strings.

## Risks / Trade-offs

- [The Microsoft Store AUMIDs and Steam app IDs above are from memory and could be wrong, especially `Microsoft.Limitless` for MSFS 2024] → Confirm them before the release: check `Get-StartApps` on a machine with each Store version, and the Steam store URLs for the IDs. Presets are just data in one file, so fixing one is a one-line patch. The typed field lets users work around a wrong preset meanwhile.
- [Steam may take longer than 5 minutes to start the simulator (updates, shader cache, cloud sync)] → The timeout goes into the close delay rather than closing right away, and the trigger reappearing during `closePending` already continues the session. A slow start therefore still ends well, as long as it fits in close delay + 5 min.
- [A Steam link on a PC without Steam makes Windows offer to find an app for the link] → This is the expected shell behavior, and it matches opening the link by hand.
- [A 20-session history with long sessions grows the file] → A session has on the order of 10–200 entries, so 21 sessions stay well under 1 MB. Written only when a session finishes.
- [An item restricted to a trigger that is later removed silently starts applying to every simulator] → This is intentional and matches "empty means all". The trigger removal is an explicit user action in the same screen.
- [The launcher-follow logic (`app_watch`) could mistake the simulator for a helper's child process] → The simulator is started outside `launch_item`, so it never goes into any item's tracked PID set. `start_simulator` does no process tracking.

## Migration Plan

- There is no schema version bump. New fields default to empty/false/None, and a v0.3.x `profiles.json` loads unchanged.
- `last-session.json` is migrated into `session-history.json` on the first start. The old file is removed only after the new file is written.
- Rollback: an older version ignores `session-history.json`, so its Logs screen starts empty. It also ignores the unknown fields, because serde drops them. A URL item with a Steam link would fail the old version's validation only when that profile is saved again.
