<!--
Phases: each phase ends with a CHECKPOINT. Stop, present the summary to the maintainer and only continue after validation.
Conventions: code in English, Clean Code, comments only when necessary (see CLAUDE.md).
Delivery: complete files, never "rest unchanged".
-->

## 1. Phase 1 — Scaffold, type contract and Rust core

- [x] 1.1 Before writing code, present the folder tree (design D2) and the resolved versions (design D1) to the maintainer and wait for approval
- [x] 1.2 Scaffold with `pnpm create tauri-app` (React + TS + Vite), `identifier = com.mariopaglia.autostart`, `productName = AutoStart`, version 0.1.0, `packageManager` with a pinned pnpm and MIT LICENSE. Verify that `pnpm tauri dev` opens the default window on macOS
- [x] 1.3 Configure strict TS (`strict`, `noUncheckedIndexedAccess`), ESLint (typescript-eslint with `no-explicit-any: error`), Prettier and the `typecheck`, `lint`, `test` and `bindings` scripts. Verify with `pnpm typecheck && pnpm lint` passing
- [x] 1.4 Install Tailwind 4 (`@tailwindcss/vite`) and run `shadcn init` (radix-nova preset, CSS variables, dark theme). Add the button, card, dialog, input, label, switch, select, badge, dropdown-menu, tooltip, field, sonner, scroll-area, separator, alert, command and popover components. Verify by rendering a Button with the `dark` class active
- [x] 1.5 Install the frontend dependencies (zustand, @dnd-kit/core, @dnd-kit/sortable, @dnd-kit/utilities, zod, react-hook-form, @hookform/resolvers, i18next, react-i18next, lucide-react, vitest and the Tauri JS plugins) and the Rust crates (design D1). Verify with `cargo check` on macOS
- [x] 1.6 Create `error.rs` (`AppError` with thiserror + `Serialize` as `{ kind, message }`) and `models.rs` (Profile, Trigger, LaunchItem tagged by `type`, OnClose, Settings, ItemStatus, MonitorState, MonitorSnapshot, TimelineEntry, ExeInfo, ProcessInfo) with serde camelCase + ts-rs. Verify with `cargo test` generating `src/bindings/*.ts`
- [x] 1.7 Create the Zod schemas `src/schemas/profile.ts` and `settings.ts` with the defaults and limits from the specs, using `satisfies z.ZodType<...>` against the bindings. Verify with Vitest tests of valid and invalid cases (non-http URL, out-of-range delay, empty name, discriminated union) and with `pnpm typecheck` failing if a field diverges
- [x] 1.8 Implement `storage.rs`: data directory, reads with `#[serde(default)]`, atomic writes (`.tmp` + rename), `schemaVersion`, corrupted-file recovery (`.corrupt-<timestamp>`) and the MSFS 2024 example profile. Verify with unit tests in a temporary directory (roundtrip, missing field, corrupted file)
- [x] 1.9 Create the `platform/` layer with the single API and `fallback.rs` for macOS (design D15). Verify with `cargo check` on macOS and `cargo check --target x86_64-pc-windows-msvc` (or in CI)
- [x] 1.10 Implement `processes.rs` (list without duplicates, sorted, and find PIDs by name case-insensitively) and `platform/windows/exe_info.rs` (base64 PNG icon + ProductName/FileDescription). Verify with a unit test of name normalization and, on Windows, with `inspect_exe` on `C:\Windows\notepad.exe` returning the name and icon
- [x] 1.11 Implement `launcher.rs` (Command with `raw_arg`, workingDir, detached flags, `runas` via `ShellExecuteExW`, URL via opener and process confirmation within 10 s) and `closer.rs` + `platform/windows/windows_close.rs` (WM_CLOSE on every top-level window, polling, TerminateProcess, AccessDenied, parallel closing with a global timeout). Verify on Windows by opening and closing Notepad through the test commands
- [x] 1.12 Expose the commands in `commands.rs`: get_profiles, save_profile, delete_profile, set_active_profile, get_settings, save_settings, inspect_exe, list_running_processes, import_profile, export_profile, test_launch and test_close (no monitor yet). Create the typed wrappers in `src/lib/tauri.ts`. Verify by calling each command from a temporary debug page and checking the typed responses
- [x] 1.13 Create `.github/workflows/ci.yml` (windows-latest: pnpm install, typecheck, lint, test, clippy `-D warnings`, cargo test and `git diff --exit-code src/bindings`). Verify with the workflow green after the push
- [x] 1.15 Create `.github/workflows/preview-build.yml` (manual dispatch only, triggered at checkpoints): builds on windows-latest and publishes the NSIS installer and the portable `autostart.exe` as an artifact, so the maintainer can validate on Windows as an end user, without development tools. Verify by downloading the artifact and running it on Windows
<!-- Phase 1 validated on Windows by the maintainer on 2026-09-22 (preview build). -->
- [x] 1.14 CHECKPOINT Phase 1: present the summary and the manual Windows validation checklist (inspect_exe, test_launch/test_close with Notepad and with a launcher app) and wait for the maintainer's validation

