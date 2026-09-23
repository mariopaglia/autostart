# desktop-notifications Specification

## Purpose
Tells the pilot, through native Windows notifications, about events that need their attention while AutoStart runs hidden in the tray, such as failures and the pending close of their apps.

## Requirements

### Requirement: Notified events
When `showNotifications` is enabled, the system SHALL show a native Windows notification, in the configured language, for: an item whose launch failed (with the item name and the error message), SimConnect not becoming available, the start of the close delay (with the number of seconds and a hint that the apps can be kept open from AutoStart), an item reopened after a crash and an item that was not reopened after crashing repeatedly. Successful launches and closings SHALL NOT produce notifications. Test sessions SHALL NOT produce notifications. Notifications SHALL be shown whether or not the main window is visible.

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

### Requirement: Notifications setting
The user SHALL be able to turn notifications off in Settings. With `showNotifications = false`, the system SHALL NOT show any notification.

#### Scenario: Notifications turned off
- **WHEN** `showNotifications = false` and an item fails to launch
- **THEN** no Windows notification is shown and the failure still appears in the card and the timeline
