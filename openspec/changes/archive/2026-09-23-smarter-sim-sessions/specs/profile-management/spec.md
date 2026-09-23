## MODIFIED Requirements

### Requirement: Profile structure
The system SHALL store profiles with `id` (UUID), `name` (non-empty), `triggers` (a list of 1 to 5 triggers, each with a `processName` ending in `.exe` and a `label`, with no two `processName`s equal case-insensitively), `items` (ordered list of items) and `enabled` (boolean).

#### Scenario: Create profile
- **WHEN** the user creates a profile named "Live MSFS 2024" with trigger `FlightSimulator2024.exe`
- **THEN** the profile is persisted with a new UUID, the triggers list `[FlightSimulator2024.exe]`, an empty item list and `enabled = true` if no enabled profile uses that trigger (otherwise `enabled = false`)

#### Scenario: Invalid name
- **WHEN** the user tries to save a profile with an empty or whitespace-only name
- **THEN** the system rejects the save and shows a validation error on the field

#### Scenario: Profile for both MSFS versions
- **WHEN** the user adds `FlightSimulator.exe` to a profile whose trigger is `FlightSimulator2024.exe`
- **THEN** the profile is persisted with both triggers

#### Scenario: Last trigger
- **WHEN** the user tries to remove the only trigger of a profile
- **THEN** the system prevents the removal

#### Scenario: Repeated trigger
- **WHEN** the user tries to add `flightsimulator2024.exe` to a profile that already has `FlightSimulator2024.exe`
- **THEN** the system rejects the duplicate

### Requirement: App items
An `app` item SHALL contain `id`, `name`, `exePath`, optional `args`, optional `workingDir`, `processName`, `processNameMode` (`auto` | `manual`, default `auto`), optional `iconBase64`, `delayMs` (default 800, between 0 and 60000), `runAsAdmin` (default false), `startMinimized` (default false), `waitForSimConnect` (default false), `restartOnCrash` (default false), `onClose` (`graceful` | `force` | `keep`, default `graceful`) and `enabled` (default true). The `processName` SHALL initially be filled with the executable name and, in `auto` mode, SHALL be maintained by the system. The user SHALL be able to edit it, which switches the mode to `manual`, and SHALL be able to return to `auto` mode. Items saved by earlier versions, without the new fields, SHALL be loaded with the default values, except for the `processNameMode` of locally saved items, which SHALL be `manual` when the `processName` differs from the executable name (case-insensitive), since in that case the name was set by the user.

#### Scenario: processName different from the exe
- **WHEN** the user adds `C:\Apps\Volanta\Launcher.exe` and changes the `processName` to `Volanta.exe`
- **THEN** the item switches to `processNameMode = manual`, and pre-existence detection and closing use `Volanta.exe`

#### Scenario: New item in automatic mode
- **WHEN** the user adds `C:\Apps\SPAD\Spad.exe` without opening the Advanced section
- **THEN** the item is saved with `processName = Spad.exe` and `processNameMode = auto`

#### Scenario: Profile from the previous version
- **WHEN** the app loads a profile saved by v0.1.0 whose items have neither `processNameMode`, `startMinimized`, `waitForSimConnect` nor `restartOnCrash`
- **THEN** the items are loaded with `startMinimized = false`, `waitForSimConnect = false` and `restartOnCrash = false`, without losing the other fields

#### Scenario: Name set by the user in the previous version
- **WHEN** an item saved by v0.1.0 points to `Launcher.exe` with `processName = Volanta.exe`
- **THEN** the item is loaded with `processNameMode = manual`, and AutoStart does not change that name

#### Scenario: Name equal to the executable in the previous version
- **WHEN** an item saved by v0.1.0 points to `Spad.exe` with `processName = Spad.exe`
- **THEN** the item is loaded with `processNameMode = auto`

#### Scenario: Import an old profile
- **WHEN** the user imports a file exported by v0.1.0
- **THEN** the import is accepted and the new fields take their default values

### Requirement: Profile operations
The system SHALL allow creating, renaming, duplicating and deleting profiles, enabling/disabling them, as well as reordering, enabling/disabling, editing and removing items. Duplicating SHALL generate new UUIDs for the profile and all its items, add the suffix " (cópia)"/" (copy)" according to the language, and create the copy disabled, since it shares the original's triggers.

#### Scenario: Reorder items
- **WHEN** the user drags the third item to the first position
- **THEN** the new order is persisted and used on the next launch

#### Scenario: Duplicate an enabled profile
- **WHEN** the user duplicates the enabled profile "MSFS Online"
- **THEN** "MSFS Online (copy)" is created disabled and "MSFS Online" stays enabled

