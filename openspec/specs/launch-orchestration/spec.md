# launch-orchestration Specification

## Purpose
Launches the profile's items in the configured order when the session starts and, at the end of the session, closes only what should be closed, identifying processes by name to handle launchers and updaters.

## Requirements

### Requirement: Ordered launch with delay
When a session starts, the system SHALL consider only the enabled items that apply to the trigger that started the session: items with an empty `onlyForTriggers`, or whose `onlyForTriggers` contains that trigger (case-insensitive). Items that don't apply SHALL be ignored for the whole session: no status, no launch, no closing and no crash relaunch. The system SHALL launch the items it considers in phases, each in list order:
1. only in sessions started with "Start flight": the `app` items with `launchBeforeSimulator = true`, before the simulator is started;
2. the items that do not wait for SimConnect (and were not launched in phase 1);
3. the `app` items with `waitForSimConnect = true`, only after SimConnect is available.

In every phase, the system SHALL wait each item's `delayMs` before launching it. Disabled items SHALL be ignored without producing a status.

#### Scenario: Three apps in sequence
- **WHEN** the session starts with items A (delay 0), B (delay 800) and C (delay 2000), none waiting for SimConnect
- **THEN** A launches immediately, B about 800 ms after A, and C about 2000 ms after B

#### Scenario: Items that wait for SimConnect go last
- **WHEN** the list is A, B (waits for SimConnect), C and D (waits for SimConnect)
- **THEN** A and C launch first, in that order, and B and D launch afterwards, in that order, once SimConnect is available

#### Scenario: Item only for the other simulator
- **WHEN** a profile has the triggers MSFS 2020 and MSFS 2024, item A has `onlyForTriggers = [FlightSimulator2024.exe]` and the user opens MSFS 2020
- **THEN** item A is not launched, does not appear with a status in the session, and is not closed when the session ends

#### Scenario: Item for the running simulator
- **WHEN** the same profile's session is started by MSFS 2024
- **THEN** item A is launched in its list position

#### Scenario: Before-simulator phase
- **WHEN** a flight is started with "Start flight" and the list is A, B (`launchBeforeSimulator`), C
- **THEN** B launches before the simulator is started, and A and C launch, in that order, after the simulator is detected
### Requirement: Pre-existing app detection
Before launching an `app` item, the system SHALL check whether a process with the item's `processName` already exists. If it does, the item SHALL NOT be launched and SHALL get the `skipped` status with the reason "pre-existing".

#### Scenario: App already running
- **WHEN** Navigraph Charts is already running when the session starts
- **THEN** it is not launched again and the card shows "Already open"

### Requirement: Item execution
An `app` item SHALL be started with its `args` and `workingDir` (default: the exe folder). A `url` item with an `http` or `https` address SHALL be opened in the default browser. A `url` item with a Steam launch link SHALL be handed to Windows, which opens it with Steam. A launch failure SHALL mark the item with the `error` status and the cause's message, without interrupting the following items.

#### Scenario: Removed executable
- **WHEN** an item's `exePath` no longer exists
- **THEN** the item gets the `error` status ("Executable not found") and the next items keep being launched

#### Scenario: Launch confirmation
- **WHEN** an app is started and, within 10 seconds, the started process, one of its descendants or a process with the item's `processName` is running
- **THEN** the item moves to `running`; if none of these processes exists in time, the item gets `error` with the message "The process was not detected after launch"

#### Scenario: Launcher that exits right after opening the app
- **WHEN** AutoStart starts `Launcher.exe`, which opens `Volanta.exe` and exits within 3 seconds
- **THEN** the item moves to `running`, because a process started by the launcher keeps running

#### Scenario: Steam link
- **WHEN** a session launches a URL item with `steam://rungameid/1234560`
- **THEN** Windows opens the link with Steam, the item moves to `running`, and the item is not closed at the end of the session
### Requirement: Tracking of items launched by AutoStart
The session SHALL record which items were launched by AutoStart and which were pre-existing.

#### Scenario: Query the session
- **WHEN** a session is in progress
- **THEN** the exposed state tells, per item, whether it was launched by AutoStart or was pre-existing

### Requirement: Closing rules
At the end of the session, the system SHALL close the `app` items whose `onClose` is not `keep`. When `closeOnlyIfLaunchedByApp` is enabled, it SHALL close only the items launched by AutoStart in that session. Closing SHALL reach every process with the item's `processName`, not only the originally started PID.

