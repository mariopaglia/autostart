# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2026-09-24

### Added

- "Start flight": set how each simulator starts (Steam, Microsoft Store or its executable) and start the flight from the top bar or the tray. Apps marked "Open before the simulator" open first, then the simulator, then the rest.
- Session history: the Logs screen keeps your last 20 sessions and your latest test, with a session picker, an error count per session and "Clear history".
- URL items accept Steam links (`steam://rungameid/…`), and Steam desktop shortcuts can be dragged onto the window.
- "Only with" on apps and URLs: in a profile with several simulators, an item can open only with some of them.

## [0.4.0] - 2026-09-23

### Added

- "See what's new" next to the version in Settings → About shows the full version history, in the app's language and offline.

### Fixed

- The update dialog now shows the release notes formatted (headings, lists, bold) instead of raw Markdown, and in the app's language.

## [0.3.1] - 2026-09-23

### Changed

- Clearer closing settings: "Wait after the simulator exits" is the main option, and the time an app gets to close on its own is now set in seconds, explains that it only applies to apps set to "Close normally" and lives under "Advanced".
- AutoStart is now licensed under the GNU GPL v3.0 or later (previously MIT). The installer and the About section in Settings show the license, and forks must use a different name and icon.

### Fixed

- Clicking a profile in the sidebar while on Logs or Settings now opens it on the Profiles screen, and the profile is only highlighted while that screen is open.

## [0.3.0] - 2026-09-23

### Added

- "Add app" now opens a picker listing the apps installed on your computer (Start Menu and desktop) and the apps open right now, with search; no more hunting for the `.exe` in folders.
- Popular flight sim tools (Navigraph, SimBrief, Little Navmap, Volanta, vPilot, FSUIPC, SPAD.neXt, GSX and others) are suggested at the top of the installed apps list.
- Drag and drop: drop an app shortcut, an `.exe` or a web shortcut (`.url`) onto the window to add it; dropping several files adds them all at once.
- Apps already in the profile are marked in the picker, and the form warns when the same executable is added twice.
- Every enabled profile is now watched: open any simulator and its profile runs, with no profile to switch first. Only one enabled profile can watch a given simulator; enabling one turns off the other and tells you.
- A profile can have several simulators as triggers, e.g. MSFS 2020 and MSFS 2024 sharing the same apps.
- Apps are closed 60 seconds after the simulator exits (configurable, 0 closes right away). If the simulator comes back in that time, e.g. after a crash to desktop, nothing is closed. A countdown offers "Close now" and "Keep apps open", also in the tray menu.
- Windows notifications when an app could not be opened, SimConnect did not become available, the closing countdown starts or a crashed app was reopened. They can be turned off in Settings.
- Per-app "Reopen if it crashes" option: during a flight, an app that crashes is opened again (up to 3 times per session); apps you close yourself are not reopened.

### Changed

- The "active profile" is gone: the switch next to each profile's name (and the tray's profile menu) now decides which profiles are watched. On the first start after updating, profiles that shared a simulator with the previously active one are turned off.
- Profiles and settings saved by this version cannot be read by earlier versions.

## [0.2.0] - 2026-09-22

### Added

- Each app's process name is detected automatically, including apps that use a launcher; manual configuration is still available under "Advanced".
- Per-app "Start minimized" option.
- Per-app "Wait for SimConnect" option (MSFS 2020 and 2024): these items open after the others, once the simulator accepts add-on connections.

### Changed

- All apps are now started through the Windows shell, which makes it possible to follow the processes they start and to open them minimized.

## [0.1.0] - 2026-09-22

First public release.

### Added

- Profiles with a trigger (the simulator process) and app or URL items, with presets for MSFS 2024, MSFS 2020 and X-Plane 12.
- Ordered launch with a configurable delay, detection of apps that are already running and confirmation of the started process.
- Per-item graceful close, force close or "keep open", run in parallel with a timeout; option to close only what AutoStart opened.
- Launcher support: closing reaches every process with the configured name.
- Running items as administrator and restarting AutoStart elevated.
- Buttons to test launching and closing without the simulator.
- Tray icon with a profile menu, monitoring pause, single instance and a start-with-Windows option.
- Logs screen with a timeline of the last session, plus technical logs in a rotating file.
- Profile import and export as JSON.
- Interface in Portuguese (Brazil) and English, with light, dark and system themes.
- First-run wizard.
- Per-user installer (no administrator rights) and signed automatic updates.

[Unreleased]: https://github.com/mariopaglia/autostart/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/mariopaglia/autostart/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/mariopaglia/autostart/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/mariopaglia/autostart/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/mariopaglia/autostart/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/mariopaglia/autostart/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/mariopaglia/autostart/releases/tag/v0.1.0
