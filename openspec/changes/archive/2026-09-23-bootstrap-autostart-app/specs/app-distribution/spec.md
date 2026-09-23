## Purpose

Lets any pilot in the community install AutoStart without administrator rights and receive updates automatically, with a reproducible release process.

## ADDED Requirements

### Requirement: Per-user installer
The project SHALL produce an NSIS installer for Windows x64 that installs the app per user (without requiring administrator), creates a Start menu shortcut and supports uninstalling through Windows. The installer SHALL be available in pt-BR and en.

#### Scenario: Install without admin
- **WHEN** a standard user runs the installer
- **THEN** the app is installed under `%LOCALAPPDATA%` without a UAC prompt and appears in the Start menu

### Requirement: Signed auto update
The app SHALL check for updates on the public repository's GitHub Releases (automatically on startup, if enabled, and through the manual button). When an update is available, it SHALL show the version and notes and install only after the user accepts, then restart the app. Update packages SHALL be verified with the updater public key.

#### Scenario: New version available
- **WHEN** a newer release exists and the user clicks "Update"
- **THEN** the package is downloaded, verified and installed, and the app restarts on the new version

#### Scenario: No connection
- **WHEN** the check fails due to no network
- **THEN** the automatic check fails silently (log only) and the manual check shows an error message

#### Scenario: Invalid signature
- **WHEN** the downloaded package does not match the public key
- **THEN** the update is aborted and the error is shown and logged

### Requirement: Release pipeline
Creating a `v*` tag in the repository SHALL trigger a workflow on `windows-latest` that builds the app, produces the updater-signed installer and publishes a GitHub Release with the installer and `latest.json`. The tag version SHALL match the app version.

#### Scenario: Publish a version
- **WHEN** the maintainer pushes the `v0.1.0` tag
- **THEN** a `v0.1.0` release is published with the `.exe` installer, the `.sig` signature and `latest.json`

#### Scenario: Version mismatch
- **WHEN** the tag is `v0.2.0` but the app version is `0.1.0`
- **THEN** the workflow fails before publishing

### Requirement: Continuous verification
Pushes and pull requests SHALL trigger a workflow that runs the frontend typecheck, lint and tests, plus Rust clippy and tests, on `windows-latest`.

#### Scenario: PR with a type error
- **WHEN** a PR introduces a TypeScript error
- **THEN** the workflow fails

### Requirement: Prepared code signing
The workflow and configuration SHALL contain the Azure Trusted Signing step, disabled and documented, which can be enabled just by configuring secrets and uncommenting the step.

#### Scenario: Enable signing
- **WHEN** the maintainer follows the documentation and configures the Azure secrets
- **THEN** subsequent releases ship with a signed installer

### Requirement: Documentation
The README SHALL cover: what the app does, installation, how to find a process name (Task Manager → Details, and the app's process picker), development (prerequisites, `pnpm tauri dev` on Windows and on macOS with stubs), build, updater key generation, GitHub secrets, the release process and enabling signing.

#### Scenario: New contributor
- **WHEN** someone clones the repository and follows the README
- **THEN** they can run the app in development mode without further instructions
