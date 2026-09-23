## Why

In v0.1.0, users must manually get the `processName` right for apps that use a launcher (the chosen `.exe` starts another process and exits), opening apps minimized requires knowing each app's specific command-line arguments, and addons that depend on SimConnect fail if they open before MSFS is ready. Tools like Addon Linker already solve these points, and the community expects the same. This version (v0.2.0) also serves as the first real update delivered through the updater, validating task 4.2 of the `bootstrap-autostart-app` change end to end.

## What Changes

- **Automatically learned process name**: when launching an `app` item, AutoStart tracks the processes it started (the launched process and its descendants, falling back to new processes whose executable lives in the same install folder). When the started `.exe` exits and the real app keeps running, that process's name is saved on the item. The user only provides the name and executable path.
- **Process name mode**: each item gets `processNameMode` (`auto` | `manual`). In the form, the "Process name" field moves out of the main area into an "Advanced" section, labeled "Detected automatically"; editing the field switches the mode to `manual`, and it is possible to switch back to automatic.
- **Start minimized**: new per-`app`-item checkbox (`startMinimized`). The app is started with a minimized-window request and, during the first seconds, AutoStart minimizes the process's windows. Apps that ignore the request or restore themselves are a documented limitation.
- **Wait for SimConnect**: new per-`app`-item checkbox (`waitForSimConnect`). When the session launches, items without the option open first, in list order; checked items open afterwards, also in order, once MSFS's SimConnect is available. If SimConnect does not become available within the time limit, the item gets an error. The option only applies to MSFS 2020/2024 triggers and is disabled, with a hint, for other triggers.
- **New item status** `waitingSimConnect`, shown on the card and in the timeline.
- New card **badges**: "Minimized" and "SimConnect".
- Compatible with v0.1.0 profiles: the new fields have default values and old profiles remain valid (import/export included).

## Capabilities

### New Capabilities

None. The changes extend existing capabilities.

### Modified Capabilities

- `launch-orchestration`: launching now has two phases (regular items and items that wait for SimConnect); launch confirmation now considers the started process tree; new requirements for process name learning, minimized launch and waiting for SimConnect; manual tests do not wait for SimConnect.
- `profile-management`: the `app` item gains `processNameMode`, `startMinimized` and `waitForSimConnect`; the `processName` is no longer the user's responsibility by default.
- `process-monitor`: item status events now include `waitingSimConnect`.
- `app-interface`: the item form gains the new checkboxes and the "Advanced" section; the cards gain the new badges and status.

## Impact

- **Rust**: `models.rs` (new fields on `AppItem`, `ProcessNameMode`, `WaitingSimConnect` status), `launcher.rs` (process tracking, learning, minimizing), `monitor/runner.rs` (two launch phases), `processes.rs` (process tree by parent PID and by folder), `platform/windows/` (minimized window, minimizing a PID's windows, SimConnect named pipe detection, PID of the elevated process) and the stubs in `platform/fallback.rs`; persistence of the learned name through `AppState` with an event for the UI.
- **Frontend**: Zod schemas, `AppItemForm` (checkboxes and Advanced section), `ItemCard`/`ItemStatusBadge` (badges and status), timeline, profiles store (receives the profile updated by learning) and pt-BR/en translations.
- **Bindings** regenerated (`AppItem`, `ItemStatus`, `ProcessNameMode`).
- **Docs**: README (sections on process name, start minimized and SimConnect) and the v0.2.0 CHANGELOG.
- **Ordering dependency**: this change modifies requirements created by `bootstrap-autostart-app`, which must be archived before this one.
