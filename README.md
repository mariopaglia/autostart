# AutoStart

[![CI](https://github.com/mariopaglia/autostart/actions/workflows/ci.yml/badge.svg)](https://github.com/mariopaglia/autostart/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/mariopaglia/autostart?label=release)](https://github.com/mariopaglia/autostart/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/mariopaglia/autostart/total)](https://github.com/mariopaglia/autostart/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Platform: Windows 10/11](https://img.shields.io/badge/platform-Windows%2010%2F11-0078d4)

Automatically opens your flight simulator's helper apps when it starts and closes them all when it exits.

AutoStart is a free Windows tray app for virtual pilots. Pick a trigger process (e.g. `FlightSimulator2024.exe`), add your helper apps and websites (Volanta, Navigraph, SimBrief, SPAD.neXt…) and AutoStart launches them in order when the simulator starts and closes them when it exits. The app is available in Portuguese (Brazil) and English.

## What it does

- **Profiles**: each profile has one or more triggers (simulator processes) and a list of items (apps or URLs). Every enabled profile is watched, so switching simulators needs no clicks.
- **Ordered launch**: when the trigger appears, the enabled items open in list order, with a configurable delay before each one. Apps that are already running are skipped.
- **Smart shutdown**: when the simulator exits, each app is closed normally (like clicking the X), force-closed, or kept open, as you choose. By default, only what AutoStart itself opened gets closed, and only after a short wait, so a simulator reopened after a crash keeps its apps.
- **Crash recovery**: apps marked "Reopen if it crashes" are opened again if they crash mid-flight.
- **Unobtrusive**: lives in the system tray, can start with Windows, shows Windows notifications when something needs your attention and keeps a log of every session.

## Installation

1. Download the **[AutoStart-Setup.exe](https://github.com/mariopaglia/autostart/releases/latest/download/AutoStart-Setup.exe)** installer. This link always serves the latest version; release notes for each version are on the [releases page](https://github.com/mariopaglia/autostart/releases/latest). Download **only** from these official links.
2. Run the installer. It installs for your user only, under `%LOCALAPPDATA%`, **without asking for administrator rights**, and adds a Start menu shortcut.
3. **SmartScreen warning**: since the installer is not code-signed yet, Windows may show "Windows protected your PC". Click **More info → Run anyway**.

To uninstall, go to **Windows Settings → Apps → AutoStart → Uninstall**.

### Updates

AutoStart checks for new versions on startup (you can turn this off in Settings) and via the **Check for updates now** button. Every update is signed and is only installed if its signature matches the public key embedded in the app.

## How to use

1. On first run, a wizard creates the sample profile and asks which simulator you use (MSFS 2024, MSFS 2020, X-Plane 12 or custom).
2. On the main screen, use **Add app** to pick one of your apps (see [Adding apps](#adding-apps)) or **Add URL** to open a website.
3. Drag the cards to set the order. The keyboard works too: focus the handle, press Space and use the arrow keys.
4. Use **Test launch** and **Test close** to check everything without opening the simulator.
5. Every **enabled** profile is watched (the switch next to the profile name, also available in the tray's profile menu). When a simulator starts, the profile that has it as a trigger runs.

### Profiles and simulators

A profile can have several triggers: add **MSFS 2020** and **MSFS 2024** to the same profile to use the same apps with both. Use **Add simulator** next to the profile name to pick a preset, a running process or type a process name.

Only one enabled profile can watch a given simulator. If you keep variations of the same setup (e.g. "MSFS Online" and "MSFS Offline"), enabling one turns off the other and AutoStart tells you which one was turned off. New, duplicated and imported profiles start turned off when their simulator is already watched by another profile.

### Closing delay

When the simulator exits, AutoStart waits **60 seconds** before closing your apps (configurable in **Settings → Closing**, 0 closes right away). If the simulator is reopened meanwhile, for example after a crash to desktop, the session continues and nothing is closed or opened again. During the wait, the top of the window shows a countdown with **Close now** and **Keep apps open**; the same actions are in the tray menu.

### Reopen if it crashes

Turn on **Reopen if it crashes** on an app to have AutoStart open it again when it crashes while the simulator is running (up to 3 times per session). AutoStart tells a crash from a normal close by the app's exit code: apps you close yourself are not reopened. The option does not apply to **Test launch**, and crashes in the first 30 seconds after an app opens are not detected.

### Notifications

AutoStart shows Windows notifications when an app could not be opened, SimConnect did not become available, the closing countdown starts, and an app was reopened after a crash (or kept crashing). Successful launches and test sessions never notify. Turn them off in **Settings → Notifications**.

### Adding apps

You don't need to know where an app is installed. **Add app** opens a picker with a search box and two lists:

- **Installed**: apps found in the Start Menu and on the desktop. Popular flight sim tools (Navigraph, SimBrief, Little Navmap, Volanta, vPilot, FSUIPC, SPAD.neXt, GSX and others) appear first under **Suggested for flight sim**.
- **Open now**: apps that have a window open right now. Handy for apps that are not in the Start Menu: open the app, then pick it here.

Picking an app opens the item form already filled in (name, icon, path, arguments and process) so you can review it and save. If the app is in neither list, **Browse for file…** lets you choose the `.exe` yourself.

You can also **drag** an app's shortcut (from the desktop or the Start Menu), an `.exe` file or a web shortcut (`.url`) onto the AutoStart window:

- one file opens the item form already filled in;
- several files are added to the end of the profile at once;
- files that are not apps (documents, folders, `steam://` shortcuts…) are skipped with a notice.

Apps already in the profile are marked "Already in profile" in the lists, and the form warns when you add the same executable twice (it is still allowed, e.g. with different arguments).

Limitations: Microsoft Store apps are not listed and cannot be added, since they have no regular `.exe` to open. Dragging files in does not work while AutoStart runs **as administrator** (Windows blocks dragging from non-elevated windows); use **Add app** instead.

### Process name (detected automatically)

AutoStart identifies apps by their **process name** (e.g. `Volanta.exe`), which it uses to tell whether the app was already running and to close it later. You don't need to configure this: just pick the executable.

Many apps use a _launcher_: you open `Launcher.exe`, it starts the real app and exits. The first time AutoStart opens the item (in a session or with **Test launch**), it follows the processes that get started and saves the name of the one that keeps running. The result shows up under **Advanced → Process name**, marked "Detected automatically", and on the Logs screen.

If detection gets it wrong, edit the field under **Advanced**: the item switches to manual mode and AutoStart no longer changes the name. The **Detect automatically** button switches back to automatic mode. To find the right name:

- **In AutoStart**: in the trigger picker, the "Running processes" list shows everything that is open right now.
- **In Task Manager**: `Ctrl + Shift + Esc` → **Details** tab → the **Name** column shows the executable.

### Start minimized

Turn on **Start minimized** on an item to open the app straight to the taskbar without stealing focus from the simulator. AutoStart asks Windows for a minimized window and, for the first 10 seconds, minimizes any windows the app opens.

Limitations: apps that open only in the tray have nothing to minimize (and work normally); apps that restore their own window after that period may pop back up. In those cases, check whether the app itself has a start-minimized option.

### Wait for SimConnect (MSFS)

Some add-ons need Microsoft Flight Simulator's **SimConnect** to be ready before they work. Turn on **Wait for SimConnect** on those items:

1. When the simulator starts, items **without** this option open first, in list order.
2. Items **with** the option show as "Waiting for SimConnect" and open, also in order, as soon as SimConnect accepts connections (usually around the main menu).
3. If SimConnect doesn't become available within 10 minutes, those items are marked as failed.

The option only works when the session was started by **MSFS 2020** (`FlightSimulator.exe`) or **MSFS 2024** (`FlightSimulator2024.exe`); in a profile that also has another simulator, those items open without waiting when the other simulator starts. **Test launch** skips the wait so you can check everything without the simulator. "SimConnect available" means the simulator accepts add-on connections, not that the flight has loaded. If you changed the simulator's `SimConnect.xml` (pipe name or TCP only), detection may not work.

### Importing and exporting profiles

In a profile's `…` menu, **Export** saves a `.json` file you can share. **Import profile** (in the sidebar) validates the file and creates a copy with new identifiers. If app paths differ on your PC, edit the items after importing (cards show a warning when the executable doesn't exist).

### Apps that need administrator rights

- Check **Run as administrator** on the item. Windows will ask for confirmation (UAC) when it opens.
- To **close** an app running as administrator, AutoStart must be elevated too. In that case the main screen shows a warning with a **Restart as administrator** button.
- While AutoStart is elevated, every app it opens inherits the elevation.
- "Start with Windows" does not start AutoStart elevated. If you always need that, create a **Task Scheduler** task with "Run with highest privileges" pointing to `autostart.exe` with the `--minimized` argument.

### Where data is stored

| What                                        | Where                                            |
| ------------------------------------------- | ------------------------------------------------ |
| Profiles, settings and last session         | `%APPDATA%\com.mariopaglia.autostart\`           |
| Technical logs (5 files of up to 5 MB each) | `%LOCALAPPDATA%\com.mariopaglia.autostart\logs\` |

The Settings screen has an **Open log folder** button. Logs record app names and paths, but never their arguments. When reporting a problem, attach the most recent log file.

## Development

Stack: Tauri 2, React 19, TypeScript, Vite, Tailwind CSS 4 + shadcn/ui, Zustand, Zod and i18next on the frontend; Rust with `sysinfo`, `windows`, `tokio` and `ts-rs` on the backend. Planning lives in [`openspec/`](openspec/) and code conventions in [`CLAUDE.md`](CLAUDE.md).

### Prerequisites

- [Node.js 24 LTS](https://nodejs.org/) and pnpm (pinned in `package.json`): `corepack enable`
- [Stable Rust](https://rustup.rs/)
- **Windows**: [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) ("Desktop development with C++" workload) and WebView2 (included in up-to-date Windows 10/11)
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)

Details in [Tauri — Prerequisites](https://v2.tauri.app/start/prerequisites/).

### Running

```bash
pnpm install
pnpm tauri dev
```

The window starts hidden when **Start minimized** is on (the default). In that case, click the tray icon (or the menu bar icon on macOS).

**On macOS**, the Windows-only APIs (executable icon, elevation, window-based closing) use the stubs in `src-tauri/src/platform/fallback.rs`. The monitor works end to end: use any Mac process as the trigger (e.g. `TextEdit.exe`, since the trigger name must end in `.exe`; on the Mac, AutoStart compares the name without the extension).

### Checks

```bash
pnpm typecheck && pnpm lint && pnpm test          # frontend
cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test   # Rust
```

`cargo test` also regenerates the TypeScript types in `src/bindings/` from the Rust structs (`pnpm bindings` does only that). CI (`.github/workflows/ci.yml`, on `windows-latest`) runs all of this on every push and fails if the bindings are out of date. It also runs Windows-only integration tests that drive real processes and windows through the Win32 APIs (`src-tauri/src/platform/windows/tests.rs`, compiled only on Windows).

### Build

```bash
pnpm tauri build --config '{"bundle":{"createUpdaterArtifacts":false}}'
```

The inline config turns off the updater artifacts, which need the private signing key. The installer lands in `src-tauri/target/release/bundle/nsis/`.

## Publishing (maintainer)

### Updater keys

Done only once. **Losing the private key permanently breaks automatic updates** for existing installs; keep the key and its password in a password manager.

1. Generate the key pair in an interactive terminal (the password is asked twice):

   ```bash
   pnpm tauri signer generate -w ~/.tauri/autostart.key
   chmod 600 ~/.tauri/autostart.key
   ```

2. Put the contents of `~/.tauri/autostart.key.pub` in `plugins.updater.pubkey` in `src-tauri/tauri.conf.json`. The public key can live in the repository.
3. Add the repository secrets (the private key **never** goes into the repository):

   ```bash
   gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.tauri/autostart.key
   gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD   # paste the password when prompted
   ```

### Releasing a version

1. While developing, add user-facing entries under `## [Unreleased]` in `CHANGELOG.md`. The release fails if that section is empty.
2. When ready, open **Actions → Release → Run workflow** on `main` and choose `patch`, `minor` or `major` (or run `gh workflow run release.yml -f bump=patch`). The optional `dry_run` input builds and smoke-tests the installer without publishing anything, and works from any branch.
3. The **Release** workflow (`.github/workflows/release.yml`):
   1. runs all CI checks;
   2. bumps the version in `package.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` and `src-tauri/tauri.conf.json`, and turns the Unreleased section into `## [X.Y.Z] - date`;
   3. commits `chore: release X.Y.Z` (authored by `github-actions[bot]`) and creates the tag `vX.Y.Z`;
   4. builds and uploads the installer `AutoStart-Setup.exe` (fixed name, so the direct download link never changes), its `.sig` signature and `latest.json` (the file installed apps check for updates) to a **draft** release, using the changelog section as release notes;
   5. smoke-tests the installer: silent install, launch, check that the app stays alive, uninstall;
   6. only then pushes the commit to `main` and publishes the release.

If any step fails, the draft release and the tag are deleted and `main` is left untouched. Installed apps only see published releases, so a broken build never reaches users.

To pull a version, delete its release on GitHub: `latest.json` falls back to the previous release's.

### Code signing (Azure Trusted Signing)

Prepared but disabled. With it, SmartScreen stops warning and Windows shows the installer's publisher.

1. Create an [Azure Trusted Signing](https://learn.microsoft.com/azure/trusted-signing/) account with a certificate profile, and an App Registration with the _Trusted Signing Certificate Profile Signer_ role.
2. Add to the repository:
   - Secrets: `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID`
   - Variables: `AZURE_SIGNING_ENDPOINT` (e.g. `https://eus.codesigning.azure.net`), `AZURE_SIGNING_ACCOUNT`, `AZURE_CERTIFICATE_PROFILE`
3. In `release.yml`, uncomment the **Install Azure Trusted Signing CLI** step and the `AZURE_*` and `TAURI_CONFIG` variables of the **Build and upload to a draft release** step. `TAURI_CONFIG` sets `bundle.windows.signCommand` using [`trusted-signing-cli`](https://github.com/Levminer/trusted-signing-cli):

   ```
   trusted-signing-cli -e <endpoint> -a <account> -c <profile> -d AutoStart %1
   ```

From then on, releases ship with a signed installer and executable.

## Community

- **Contributing**: read the [contributing guide](CONTRIBUTING.md) before opening a pull request.
- **Bugs and ideas**: open an [issue](https://github.com/mariopaglia/autostart/issues/new/choose) using the provided templates.
- **Security**: report vulnerabilities privately, as described in the [Security Policy](SECURITY.md).
- **Conduct**: every project space follows the [Code of Conduct](CODE_OF_CONDUCT.md).
- **What's new**: every version is described in the [CHANGELOG](CHANGELOG.md).

## License

[MIT](LICENSE) © Mario Paglia
