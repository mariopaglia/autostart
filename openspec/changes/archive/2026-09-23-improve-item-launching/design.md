## Context

Motivation and scope in `proposal.md`; behavior in the deltas under `specs/`. Relevant current state (v0.1.0):

- `launcher.rs` starts regular apps with `std::process::Command` (detached flags + `raw_arg`) and elevated apps with `ShellExecuteExW("runas")` in `platform/windows/elevation.rs`. Neither path returns the PID to the caller: confirmation (`wait_for_process`) and closing rely only on the item's `processName`.
- `monitor/runner.rs::launch_items` walks the enabled items in a single sequential pass; the `Session` holds a frozen copy of the profile, used for closing (`close_targets`).
- `processes.rs` uses `sysinfo` (names, PIDs and executable paths); `sysinfo` also exposes each process's parent PID and start time.
- `AppState` owns the profiles; the frontend receives backend-made changes through events (`settings://changed`, used by the tray).
- Constraints: small, flat Rust, `unsafe` isolated in `platform/windows/`, stubs in `platform/fallback.rs` for macOS, logic in testable pure functions.

## Goals / Non-Goals

**Goals:**
- A single launch entry point in the `platform` layer that returns the PID and accepts the "elevated" and "minimized" options.
- Tracking and choice of the learned process as pure functions, testable without the OS.
- SimConnect detection without depending on the MSFS SDK.

**Non-Goals:**
- Actually connecting to SimConnect (reading flight data, aircraft state).
- Supporting custom `SimConnect.xml` configurations (a pipe with another name or TCP only); in that case the wait ends in a timeout error, as the spec states.
- Learning the process name of items in manual mode or of URL items.
- Forcing minimization of apps that restore their own window after the observation period.

## Decisions

### D1. Unified launch via `ShellExecuteExW`, returning the PID
`platform::launch(exe, args, working_dir, LaunchOptions { elevated, minimized }) -> AppResult<u32>` replaces `launch_elevated` and the `Command` in `launcher.rs`. On Windows, it uses `ShellExecuteExW` with `SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC`, the `runas` verb when elevated (or no verb, equivalent to "open"), `nShow = SW_SHOWMINNOACTIVE` when minimized (otherwise `SW_SHOWNORMAL`), and gets the PID with `GetProcessId(hProcess)`, closing the handle in a `Drop` guard. `lpParameters` receives the `args` as a single string, preserving the user's quotes as `raw_arg` did.
- *Rejected alternative:* keeping `Command` and using `CommandExt::show_window`, which is still unstable in stable Rust; or calling `CreateProcessW` directly, which would add a second large `unsafe` block just for the minimized case.
- *Consequence:* a single launch path on Windows and the PID available for elevated items too. The `DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP` flags are no longer needed: a process created by the shell does not inherit AutoStart's console or group. Since this replaces a path already validated in v0.1.0, the phase checkpoint repeats the regular launch test, with arguments and working folder.
- On macOS, the stub uses `Command::spawn` and returns `child.id()`; `minimized` is ignored.

### D2. Process tree tracking and name choice (pure functions)
New `process_tracker.rs` module, with no OS access:
- `ProcessSample { pid, parent_pid, name, exe_path, start_time }` describes a process in one `sysinfo` reading.
- `expand_tracked(tracked: &HashSet<u32>, samples: &[ProcessSample]) -> HashSet<u32>` adds every process whose parent is already in the set. The set only grows, so grandchildren stay tracked even after the intermediate launcher has exited.
- `choose_process_name(root, tracked_alive, new_in_install_dir) -> Option<String>` applies the spec's priority: the live root process; otherwise the oldest live descendant (by `start_time`, ties broken by name); otherwise the oldest new process whose executable is under the item's `exePath` folder (case-insensitive path comparison); otherwise `None`.
- A "new process" is any PID missing from the snapshot taken immediately before the launch.

`launcher.rs` does the polling (every 500 ms): up to 10 s for launch confirmation (any live tracked PID or the `processName` running) and up to 30 s for learning. Learning runs in a separate task per item, so it does not delay the launch of the following items.
- *Rejected alternative:* Windows Job Objects (which follow the tree natively), because they require creating the process suspended and more `unsafe`, and they do not catch apps the launcher opens through the shell. The install-folder heuristic covers that case.

### D3. Applying the learned name
When `choose_process_name` returns a name different from the current one and the item is in `ProcessNameMode::Auto`:
1. `Session::learn_process_name(item_id, name)` updates the session's profile copy, and that session's closing already uses the new name.
2. `AppState::learn_process_name(profile_id, item_id, name)` updates the persisted profile, only if the item still exists and is still in `Auto` mode (the user may have edited the item mid-session), and writes it to disk.
3. The backend emits `profiles://changed` with the updated `Profile`; `profiles-store` upserts it, and the form and card reflect the name without reloading.
- Learning also runs on "Test launch", which lets the user configure a launcher without starting the simulator.

### D4. Minimizing after launch
`platform::minimize_new_windows(pids, already_minimized: &mut HashSet<isize>)` enumerates the visible top-level windows (`EnumWindows` + `IsWindowVisible`) of the tracked PIDs and applies `ShowWindow(SW_SHOWMINNOACTIVE)` to each window we have not minimized yet, recording the handle. The launcher calls this function every 250 ms for 10 s after the item becomes `running`. Since each window is minimized only once, if the user restores the window during that period AutoStart does not fight them. `SW_SHOWMINNOACTIVE` rather than `SW_MINIMIZE`, because the latter activates the next window and can steal focus from the simulator.
- On macOS, an empty stub.

