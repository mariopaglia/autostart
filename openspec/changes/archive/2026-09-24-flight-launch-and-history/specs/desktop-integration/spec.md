## MODIFIED Requirements

### Requirement: Per-state tray icon
The app SHALL show a system tray icon with distinct visual variants for `idle`, `simStarting`/`simRunning`/`closePending`/`closing` and `paused`, and a tooltip with the app name and the current state. During a session the tooltip SHALL name the session's profile; otherwise it SHALL show how many profiles are being watched.

#### Scenario: Simulator starts
- **WHEN** the monitor enters `simRunning` with the "MSFS 2024" profile
- **THEN** the tray icon switches to the "running" variant and the tooltip shows "MSFS 2024" and "Simulator running"

#### Scenario: Waiting for the simulator
- **WHEN** the monitor is `idle` with two enabled profiles
- **THEN** the tooltip shows that 2 profiles are being watched

#### Scenario: Starting the simulator
- **WHEN** the monitor enters `simStarting` with the "MSFS" profile
- **THEN** the tray icon switches to the "running" variant and the tooltip shows "MSFS" and "Starting simulator"

### Requirement: Tray menu
The tray menu SHALL contain: a "Start flight" submenu, shown only when at least one profile has a trigger with a start target, listing each such trigger as "<profile> — <simulator>" and enabled only while the monitor is `idle`; a submenu with the profiles, each one checked when enabled, where picking a profile enables or disables it following the one-enabled-profile-per-trigger rule; "Cancel start" while the monitor is in `simStarting`; "Close apps now" and "Keep apps open" while the monitor is in `closePending`; "Pause monitoring"/"Resume monitoring"; "Open AutoStart" and "Quit". The labels SHALL follow the configured language, and the menu SHALL reflect profile and state changes without restarting the app.

#### Scenario: Switch profile from the tray
- **WHEN** the user picks the disabled profile "MSFS Offline" in the submenu while "MSFS Online", with the same trigger, is enabled
- **THEN** "MSFS Offline" becomes enabled, "MSFS Online" becomes disabled, and the UI (if open) reflects the change

#### Scenario: Keep apps open from the tray
- **WHEN** the monitor is in `closePending` and the user picks "Keep apps open"
- **THEN** the session ends without closing any item

#### Scenario: Icon click
- **WHEN** the user left-clicks the tray icon
- **THEN** the main window is shown and focused

#### Scenario: Start a flight from the tray
- **WHEN** the monitor is `idle` and the user picks "Start flight → MSFS — MSFS 2024"
- **THEN** the flight starts as described in the Start flight action, without opening the main window

#### Scenario: No start targets
- **WHEN** no trigger of any profile has a start target
- **THEN** the tray menu has no "Start flight" submenu
