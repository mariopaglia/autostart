## MODIFIED Requirements

### Requirement: Empty state
A profile without items SHALL show an empty state with the "Add app" and "Add URL" actions and a hint that apps can also be dragged into the window. "Add app" SHALL open the app picker.

#### Scenario: New profile
- **WHEN** the user creates a profile
- **THEN** the central area shows the empty state with both actions and the drag-and-drop hint

### Requirement: Item form
The form SHALL allow adding/editing an `app` item, choosing the `.exe` through the native dialog (filling in name, icon and process automatically), or a `url` item. The form SHALL also open pre-filled from an app or URL candidate (picker or drag and drop), with every field still editable before saving. Main app fields: name, path, arguments, delay, run as admin, start minimized, wait for SimConnect and close behavior. An "Advanced" section, collapsed by default, SHALL contain the working folder and the process name, indicating when the name is detected automatically and offering an action to return to automatic mode when in manual mode. The "Wait for SimConnect" option SHALL be disabled, with a hint explaining why, when the profile's trigger is not MSFS 2020 or 2024. URL fields: name, URL, delay. Validation SHALL use the same schemas as persistence and show per-field errors. When the executable path is already used by another item of the same profile, the form SHALL show a non-blocking "already in this profile" warning.

#### Scenario: Add an app through the dialog
- **WHEN** the user opens the app picker, chooses "Browse for file…" and selects an `.exe`
- **THEN** name, icon and process are filled in, and the user can save without opening the Advanced section

#### Scenario: Form pre-filled from a candidate
- **WHEN** the user picks "Little Navmap" in the app picker
- **THEN** the form opens with name, icon, path, arguments, working folder and process name taken from the candidate

#### Scenario: Automatically detected name
- **WHEN** the user opens the Advanced section of an item in automatic mode
- **THEN** the process name field shows the current name with the "Detected automatically" label

#### Scenario: Return to automatic mode
- **WHEN** the item is in manual mode and the user clicks "Detect automatically"
- **THEN** the item returns to `processNameMode = auto` and the name is maintained by the system

#### Scenario: SimConnect unavailable for the trigger
- **WHEN** the profile has trigger `X-Plane.exe` and the user opens an app's form
- **THEN** the "Wait for SimConnect" option appears disabled with the hint that it only applies to MSFS 2020/2024

#### Scenario: Duplicate executable
- **WHEN** the profile already has an item for `C:\Tools\vPilot\vPilot.exe` and the user adds the same executable again
- **THEN** the form shows the "already in this profile" warning and still allows saving

## ADDED Requirements

### Requirement: App picker
"Add app" SHALL open an app picker dialog with a search field and two lists: "Installed" (default) and "Open now". Each entry SHALL show the icon (or a generic icon), the name and the shortened executable path; entries whose executable is already in the current profile SHALL be marked "Already in profile". The "Installed" list SHALL show flight sim suggestions in a "Suggested" group above the other apps. Search SHALL filter by name and executable path, case-insensitively. The picker SHALL show a loading state while a list is being gathered, an empty state when a list has no entries, and a "Browse for file…" action that opens the native `.exe` dialog. Choosing an entry or a file SHALL close the picker and open the item form pre-filled.

#### Scenario: Pick an installed app
- **WHEN** the user clicks "Add app", types "navi" and chooses "Navigraph Charts"
- **THEN** the picker closes and the item form opens pre-filled with Navigraph Charts' data

#### Scenario: Pick an open app
- **WHEN** the user switches to "Open now" and chooses the running "vPilot"
- **THEN** the item form opens pre-filled, with the process name exactly as it is running

#### Scenario: Suggestions first
- **WHEN** Little Navmap and a text editor are installed
- **THEN** Little Navmap appears in the "Suggested" group above the rest of the installed apps

#### Scenario: App already added
- **WHEN** the profile already has an item for Volanta
- **THEN** Volanta appears in the lists with the "Already in profile" mark and can still be chosen

#### Scenario: Nothing found
- **WHEN** the search matches no entry
- **THEN** the picker shows an empty message and keeps the "Browse for file…" action available

### Requirement: Drag and drop to add
While the main screen shows a profile, the window SHALL accept files dragged from Windows Explorer or the desktop. While files are dragged over the window, an overlay SHALL indicate that they can be dropped to add them to the displayed profile. On drop:
- exactly one app or URL candidate → the item form opens pre-filled with it;
- several candidates → all of them are added to the end of the profile as enabled items with default options, in drop order, and a notification reports how many were added;
- rejected paths → a notification lists the files that could not be added and why, without blocking the accepted ones.
Dropping SHALL be ignored while any dialog is open. When AutoStart runs as administrator, Windows blocks dragging from non-elevated apps, so the empty-state hint SHALL say that dragging is unavailable while running as administrator instead of inviting the user to drag.

#### Scenario: Drop one shortcut
- **WHEN** the user drags "Navigraph Charts" from the desktop onto the window
- **THEN** the overlay appears during the drag, and on drop the item form opens pre-filled with Navigraph Charts

#### Scenario: Drop several apps
- **WHEN** the user drops the shortcuts of Little Navmap, Volanta and vPilot together
- **THEN** the three apps are added to the end of the profile in that order and a notification says 3 apps were added

#### Scenario: Mixed drop
- **WHEN** the user drops a valid shortcut and a `.pdf` file
- **THEN** the valid app is handled as a single-candidate drop and a notification says the `.pdf` could not be added

#### Scenario: Running as administrator
- **WHEN** AutoStart was relaunched as administrator and the profile is empty
- **THEN** the empty state says dragging is unavailable while running as administrator, and "Add app" still works

#### Scenario: Drop while a dialog is open
- **WHEN** the item form is open and the user drops a file on the window
- **THEN** nothing is added and the form stays unchanged
