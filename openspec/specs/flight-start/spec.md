# flight-start Specification

## Purpose
Lets the pilot start a flight from AutoStart: open the tools that must run before the simulator, start the simulator itself and continue with the normal session, from the main window or the tray.

## Requirements

### Requirement: Trigger start target
A trigger SHALL have an optional `launchTarget`, which says how AutoStart starts that simulator. It SHALL be exactly one of: an absolute path to an `.exe` file, a Steam launch link (`steam://rungameid/<digits>` or `steam://run/<digits>`), or a Microsoft Store app in the form `shell:AppsFolder\<AppUserModelID>`. Any other value SHALL be rejected by validation. A trigger without `launchTarget` SHALL keep working as today (detection only). For the MSFS 2020 and MSFS 2024 presets, the system SHALL offer ready-made "Steam" and "Microsoft Store" targets. For every trigger, it SHALL also offer choosing an `.exe` file or typing a target.

#### Scenario: MSFS 2024 on Steam
- **WHEN** the user picks "Steam" as the start target of the MSFS 2024 trigger
- **THEN** the trigger is saved with `launchTarget = "steam://rungameid/2537590"`

#### Scenario: X-Plane from its executable
- **WHEN** the user chooses `D:\X-Plane 12\X-Plane.exe` as the start target of the X-Plane 12 trigger
- **THEN** the trigger is saved with that path as `launchTarget`

#### Scenario: Unsupported target
- **WHEN** a profile, edited or imported, has a trigger with `launchTarget = "ms-msdt:/id PCWDiagnostic"` or a relative path
- **THEN** the profile is rejected with a validation error on that field

#### Scenario: Profile from the previous version
- **WHEN** the app loads or imports a profile whose triggers have no `launchTarget`
- **THEN** the profile loads unchanged and "Start flight" is not offered for those triggers

### Requirement: Start flight action
The system SHALL offer "Start flight" for every trigger that has a `launchTarget`, from the main window and from the tray. The action SHALL be available only while the monitor is `idle`, and SHALL be available for disabled profiles too, since the user explicitly picks the profile. Starting a flight SHALL:
1. start a real session for that profile and trigger right away, as if that trigger had been detected (the session is not a test);
2. launch, in list order and with their delays, the enabled items with `launchBeforeSimulator = true` that apply to that trigger;
3. start the simulator from `launchTarget`, unless a process with the trigger's `processName` is already running;
4. continue with the normal launch phases for the remaining items once the simulator is detected.

#### Scenario: Start MSFS 2024 with head tracking first
- **WHEN** the monitor is `idle`, the "MSFS" profile has TrackIR with `launchBeforeSimulator = true`, and Volanta and SimBrief without it, and the user picks "Start flight → MSFS 2024"
- **THEN** TrackIR opens, then MSFS 2024 is started, and Volanta and SimBrief open after MSFS 2024 is detected

#### Scenario: Simulator already running
- **WHEN** the user starts a flight for a trigger whose process is already running
- **THEN** the simulator is not started again, and the session continues as if it had been detected

#### Scenario: Unavailable during a session
- **WHEN** the monitor is in `simStarting`, `simRunning`, `closePending`, `closing` or `paused`
- **THEN** "Start flight" is disabled in the window and in the tray

#### Scenario: Disabled profile
- **WHEN** the profile "MSFS Offline" is disabled and the user starts a flight for it
- **THEN** the session runs with that profile's items, and the profile stays disabled afterwards

### Requirement: Items that open before the simulator
An `app` item SHALL have `launchBeforeSimulator` (default false). In a session started with "Start flight", those items SHALL open before the simulator is started. In a session started because the simulator was detected, and in "Test launch", the flag SHALL be ignored and the items SHALL open in the normal list order. An item with both `launchBeforeSimulator` and `waitForSimConnect` SHALL NOT be allowed. Items opened before the simulator SHALL count as launched by AutoStart for closing and crash relaunch.

#### Scenario: Simulator opened by hand
- **WHEN** the simulator is opened outside AutoStart and the profile has an item with `launchBeforeSimulator = true`
- **THEN** that item opens in its list position together with the others

#### Scenario: Conflicting options
- **WHEN** the user turns on "Open before the simulator" on an item with "Wait for SimConnect"
- **THEN** the form turns "Wait for SimConnect" off, and the backend rejects a profile that has both

#### Scenario: Closed at the end of the flight
- **WHEN** a flight started with "Start flight" ends and TrackIR was opened before the simulator with `onClose = graceful`
- **THEN** TrackIR is closed like the other items launched by AutoStart

### Requirement: Waiting for the simulator
After starting the simulator, the monitor SHALL wait up to 5 minutes, in the `simStarting` state, for a process with the trigger's `processName`. While waiting, only the items that open before the simulator SHALL be launched. If the process appears, the session SHALL move to `simRunning` and launch the remaining items. If starting the simulator fails, or the process does not appear in time, the timeline SHALL record it, and the session SHALL follow the close delay rules as if the simulator had exited (`closePending` when `closeDelayMs > 0`, otherwise `closing`). The user SHALL be able to cancel the wait, which ends the session the same way.

#### Scenario: Simulator detected
- **WHEN** the simulator process appears 40 seconds after "Start flight"
- **THEN** the state changes from `simStarting` to `simRunning` and the remaining items start launching

#### Scenario: Simulator never appears
- **WHEN** 5 minutes pass without the trigger's process, with `closeDelayMs = 60000`
- **THEN** the timeline records that the simulator did not start, the state becomes `closePending`, and the items opened before the simulator are closed after 60 seconds unless the user keeps them open

#### Scenario: Start target fails
- **WHEN** the `launchTarget` executable no longer exists
- **THEN** the timeline records the error with the reason "Executable not found", and the session goes into the close delay right away

#### Scenario: Cancel while waiting
- **WHEN** the state is `simStarting` and the user chooses "Cancel"
- **THEN** the session goes into the close delay (or closes right away when `closeDelayMs = 0`) without waiting any longer