## 2. Phase 2 — Monitor and main UI

- [x] 2.1 Implement `monitor/state_machine.rs` as a pure function (design D6). Verify with unit tests of every scenario in the process-monitor spec (start, 2 missed ticks, flapping, reopening during closing, pause/resume, app started with the sim already running)
- [x] 2.2 Implement `monitor/session.rs` (profile snapshot, launched × pre-existing, timeline and persistence to `last-session.json`). Verify with unit tests of the tracking and the `closeOnlyIfLaunchedByApp` rule
- [x] 2.3 Implement `monitor/mod.rs` (2 s loop, `MonitorCommand` channel, launch/close tasks, `monitor://state`, `monitor://item-status` and `monitor://log` events) and the get_monitor_state, pause_monitor and resume_monitor commands. Move test_launch/test_close to the monitor channel, blocking them during a session. Verify on macOS using `TextEdit` as the trigger: opening/closing TextEdit opens/closes the items and the events reach the frontend
- [x] 2.4 Create the Zustand stores (profiles, settings, monitor) and the `useMonitorEvents` hook (initial snapshot + listen). Verify with Vitest tests of the stores with a mocked `invoke`
- [x] 2.5 Build the AppShell layout (Sidebar + TopBar + central area) with the dark theme and `main | logs | settings` screen switching. Verify visually at 1000x680 and at the 800x560 minimum
- [x] 2.6 Build the profile Sidebar: list, mark active, create, rename, duplicate (new UUIDs + suffix), delete (blocking the last one), export and import (dialog + Zod + new UUIDs + per-field errors). Verify by running each action and checking `profiles.json`
- [x] 2.7 Build the TopBar: live monitor StatusBadge, TriggerSelector (presets + ProcessPicker with search via list_running_processes + custom) and test buttons with a disabled state and tooltip. Verify by changing the trigger and watching the status change
- [x] 2.8 Build the ItemList with dnd-kit (mouse + keyboard), ItemCard (icon/globe, name, shortened path, badges, toggle, session status, missing-exe warning) and EmptyState. Verify by reordering with mouse and keyboard and checking the persisted order
- [x] 2.9 Build the ItemFormDialog (app via native dialog + inspect_exe with autofill, editable processName, or URL) with react-hook-form + Zod. Verify by adding an app and a URL, editing them and seeing the per-field validation errors
- [x] 2.10 CHECKPOINT Phase 2: present the summary and the manual Windows validation checklist (real flow: start MSFS or a substitute trigger, items opening in order, pre-existing item skipped, graceful and forced closing, launcher that switches process) and wait for the maintainer's validation

## 3. Phase 3 — System integration, logs, settings, i18n and onboarding

