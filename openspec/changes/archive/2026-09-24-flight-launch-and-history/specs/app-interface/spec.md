## MODIFIED Requirements

### Requirement: Main screen
The main screen SHALL contain: a sidebar with the profile list (create, rename, duplicate, delete, export, import), showing for each profile whether it is enabled and which one is in the current session, always visible so that clicking a profile from any screen opens it on the Profiles screen; a top bar with the monitor status (Waiting for simulator / Starting <simulator> / Simulator running / Closing soon / Closing apps / Paused), the enabled toggle and the triggers of the displayed profile, the "Start flight" button, and the "Test launch" and "Test close" buttons; and a central area with the profile's items as cards. "Start flight" SHALL be shown only when a trigger of the displayed profile has a start target. With several such triggers, it SHALL let the user pick the simulator. While the monitor is in `simStarting`, the top bar SHALL offer "Cancel". While the monitor is in `closePending`, the window SHALL show, above whichever screen is open, the countdown with the "Close now" and "Keep apps open" actions.

#### Scenario: Live status
- **WHEN** the monitor changes state
- **THEN** the top bar updates without reloading the screen

#### Scenario: Close delay countdown
- **WHEN** the simulator exits with `closeDelayMs = 60000`
- **THEN** the window shows "Closing apps in 60 s", counting down every second, with the "Close now" and "Keep apps open" buttons

#### Scenario: Enabling a conflicting profile
- **WHEN** the user turns on the enabled toggle of "MSFS Offline" while "MSFS Online" uses the same trigger
- **THEN** the sidebar shows "MSFS Online" as disabled and a notice names it

#### Scenario: Several triggers in the top bar
- **WHEN** the displayed profile has the triggers MSFS 2020 and MSFS 2024
- **THEN** the top bar shows both, lets the user remove either one while at least one remains, and lets the user add another from the presets, the running processes or a typed name

#### Scenario: Start flight with two simulators
- **WHEN** the displayed profile has start targets for MSFS 2020 and MSFS 2024 and the user clicks "Start flight"
- **THEN** a menu lets the user pick MSFS 2020 or MSFS 2024, and picking one starts the flight

#### Scenario: Status while the simulator starts
- **WHEN** the monitor is in `simStarting` for MSFS 2024
- **THEN** the top bar shows "Starting MSFS 2024" and a "Cancel" button

### Requirement: Item form
The form SHALL allow adding/editing an `app` item, choosing the `.exe` through the native dialog (filling in name, icon and process automatically), or a `url` item. The form SHALL also open pre-filled from an app or URL candidate (picker or drag and drop), with every field still editable before saving. Main app fields: name, path, arguments, delay, run as admin, start minimized, wait for SimConnect, open before the simulator, reopen if it crashes and close behavior. An "Advanced" section, collapsed by default, SHALL contain the working folder and the process name, indicating when the name is detected automatically and offering an action to return to automatic mode when in manual mode. The "Wait for SimConnect" option SHALL be disabled, with a hint explaining why, when none of the profile's triggers is MSFS 2020 or 2024. Turning on "Open before the simulator" SHALL turn off "Wait for SimConnect", and turning on "Wait for SimConnect" SHALL turn off "Open before the simulator". URL fields: name, URL (web address or Steam launch link), delay. When the profile has two or more triggers, both app and URL forms SHALL show an "Only with" field listing the profile's simulators, where no selection means every simulator. Validation SHALL use the same schemas as persistence and show per-field errors. When the executable path is already used by another item of the same profile, the form SHALL show a non-blocking "already in this profile" warning.

#### Scenario: Add an app through the dialog
- **WHEN** the user opens the app picker, chooses "Browse for file…" and selects an `.exe`
- **THEN** name, icon and process are filled in, and the user can save without opening the Advanced section

#### Scenario: Form pre-filled from a candidate
- **WHEN** the user picks "Little Navmap" in the app picker
- **THEN** the form opens with name, icon, path, arguments, working folder and process name taken from the candidate

#### Scenario: Automatically detected name
- **WHEN** the user opens the Advanced section of an item in automatic mode
- **THEN** the process name field shows the current name with the "Detected automatically" label

#### Scenario: Return to automatic mode
- **WHEN** the item is in manual mode and the user clicks "Detect automatically"
- **THEN** the item returns to `processNameMode = auto` and the name is maintained by the system

#### Scenario: SimConnect unavailable for the trigger
- **WHEN** the profile's only trigger is `X-Plane.exe` and the user opens an app's form
- **THEN** the "Wait for SimConnect" option appears disabled with the hint that it only applies to MSFS 2020/2024

#### Scenario: Duplicate executable
- **WHEN** the profile already has an item for `C:\Tools\vPilot\vPilot.exe` and the user adds the same executable again
- **THEN** the form shows the "already in this profile" warning and still allows saving

#### Scenario: Reopen if it crashes
- **WHEN** the user turns on "Reopen if it crashes" and saves the item
- **THEN** the item is saved with `restartOnCrash = true` and the option's hint explains that apps closed normally are not reopened

#### Scenario: Profile with a single simulator
- **WHEN** the profile has only the MSFS 2024 trigger
- **THEN** the form does not show the "Only with" field

#### Scenario: Open before the simulator hint
- **WHEN** the user turns on "Open before the simulator"
- **THEN** the option's hint explains that it applies only when the flight is started from AutoStart

## ADDED Requirements

### Requirement: Trigger start target editor
Each trigger shown in the top bar SHALL offer a way to set, change or remove its start target. For MSFS 2020 and MSFS 2024, the editor SHALL offer "Steam" and "Microsoft Store" as one-click choices. For every trigger, it SHALL offer "Choose executable…" (native file dialog, `.exe` only) and a text field for a Steam link or a Microsoft Store app. A trigger with a start target SHALL show a visual mark. Invalid values SHALL show the validation message on the field.

#### Scenario: Set MSFS 2020 from the Microsoft Store
- **WHEN** the user opens the editor of the MSFS 2020 trigger and picks "Microsoft Store"
- **THEN** the trigger is saved with the Microsoft Store start target for MSFS 2020, and "Start flight" appears in the top bar

#### Scenario: Remove the start target
- **WHEN** the user removes the only start target of the displayed profile
- **THEN** the "Start flight" button disappears from the top bar
