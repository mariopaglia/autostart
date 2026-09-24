## Why

AutoStart already covers its core loop (detect the simulator, open the helpers, close them afterwards), but four gaps remain in day-to-day use:

- Only the last session is kept, so a problem from yesterday's flight is lost as soon as a test or a new flight runs.
- Tools started through Steam (`steam://` shortcuts) cannot be added at all.
- A profile shared by MSFS 2020 and MSFS 2024 opens every item with both simulators, which pushes pilots into keeping near-identical profiles.
- Everything depends on the pilot starting the simulator by hand, and nothing can open before it. Head-tracking and hardware tools often have to be running before the simulator.

## What Changes

- **Session history**: keep the last 20 real sessions plus the most recent test session, persisted across restarts. The Logs screen gets a session selector and a "Clear history" action. The existing `last-session.json` is migrated into the history.
- **Steam links as URL items**: URL items also accept Steam launch links (`steam://rungameid/<id>` and `steam://run/<id>`). As before, they are open-only and never closed. Dropping a Steam `.url` shortcut (the kind Steam puts on the desktop) now creates a URL item instead of being rejected.
- **Items for specific simulators**: apps and URLs get an optional list of the profile's simulators they apply to ("Only with"). Empty means all simulators, which is the current behavior. Items that don't apply to the simulator that started the session are ignored in that session. The option only shows when the profile has two or more simulators.
- **Start flight**: each trigger can have an optional "how to start" target: an `.exe`, a Steam link or a Microsoft Store app. MSFS 2020/2024 get ready-made Steam and Microsoft Store choices. A new "Start flight" action in the top bar and in the tray starts a session right away, opens the items marked "Open before the simulator", starts the simulator and then continues the normal launch. If the simulator does not appear within 5 minutes, the session goes through the usual close delay. A new monitor state `simStarting` covers that wait.
- Timeline, notifications, tray icon and tooltip cover the new flow: simulator started, simulator failed to start, simulator did not appear.

No breaking changes. All new fields are optional, and their defaults keep today's behavior, so profiles and settings from earlier versions load unchanged.

## Capabilities

### New Capabilities
- `flight-start`: starting the simulator from AutoStart. Covers the per-trigger start target, the "Start flight" action, the items that open before the simulator and the wait for the simulator to appear.

### Modified Capabilities
- `activity-log`: the last-session timeline becomes a persisted history of sessions, with a selector and a clear action.
- `profile-management`: triggers gain an optional start target; app and URL items gain `onlyForTriggers`; app items gain `launchBeforeSimulator`; URL items accept Steam launch links.
- `launch-orchestration`: the launch skips items that don't apply to the session's simulator, gains a before-simulator phase in sessions started with "Start flight", and opens Steam links.
- `process-monitor`: new `simStarting` state and transitions, plus sessions started on demand instead of by detection.
- `app-discovery`: dropped `.url` files with Steam launch links become URL candidates.
- `desktop-integration`: a "Start flight" submenu in the tray, and `simStarting` in the tray icon and tooltip.
- `desktop-notifications`: notify when the simulator could not be started or did not appear.
- `app-interface`: "Start flight" in the top bar, the `simStarting` status, the new item form fields and the trigger start-target editor.

## Impact

- **Rust**: `models.rs` (Trigger, AppItem, UrlItem, MonitorState, TimelineKind), `validation.rs`, `storage.rs` (history file and migration), `monitor/` (state machine, session, runner, new start command), `launcher.rs`, `app_discovery.rs`, `tray.rs`, `commands.rs`, `platform/` (launching a start target: exe, URI, AUMID), plus Windows integration tests.
- **Frontend**: `src/schemas/profile.ts`, `src/lib/tauri.ts`, the monitor store, `LogsScreen`, `TopBar`, `TriggerSelector` (start target editor), `AppItemForm`/`UrlItemForm`, `StatusBadge`, `session-notifications.ts`, both i18n locales and the regenerated `src/bindings/`.
- **Docs**: README (Start flight, Steam links, per-simulator items, history) and both changelogs.
- **Dependencies**: none.
