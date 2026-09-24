## MODIFIED Requirements

### Requirement: Trigger detection by name
The monitor SHALL check every 2 seconds whether any process exists whose executable name matches one of the triggers of an enabled profile, using a case-insensitive comparison. When no session is in progress, the first enabled profile in sidebar order with a running trigger SHALL start the session. While a session is in progress, the monitor SHALL watch only the triggers of the session's profile, and the simulator SHALL count as running while any of them is running. A session can also be started on demand with "Start flight", for a given profile and trigger; that session SHALL begin in `simStarting` and SHALL move to `simRunning` when the chosen trigger is detected.

#### Scenario: Simulator starts
- **WHEN** the state is `idle` and a `flightsimulator2024.exe` process appears, with an enabled profile whose triggers include `FlightSimulator2024.exe`
- **THEN** within 2 seconds the state changes to `simRunning` and that profile's items start launching

#### Scenario: Another simulator without switching profiles
- **WHEN** the enabled profiles are "MSFS" (trigger `FlightSimulator2024.exe`) and "X-Plane" (trigger `X-Plane.exe`) and the user opens X-Plane
- **THEN** the "X-Plane" profile's session starts without the user selecting any profile

#### Scenario: Profile with two triggers
- **WHEN** a profile has the triggers `FlightSimulator.exe` and `FlightSimulator2024.exe` and the user opens MSFS 2020
- **THEN** the profile's session starts, and it lasts while either process is running

#### Scenario: Disabled profile
- **WHEN** a profile is disabled and its trigger process appears
- **THEN** no session starts for that profile

#### Scenario: Two different simulators at once
- **WHEN** no session is in progress and the triggers of two enabled profiles are running in the same check
- **THEN** only the profile higher in the sidebar starts a session

#### Scenario: Session started on demand
- **WHEN** the user starts a flight for MSFS 2024 in the "MSFS" profile
- **THEN** the state changes to `simStarting` at once, without waiting for the next check

### Requirement: State machine
The monitor SHALL operate with the states `idle`, `simStarting`, `simRunning`, `closePending`, `closing` and `paused`, with the transitions: `idle → simRunning` (trigger detected), `idle → simStarting` ("Start flight"), `simStarting → simRunning` (the chosen trigger is detected), `simStarting → closePending` (the simulator failed to start, did not appear within 5 minutes, or the user cancelled, with `closeDelayMs > 0`), `simStarting → closing` (the same causes, with `closeDelayMs = 0`), `simRunning → closePending` (triggers missing in 2 consecutive checks and `closeDelayMs > 0`), `simRunning → closing` (triggers missing in 2 consecutive checks and `closeDelayMs = 0`), `closePending → simRunning` (a trigger of the session's profile reappears), `closePending → closing` (the delay expires or the user chooses "close now"), `closePending → idle` (the user chooses "keep apps open"), `closing → idle` (closing finished), any state → `paused` (user pauses) and `paused → idle` (user resumes). While in `closing`, the monitor SHALL NOT start a new launch. Pausing during `simStarting` SHALL discard the session like pausing during `simRunning`.

#### Scenario: Simulator exits
- **WHEN** the state is `simRunning`, `closeDelayMs = 0` and the triggers are missing in two consecutive checks
- **THEN** the state changes to `closing`, the items are closed according to the rules and the state returns to `idle`

#### Scenario: Simulator exits with a close delay
- **WHEN** the state is `simRunning`, `closeDelayMs = 60000` and the triggers are missing in two consecutive checks
- **THEN** the state changes to `closePending`, and the items are closed only if no trigger of the session's profile reappears within 60 seconds

#### Scenario: Momentary flapping
- **WHEN** the trigger disappears in one check and reappears in the next
- **THEN** the state stays `simRunning` and nothing is closed

#### Scenario: Simulator reopened during closing
- **WHEN** the trigger reappears while the state is `closing`
- **THEN** closing finishes, the state goes to `idle` and, on the next check, it enters `simRunning` and launches the items again

#### Scenario: Missing trigger while starting
- **WHEN** the state is `simStarting` and the trigger has not appeared yet
- **THEN** the state stays `simStarting` until the trigger appears, the 5-minute limit passes or the user cancels

#### Scenario: Pause while starting
- **WHEN** the user pauses while the state is `simStarting`
- **THEN** the state becomes `paused` and no item is closed

### Requirement: Events for the interface
The monitor SHALL emit events to the frontend on every monitor state change, on every item status change (`pending`, `waitingSimConnect`, `launching`, `running`, `restarting`, `skipped`, `closing`, `closed`, `error`) and on every log entry. While in `closePending`, the state SHALL include the moment when the items will be closed. While in `simStarting`, the state SHALL include the label of the simulator being started. The current state SHALL also be available on demand.

#### Scenario: Window opened after the start
- **WHEN** the window is opened from the tray in the middle of a session
- **THEN** the UI queries the current state and correctly shows the status of the monitor and of each item

#### Scenario: Item waiting for SimConnect
- **WHEN** an item starts waiting for SimConnect during the session
- **THEN** the UI receives the item's `waitingSimConnect` status without needing to reload

#### Scenario: Countdown after the window opens
- **WHEN** the window is opened while the state is `closePending`
- **THEN** the UI shows how many seconds remain before the items are closed

#### Scenario: Window opened while the simulator starts
- **WHEN** the window is opened while the state is `simStarting` for MSFS 2024
- **THEN** the UI shows that MSFS 2024 is starting
