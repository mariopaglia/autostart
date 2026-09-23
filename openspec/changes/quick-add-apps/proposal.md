## Why

Adding apps to a profile is the core task of AutoStart, but today the only way is the native file dialog: the user must know where the app's `.exe` lives (often deep inside `Program Files` or `AppData`). Many virtual pilots in the community are not comfortable browsing install folders, so they get stuck on the very first step. Windows already knows where the apps are (Start Menu and desktop shortcuts, running processes), so AutoStart can offer them instead of asking the user to find them.

## What Changes

- **Drag and drop**: the user can drag a desktop/Start Menu shortcut (`.lnk`), an `.exe` or an internet shortcut (`.url` with an http/https address) onto the AutoStart window. A drop overlay appears while dragging. Shortcuts are resolved to their target executable (and arguments/working folder), then:
  - a single dropped app opens the item form already filled in (name, icon, path, arguments, process), ready to save;
  - several dropped files are added at once, with a summary notification;
  - unsupported files (folders, documents, `.url` with non-http addresses, broken shortcuts) are reported without adding anything.
- **App picker**: "Add app" opens a searchable picker instead of going straight to the form. It has two lists and a fallback:
  - **Installed** — apps found in the Start Menu (all users and current user) and on the desktop, with icon and name, excluding uninstallers and Windows components.
  - **Open now** — apps that currently have a visible window, with icon and name, excluding AutoStart itself and Windows system processes. Picking one also records the exact running process name.
  - **Browse for file…** — the existing native dialog, for anything not listed.
  Choosing an entry opens the item form pre-filled, the same review step used by the file dialog.
- **Suggested for flight sim** (new suggestion): the Installed list pins, at the top, installed apps matching a curated list of popular flight sim tools (e.g. Navigraph Charts, SimBrief Downloader, Little Navmap, Volanta, vPilot, FSUIPC, SPAD.neXt, GSX/Couatl).
- **Already added** (new suggestion): entries whose executable is already in the current profile are marked "Already in profile"; saving a duplicate from any path shows a warning but is allowed (same exe with different arguments is a valid use).
- The empty state and the "Add app" buttons open the picker; the empty state also hints that apps can be dragged in.
- No profile format change: all paths produce a regular `app` (or `url`) item.

### Non-goals (future ideas)

- Microsoft Store/UWP apps (no launchable `.exe` path; would need `shell:AppsFolder` activation).
- `.url` shortcuts with custom schemes such as `steam://` (URL items only accept http/https today).
- Scanning Steam/other launcher libraries directly.

## Capabilities

### New Capabilities

- `app-discovery`: resolving dropped files and shortcuts into app candidates, listing installed apps (Start Menu/desktop shortcuts), listing open apps (processes with visible windows) and matching flight sim suggestions.

### Modified Capabilities

- `app-interface`: the empty state and "Add app" open the new app picker; new drag-and-drop requirement; the item form can be opened pre-filled from a candidate and warns about duplicates.

## Impact

- **Rust**: new `app_discovery.rs` (candidate building, filtering, deduplication, suggestions — pure and unit-tested); new Windows modules for shortcut resolution (`IShellLinkW`/`IPersistFile` via COM), known folders (Start Menu, Desktop) and processes with visible windows (reusing `top_level_windows`); stubs in `platform/fallback.rs`; new commands `resolve_dropped_paths`, `list_installed_apps`, `list_open_apps`; new `AppCandidate` model exported to `src/bindings/`. `windows` crate gains the `Win32_System_Com` feature.
- **Frontend**: new `AppPickerDialog` (tabs + `Command` search list), drag-and-drop hook using Tauri's webview `onDragDropEvent`, `ItemFormDialog`/`AppItemForm` accepting an initial candidate, duplicate warning, `EmptyItems`/`MainScreen` wiring, new Zod schema for `AppCandidate`, typed wrappers in `src/lib/tauri.ts`, pt-BR/en translations.
- **Tests**: Rust unit tests for filtering/dedup/suggestions; Windows integration tests creating real `.lnk` files and opening `notepad.exe` to verify it appears in "Open now"; Vitest tests for drop classification and candidate-to-form mapping.
- **Docs**: README section on adding apps; CHANGELOG entry.
- **Ordering dependency**: this change modifies `app-interface` requirements created by `bootstrap-autostart-app` and changed by `improve-item-launching`; both must be archived before this one.
