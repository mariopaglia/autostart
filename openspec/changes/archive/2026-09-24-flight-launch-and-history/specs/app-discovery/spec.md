## MODIFIED Requirements

### Requirement: Resolve dropped files
Given a list of dropped file paths, the system SHALL return, for each path, either an app candidate, a URL candidate or a rejection reason:
- `.exe` → app candidate;
- `.lnk` → app candidate for the shortcut's target executable, keeping the shortcut's arguments and working folder;
- `.url` whose address is http, https or a Steam launch link (`steam://rungameid/<digits>` or `steam://run/<digits>`) → URL candidate (name from the file name, the address as URL);
- anything else (folders, non-executable files, shortcuts to non-`.exe` targets or missing targets, `.url` with other schemes or other Steam links) → rejected with a reason.

#### Scenario: Drop a desktop shortcut
- **WHEN** the user drops `Navigraph Charts.lnk` whose target is `C:\Program Files\Navigraph\Charts\Navigraph Charts.exe`
- **THEN** the result is an app candidate for that executable named "Navigraph Charts"

#### Scenario: Shortcut with arguments
- **WHEN** the dropped shortcut targets `C:\Tools\App\App.exe` with arguments `--minimized` and working folder `C:\Tools\App`
- **THEN** the candidate carries the arguments `--minimized` and the working folder `C:\Tools\App`

#### Scenario: Web shortcut
- **WHEN** the user drops `SimBrief.url` pointing to `https://www.simbrief.com`
- **THEN** the result is a URL candidate named "SimBrief" with that address

#### Scenario: Steam shortcut
- **WHEN** the user drops `SimHub.url`, created by Steam on the desktop, pointing to `steam://rungameid/1234560`
- **THEN** the result is a URL candidate named "SimHub" with that address

#### Scenario: Unsupported file
- **WHEN** the user drops a `.pdf` file, a folder, a shortcut to a folder or a `.url` pointing to `steam://uninstall/123` or `ms-settings:display`
- **THEN** that path is rejected with a reason and no candidate is produced for it

#### Scenario: Broken shortcut
- **WHEN** the dropped shortcut's target executable no longer exists
- **THEN** that path is rejected with the "executable not found" reason
