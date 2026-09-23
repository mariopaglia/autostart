# app-interface Specification

## Purpose
Defines the screens and interactions of the AutoStart interface: a modern stream deck-style look, visual management of profiles and items, onboarding, languages and themes.

## Requirements

### Requirement: Main screen
The main screen SHALL contain: a sidebar with the profile list (create, rename, duplicate, delete, export, import), showing for each profile whether it is enabled and which one is in the current session; a top bar with the monitor status (Waiting for simulator / Simulator running / Closing soon / Closing apps / Paused), the enabled toggle and the triggers of the displayed profile, and the "Test launch" and "Test close" buttons; and a central area with the profile's items as cards. While the monitor is in `closePending`, the window SHALL show, above whichever screen is open, the countdown with the "Close now" and "Keep apps open" actions.

#### Scenario: Live status
- **WHEN** the monitor changes state
- **THEN** the top bar updates without reloading the screen

#### Scenario: Close delay countdown
- **WHEN** the simulator exits with `closeDelayMs = 60000`
- **THEN** the window shows "Closing apps in 60 s", counting down every second, with the "Close now" and "Keep apps open" buttons

#### Scenario: Enabling a conflicting profile
- **WHEN** the user turns on the enabled toggle of "MSFS Offline" while "MSFS Online" uses the same trigger
- **THEN** the sidebar shows "MSFS Online" as disabled and a notice names it

#### Scenario: Several triggers in the top bar
- **WHEN** the displayed profile has the triggers MSFS 2020 and MSFS 2024
- **THEN** the top bar shows both, lets the user remove either one while at least one remains, and lets the user add another from the presets, the running processes or a typed name

### Requirement: Item cards
Each card SHALL show the icon (or a generic/globe icon for URLs), name, shortened path or URL, badges (Admin, Minimized, SimConnect, Reopen if it crashes, delay in seconds, close behavior), the current session status when there is one (including "Waiting for SimConnect" and "Reopening"), and an enabled toggle. Cards SHALL be reorderable by drag and drop (mouse and keyboard). Clicking a card SHALL open it for editing.

#### Scenario: Toggle disables
- **WHEN** the user turns off a card's toggle
- **THEN** the card is visually dimmed and the item is ignored on the next launch

#### Scenario: Reorder via keyboard
- **WHEN** the user focuses the drag handle, presses space and uses the arrow keys
- **THEN** the card changes position and the order is persisted

#### Scenario: Launch option badges
- **WHEN** an item has `startMinimized = true`, `waitForSimConnect = true` and `restartOnCrash = true`
- **THEN** the card shows the "Minimized", "SimConnect" and "Reopen if it crashes" badges

#### Scenario: Item being reopened
- **WHEN** an item crashed and is waiting to be relaunched
- **THEN** its card shows the "Reopening" status

### Requirement: Empty state
A profile without items SHALL show an empty state with the "Add app" and "Add URL" actions and a hint that apps can also be dragged into the window. "Add app" SHALL open the app picker.

#### Scenario: New profile
- **WHEN** the user creates a profile
- **THEN** the central area shows the empty state with both actions and the drag-and-drop hint

### Requirement: Item form
The form SHALL allow adding/editing an `app` item, choosing the `.exe` through the native dialog (filling in name, icon and process automatically), or a `url` item. The form SHALL also open pre-filled from an app or URL candidate (picker or drag and drop), with every field still editable before saving. Main app fields: name, path, arguments, delay, run as admin, start minimized, wait for SimConnect, reopen if it crashes and close behavior. An "Advanced" section, collapsed by default, SHALL contain the working folder and the process name, indicating when the name is detected automatically and offering an action to return to automatic mode when in manual mode. The "Wait for SimConnect" option SHALL be disabled, with a hint explaining why, when none of the profile's triggers is MSFS 2020 or 2024. URL fields: name, URL, delay. Validation SHALL use the same schemas as persistence and show per-field errors. When the executable path is already used by another item of the same profile, the form SHALL show a non-blocking "already in this profile" warning.

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
- **WHEN** the profile's only trigger is `X-Plane.exe` and the user opens an app's form
- **THEN** the "Wait for SimConnect" option appears disabled with the hint that it only applies to MSFS 2020/2024

#### Scenario: Duplicate executable
- **WHEN** the profile already has an item for `C:\Tools\vPilot\vPilot.exe` and the user adds the same executable again
- **THEN** the form shows the "already in this profile" warning and still allows saving

#### Scenario: Reopen if it crashes
- **WHEN** the user turns on "Reopen if it crashes" and saves the item
- **THEN** the item is saved with `restartOnCrash = true` and the option's hint explains that apps closed normally are not reopened

### Requirement: Settings screen
The settings screen SHALL expose: start with Windows, start minimized, close delay after the simulator exits (in seconds), only close what AutoStart opened, the graceful close timeout (in seconds, inside a collapsed "Advanced" area of the closing section, explaining that it only applies to items set to close normally), show notifications, theme, language, check for updates (automatic on startup and manual button), open log folder and an "About" section (version, repository link, license).

#### Scenario: Change theme
- **WHEN** the user picks the `light` theme
- **THEN** the interface switches to the light theme immediately

#### Scenario: Turn off the close delay
- **WHEN** the user sets the close delay to 0 seconds
- **THEN** the items are closed right after the simulator exits, as in earlier versions

### Requirement: Internationalization
All user-visible text (UI, tray, notifications, displayed error messages) SHALL come from translation files, with pt-BR as the default and en as the alternative. Keys missing an en translation SHALL fall back to pt-BR.

#### Scenario: English language
- **WHEN** the language is `en`
- **THEN** no Portuguese text appears in the UI

### Requirement: Themes
The app SHALL support the `dark` (default), `light` and `system` themes, the last one following the Windows theme in real time.

#### Scenario: System theme
- **WHEN** the theme is `system` and Windows switches to light mode
- **THEN** the app switches to the light theme without restarting

### Requirement: Onboarding
On first run, the system SHALL create an example profile "MSFS 2024" with the trigger `FlightSimulator2024.exe` and show a short wizard (3 to 4 steps) explaining: how the trigger works, the choice of simulator preset (MSFS 2024, MSFS 2020, X-Plane 12, custom), how to add apps and the start with Windows options. Finishing or skipping the wizard SHALL set `onboardingCompleted = true`.

#### Scenario: First run
- **WHEN** the app is opened for the first time
- **THEN** the wizard appears over the main screen and the example profile exists

#### Scenario: Preset choice in the wizard
- **WHEN** the user picks "X-Plane 12" in the wizard
- **THEN** the example profile's triggers become `[X-Plane.exe]` and its name becomes "X-Plane 12"

#### Scenario: Subsequent runs
- **WHEN** the app is opened after onboarding was completed
- **THEN** the wizard does not appear

### Requirement: Visuals and accessibility
The interface SHALL use rounded corners, cards with icons, subtle animations that respect `prefers-reduced-motion`, visible focus on every interactive control and AA contrast in both themes.

#### Scenario: Reduced motion
- **WHEN** Windows has animations turned off
- **THEN** UI transitions are disabled

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
