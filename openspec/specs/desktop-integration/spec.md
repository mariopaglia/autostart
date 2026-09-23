# desktop-integration Specification

## Purpose
Makes AutoStart behave like an unobtrusive tray utility: always available, with a single instance, starting with Windows if the user wants.

## Requirements

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

### Requirement: Closing the window minimizes to the tray
Closing the main window SHALL hide it without quitting the app. Only "Quit" in the tray menu SHALL terminate the process. Quitting during `simRunning` SHALL NOT close the launched items.

#### Scenario: Close with the X
- **WHEN** the user clicks the window's X
- **THEN** the window disappears, the icon stays in the tray and the monitor keeps running

### Requirement: Single instance
Starting AutoStart when it is already running SHALL only show and focus the existing instance's window.

#### Scenario: Double click on the shortcut
- **WHEN** the user opens AutoStart from the Start menu while the app is already running in the tray
- **THEN** no new instance keeps running and the existing window is brought to the foreground

### Requirement: Start with Windows
When `startWithWindows` is enabled, the app SHALL be registered to start at user login (without elevation); when disabled, the registration SHALL be removed. The displayed state SHALL reflect the actual system registration.

#### Scenario: Enable autostart
- **WHEN** the user enables "Start with Windows" and restarts the computer
- **THEN** AutoStart starts at login, according to the `startMinimized` setting

### Requirement: Start minimized
When `startMinimized` is enabled, the app SHALL start in the tray only, without showing the window. On first run (onboarding pending), the window SHALL be shown regardless of this setting.

#### Scenario: Minimized start
- **WHEN** the app starts with `startMinimized = true` and onboarding completed
- **THEN** only the tray icon appears

### Requirement: Main window
The window SHALL be resizable, with an initial size of 1000x680, a minimum size of 800x560 and the title "AutoStart".

#### Scenario: Resize
- **WHEN** the user tries to shrink the window below 800x560
- **THEN** the window stops at the minimum size
