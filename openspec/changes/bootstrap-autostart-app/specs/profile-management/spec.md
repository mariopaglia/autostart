## Purpose

Lets the pilot organize their helper programs into profiles, each tied to a trigger process (simulator), with an ordered list of apps and URLs to open and close.

## ADDED Requirements

### Requirement: Profile structure
The system SHALL store profiles with `id` (UUID), `name` (non-empty), `trigger` (`processName` ending in `.exe` and `label`), `items` (ordered list of items) and `enabled` (boolean).

#### Scenario: Create profile
- **WHEN** the user creates a profile named "Live MSFS 2024" with trigger `FlightSimulator2024.exe`
- **THEN** the profile is persisted with a new UUID, `enabled = true` and an empty item list

#### Scenario: Invalid name
- **WHEN** the user tries to save a profile with an empty or whitespace-only name
- **THEN** the system rejects the save and shows a validation error on the field

### Requirement: App items
An `app` item SHALL contain `id`, `name`, `exePath`, optional `args`, optional `workingDir`, `processName`, optional `iconBase64`, `delayMs` (default 800, between 0 and 60000), `runAsAdmin` (default false), `onClose` (`graceful` | `force` | `keep`, default `graceful`) and `enabled` (default true). The `processName` SHALL be editable by the user to cover apps whose launcher spawns a process with a different name.

#### Scenario: processName different from the exe
- **WHEN** the user adds `C:\Apps\Volanta\Launcher.exe` and changes the `processName` to `Volanta.exe`
- **THEN** pre-existence detection and closing use `Volanta.exe`

### Requirement: URL items
A `url` item SHALL contain `id`, `name`, `url` (`http` or `https` only), `delayMs` and `enabled`. URL items SHALL NOT be closed by the system.

#### Scenario: Invalid URL
- **WHEN** the user enters `ftp://example.com` or text that is not a URL
- **THEN** the form rejects the value with a validation message

### Requirement: Profile operations
The system SHALL allow creating, renaming, duplicating and deleting profiles, as well as reordering, enabling/disabling, editing and removing items. Duplicating SHALL generate new UUIDs for the profile and all its items and add the suffix " (cópia)"/" (copy)" according to the language.

#### Scenario: Reorder items
- **WHEN** the user drags the third item to the first position
- **THEN** the new order is persisted and used on the next launch

#### Scenario: Delete the active profile
- **WHEN** the user deletes the active profile and other profiles exist
- **THEN** the first remaining profile becomes the active one

#### Scenario: Delete the last profile
- **WHEN** the user tries to delete the only existing profile
- **THEN** the system prevents the deletion and explains that at least one profile must exist

### Requirement: Active profile
Exactly one profile SHALL be active at a time, and only the active profile is considered by the monitor. An active profile with `enabled = false` SHALL make the monitor ignore the trigger.

#### Scenario: Switch active profile
- **WHEN** the user selects another profile as active (from the UI or the tray)
- **THEN** `activeProfileId` is persisted and the monitor starts watching the new profile's trigger

### Requirement: Durable persistence
Profiles and settings SHALL be persisted as JSON files in the app data directory, with a schema version field and atomic writes, so that a crash during a write does not corrupt the previous file.

#### Scenario: App restart
- **WHEN** the app is closed and opened again
- **THEN** profiles, item order, active profile and settings are restored unchanged

#### Scenario: Corrupted file
- **WHEN** the profiles file exists but is not valid JSON
- **THEN** the system renames the file to `.corrupt-<timestamp>`, starts with the example profile and logs an error

### Requirement: Export and import profile
The system SHALL export a profile to a `.json` file chosen by the user and import profiles from `.json` files validated by the same schema. Import SHALL generate new UUIDs and SHALL NOT overwrite existing profiles.

#### Scenario: Valid import
- **WHEN** the user imports a file exported by another pilot
- **THEN** a new profile appears in the sidebar with the file's items

#### Scenario: Invalid import
- **WHEN** the file does not follow the schema (missing required field, wrong type)
- **THEN** nothing is saved and the user sees a message pointing to the invalid field

#### Scenario: Missing paths after import
- **WHEN** an imported item points to an `exePath` that does not exist on this machine
- **THEN** the item's card shows an "Executable not found" warning
