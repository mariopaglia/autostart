## Purpose

Finds apps the user may want to add without browsing folders: resolves dropped files and shortcuts, lists installed apps and currently open apps as ready-to-use candidates.

## ADDED Requirements

### Requirement: App candidate
Every discovery source SHALL produce app candidates with: display name, executable path, optional arguments, optional working folder, process name, optional icon (PNG in base64), the source (`installed`, `open` or `dropped`) and whether it is a flight sim suggestion. The display name SHALL be the shortcut name when there is one, otherwise the executable's product name, falling back to the file name without extension.

#### Scenario: Candidate from a shortcut
- **WHEN** a candidate is built from the shortcut `Little Navmap.lnk` pointing to `C:\Tools\LittleNavmap\littlenavmap.exe`
- **THEN** the candidate has name "Little Navmap", the executable path, process name `littlenavmap.exe` and the executable's icon

#### Scenario: Candidate from a bare executable
- **WHEN** a candidate is built from `C:\Tools\vPilot\vPilot.exe` with product name "vPilot"
- **THEN** the candidate has name "vPilot" and process name `vPilot.exe`

### Requirement: Resolve dropped files
Given a list of dropped file paths, the system SHALL return, for each path, either an app candidate, a URL candidate or a rejection reason:
- `.exe` → app candidate;
- `.lnk` → app candidate for the shortcut's target executable, keeping the shortcut's arguments and working folder;
- `.url` whose address is http or https → URL candidate (name from the file name, the address as URL);
- anything else (folders, non-executable files, shortcuts to non-`.exe` targets or missing targets, `.url` with other schemes) → rejected with a reason.

#### Scenario: Drop a desktop shortcut
- **WHEN** the user drops `Navigraph Charts.lnk` whose target is `C:\Program Files\Navigraph\Charts\Navigraph Charts.exe`
- **THEN** the result is an app candidate for that executable named "Navigraph Charts"

#### Scenario: Shortcut with arguments
- **WHEN** the dropped shortcut targets `C:\Tools\App\App.exe` with arguments `--minimized` and working folder `C:\Tools\App`
- **THEN** the candidate carries the arguments `--minimized` and the working folder `C:\Tools\App`

#### Scenario: Web shortcut
- **WHEN** the user drops `SimBrief.url` pointing to `https://www.simbrief.com`
- **THEN** the result is a URL candidate named "SimBrief" with that address

#### Scenario: Unsupported file
- **WHEN** the user drops a `.pdf` file, a folder, a shortcut to a folder or a `.url` pointing to `steam://rungameid/123`
- **THEN** that path is rejected with a reason and no candidate is produced for it

#### Scenario: Broken shortcut
- **WHEN** the dropped shortcut's target executable no longer exists
- **THEN** that path is rejected with the "executable not found" reason

### Requirement: List installed apps
The system SHALL list installed apps from the shortcuts in the all-users and current-user Start Menu (including subfolders) and on the all-users and current-user desktops. Only shortcuts whose target is an existing `.exe` SHALL be listed. Uninstallers (target or shortcut name matching uninstall patterns such as `unins*.exe` and "Uninstall"), executables under the Windows folder and duplicates (same target executable and arguments, compared case-insensitively) SHALL be excluded. The list SHALL be sorted by name, case-insensitively.

#### Scenario: App installed for the current user
- **WHEN** an app installed per user has a Start Menu shortcut in the user's profile
- **THEN** it appears in the installed list with its name and icon

#### Scenario: Uninstaller hidden
- **WHEN** an app's Start Menu folder contains "App" and "Uninstall App" shortcuts
- **THEN** only "App" is listed

#### Scenario: Same app in Start Menu and desktop
- **WHEN** the Start Menu and the desktop both have a shortcut to the same executable
- **THEN** the app is listed once

### Requirement: List open apps
The system SHALL list apps that currently have a visible top-level window, one entry per executable, with the process name exactly as it is running. AutoStart itself, processes whose executable path cannot be read and executables under the Windows folder SHALL be excluded. The list SHALL be sorted by name, case-insensitively.

#### Scenario: Pick an open app
- **WHEN** Little Navmap is running with its window open
- **THEN** it appears in the open list with its icon, executable path and running process name

#### Scenario: Background processes hidden
- **WHEN** a process has no visible window (a service or background helper)
- **THEN** it does not appear in the open list

#### Scenario: Several instances
- **WHEN** two instances of the same executable are open
- **THEN** the app appears once

### Requirement: Flight sim suggestions
Installed apps whose executable or name matches a curated list of popular flight sim tools SHALL be flagged as suggestions. The curated list SHALL live in the app (not downloaded) and match case-insensitively.

#### Scenario: Navigraph installed
- **WHEN** Navigraph Charts is among the installed apps
- **THEN** its candidate is flagged as a flight sim suggestion

#### Scenario: Unrelated app
- **WHEN** a text editor is among the installed apps
- **THEN** its candidate is not flagged as a suggestion

### Requirement: Discovery on non-Windows hosts
On non-Windows development hosts, discovery SHALL compile and return empty installed/open lists, and dropped `.lnk` shortcuts SHALL be rejected as unsupported (other dropped files follow the same rules as on Windows), so the UI can run without errors.

#### Scenario: Development on macOS
- **WHEN** the picker is opened on macOS
- **THEN** both lists are empty and "Browse for file…" still works
