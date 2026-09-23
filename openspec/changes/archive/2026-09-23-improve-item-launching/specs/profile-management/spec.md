## MODIFIED Requirements

### Requirement: App items
An `app` item SHALL contain `id`, `name`, `exePath`, optional `args`, optional `workingDir`, `processName`, `processNameMode` (`auto` | `manual`, default `auto`), optional `iconBase64`, `delayMs` (default 800, between 0 and 60000), `runAsAdmin` (default false), `startMinimized` (default false), `waitForSimConnect` (default false), `onClose` (`graceful` | `force` | `keep`, default `graceful`) and `enabled` (default true). The `processName` SHALL initially be filled with the executable name and, in `auto` mode, SHALL be maintained by the system. The user SHALL be able to edit it, which switches the mode to `manual`, and SHALL be able to return to `auto` mode. Items saved by earlier versions, without the new fields, SHALL be loaded with the default values, except for the `processNameMode` of locally saved items, which SHALL be `manual` when the `processName` differs from the executable name (case-insensitive), since in that case the name was set by the user.

#### Scenario: processName different from the exe
- **WHEN** the user adds `C:\Apps\Volanta\Launcher.exe` and changes the `processName` to `Volanta.exe`
- **THEN** the item switches to `processNameMode = manual`, and pre-existence detection and closing use `Volanta.exe`

#### Scenario: New item in automatic mode
- **WHEN** the user adds `C:\Apps\SPAD\Spad.exe` without opening the Advanced section
- **THEN** the item is saved with `processName = Spad.exe` and `processNameMode = auto`

#### Scenario: Profile from the previous version
- **WHEN** the app loads a profile saved by v0.1.0 whose items have neither `processNameMode`, `startMinimized` nor `waitForSimConnect`
- **THEN** the items are loaded with `startMinimized = false` and `waitForSimConnect = false`, without losing the other fields

#### Scenario: Name set by the user in the previous version
- **WHEN** an item saved by v0.1.0 points to `Launcher.exe` with `processName = Volanta.exe`
- **THEN** the item is loaded with `processNameMode = manual`, and AutoStart does not change that name

#### Scenario: Name equal to the executable in the previous version
- **WHEN** an item saved by v0.1.0 points to `Spad.exe` with `processName = Spad.exe`
- **THEN** the item is loaded with `processNameMode = auto`

#### Scenario: Import an old profile
- **WHEN** the user imports a file exported by v0.1.0
- **THEN** the import is accepted and the new fields take their default values