### D5. SimConnect detection via named pipe
The MSFS SimConnect server opens the named pipe `\\.\pipe\Microsoft Flight Simulator\SimConnect` when it is ready for connections. `platform::is_simconnect_available() -> bool` calls `WaitNamedPipeW(name, 1)`: success or `ERROR_SEM_TIMEOUT` (the pipe exists, but all instances are busy) mean available; `ERROR_FILE_NOT_FOUND` means unavailable. The pipe name lives in a single constant.
- *Rejected alternative:* using the SimConnect SDK (`SimConnect.dll` and `SimConnect_Open`). It gives the strongest confirmation, but requires redistributing the SDK DLL, additional FFI and coupling to the MSFS version, for a small gain over the pipe's existence.
- `supports_simconnect(trigger_process_name) -> bool` (pure function) accepts `FlightSimulator.exe` and `FlightSimulator2024.exe`, case-insensitively. The frontend mirrors the list in `trigger-presets.ts` (`SIMCONNECT_TRIGGERS`) to disable the checkbox.
- On macOS, the stub returns `true`, so the second-phase flow can be exercised during development.

### D6. Two-phase launch in the runner
`runner::launch_items` splits the enabled items into `immediate` and `deferred` (`waitForSimConnect && item is app`), preserving relative order (pure, testable `split_launch_phases` function). After the first phase:
- Real session with a trigger that supports SimConnect: every `deferred` item gets `ItemStatus::WaitingSimConnect`; the runner checks every 2 s for up to 10 min. Once SimConnect is available, it records `TimelineKind::SimConnectReady` and launches the `deferred` items in order, with their delays. On timeout, each waiting item gets `error` with `AppError::SimConnectUnavailable`.
- Test session: records `TimelineKind::SimConnectWaitSkipped` and launches the `deferred` items without waiting.
- Unsupported trigger: launches the `deferred` items without waiting.
- Ending the session already aborts the launch task (`abort(self.launch_task)`), which stops the wait and prevents the waiting items from launching, with no new code.
- A 10 min limit rather than 5: MSFS 2024 can take several minutes to reach the main menu on modest machines, and a short limit would produce false errors.

### D7. Model and compatibility
- `AppItem` gains `process_name_mode: ProcessNameMode` (`auto` | `manual`, `#[serde(default)]` = `Auto`), `start_minimized: bool` and `wait_for_simconnect: bool` (`#[serde(default)]` = false). `ItemStatus` gains `WaitingSimConnect`; `TimelineKind` gains `SimConnectReady` and `SimConnectWaitSkipped`; `ErrorKind`/`AppError` gain `SimConnectUnavailable`. The bindings are regenerated, and the Zod schemas gain the same fields with `.default(...)`, which keeps importing v0.1.0 files and loading old `profiles.json` files valid.
- Local migration: when loading `profiles.json`, `storage.rs` fills a missing `processNameMode` with `manual` when the `processName` differs from the executable name (the user manually fixed a launcher in v0.1.0) and with `auto` otherwise. Without this, learning could replace the user's fix with the name of a launcher that stays open. Imported files use the schema default (`auto`).
- `schemaVersion` stays at 1: there are only new fields with default values. On a downgrade to v0.1.0, unknown fields are ignored by serde and dropped by Zod, without errors.

### D8. Form
- `AppItemForm`: "Start minimized" and "Wait for SimConnect" use the same control as "Run as administrator" (`Switch` with a label), for visual consistency.
- New "Advanced" section with shadcn's `collapsible` component (added via the CLI), containing the working folder and the process name. Editing the name sets `processNameMode = manual`; the "Detect automatically" button sets `auto` and restores the name from the executable.
- The form receives the profile's trigger to decide whether SimConnect is supported.

## Risks / Trade-offs

- [Replacing `Command` with `ShellExecuteExW` may change the behavior of apps that already worked] → Same handling of `args` (raw string) and `workingDir`. The checkpoint repeats the v0.1.0 regular launch test, with arguments and working folder.
- [The heuristic picks the wrong process (e.g. the launcher opens an updater and the app)] → Priority for the oldest descendant, scope restricted to the install folder, and manual mode as a way out. The learned name appears in the form, and the timeline records the learning.
- [Unrelated new processes in the same folder (e.g. two apps in the same directory)] → The folder heuristic is only used when there is no live descendant, and only with processes that appeared after the launch.
- [Different pipe name in MSFS 2024, or a custom `SimConnect.xml`] → Single constant, easy to adjust. The checkpoint validates the name on MSFS 2020 and 2024 before the release. A custom configuration results in a clear timeout error.
- [SimConnect available does not mean "aircraft loaded"] → The spec only promises "available for connections", which is what addons need to start. The limitation is documented in the README.
- [Apps that restore themselves after 10 s or ignore `nShow`] → Limitation accepted by the spec and documented.
- [Learning writes to the profile while the user edits the same item] → `AppState` only applies the name if the item is still in `auto` mode, and the user's save, which sends the whole item, wins as the most recent write.

## Migration Plan

1. Implement and validate on Windows (checkpoints in `tasks.md`).
2. Archive the `bootstrap-autostart-app` change, a prerequisite for archiving this one, which modifies its requirements.
3. Bump the version to 0.2.0 in `package.json`, `Cargo.toml` and `tauri.conf.json`, update the CHANGELOG, merge into `main` and push the `v0.2.0` tag.
4. With v0.1.0 installed, validate the update to v0.2.0 through the app (task 4.2 of the `bootstrap-autostart-app` change).
- Rollback: unpublishing the v0.2.0 release makes `latest.json` point back to v0.1.0. Profiles saved by v0.2.0 remain readable by v0.1.0 (D7).