- [x] 3.1 Implement `tray.rs` (three per-state icons, tooltip, menu with a profiles submenu, pause/resume, open, quit, per-language labels and `refresh` on changes). Verify by switching profile and pausing from the tray, with the UI reflecting it
- [x] 3.2 Configure close to tray (`CloseRequested` → hide), left click on the icon to show/focus, window with `visible: false` and conditional display (startMinimized, onboarding, `--minimized`). Verify by closing with the X and reopening from the tray
- [x] 3.3 Configure the single-instance plugin (focus the existing window) and the `--wait-for-pid` flow in `main.rs` (design D10). Verify by opening the app twice on Windows: it stays a single instance and the window gets focus
- [x] 3.4 Integrate the autostart plugin with `--minimized`, synced with `startWithWindows`, and with the state read from the system shown on screen. Verify by enabling, restarting Windows and disabling
- [x] 3.5 Implement `platform/windows/elevation.rs` (is_elevated, relaunch_as_admin) and the ElevationBanner with the conditions from the privilege-elevation spec. Verify on Windows: the banner appears with an admin item, the elevated restart works and a denied UAC keeps the instance
- [x] 3.6 Configure tauri-plugin-log (LogDir + 5 × 5 MB rotation, no args) and the "Open log folder" action. Build the LogsScreen with the live timeline and the persisted last session. Verify by running a launch/close test and restarting the app
- [x] 3.7 Build the SettingsScreen (every field from the app-settings spec, Zod validation, immediate application, About section with version and repo link). Verify by changing each setting and restarting
- [x] 3.8 Configure i18n (i18next, pt-BR default, en, fallback) covering 100% of UI texts and error messages by `kind`, plus the `system | light | dark` theme reacting to `prefers-color-scheme`. Verify with a Vitest test comparing the pt-BR and en keys and by switching language/theme at runtime
- [x] 3.9 Build the OnboardingWizard (3–4 steps, simulator presets applied to the example profile, skip/finish → `onboardingCompleted`). Verify by deleting the app data dir and opening the app
- [x] 3.10 Review accessibility and motion: visible focus, `prefers-reduced-motion`, contrast in both themes and subtle card animations. Verify by navigating with the keyboard only
<!-- Phases 2 and 3 validated by the maintainer on 2026-09-22 (Phase 3 on Windows with the Phase 4 preview build). -->
- [x] 3.11 CHECKPOINT Phase 3: present the summary and the manual Windows validation checklist (tray, single-instance, autostart after reboot, elevation, logs, language, onboarding) and wait for the maintainer's validation

## 4. Phase 4 — Distribution and documentation

- [x] 4.1 Generate the app and tray icons (`pnpm tauri icon` with a placeholder) and configure the NSIS bundle (currentUser, PortugueseBR + English, Start menu shortcut). Verify by building the installer on Windows and installing without UAC
- [ ] 4.2 Generate the updater keys (`pnpm tauri signer generate`), configure `pubkey` + GitHub Releases endpoint + `createUpdaterArtifacts`, integrate plugin-updater + plugin-process (check on startup if enabled, manual button, dialog with notes, install and restart). Guide the maintainer to register the secrets in the repository. Verify with a test key and a `latest.json` pointing to a higher version
- [ ] 4.3 Create `.github/workflows/release.yml` (`v*` tag, windows-latest, tag × version check, tauri-action publishing the installer, `.sig` and `latest.json`) with the Azure Trusted Signing step commented out and the `signCommand` documented. Verify with the `v0.1.0` tag producing the complete release
- [ ] 4.4 Write the README (pt-BR with an en summary): what it is, installation and the SmartScreen warning, how to find a process name, using profiles and import/export, elevation, development on Windows and macOS, build, updater keys, secrets, release and enabling signing. Verify by following the README on a clean clone
<!-- 4.1–4.4: implemented (branch feat/phase-4-distribution), updater key generated and secrets registered on 2026-09-22. Installer validated on Windows (4.1). Pending verifications: v0.1.0 tag (4.3), end-to-end update with v0.1.1 (4.2) and README on a clean clone (4.4). Community docs added (CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, CHANGELOG, templates). -->
- [ ] 4.5 Final validation against the specs: run `openspec validate bootstrap-autostart-app --strict`, green CI, and walk through each spec's scenarios on Windows with the release installer, recording the results
- [ ] 4.6 CHECKPOINT Phase 4: present the v0.1.0 release summary and the open items (final icon artwork, code signing) and wait for the maintainer's validation
