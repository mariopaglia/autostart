# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- "Add app" now opens a picker listing the apps installed on your computer (Start Menu and desktop) and the apps open right now, with search; no more hunting for the `.exe` in folders.
- Popular flight sim tools (Navigraph, SimBrief, Little Navmap, Volanta, vPilot, FSUIPC, SPAD.neXt, GSX and others) are suggested at the top of the installed apps list.
- Drag and drop: drop an app shortcut, an `.exe` or a web shortcut (`.url`) onto the window to add it; dropping several files adds them all at once.
- Apps already in the profile are marked in the picker, and the form warns when the same executable is added twice.

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

[Unreleased]: https://github.com/mariopaglia/autostart/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/mariopaglia/autostart/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/mariopaglia/autostart/releases/tag/v0.1.0
