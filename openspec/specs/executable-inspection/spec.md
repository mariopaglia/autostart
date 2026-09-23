# executable-inspection Specification

## Purpose
Reduces manual work when registering items and triggers: automatically extracts data from the chosen executable and lists running processes for selection.

## Requirements

### Requirement: Inspect executable
Given the path of an `.exe`, the system SHALL return `processName` (the file name), `productName` (from the file version information, falling back to `FileDescription` and then to the file name without extension) and `iconBase64` (32x32 or larger PNG in base64, optional).

#### Scenario: Executable with metadata
- **WHEN** the user selects `C:\Program Files\Navigraph\Charts\Navigraph Charts.exe`
- **THEN** the form is filled in with the name "Navigraph Charts", the process `Navigraph Charts.exe` and the app icon

#### Scenario: Executable without icon or version
- **WHEN** the exe has neither an icon nor a version resource
- **THEN** the name comes from the file, `iconBase64` is empty and the card shows a generic icon

#### Scenario: Invalid file
- **WHEN** the path does not exist or is not an `.exe`
- **THEN** the command returns a typed error and the form shows the message

### Requirement: List running processes
The system SHALL list running processes with their name and executable path (when accessible), without duplicates by name and in alphabetical order, so the user can choose the trigger process.

#### Scenario: Choose the trigger from the list
- **WHEN** the user opens the trigger selector with X-Plane running and searches for "x-plane"
- **THEN** `X-Plane.exe` appears in the list and, when selected, becomes the trigger's `processName`

### Requirement: Trigger presets
The system SHALL offer the presets MSFS 2024 (`FlightSimulator2024.exe`), MSFS 2020 (`FlightSimulator.exe`), X-Plane 12 (`X-Plane.exe`) and "custom process" (typed or chosen from the process list).

#### Scenario: Select a preset
- **WHEN** the user picks the "MSFS 2020" preset
- **THEN** the trigger becomes `{ processName: "FlightSimulator.exe", label: "MSFS 2020" }`
