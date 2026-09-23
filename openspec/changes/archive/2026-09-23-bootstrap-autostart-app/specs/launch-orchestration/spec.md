## Purpose

Launches the profile's items in the configured order when the session starts and, at the end of the session, closes only what should be closed, identifying processes by name to handle launchers and updaters.

## ADDED Requirements

### Requirement: Ordered launch with delay
When a session starts, the system SHALL process the profile's enabled items in list order, waiting each item's `delayMs` before launching it. Disabled items SHALL be ignored without producing a status.

#### Scenario: Three apps in sequence
- **WHEN** the session starts with items A (delay 0), B (delay 800) and C (delay 2000)
- **THEN** A launches immediately, B about 800 ms after A, and C about 2000 ms after B

### Requirement: Pre-existing app detection
Before launching an `app` item, the system SHALL check whether a process with the item's `processName` already exists. If it does, the item SHALL NOT be launched and SHALL get the `skipped` status with the reason "pre-existing".

#### Scenario: App already running
- **WHEN** Navigraph Charts is already running when the session starts
- **THEN** it is not launched again and the card shows "Already open"

### Requirement: Item execution
An `app` item SHALL be started with its `args` and `workingDir` (default: the exe folder). A `url` item SHALL be opened in the default browser. A launch failure SHALL mark the item with the `error` status and the cause's message, without interrupting the following items.

#### Scenario: Removed executable
- **WHEN** an item's `exePath` no longer exists
- **THEN** the item gets the `error` status ("Executable not found") and the next items keep being launched

#### Scenario: Launch confirmation
- **WHEN** an app is started and, within 10 seconds, a process with the item's `processName` appears
- **THEN** the item moves to `running`; if no process appears in time, the item gets `error` with the message "The process was not detected after launch"

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
The user SHALL be able to run "Test launch" and "Test close" for a profile without the simulator running, using exactly the same rules as the real session. "Test close" SHALL close the items launched by the last "Test launch" (or, if there was no previous test, SHALL treat every running item as launched by AutoStart, after user confirmation). The tests SHALL be unavailable while the monitor is in `simRunning` or `closing`.

#### Scenario: Test launch
- **WHEN** the user clicks "Test launch" with the monitor in `idle`
- **THEN** the enabled items launch in order, with their statuses updating on the cards

#### Scenario: Test blocked during a flight
- **WHEN** the monitor is in `simRunning`
- **THEN** the test buttons are disabled, with a hint explaining why
