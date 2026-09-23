## Purpose

Gives visibility into what AutoStart did in each session and keeps a technical history on disk to diagnose problems reported by the community.

## ADDED Requirements

### Requirement: Last-session timeline
The system SHALL record, for the current or most recent session (real or test), a timeline of events with time, item, kind (session started, launched, skipped, closed gracefully, force closed, kept, error, session ended) and message. The last-session timeline SHALL survive an app restart.

#### Scenario: View session
- **WHEN** the user opens the Logs screen after a flight
- **THEN** they see, in chronological order, what was launched, what was skipped, what was closed and the errors

#### Scenario: After restarting the app
- **WHEN** AutoStart is restarted after a session
- **THEN** the Logs screen still shows that session's timeline

#### Scenario: Live update
- **WHEN** the Logs screen is open during a session
- **THEN** new events appear without reloading

### Requirement: Rotating file log
The system SHALL write technical logs to files in the app log directory, with size-based rotation (max 5 MB per file, keeping the 5 most recent files). The settings screen SHALL offer an action to open the log folder.

#### Scenario: Open log folder
- **WHEN** the user clicks "Open log folder"
- **THEN** Explorer opens the folder containing the `.log` files

### Requirement: No sensitive data
Logs SHALL NOT contain the content of arguments marked as sensitive; the system SHALL log paths and process names, but SHALL omit item `args` from the log files.

#### Scenario: Args with a token
- **WHEN** an item has `args = "--token abc123"`
- **THEN** the log file records the item launch without the argument values
