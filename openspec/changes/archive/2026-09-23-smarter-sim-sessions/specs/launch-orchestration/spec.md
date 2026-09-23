## MODIFIED Requirements

### Requirement: Manual tests
The user SHALL be able to run "Test launch" and "Test close" for a profile without the simulator running, using the same rules as the real session, with two exceptions: in "Test launch", items that wait for SimConnect SHALL launch in the second phase without waiting for SimConnect, and the timeline SHALL record that the wait was skipped in the test; and items with `restartOnCrash` SHALL NOT be relaunched after a crash. "Test close" SHALL close the items launched by the last "Test launch" (or, if there was no previous test, SHALL treat every running item as launched by AutoStart, after user confirmation). The tests SHALL be unavailable while the monitor is in `simRunning`, `closePending` or `closing`.

#### Scenario: Test launch
- **WHEN** the user clicks "Test launch" with the monitor in `idle`
- **THEN** the enabled items launch in the order of the two phases, with their statuses updating on the cards

#### Scenario: Test with an item that waits for SimConnect
- **WHEN** the user tests the launch of a profile with an item that waits for SimConnect and the simulator is not running
- **THEN** the item launches in the second phase without waiting, and the timeline shows that the SimConnect wait was skipped in the test

#### Scenario: Test blocked during a flight
- **WHEN** the monitor is in `simRunning` or `closePending`
- **THEN** the test buttons are disabled, with a hint explaining why

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

## ADDED Requirements

### Requirement: Relaunch after a crash
During a real session in `simRunning`, the system SHALL watch every `app` item with `restartOnCrash = true` whose status is `running` (whether launched by AutoStart or pre-existing). When every process of the item has exited and at least one of them exited with a non-zero exit code, the system SHALL consider it a crash: the item SHALL get the `restarting` status and SHALL be launched again after 5 seconds, following the item's launch rules. When all of the item's processes exited with exit code 0, the system SHALL consider it closed by the user and SHALL stop watching it. The system SHALL relaunch an item at most 3 times per session; after the third crash, the item SHALL get the `error` status with the message "The app kept crashing and was not reopened". A relaunched item SHALL count as launched by AutoStart for the closing rules. Relaunch SHALL stop as soon as the session leaves `simRunning`.

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
