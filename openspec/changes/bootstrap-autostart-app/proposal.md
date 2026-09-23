## Why

Before flying or streaming, virtual pilots manually open several helper programs (Navigraph Charts, Volanta, JoyToKey, SPAD.neXt, PilotUI, websites, etc.), and after the simulator closes those programs stay open, consuming resources. AutoStart removes this ritual: it sits in the tray, detects when the simulator starts or exits, and brings the pilot's "kit" up or down automatically, without changing how the simulator is launched. The app is free and will be distributed to the community of the maintainer's YouTube channel.

## What Changes

Greenfield project. This change creates the complete application, version 0.1.0:

- Windows desktop app on Tauri 2 (React 19 + TypeScript + Vite, Tailwind 4 + shadcn/ui, Zustand, dnd-kit, Zod, i18next) with a lean Rust core.
- **Profiles** with a trigger process and an ordered list of items (`.exe` apps or URLs), persisted as JSON in the app data dir, plus schema-validated import and export.
- Background **process monitor** (polling every 2s) with an `Idle → SimRunning → Closing → Idle` state machine that can be paused.
- **Orchestrated launch** of items (order, delay, detection of already-running apps, run as administrator) and graceful (WM_CLOSE → timeout → TerminateProcess) or forced **closing**, identifying processes by name.
- **Executable inspection**: icon, product name and process name extracted from the `.exe`; list of running processes to pick the trigger from.
- **System integration**: tray with per-state icon and menu, closing the window minimizes to the tray, single instance, start with Windows, start minimized, elevation detection with an option to restart as administrator.
- **UI**: stream deck-style main screen (profile sidebar, draggable cards, monitor status bar, test launch/close), item form, last-session log timeline, settings, first-run onboarding, pt-BR/en i18n and light/dark/system theme.
- **Distribution**: per-user NSIS installer, auto update via GitHub Releases (public repo `mariopaglia/autostart`), GitHub Actions workflow triggered by `v*` tags, Azure Trusted Signing step prepared and commented out, README.
- Code conventions (English, Clean Code, few comments) recorded in `CLAUDE.md`.

## Capabilities

### New Capabilities
- `profile-management`: profiles, launch items (app/URL), active profile, persistence, validated import/export.
- `app-settings`: persisted global settings and their default values.
- `process-monitor`: trigger process detection, state machine, pause and state events.
- `launch-orchestration`: ordered item launch, detection of pre-existing apps, tracking of what was launched, graceful/forced closing, manual launch/close tests.
- `executable-inspection`: icon/metadata extraction from `.exe` files and listing of running processes.
- `privilege-elevation`: running items as administrator, elevation detection and elevated restart.
- `desktop-integration`: tray, single instance, close to tray, start with Windows, start minimized.
- `app-interface`: screens (main, item form, logs, settings), onboarding, i18n and theme.
- `activity-log`: last-session timeline and rotating file log.
- `app-distribution`: installer, auto update, release pipeline and documentation.

### Modified Capabilities
<!-- None: there are no existing specs. -->

## Impact

- **New code**: `src/` (frontend), `src-tauri/` (Rust), `.github/workflows/`, `README.md`, `CLAUDE.md`.
- **Dependencies**: Tauri 2 and plugins (tray via core, autostart, updater, dialog, opener, single-instance, log, process); crates `sysinfo`, `windows`, `serde`, `tokio`, `thiserror`, `ts-rs`, `uuid`, `image`.
- **External systems**: GitHub Releases (updater endpoint, must be public), GitHub Actions (secrets `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`), optionally Azure Trusted Signing.
- **Platform**: Windows 10/11 x64 only. Development happens on macOS with stubs, and real validation is done on Windows.
- **Deviations from the original request** (detailed in design.md): React 19 instead of 18, JSON persistence through Rust instead of `plugin-store`/`plugin-fs`, and minimal code comments.
