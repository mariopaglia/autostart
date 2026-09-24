## 1. Session history

- [x] 1.1 Add `src-tauri/src/session_history.rs` with a pure `record` (a new test replaces any earlier test session; keeps at most 20 real sessions, newest first); verify with unit tests for replacing tests, the 21st real session and ordering
- [x] 1.2 Store the history in `session-history.json` (`schemaVersion` + `sessions`) with atomic writes and quarantine on corruption, and migrate `last-session.json` into it on the first load, deleting the old file only after the write; verify with `storage.rs` tests (fresh start, migration, corrupt file)
- [x] 1.3 Replace `save_last_session`/`load_last_session` in `AppState` and `Reporter::finish` with `record_session`, and add the `get_session_history` and `clear_session_history` commands; verify with `cargo clippy -- -D warnings && cargo test`
- [x] 1.4 Add the typed wrappers in `src/lib/tauri.ts` and a `src/stores/logs-store.ts` that loads the history, puts the live session in front, refetches when a `sessionEnded` entry arrives and clears the history; verify with store unit tests
- [x] 1.5 Add the session selector (profile, date/time, Test badge, error count) and "Clear history" with `ConfirmDialog` to `LogsScreen`. The default selection follows the live/latest session, and a manual pick stays put; verify with a `LogsScreen.test.tsx` covering picking, the error count and a new session arriving while an older one is selected

## 2. Steam links in URL items

- [x] 2.1 Add `is_supported_item_url` to `validation.rs` (http/https, `steam://rungameid/<digits>`, `steam://run/<digits>`), use it in `validate_item` and in `app_discovery::url_from_internet_shortcut`; verify with a shared accept/reject table test (including `steam://uninstall/1`, `ms-msdt:`, `file:`) and the updated drop test where `steam://rungameid/123` becomes a URL candidate
- [x] 2.2 Add `itemUrlSchema` to `src/schemas/profile.ts` with the same rule and use it in `urlItemSchema`; verify with `profile.test.ts` using the same table
- [x] 2.3 Update `UrlItemForm` so the URL field accepts Steam links, with a hint that AutoStart opens them but doesn't close them; add the pt-BR and en strings and verify with `locales.test.ts` and a form test
- [x] 2.4 Add a Windows integration test in `platform/windows/tests.rs` that opens a URL item with a custom scheme registered under HKCU for the test (pointing to `notepad.exe`) through the same code path as `launcher::launch_item`, asserts that notepad starts, then cleans up

## 3. Items for specific simulators

