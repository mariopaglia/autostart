## Purpose

Defines the screens and interactions of the AutoStart interface: a modern stream deck-style look, visual management of profiles and items, onboarding, languages and themes.

## ADDED Requirements

### Requirement: Main screen
The main screen SHALL contain: a sidebar with the profile list (create, rename, duplicate, delete, export, import, mark as active); a top bar with the monitor status (Waiting for simulator / Simulator running / Closing apps / Paused), the trigger selector of the displayed profile and the "Test launch" and "Test close" buttons; and a central area with the profile's items as cards.

#### Scenario: Live status
- **WHEN** the monitor changes state
- **THEN** the top bar updates without reloading the screen

### Requirement: Item cards
Each card SHALL show the icon (or a generic/globe icon for URLs), name, shortened path or URL, badges (Admin, delay in seconds, close behavior), the current session status when there is one, and an enabled toggle. Cards SHALL be reorderable by drag and drop (mouse and keyboard). Clicking a card SHALL open it for editing.

#### Scenario: Toggle disables
- **WHEN** the user turns off a card's toggle
- **THEN** the card is visually dimmed and the item is ignored on the next launch

#### Scenario: Reorder via keyboard
- **WHEN** the user focuses the drag handle, presses space and uses the arrow keys
- **THEN** the card changes position and the order is persisted

### Requirement: Empty state
A profile without items SHALL show an empty state with the "Add app" and "Add URL" actions.

#### Scenario: New profile
- **WHEN** the user creates a profile
- **THEN** the central area shows the empty state with both actions

### Requirement: Item form
The form SHALL allow adding/editing an `app` item, choosing the `.exe` through the native dialog (filling in name, icon and process automatically), or a `url` item. App fields: name, path, arguments, working folder, process name, delay, run as admin, close behavior. URL fields: name, URL, delay. Validation SHALL use the same schemas as persistence and show per-field errors.

#### Scenario: Add an app through the dialog
- **WHEN** the user clicks "Add app" and selects an `.exe`
- **THEN** name, icon and process are filled in and can be adjusted before saving

### Requirement: Settings screen
The settings screen SHALL expose: start with Windows, start minimized, graceful close timeout, only close what AutoStart opened, theme, language, check for updates (automatic on startup and manual button), open log folder and an "About" section (version, repository link, license).

#### Scenario: Change theme
- **WHEN** the user picks the `light` theme
- **THEN** the interface switches to the light theme immediately

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
On first run, the system SHALL create an example profile "MSFS 2024" with trigger `FlightSimulator2024.exe` and show a short wizard (3 to 4 steps) explaining: how the trigger works, the choice of simulator preset (MSFS 2024, MSFS 2020, X-Plane 12, custom), how to add apps and the start with Windows options. Finishing or skipping the wizard SHALL set `onboardingCompleted = true`.

#### Scenario: First run
- **WHEN** the app is opened for the first time
- **THEN** the wizard appears over the main screen and the example profile exists

#### Scenario: Preset choice in the wizard
- **WHEN** the user picks "X-Plane 12" in the wizard
- **THEN** the example profile's trigger becomes `X-Plane.exe` and its name becomes "X-Plane 12"

#### Scenario: Subsequent runs
- **WHEN** the app is opened after onboarding was completed
- **THEN** the wizard does not appear

### Requirement: Visuals and accessibility
The interface SHALL use rounded corners, cards with icons, subtle animations that respect `prefers-reduced-motion`, visible focus on every interactive control and AA contrast in both themes.

#### Scenario: Reduced motion
- **WHEN** Windows has animations turned off
- **THEN** UI transitions are disabled
