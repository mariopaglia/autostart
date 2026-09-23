<!--
Phases: each phase ends with a CHECKPOINT. Stop, present the summary to the maintainer and only continue after validation.
Conventions: code in English, Clean Code, comments only when necessary (see CLAUDE.md).
Branch: feat/improve-item-launching from main.
-->

## 1. Phase 1 — Model and unified launch

- [x] 1.1 Add to `models.rs` the `ProcessNameMode` (`auto` | `manual`), the `process_name_mode`, `start_minimized` and `wait_for_sim_connect` fields on `AppItem` (with `#[serde(default)]`), `ItemStatus::WaitingSimConnect`, `TimelineKind::SimConnectReady`/`SimConnectWaitSkipped` (`AppError::SimConnectUnavailable` comes in 3.3, where it starts being used, so CI's `clippy -D warnings` does not fail earlier). Verify with a deserialization test of a v0.1.0 item (without the new fields) and with `cargo test` regenerating the bindings
- [x] 1.2 Update the Zod schemas (`appItemSchema` with the new fields and defaults) and the derived frontend types. Verify with Vitest tests of a v0.1.0 profile accepted with the defaults and with `pnpm typecheck` passing against the bindings
- [x] 1.3 Create `platform::launch(exe, args, working_dir, LaunchOptions) -> AppResult<u32>` (design D1): `ShellExecuteExW` with `SEE_MASK_NOCLOSEPROCESS`, `runas` verb when elevated, `SW_SHOWMINNOACTIVE` when minimized and PID via `GetProcessId`, with the handle in a `Drop` guard; stub in `fallback.rs` with `Command::spawn`. Remove `launch_elevated`/`configure_launch` and use the new function in `launcher.rs`. Verify with `cargo clippy -D warnings` on macOS and in CI (Windows)
<!-- Intermediate checkpoints waived by the maintainer on 2026-09-22: implement straight through and validate on the final build. -->
- [x] 1.4 CHECKPOINT Phase 1: produce the preview build and ask the maintainer to validate on Windows that regular launch, with arguments, with working folder and as administrator (UAC accepted and denied) still behaves as in v0.1.0

## 2. Phase 2 — Process name learning

- [x] 2.1 Create `process_tracker.rs` with `ProcessSample`, `expand_tracked` and `choose_process_name` as pure functions (design D2). Verify with unit tests: live root, launcher that exits leaving a child, grandchild after the intermediate dies, install-folder fallback, start-time tie and no candidate
- [x] 2.2 Expose in `processes.rs` the reading of `ProcessSample` (PID, parent PID, name, path, start) from `sysinfo`. Verify with a test that the reading includes the test process itself with the correct parent PID
- [x] 2.3 Rewrite the launch confirmation in `launcher.rs` to consider the root PID, the tracked descendants and the `processName` (10 s), and start the 30 s learning task for items in `auto` mode. Verify with `cargo test` and, on macOS, with an item pointing to a script that opens another app and exits
- [x] 2.4 Apply the learned name (design D3): `Session::learn_process_name`, `AppState::learn_process_name` (only in `auto` mode and if the item still exists), disk write and `profiles://changed` event; timeline entry with the learned name. Verify with unit tests of `Session` and `AppState` (manual mode preserved, removed item ignored)
- [x] 2.5 Handle `profiles://changed` in the frontend (wrapper in `src/lib/tauri.ts`, hook and upsert in `profiles-store`). Verify with a Vitest test of the store and by watching the form update without reloading after a "Test launch"
- [x] 2.6 Move the working folder and the process name into the "Advanced" section (shadcn `collapsible` component), with the "Detected automatically" label, editing switching to `manual` and the "Detect automatically" button. Texts in pt-BR and en. Verify by adding an app without opening the section and toggling between the modes
- [x] 2.7 CHECKPOINT Phase 2: preview build and Windows checklist: app without a launcher keeps its name; a real launcher (e.g. Volanta or another that switches process) learns the right name through "Test launch" and closes on "Test close"; manual mode preserved; timeline shows the learning

## 3. Phase 3 — Start minimized and wait for SimConnect

- [x] 3.1 Implement `platform::minimize_new_windows` (design D4) with a macOS stub and call it every 250 ms for 10 s after the item becomes `running` when `startMinimized = true`. Verify with `cargo clippy` and, on Windows, in the phase check
- [x] 3.2 Implement `platform::is_simconnect_available` (design D5, `WaitNamedPipeW` with the pipe name in a constant) with a macOS stub, and `supports_simconnect(trigger)` as a pure function. Verify with unit tests of `supports_simconnect` (MSFS 2020, MSFS 2024, casing, X-Plane)
- [x] 3.3 Implement the two-phase launch in `runner.rs` (design D6), with `AppError`/`ErrorKind::SimConnectUnavailable`: pure `split_launch_phases`, `waitingSimConnect` status, 2 s polling with a 10 min limit, timeout error, skipping the wait in tests and unsupported triggers. Verify with unit tests of `split_launch_phases` and, on macOS, with "Test launch" recording `simConnectWaitSkipped` in the timeline
- [x] 3.4 Add to the form the "Start minimized" and "Wait for SimConnect" switches (disabled with a hint when the trigger is not MSFS 2020/2024, via `SIMCONNECT_TRIGGERS`), the "Minimized" and "SimConnect" badges on the cards, the "Waiting for SimConnect" status in `ItemStatusBadge` and the new kinds in the timeline, in pt-BR and en. Verify with the i18n key parity test and visually on macOS
- [x] 3.5 Update the README (automatic process name, start minimized and its limitations, wait for SimConnect and its limitations) and the "Unreleased" section of the CHANGELOG. Verify with `pnpm format:check`
- [x] 3.6 CHECKPOINT Phase 3: preview build and Windows checklist: minimized item without stealing focus; item that only opens in the tray becomes `running`; with MSFS 2020 and/or 2024, SimConnect items wait and open after the main menu (confirming the pipe name); closing the simulator during the wait does not launch the items; X-Plane trigger disables the option

## 4. Phase 4 — v0.2.0 release and updater test

<!-- 4.1: open item recorded on 2026-09-22. bootstrap-autostart-app can only be archived after the updater test with this v0.2.0 (its task 4.2); archive bootstrap before this one. -->
- [x] 4.1 Archive the `bootstrap-autostart-app` change (prerequisite for archiving this one) when its remaining tasks allow it, or record the open item. Verify with `openspec validate improve-item-launching --strict` without archiving warnings
- [x] 4.2 Bump the version to 0.2.0 in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`, date the CHANGELOG, merge into `main` and, with the maintainer's confirmation, push the `v0.2.0` tag. Verify the published release with `AutoStart-Setup.exe`, `.sig` and a `latest.json` pointing to `AutoStart-Setup.exe`
- [ ] 4.3 CHECKPOINT Phase 4: with v0.1.0 installed on Windows, the maintainer uses "Check for updates now", accepts v0.2.0 and confirms the download, signature verification, reinstall and restart on the new version with profiles preserved (this also closes task 4.2 of the `bootstrap-autostart-app` change)