- [x] 3.1 Add `only_for_triggers: Vec<String>` to `AppItem` and `UrlItem` (serde default, skipped when empty) and `LaunchItem::applies_to`; validate that entries are triggers of the same profile and not repeated; verify with `models.rs` defaults tests and `validation.rs` tests (unknown trigger, repeat, case-insensitive match)
- [x] 3.2 Filter the items in `Session::start` by `started_by` (test sessions keep all items); verify with `session.rs` tests: a filtered item has no runtime and is not a close target, the matching trigger includes it, and a test session includes everything
- [x] 3.3 Mirror the field in the Zod schemas (`onlyForTriggers` with a default of `[]`, refined against the profile's triggers) and make `removeTrigger` strip the removed process name from every item; verify with `profile.test.ts` and `trigger-presets.test.ts`
- [x] 3.4 Add the "Only with" toggle chips to `AppItemForm` and `UrlItemForm`, shown only with 2+ triggers (pass `triggers` instead of `simConnectSupported`), and show the restriction on `ItemCard`; verify with form tests for 1 and 2 triggers and the saved value

## 4. Start flight: model and validation

- [x] 4.1 Add `launch_target: Option<String>` to `Trigger` and `launch_before_simulator: bool` to `AppItem`; validate the target (absolute `.exe` path, Steam launch link, `shell:AppsFolder\<AUMID>`) and reject `launch_before_simulator` together with `wait_for_sim_connect`; verify with `validation.rs` tests, including a relative path, `ms-msdt:` and an AUMID with a backslash
- [x] 4.2 Mirror both fields in the Zod schemas with the same rules; verify with `profile.test.ts`, including importing a v0.3.1 profile without the fields

## 5. Start flight: monitor

- [x] 5.1 Extend the pure state machine with `SimStarting { remaining_ticks }` (5 min of polls), the `StartFlight` and `SimulatorFailed` inputs, the `SimulatorArrived` action, and the transitions from `SimStarting` for `TriggerSeen`, `TriggerMissing` (countdown, then like `SimulatorFailed`) and `Pause`; include it in `has_session`; verify with `state_machine.rs` tests for every new transition, with a zero and a non-zero close delay
- [x] 5.2 Add `MonitorState::SimStarting`, `MonitorSnapshot.starting_simulator`, and the `SimulatorStarted`/`SimulatorNotStarted` timeline kinds; replace `Session::start`'s `is_test` with `StartMode` (`Detected`, `Flight`, `Test`); verify with the existing and updated `session.rs` tests and regenerated bindings (CI fails if they are stale)
- [x] 5.3 Add the `StartFlight { profile_id, trigger_process_name, reply }` and `CancelStart` commands to the monitor. Starting requires `Idle` and a trigger with a target, and works for disabled profiles. While `SimStarting`, `poll` watches only the chosen trigger. Add the `start_flight` and `cancel_flight_start` Tauri commands; verify with `detection.rs` unit tests for the chosen-trigger rule and `cargo test`
- [x] 5.4 Add `launcher::start_simulator` (skips when the trigger's process is already running; otherwise `platform::launch` with the exe folder or an empty working dir for URIs) and split `runner::launch_items` into before-simulator → start simulator → wait for arrival (`Notify`) → immediate → SimConnect phases, sending `SimulatorFailed` to the monitor on failure; verify with a `runner.rs` unit test for the phase split and the unix `launcher.rs` test pattern for "already running skips launch"
- [x] 5.5 Allow crash relaunch during `SimStarting` (relaunch stays allowed from the start of a `Flight` session); verify with a `session.rs` test
- [x] 5.6 Add Windows integration tests in `platform/windows/tests.rs` that call `start_simulator` with a `notepad.exe` target (asserts that the process starts and that a second call with it already running does not start another) and with an invalid exe path (asserts `ExecutableNotFound`)

## 6. Start flight: tray and notifications

- [x] 6.1 Add the "Start flight" submenu ("<profile> — <label>" for every trigger with a target, enabled only when `idle`, hidden when there are none), "Cancel start" in `SimStarting`, the running icon and "Starting simulator" tooltip for `SimStarting`, and the pt-BR/en labels in `tray.rs`; verify with unit tests for the tooltip and a pure function that lists the menu's start entries
- [x] 6.2 Notify when the simulator could not be started or did not appear (not on success, not in tests) in `session-notifications.ts`; verify with `session-notifications.test.ts` cases for `simulatorNotStarted` and an error entry with no item

## 7. Start flight: interface

- [x] 7.1 Add `LAUNCH_TARGET_PRESETS` (Steam and Microsoft Store for MSFS 2020/2024) to `trigger-presets.ts`, and the start target editor popover on each trigger chip (presets, "Choose executable…", typed field, remove, a mark when set); verify with `trigger-presets.test.ts` and a `TriggerSelector.test.tsx` case that sets and removes a target
- [x] 7.2 Add "Start flight" to `TopBar` (button with one target, dropdown with several, hidden with none, disabled outside `idle`), "Cancel" and the "Starting <label>" status in `simStarting`, and the new status in `StatusBadge`; make the test buttons disabled in `simStarting`; verify with a `TopBar.test.tsx` covering 0/1/2 targets and the `simStarting` state
- [x] 7.3 Add the "Open before the simulator" switch with its hint to `AppItemForm`, mutually exclusive with "Wait for SimConnect"; verify with `AppItemForm.test.tsx`
- [x] 7.4 Add the timeline labels for `simulatorStarted` and `simulatorNotStarted` to `TimelineRow`, and every new string to both locales; verify with `locales.test.ts` (same keys in pt-BR and en)

## 8. Docs and release notes

- [x] 8.1 Update README ("Start flight", Steam links, "Only with", "Open before the simulator", the session history and the new data file) and verify that the sections match the specs
- [x] 8.2 Add the user-facing entries under `## [Unreleased]` in `CHANGELOG.md` and the same entries, translated, in `CHANGELOG.pt-BR.md`; verify with `pnpm test` (changelog parsing tests)
- [x] 8.3 Run `pnpm typecheck && pnpm lint && pnpm test` and `cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test`, and confirm that everything passes and that `src/bindings/` is committed up to date
