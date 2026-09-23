## MODIFIED Requirements

### Requirement: Per-state tray icon
The app SHALL show a system tray icon with distinct visual variants for `idle`, `simRunning`/`closePending`/`closing` and `paused`, and a tooltip with the app name and the current state. During a session the tooltip SHALL name the session's profile; otherwise it SHALL show how many profiles are being watched.

#### Scenario: Simulator starts
- **WHEN** the monitor enters `simRunning` with the "MSFS 2024" profile
- **THEN** the tray icon switches to the "running" variant and the tooltip shows "MSFS 2024" and "Simulator running"

#### Scenario: Waiting for the simulator
- **WHEN** the monitor is `idle` with two enabled profiles
- **THEN** the tooltip shows that 2 profiles are being watched

### Requirement: Tray menu
The tray menu SHALL contain: a submenu with the profiles, each one checked when enabled, where picking a profile enables or disables it following the one-enabled-profile-per-trigger rule; "Close apps now" and "Keep apps open" while the monitor is in `closePending`; "Pause monitoring"/"Resume monitoring"; "Open AutoStart" and "Quit". The labels SHALL follow the configured language, and the menu SHALL reflect profile and state changes without restarting the app.

#### Scenario: Switch profile from the tray
- **WHEN** the user picks the disabled profile "MSFS Offline" in the submenu while "MSFS Online", with the same trigger, is enabled
- **THEN** "MSFS Offline" becomes enabled, "MSFS Online" becomes disabled, and the UI (if open) reflects the change

#### Scenario: Keep apps open from the tray
- **WHEN** the monitor is in `closePending` and the user picks "Keep apps open"
- **THEN** the session ends without closing any item

#### Scenario: Icon click
- **WHEN** the user left-clicks the tray icon
- **THEN** the main window is shown and focused
