## MODIFIED Requirements

### Requirement: Notified events
When `showNotifications` is enabled, the system SHALL show a native Windows notification, in the configured language, for: an item whose launch failed (with the item name and the error message), SimConnect not becoming available, the simulator failing to start or not appearing after "Start flight" (with the simulator name and the reason), the start of the close delay (with the number of seconds and a hint that the apps can be kept open from AutoStart), an item reopened after a crash and an item that was not reopened after crashing repeatedly. Successful launches and closings, including a simulator started successfully, SHALL NOT produce notifications. Test sessions SHALL NOT produce notifications. Notifications SHALL be shown whether or not the main window is visible.

#### Scenario: Launch failure while in the tray
- **WHEN** the simulator starts, AutoStart is hidden in the tray and the Volanta executable no longer exists
- **THEN** a Windows notification says that Volanta could not be opened, with the reason "Executable not found"

#### Scenario: Close delay started
- **WHEN** the simulator exits and the close delay of 60 seconds starts
- **THEN** a Windows notification says the apps will be closed in 60 seconds and that they can be kept open from AutoStart

#### Scenario: Normal session
- **WHEN** every item launches and closes successfully
- **THEN** no notification is shown

#### Scenario: Test launch with a failure
- **WHEN** an item fails during "Test launch"
- **THEN** no notification is shown and the failure appears only in the interface

#### Scenario: Simulator did not appear
- **WHEN** the pilot started a flight from the tray and MSFS 2024 did not appear within 5 minutes
- **THEN** a Windows notification says that MSFS 2024 did not start
