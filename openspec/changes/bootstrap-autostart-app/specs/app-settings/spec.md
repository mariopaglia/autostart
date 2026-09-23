## Purpose

Defines AutoStart's global settings, their default values and how they affect the app's behavior.

## ADDED Requirements

### Requirement: Settings and default values
The system SHALL persist the settings `activeProfileId`, `startWithWindows` (default false), `startMinimized` (default true), `gracefulTimeoutMs` (default 5000, between 1000 and 60000), `closeOnlyIfLaunchedByApp` (default true), `theme` (`system` | `light` | `dark`, default `dark`), `language` (`pt-BR` | `en`, default `pt-BR`), `onboardingCompleted` (default false) and `checkUpdatesOnStartup` (default true).

#### Scenario: First run
- **WHEN** the app starts without a settings file
- **THEN** the settings are created with the default values

#### Scenario: New field in a future version
- **WHEN** the settings file saved by an earlier version lacks a new field
- **THEN** the missing field takes its default value and the others are preserved

### Requirement: Settings validation
The system MUST reject out-of-range settings (e.g. `gracefulTimeoutMs` < 1000) both in the UI and in the backend.

#### Scenario: Invalid timeout
- **WHEN** the user enters a 200 ms graceful timeout
- **THEN** the field shows an error and the value is not saved

### Requirement: Immediate application
Setting changes SHALL take effect without restarting the app (theme, language, timeout, autostart, closing rule).

#### Scenario: Language change
- **WHEN** the user changes the language to `en`
- **THEN** the interface and the tray menu show English text immediately
