## MODIFIED Requirements

### Requirement: Ordered launch with delay
When a session starts, the system SHALL launch the enabled items in two phases. In the first, it SHALL process, in list order, the items that do not wait for SimConnect. In the second, it SHALL process, in list order, the `app` items with `waitForSimConnect = true`, only after SimConnect is available. In both phases, the system SHALL wait each item's `delayMs` before launching it. Disabled items SHALL be ignored without producing a status.

#### Scenario: Three apps in sequence
- **WHEN** the session starts with items A (delay 0), B (delay 800) and C (delay 2000), none waiting for SimConnect
- **THEN** A launches immediately, B about 800 ms after A, and C about 2000 ms after B

#### Scenario: Items that wait for SimConnect go last
- **WHEN** the list is A, B (waits for SimConnect), C and D (waits for SimConnect)
- **THEN** A and C launch first, in that order, and B and D launch afterwards, in that order, once SimConnect is available

### Requirement: Item execution
An `app` item SHALL be started with its `args` and `workingDir` (default: the exe folder). A `url` item SHALL be opened in the default browser. A launch failure SHALL mark the item with the `error` status and the cause's message, without interrupting the following items.

#### Scenario: Removed executable
- **WHEN** an item's `exePath` no longer exists
- **THEN** the item gets the `error` status ("Executable not found") and the next items keep being launched

#### Scenario: Launch confirmation
- **WHEN** an app is started and, within 10 seconds, the started process, one of its descendants or a process with the item's `processName` is running
- **THEN** the item moves to `running`; if none of these processes exists in time, the item gets `error` with the message "The process was not detected after launch"

#### Scenario: Launcher that exits right after opening the app
- **WHEN** AutoStart starts `Launcher.exe`, which opens `Volanta.exe` and exits within 3 seconds
- **THEN** the item moves to `running`, because a process started by the launcher keeps running

### Requirement: Manual tests
The user SHALL be able to run "Test launch" and "Test close" for a profile without the simulator running, using the same rules as the real session, with one exception: in "Test launch", items that wait for SimConnect SHALL launch in the second phase without waiting for SimConnect, and the timeline SHALL record that the wait was skipped in the test. "Test close" SHALL close the items launched by the last "Test launch" (or, if there was no previous test, SHALL treat every running item as launched by AutoStart, after user confirmation). The tests SHALL be unavailable while the monitor is in `simRunning` or `closing`.

#### Scenario: Test launch
- **WHEN** the user clicks "Test launch" with the monitor in `idle`
- **THEN** the enabled items launch in the order of the two phases, with their statuses updating on the cards

#### Scenario: Test with an item that waits for SimConnect
- **WHEN** the user tests the launch of a profile with an item that waits for SimConnect and the simulator is not running
- **THEN** the item launches in the second phase without waiting, and the timeline shows that the SimConnect wait was skipped in the test

#### Scenario: Test blocked during a flight
- **WHEN** the monitor is in `simRunning`
- **THEN** the test buttons are disabled, with a hint explaining why

## ADDED Requirements

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
An `app` item with `waitForSimConnect = true` SHALL be launched only when the simulator's SimConnect is available for connections. While waiting, the item SHALL have the `waitingSimConnect` status. The system SHALL check availability every 2 seconds for up to 10 minutes from the start of the second phase; once the limit expires, items still waiting SHALL get the `error` status with the message "SimConnect did not become available". If the session ends during the wait, the waiting items SHALL NOT be launched. The option SHALL be honored only when the profile's trigger is MSFS 2020 (`FlightSimulator.exe`) or MSFS 2024 (`FlightSimulator2024.exe`); for other triggers, the item SHALL launch in the second phase without waiting.

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
