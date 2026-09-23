# privilege-elevation Specification

## Purpose
Supports helper apps that must run as administrator and makes it clear to the user when AutoStart needs to be elevated to be able to close them.

## Requirements

### Requirement: Run item as administrator
An item with `runAsAdmin = true` SHALL be started with an elevation request (UAC prompt). If the user denies UAC, the item SHALL get the `error` status and the message "Elevation denied".

#### Scenario: UAC accepted
- **WHEN** the session launches an item with `runAsAdmin = true` and the user accepts UAC
- **THEN** the app runs elevated and the item becomes `running`

#### Scenario: UAC denied
- **WHEN** the user denies UAC
- **THEN** the item becomes `error` and the following items keep being launched

### Requirement: Detect AutoStart elevation
The system SHALL report whether AutoStart itself is running elevated. When the active profile has an enabled item with `runAsAdmin = true`, a close behavior other than `keep`, and AutoStart is not elevated, the UI SHALL show a persistent warning explaining that this app may not be closed, with the "Restart as administrator" action. AutoStart SHALL NOT require elevation by default.

#### Scenario: Warning shown
- **WHEN** the active profile has SPAD.neXt with `runAsAdmin = true` and `onClose = graceful`, and AutoStart is not elevated
- **THEN** the main screen shows the warning banner with the "Restart as administrator" button

### Requirement: Restart as administrator
The "Restart as administrator" action SHALL open a new elevated instance (via UAC) and terminate the current instance only after the new one is accepted. If UAC is denied, the current instance SHALL keep running and report the failure.

#### Scenario: Restart accepted
- **WHEN** the user clicks "Restart as administrator" and accepts UAC
- **THEN** the old instance exits, the new one opens elevated and the warning disappears

#### Scenario: Restart denied
- **WHEN** the user denies UAC
- **THEN** the current instance stays open and shows "Elevation cancelled"

### Requirement: Permission error when closing
If terminating a process fails due to missing permission, the item SHALL get the `error` status and the message SHALL state that AutoStart must be run as administrator.

#### Scenario: Access denied when closing
- **WHEN** a non-elevated AutoStart tries to terminate an elevated process
- **THEN** the item becomes `error` with the message "Access denied: run AutoStart as administrator"