#### Scenario: Launcher that switches process
- **WHEN** AutoStart started `Launcher.exe`, which exited and left `Volanta.exe` running (the item's processName = `Volanta.exe`)
- **THEN** at the end of the session `Volanta.exe` is closed

#### Scenario: Pre-existing app preserved
- **WHEN** `closeOnlyIfLaunchedByApp` is enabled and SPAD.neXt was already running before the simulator
- **THEN** SPAD.neXt stays open after the simulator exits

#### Scenario: Keep item
- **WHEN** an item has `onClose = keep`
- **THEN** it is never closed by AutoStart

### Requirement: Graceful close
For `onClose = graceful`, the system SHALL request closing of every top-level window of the item's processes and wait up to `gracefulTimeoutMs`. If any process still exists after the deadline, it SHALL force-terminate it. The final status SHALL be `closed`, or `error` if termination fails.

#### Scenario: App closes on its own
- **WHEN** the app responds to the close request within 1 second
- **THEN** the item becomes `closed` without forced termination, and the log records "Closed gracefully"

#### Scenario: App hangs while closing
- **WHEN** the app shows a "Save changes?" dialog and does not exit within `gracefulTimeoutMs`
- **THEN** the process is force-terminated and the log records "force closed after timeout"

### Requirement: Forced close
For `onClose = force`, the system SHALL immediately terminate every process with the item's `processName`.

#### Scenario: Force
- **WHEN** the session ends with a running `force` item
- **THEN** the process is terminated without waiting and the item becomes `closed`

### Requirement: Parallel closing with a time limit
Items SHALL be closed in parallel, and the `closing` state SHALL end within at most `gracefulTimeoutMs` + 5 seconds.

#### Scenario: Several graceful apps
- **WHEN** five graceful apps are closed with a 5000 ms timeout
- **THEN** the monitor returns to `idle` within 10 seconds

### Requirement: Manual tests
The user SHALL be able to run "Test launch" and "Test close" for a profile without the simulator running, using the same rules as the real session, with these exceptions: in "Test launch", items that wait for SimConnect SHALL launch in the last phase without waiting for SimConnect, and the timeline SHALL record that the wait was skipped in the test; items with `restartOnCrash` SHALL NOT be relaunched after a crash; every enabled item SHALL be considered regardless of `onlyForTriggers`; and `launchBeforeSimulator` SHALL be ignored. "Test close" SHALL close the items launched by the last "Test launch" (or, if there was no previous test, SHALL treat every running item as launched by AutoStart, after user confirmation). The tests SHALL be unavailable while the monitor is in `simStarting`, `simRunning`, `closePending` or `closing`.

#### Scenario: Test launch
- **WHEN** the user clicks "Test launch" with the monitor in `idle`
- **THEN** the enabled items launch in list order, with the items that wait for SimConnect last, and their statuses update on the cards

#### Scenario: Test with an item that waits for SimConnect
- **WHEN** the user tests the launch of a profile with an item that waits for SimConnect and the simulator is not running
- **THEN** the item launches in the last phase without waiting, and the timeline shows that the SimConnect wait was skipped in the test

#### Scenario: Test blocked during a flight
- **WHEN** the monitor is in `simStarting`, `simRunning` or `closePending`
- **THEN** the test buttons are disabled, with a hint explaining why

#### Scenario: Test covers every simulator
- **WHEN** the user tests the launch of a profile where one item applies only to MSFS 2024 and another only to MSFS 2020
- **THEN** both items are launched
### Requirement: Process name learning
For `app` items with `processNameMode = auto`, the system SHALL identify which process represents the app after starting it and SHALL save that name to the item's `processName`. If the started process keeps running until the end of the observation window (30 seconds), the learned name SHALL be its own. If it exits earlier, the learned name SHALL be that of a process it started (directly or indirectly) that is still running; with no descendants, it SHALL be that of a process that appeared after the launch with an executable inside the item's install folder. If there is no candidate, the `processName` SHALL remain unchanged. The learned name SHALL apply immediately to closing the ongoing session and SHALL be persisted in the profile, reflected in the interface without restarting. Items with `processNameMode = manual` SHALL NOT have their `processName` changed.

#### Scenario: App without a launcher
- **WHEN** AutoStart launches `C:\Apps\SPAD\Spad.exe` and it keeps running
- **THEN** the item's `processName` stays `Spad.exe`

#### Scenario: Launcher that switches process
- **WHEN** the item points to `C:\Apps\Volanta\Launcher.exe` in automatic mode, and the launcher opens `Volanta.exe` and exits
- **THEN** the item's `processName` becomes `Volanta.exe`, the session's closing closes `Volanta.exe`, and the item's form shows `Volanta.exe` as detected automatically

#### Scenario: App opened outside the process tree
- **WHEN** the launcher in `C:\Apps\Foo\` exits and the real app `C:\Apps\Foo\bin\Foo.exe` appears without being its descendant
- **THEN** the item's `processName` becomes `Foo.exe`

#### Scenario: Manual mode preserved
- **WHEN** the item has `processNameMode = manual` with `processName = Volanta.exe`
- **THEN** the system never changes that item's `processName`

### Requirement: Minimized launch
An `app` item with `startMinimized = true` SHALL be started with a request to Windows for a minimized window, without stealing focus. In addition, during the first 10 seconds after the item becomes `running`, the system SHALL minimize the visible top-level windows that appear in the item's processes. Apps that ignore the request or restore their own window after that period SHALL NOT produce an error.

#### Scenario: Minimized app
- **WHEN** the session launches an item with `startMinimized = true`
- **THEN** the app window stays minimized in the taskbar and focus stays where it was

#### Scenario: App that opens only in the tray
- **WHEN** an item with `startMinimized = true` creates no visible window
- **THEN** the item becomes `running` normally, without an error

### Requirement: Waiting for SimConnect
An `app` item with `waitForSimConnect = true` SHALL be launched only when the simulator's SimConnect is available for connections. While waiting, the item SHALL have the `waitingSimConnect` status. The system SHALL check availability every 2 seconds for up to 10 minutes from the start of the second phase; once the limit expires, items still waiting SHALL get the `error` status with the message "SimConnect did not become available". If the session ends during the wait, the waiting items SHALL NOT be launched. The option SHALL be honored only when the trigger that started the session is MSFS 2020 (`FlightSimulator.exe`) or MSFS 2024 (`FlightSimulator2024.exe`); otherwise, the item SHALL launch in the second phase without waiting.

#### Scenario: SimConnect becomes available
- **WHEN** MSFS 2024 starts and the Volanta item has `waitForSimConnect = true`
- **THEN** the card shows "Waiting for SimConnect" and Volanta launches as soon as SimConnect becomes available

#### Scenario: SimConnect does not become available
- **WHEN** SimConnect does not become available within 10 minutes
- **THEN** the waiting items become `error` with "SimConnect did not become available" and the timeline records the error

#### Scenario: Simulator closed during the wait
- **WHEN** the simulator is closed while an item waits for SimConnect
- **THEN** the item is not launched and the session proceeds to closing normally

#### Scenario: Trigger that is not MSFS
- **WHEN** the profile has trigger `X-Plane.exe` and an item with `waitForSimConnect = true`
- **THEN** the item launches in the second phase without waiting for SimConnect

#### Scenario: Profile with MSFS and another simulator
- **WHEN** a profile has the triggers `FlightSimulator2024.exe` and `X-Plane.exe`, and the session was started by `X-Plane.exe`
- **THEN** items with `waitForSimConnect = true` launch in the second phase without waiting for SimConnect

### Requirement: Relaunch after a crash
During a real session in `simStarting` or `simRunning`, the system SHALL watch every `app` item with `restartOnCrash = true` whose status is `running` (whether launched by AutoStart or pre-existing). When every process of the item has exited and at least one of them exited with a non-zero exit code, the system SHALL consider it a crash: the item SHALL get the `restarting` status and SHALL be launched again after 5 seconds, following the item's launch rules. When all of the item's processes exited with exit code 0, the system SHALL consider it closed by the user and SHALL stop watching it. The system SHALL relaunch an item at most 3 times per session; after the third crash, the item SHALL get the `error` status with the message "The app kept crashing and was not reopened". A relaunched item SHALL count as launched by AutoStart for the closing rules. Relaunch SHALL stop as soon as the session leaves `simStarting` or `simRunning`.

#### Scenario: App crashes mid-flight
- **WHEN** SPAD.neXt has `restartOnCrash = true` and crashes during the flight
- **THEN** the card shows "Reopening", SPAD.neXt is launched again about 5 seconds later, and the timeline records the crash and the relaunch

#### Scenario: User closes the app
- **WHEN** the user closes an app with `restartOnCrash = true` through its own window and it exits with code 0
- **THEN** the app is not reopened

#### Scenario: App that keeps crashing
- **WHEN** an item with `restartOnCrash = true` crashes for the fourth time in the same session
- **THEN** it is not reopened, the item gets `error` with "The app kept crashing and was not reopened" and the timeline records it

#### Scenario: Simulator closes while the app is down
- **WHEN** the simulator exits during the 5 seconds before a relaunch
- **THEN** the item is not relaunched

#### Scenario: Option disabled
- **WHEN** an item with `restartOnCrash = false` crashes during the session
- **THEN** it is not reopened and its status does not change

#### Scenario: Crash while the simulator loads
- **WHEN** an item opened before the simulator, with `restartOnCrash = true`, crashes while the state is `simStarting`
- **THEN** it is reopened following the same rules
