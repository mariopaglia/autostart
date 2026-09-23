## MODIFIED Requirements

### Requirement: Events for the interface
The monitor SHALL emit events to the frontend on every monitor state change, on every item status change (`pending`, `waitingSimConnect`, `launching`, `running`, `skipped`, `closing`, `closed`, `error`) and on every log entry. The current state SHALL also be available on demand.

#### Scenario: Window opened after the start
- **WHEN** the window is opened from the tray in the middle of a session
- **THEN** the UI queries the current state and correctly shows the status of the monitor and of each item

#### Scenario: Item waiting for SimConnect
- **WHEN** an item starts waiting for SimConnect during the session
- **THEN** the UI receives the item's `waitingSimConnect` status without needing to reload
