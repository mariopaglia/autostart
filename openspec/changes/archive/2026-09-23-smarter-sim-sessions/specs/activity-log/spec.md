## MODIFIED Requirements

### Requirement: Last-session timeline
The system SHALL record, for the current or most recent session (real or test), a timeline of events with time, item, kind (session started, launched, skipped, closed gracefully, force closed, kept, close delayed, simulator returned, apps kept by the user, crashed, relaunched, error, session ended) and message. The last-session timeline SHALL survive an app restart.

#### Scenario: View session
- **WHEN** the user opens the Logs screen after a flight
- **THEN** they see, in chronological order, what was launched, what was skipped, what was closed and the errors

#### Scenario: After restarting the app
- **WHEN** AutoStart is restarted after a session
- **THEN** the Logs screen still shows that session's timeline

#### Scenario: Live update
- **WHEN** the Logs screen is open during a session
- **THEN** new events appear without reloading

#### Scenario: Crash to desktop in the timeline
- **WHEN** the simulator crashed, the close delay started and the simulator was reopened within the delay
- **THEN** the timeline shows "close delayed" followed by "simulator returned", with no closing entries

#### Scenario: Crashed app in the timeline
- **WHEN** an item with `restartOnCrash = true` crashed and was reopened
- **THEN** the timeline shows the item's crash followed by its relaunch
