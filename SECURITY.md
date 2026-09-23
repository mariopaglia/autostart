# Security Policy

## Supported versions

Only the latest version published on [Releases](https://github.com/mariopaglia/autostart/releases/latest) receives security fixes. AutoStart itself notifies you when a new version is available.

## Reporting a vulnerability

**Do not open a public issue.** Use GitHub's [private vulnerability reporting](https://github.com/mariopaglia/autostart/security/advisories/new) (**Security → Report a vulnerability** tab) and include:

- the affected version and your Windows version;
- a description of the problem and its impact;
- steps to reproduce or a proof of concept.

You will get an initial response within 7 days. Once confirmed, the fix is shipped in a new release and the report is disclosed with due credit, unless you prefer to stay anonymous.

## What counts as a vulnerability

Examples of what we care about:

- bypassing the update signature verification;
- making AutoStart run a program the user did not configure;
- escalating privileges through AutoStart;
- leaking profile data or app arguments.

## How the project protects itself

- **Signed updates**: every update package is signed with a private key kept outside the repository (in GitHub secrets and with the maintainer). The app only installs packages whose signature matches the public key embedded in it.
- **Releases only through the pipeline**: official installers are built by the [`release.yml`](.github/workflows/release.yml) workflow in the official repository.
- **No elevation by default**: the app installs and runs without administrator privileges. Elevation only happens when the user asks for it.
- **Official download**: download AutoStart only from [github.com/mariopaglia/autostart/releases](https://github.com/mariopaglia/autostart/releases).