#### Scenario: Delete the active profile
- **WHEN** the user deletes an enabled profile and other profiles exist
- **THEN** the profile stops being watched and the other profiles keep their enabled state

#### Scenario: Delete the last profile
- **WHEN** the user tries to delete the only existing profile
- **THEN** the system prevents the deletion and explains that at least one profile must exist

### Requirement: Durable persistence
Profiles and settings SHALL be persisted as JSON files in the app data directory, with a schema version field and atomic writes, so that a crash during a write does not corrupt the previous file. Profiles saved by earlier versions with a single `trigger` SHALL be loaded with `triggers` containing that trigger.

#### Scenario: App restart
- **WHEN** the app is closed and opened again
- **THEN** profiles, item order, enabled profiles and settings are restored unchanged

#### Scenario: Corrupted file
- **WHEN** the profiles file exists but is not valid JSON
- **THEN** the system renames the file to `.corrupt-<timestamp>`, starts with the example profile and logs an error

#### Scenario: Upgrade from a version with an active profile
- **WHEN** the app loads data saved by v0.2.0 with profiles "MSFS Online" (active) and "MSFS Offline", both enabled with trigger `FlightSimulator2024.exe`, and "X-Plane", enabled with trigger `X-Plane.exe`
- **THEN** "MSFS Online" and "X-Plane" stay enabled, "MSFS Offline" is disabled, and each profile has its single trigger in `triggers`

### Requirement: Export and import profile
The system SHALL export a profile to a `.json` file chosen by the user and import profiles from `.json` files validated by the same schema. Import SHALL generate new UUIDs, SHALL NOT overwrite existing profiles and SHALL accept files with the single `trigger` of earlier versions, converting it to `triggers`. An imported profile SHALL be created enabled only if none of its triggers is used by an enabled profile; otherwise it SHALL be created disabled.

#### Scenario: Valid import
- **WHEN** the user imports a file exported by another pilot
- **THEN** a new profile appears in the sidebar with the file's items

#### Scenario: Invalid import
- **WHEN** the file does not follow the schema (missing required field, wrong type)
- **THEN** nothing is saved and the user sees a message pointing to the invalid field

#### Scenario: Missing paths after import
- **WHEN** an imported item points to an `exePath` that does not exist on this machine
- **THEN** the item's card shows an "Executable not found" warning

#### Scenario: Import a file from an earlier version
- **WHEN** the user imports a file exported by v0.2.0 with `trigger = FlightSimulator.exe`
- **THEN** the imported profile has `triggers = [FlightSimulator.exe]`

#### Scenario: Import with a trigger already watched
- **WHEN** the user imports an MSFS 2024 profile while another enabled profile uses `FlightSimulator2024.exe`
- **THEN** the imported profile is created disabled

## REMOVED Requirements

### Requirement: Active profile
**Reason**: Replaced by "Watched profiles": the monitor now watches every enabled profile, so the pilot no longer switches profiles before opening a different simulator.
**Migration**: `activeProfileId` is dropped from the settings. On the first load, enabled profiles stay enabled; when enabled profiles share a trigger, the previously active one (or, if it is not among them, the first in sidebar order) stays enabled and the others are disabled.

## ADDED Requirements

### Requirement: Watched profiles
Every enabled profile SHALL be watched by the monitor. At most one enabled profile SHALL use a given trigger process (case-insensitive). When the user enables an existing profile, or saves an enabled existing profile with a new trigger, the system SHALL disable the other enabled profiles that share one of its triggers and SHALL tell the user which profiles were disabled. A new profile (created, duplicated or imported) SHALL never disable other profiles: it SHALL be created disabled when one of its triggers is used by an enabled profile. Profiles MAY all be disabled.

#### Scenario: Switch between two variations of the same simulator
- **WHEN** "MSFS Online" is enabled and the user enables "MSFS Offline", both with trigger `FlightSimulator2024.exe`
- **THEN** "MSFS Offline" is enabled, "MSFS Online" is disabled, and the user sees a notice naming "MSFS Online"

#### Scenario: Profiles for different simulators
- **WHEN** "MSFS" (trigger `FlightSimulator2024.exe`) is enabled and the user enables "X-Plane" (trigger `X-Plane.exe`)
- **THEN** both stay enabled

#### Scenario: New profile for a watched simulator
- **WHEN** "MSFS" is enabled with trigger `FlightSimulator2024.exe` and the user creates a new profile with the same trigger
- **THEN** the new profile is created disabled and "MSFS" stays enabled

#### Scenario: Trigger added to an enabled profile
- **WHEN** "MSFS 2024" and "MSFS 2020" are enabled and the user adds `FlightSimulator.exe` to "MSFS 2024"
- **THEN** "MSFS 2020" is disabled and the user is told
