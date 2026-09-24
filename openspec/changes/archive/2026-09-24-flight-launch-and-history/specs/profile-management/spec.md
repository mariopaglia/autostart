## MODIFIED Requirements

### Requirement: Profile structure
The system SHALL store profiles with `id` (UUID), `name` (non-empty), `triggers` (a list of 1 to 5 triggers, each with a `processName` ending in `.exe`, a `label` and an optional `launchTarget`, with no two `processName`s equal case-insensitively), `items` (ordered list of items) and `enabled` (boolean).

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

#### Scenario: Removing a trigger used by items
- **WHEN** the user removes the MSFS 2020 trigger from a profile where an item has `onlyForTriggers = [FlightSimulator.exe]`
- **THEN** the trigger is removed from that item's `onlyForTriggers`, and an item left with an empty list applies to every simulator of the profile

### Requirement: App items
An `app` item SHALL contain `id`, `name`, `exePath`, optional `args`, optional `workingDir`, `processName`, `processNameMode` (`auto` | `manual`, default `auto`), optional `iconBase64`, `delayMs` (default 800, between 0 and 60000), `runAsAdmin` (default false), `startMinimized` (default false), `waitForSimConnect` (default false), `restartOnCrash` (default false), `launchBeforeSimulator` (default false), `onlyForTriggers` (default empty), `onClose` (`graceful` | `force` | `keep`, default `graceful`) and `enabled` (default true). `onlyForTriggers` SHALL list process names of the profile's own triggers, with no repeats. An empty list SHALL mean the item applies to every trigger of the profile. `launchBeforeSimulator` and `waitForSimConnect` SHALL NOT both be true. The `processName` SHALL initially be filled with the executable name and, in `auto` mode, SHALL be maintained by the system. The user SHALL be able to edit it, which switches the mode to `manual`, and SHALL be able to return to `auto` mode. Items saved by earlier versions, without the new fields, SHALL be loaded with the default values, except for the `processNameMode` of locally saved items, which SHALL be `manual` when the `processName` differs from the executable name (case-insensitive), since in that case the name was set by the user.

#### Scenario: processName different from the exe
- **WHEN** the user adds `C:\Apps\Volanta\Launcher.exe` and changes the `processName` to `Volanta.exe`
- **THEN** the item switches to `processNameMode = manual`, and pre-existence detection and closing use `Volanta.exe`

#### Scenario: New item in automatic mode
- **WHEN** the user adds `C:\Apps\SPAD\Spad.exe` without opening the Advanced section
- **THEN** the item is saved with `processName = Spad.exe` and `processNameMode = auto`

#### Scenario: Profile from the previous version
- **WHEN** the app loads a profile saved by v0.1.0 whose items have neither `processNameMode`, `startMinimized`, `waitForSimConnect`, `restartOnCrash`, `launchBeforeSimulator` nor `onlyForTriggers`
- **THEN** the items are loaded with `startMinimized = false`, `waitForSimConnect = false`, `restartOnCrash = false`, `launchBeforeSimulator = false` and an empty `onlyForTriggers`, without losing the other fields

#### Scenario: Name set by the user in the previous version
- **WHEN** an item saved by v0.1.0 points to `Launcher.exe` with `processName = Volanta.exe`
- **THEN** the item is loaded with `processNameMode = manual`, and AutoStart does not change that name

#### Scenario: Name equal to the executable in the previous version
- **WHEN** an item saved by v0.1.0 points to `Spad.exe` with `processName = Spad.exe`
- **THEN** the item is loaded with `processNameMode = auto`

#### Scenario: Import an old profile
- **WHEN** the user imports a file exported by v0.1.0
- **THEN** the import is accepted and the new fields take their default values

#### Scenario: Item only for MSFS 2024
- **WHEN** a profile has the triggers MSFS 2020 and MSFS 2024 and the user restricts an item to MSFS 2024
- **THEN** the item is saved with `onlyForTriggers = [FlightSimulator2024.exe]`

#### Scenario: Unknown trigger in the item
- **WHEN** a profile, edited or imported, has an item whose `onlyForTriggers` contains `X-Plane.exe` but the profile has no such trigger
- **THEN** the profile is rejected with a validation error on that item

### Requirement: URL items
A `url` item SHALL contain `id`, `name`, `url`, `delayMs`, `onlyForTriggers` (same rules as for app items) and `enabled`. The `url` SHALL be an `http` or `https` address, or a Steam launch link in the form `steam://rungameid/<digits>` or `steam://run/<digits>`. URL items SHALL NOT be closed by the system.

#### Scenario: Invalid URL
- **WHEN** the user enters `ftp://example.com`, `steam://uninstall/123`, `file:///C:/x.exe` or text that is not a URL
- **THEN** the form rejects the value with a validation message

#### Scenario: Steam launch link
- **WHEN** the user enters `steam://rungameid/1234560` as the address of a URL item
- **THEN** the item is saved, and the form hints that AutoStart opens it but does not close it
