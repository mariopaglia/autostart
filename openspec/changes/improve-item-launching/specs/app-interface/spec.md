## MODIFIED Requirements

### Requirement: Item cards
Each card SHALL show the icon (or a generic/globe icon for URLs), name, shortened path or URL, badges (Admin, Minimized, SimConnect, delay in seconds, close behavior), the current session status when there is one (including "Waiting for SimConnect"), and an enabled toggle. Cards SHALL be reorderable by drag and drop (mouse and keyboard). Clicking a card SHALL open it for editing.

#### Scenario: Toggle disables
- **WHEN** the user turns off a card's toggle
- **THEN** the card is visually dimmed and the item is ignored on the next launch

#### Scenario: Reorder via keyboard
- **WHEN** the user focuses the drag handle, presses space and uses the arrow keys
- **THEN** the card changes position and the order is persisted

#### Scenario: Launch option badges
- **WHEN** an item has `startMinimized = true` and `waitForSimConnect = true`
- **THEN** the card shows the "Minimized" and "SimConnect" badges

### Requirement: Item form
The form SHALL allow adding/editing an `app` item, choosing the `.exe` through the native dialog (filling in name, icon and process automatically), or a `url` item. Main app fields: name, path, arguments, delay, run as admin, start minimized, wait for SimConnect and close behavior. An "Advanced" section, collapsed by default, SHALL contain the working folder and the process name, indicating when the name is detected automatically and offering an action to return to automatic mode when in manual mode. The "Wait for SimConnect" option SHALL be disabled, with a hint explaining why, when the profile's trigger is not MSFS 2020 or 2024. URL fields: name, URL, delay. Validation SHALL use the same schemas as persistence and show per-field errors.

#### Scenario: Add an app through the dialog
- **WHEN** the user clicks "Add app" and selects an `.exe`
- **THEN** name, icon and process are filled in, and the user can save without opening the Advanced section

#### Scenario: Automatically detected name
- **WHEN** the user opens the Advanced section of an item in automatic mode
- **THEN** the process name field shows the current name with the "Detected automatically" label

#### Scenario: Return to automatic mode
- **WHEN** the item is in manual mode and the user clicks "Detect automatically"
- **THEN** the item returns to `processNameMode = auto` and the name is maintained by the system

#### Scenario: SimConnect unavailable for the trigger
- **WHEN** the profile has trigger `X-Plane.exe` and the user opens an app's form
- **THEN** the "Wait for SimConnect" option appears disabled with the hint that it only applies to MSFS 2020/2024
