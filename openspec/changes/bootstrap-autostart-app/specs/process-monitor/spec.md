## Purpose

Continuously watches system processes to detect when the simulator (the active profile's trigger process) starts or exits, triggering the launch and closing of items.

## ADDED Requirements

### Requirement: Trigger detection by name
The monitor SHALL check every 2 seconds whether any process exists whose executable name matches the active profile's `trigger.processName`, using a case-insensitive comparison.

#### Scenario: Simulator starts
- **WHEN** the state is `idle` and a `flightsimulator2024.exe` process appears, with the trigger configured as `FlightSimulator2024.exe`
- **THEN** within 2 seconds the state changes to `simRunning` and item launch begins

### Requirement: State machine
The monitor SHALL operate with the states `idle`, `simRunning`, `closing` and `paused`, with the transitions: `idle → simRunning` (trigger detected), `simRunning → closing` (trigger missing in 2 consecutive checks), `closing → idle` (closing finished), any state → `paused` (user pauses) and `paused → idle` (user resumes). While in `closing`, the monitor SHALL NOT start a new launch.

#### Scenario: Simulator exits
- **WHEN** the state is `simRunning` and the trigger is missing in two consecutive checks
- **THEN** the state changes to `closing`, the items are closed according to the rules and the state returns to `idle`

#### Scenario: Momentary flapping
- **WHEN** the trigger disappears in one check and reappears in the next
- **THEN** the state stays `simRunning` and nothing is closed

#### Scenario: Simulator reopened during closing
- **WHEN** the trigger reappears while the state is `closing`
- **THEN** closing finishes, the state goes to `idle` and, on the next check, it enters `simRunning` and launches the items again

### Requirement: Monitoring pause
The user SHALL be able to pause and resume monitoring from the tray and from the UI. Pausing during `simRunning` SHALL NOT close the launched items, and the ongoing session SHALL be discarded.

#### Scenario: Pause with the simulator running
- **WHEN** the user pauses with the simulator running and then closes the simulator
- **THEN** no item is closed

### Requirement: App started with the simulator already running
If the trigger is already running when AutoStart starts (or when monitoring is resumed), the monitor SHALL enter `simRunning` and launch normally, with already-running items marked as pre-existing.

#### Scenario: AutoStart opened after the simulator
- **WHEN** AutoStart starts with MSFS already running and Volanta already open
- **THEN** Volanta is marked `skipped` (pre-existing) and the other enabled items are launched

### Requirement: Profile snapshot for the session
The session SHALL use a copy of the active profile taken at the moment of the transition to `simRunning`. Edits to the profile or switching the active profile during the session SHALL NOT affect that session's closing.

#### Scenario: Profile switch during a flight
- **WHEN** the user switches the active profile with the simulator running
- **THEN** when the simulator exits, the items of the profile that opened the session are closed

### Requirement: Events for the interface
The monitor SHALL emit events to the frontend on every monitor state change, on every item status change (`pending`, `launching`, `running`, `skipped`, `closing`, `closed`, `error`) and on every log entry. The current state SHALL also be available on demand.

#### Scenario: Window opened after the start
- **WHEN** the window is opened from the tray in the middle of a session
- **THEN** the UI queries the current state and correctly shows the status of the monitor and of each item
