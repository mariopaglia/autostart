## MODIFIED Requirements

### Requirement: Ordered launch with delay
When a session starts, the system SHALL consider only the enabled items that apply to the trigger that started the session: items with an empty `onlyForTriggers`, or whose `onlyForTriggers` contains that trigger (case-insensitive). Items that don't apply SHALL be ignored for the whole session: no status, no launch, no closing and no crash relaunch. The system SHALL launch the items it considers in phases, each in list order:
1. only in sessions started with "Start flight": the `app` items with `launchBeforeSimulator = true`, before the simulator is started;
2. the items that do not wait for SimConnect (and were not launched in phase 1);
3. the `app` items with `waitForSimConnect = true`, only after SimConnect is available.

In every phase, the system SHALL wait each item's `delayMs` before launching it. Disabled items SHALL be ignored without producing a status.

#### Scenario: Three apps in sequence
- **WHEN** the session starts with items A (delay 0), B (delay 800) and C (delay 2000), none waiting for SimConnect
- **THEN** A launches immediately, B about 800 ms after A, and C about 2000 ms after B

#### Scenario: Items that wait for SimConnect go last
- **WHEN** the list is A, B (waits for SimConnect), C and D (waits for SimConnect)
- **THEN** A and C launch first, in that order, and B and D launch afterwards, in that order, once SimConnect is available

#### Scenario: Item only for the other simulator
- **WHEN** a profile has the triggers MSFS 2020 and MSFS 2024, item A has `onlyForTriggers = [FlightSimulator2024.exe]` and the user opens MSFS 2020
- **THEN** item A is not launched, does not appear with a status in the session, and is not closed when the session ends

#### Scenario: Item for the running simulator
- **WHEN** the same profile's session is started by MSFS 2024
- **THEN** item A is launched in its list position

#### Scenario: Before-simulator phase
- **WHEN** a flight is started with "Start flight" and the list is A, B (`launchBeforeSimulator`), C
- **THEN** B launches before the simulator is started, and A and C launch, in that order, after the simulator is detected

### Requirement: Item execution
An `app` item SHALL be started with its `args` and `workingDir` (default: the exe folder). A `url` item with an `http` or `https` address SHALL be opened in the default browser. A `url` item with a Steam launch link SHALL be handed to Windows, which opens it with Steam. A launch failure SHALL mark the item with the `error` status and the cause's message, without interrupting the following items.

#### Scenario: Removed executable
- **WHEN** an item's `exePath` no longer exists
- **THEN** the item gets the `error` status ("Executable not found") and the next items keep being launched

#### Scenario: Launch confirmation
- **WHEN** an app is started and, within 10 seconds, the started process, one of its descendants or a process with the item's `processName` is running
- **THEN** the item moves to `running`; if none of these processes exists in time, the item gets `error` with the message "The process was not detected after launch"

#### Scenario: Launcher that exits right after opening the app
- **WHEN** AutoStart starts `Launcher.exe`, which opens `Volanta.exe` and exits within 3 seconds
- **THEN** the item moves to `running`, because a process started by the launcher keeps running

#### Scenario: Steam link
- **WHEN** a session launches a URL item with `steam://rungameid/1234560`
- **THEN** Windows opens the link with Steam, the item moves to `running`, and the item is not closed at the end of the session

### Requirement: Manual tests
The user SHALL be able to run "Test launch" and "Test close" for a profile without the simulator running, using the same rules as the real session, with these exceptions: in "Test launch", items that wait for SimConnect SHALL launch in the last phase without waiting for SimConnect, and the timeline SHALL record that the wait was skipped in the test; items with `restartOnCrash` SHALL NOT be relaunched after a crash; every enabled item SHALL be considered regardless of `onlyForTriggers`; and `launchBeforeSimulator` SHALL be ignored. "Test close" SHALL close the items launched by the last "Test launch" (or, if there was no previous test, SHALL treat every running item as launched by AutoStart, after user confirmation). The tests SHALL be unavailable while the monitor is in `simStarting`, `simRunning`, `closePending` or `closing`.

#### Scenario: Test launch
- **WHEN** the user clicks "Test launch" with the monitor in `idle`
- **THEN** the enabled items launch in list order, with the items that wait for SimConnect last, and their statuses update on the cards

#### Scenario: Test with an item that waits for SimConnect
- **WHEN** the user tests the launch of a profile with an item that waits for SimConnect and the simulator is not running
- **THEN** the item launches in the last phase without waiting, and the timeline shows that the SimConnect wait was skipped in the test

#### Scenario: Test blocked during a flight
- **WHEN** the monitor is in `simStarting`, `simRunning` or `closePending`
- **THEN** the test buttons are disabled, with a hint explaining why

#### Scenario: Test covers every simulator
- **WHEN** the user tests the launch of a profile where one item applies only to MSFS 2024 and another only to MSFS 2020
- **THEN** both items are launched

### Requirement: Relaunch after a crash
During a real session in `simStarting` or `simRunning`, the system SHALL watch every `app` item with `restartOnCrash = true` whose status is `running` (whether launched by AutoStart or pre-existing). When every process of the item has exited and at least one of them exited with a non-zero exit code, the system SHALL consider it a crash: the item SHALL get the `restarting` status and SHALL be launched again after 5 seconds, following the item's launch rules. When all of the item's processes exited with exit code 0, the system SHALL consider it closed by the user and SHALL stop watching it. The system SHALL relaunch an item at most 3 times per session; after the third crash, the item SHALL get the `error` status with the message "The app kept crashing and was not reopened". A relaunched item SHALL count as launched by AutoStart for the closing rules. Relaunch SHALL stop as soon as the session leaves `simStarting` or `simRunning`.

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

#### Scenario: Crash while the simulator loads
- **WHEN** an item opened before the simulator, with `restartOnCrash = true`, crashes while the state is `simStarting`
- **THEN** it is reopened following the same rules
