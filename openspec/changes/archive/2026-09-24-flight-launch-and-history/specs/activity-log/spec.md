## MODIFIED Requirements

### Requirement: Last-session timeline
The system SHALL record, for each session (real or test), a timeline of events with time, item, kind (session started, launched, skipped, closed gracefully, force closed, kept, close delayed, simulator returned, apps kept by the user, crashed, relaunched, simulator started, simulator did not start, error, session ended) and message. The system SHALL keep a history of finished sessions that survives an app restart: the 20 most recent real sessions plus only the most recent test session. A new test session SHALL replace the previous test session in the history, and when a 21st real session finishes, the oldest real session SHALL be dropped. On the first start after updating, a last session saved by an earlier version SHALL become the first entry of the history.

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

#### Scenario: Yesterday's flight after a test
- **WHEN** the pilot flew yesterday and ran "Test launch" today
- **THEN** the history still contains yesterday's session together with today's test

#### Scenario: Only the latest test is kept
- **WHEN** the user runs "Test launch" three times in a row
- **THEN** the history contains only the last of those test sessions

#### Scenario: History limit
- **WHEN** 20 real sessions are stored and another real session finishes
- **THEN** the oldest real session is removed and the history keeps 20 real sessions

#### Scenario: Update from an earlier version
- **WHEN** AutoStart starts for the first time after updating, with a last session saved by v0.3.1
- **THEN** that session appears in the history

## ADDED Requirements

### Requirement: Browsing the session history
The Logs screen SHALL show a session selector listing the session in progress (if any) followed by the stored sessions, newest first. Each entry SHALL show the profile name, the start date and time, whether it was a test, and how many errors it had. By default, the screen SHALL show the session in progress, or the most recent one when no session is in progress. When a new session starts while the screen shows the default selection, the screen SHALL switch to it. When the user has picked an older session, it SHALL stay on the screen. The screen SHALL offer "Clear history", which, after confirmation, removes every stored session except the session in progress.

#### Scenario: Pick an older session
- **WHEN** the user picks yesterday's session in the selector
- **THEN** the timeline shows that session's entries

#### Scenario: Errors at a glance
- **WHEN** a stored session had two items that failed to open
- **THEN** its entry in the selector shows 2 errors

#### Scenario: New session while browsing
- **WHEN** the user is looking at an older session and the simulator starts
- **THEN** the older session stays on screen, and the new session appears at the top of the selector

#### Scenario: Clear history
- **WHEN** the user confirms "Clear history" with no session in progress
- **THEN** the history is emptied, including on disk, and the screen shows the empty state
