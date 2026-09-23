# Contributing

Thanks for your interest in improving AutoStart! This guide explains how to report problems, suggest ideas and submit code. Issues and pull requests are welcome in English or Portuguese.

By participating, you agree to follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Reporting a bug

1. Check whether an [issue](https://github.com/mariopaglia/autostart/issues) already exists for it.
2. Open an issue with the **Report a bug** template and include:
   - your AutoStart version (Settings → About) and Windows version;
   - the simulator and apps involved;
   - steps to reproduce, what happened and what you expected;
   - the most recent log file (Settings → **Open log folder**). Logs don't contain app arguments, but review them before attaching.

**Security issues must not be reported in public issues.** Follow the [Security Policy](SECURITY.md).

## Suggesting a feature

Open an issue with the **Suggest a feature** template describing the problem it solves. Large changes are planned up front in [`openspec/`](openspec/) (proposals, specs and tasks), so it's worth discussing in the issue before you start coding.

## Submitting code

### Environment

Follow the [Development](README.md#development) section of the README. The app runs on Windows and macOS (with stubs for the Windows-only APIs), but real behavior must be validated on Windows.

### Workflow

1. Fork the repository and create a branch from `main` (e.g. `fix/close-timeout`, `feat/profile-icons`).
2. Keep changes small and focused, with tests whenever there is new logic.
3. Run all checks:

   ```bash
   pnpm typecheck && pnpm lint && pnpm format:check && pnpm test
   cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
   ```

4. If you changed Rust structs shared with the frontend, include the regenerated files in `src/bindings/` (CI fails if they are out of date).
5. Add user-facing changes under `## [Unreleased]` in [`CHANGELOG.md`](CHANGELOG.md).
6. Open the pull request against `main` and fill in the template. CI runs on `windows-latest` and must pass.

### Conventions

The full rules are in [`CLAUDE.md`](CLAUDE.md). In short:

- **Code is 100% English**: identifiers, comments, log messages and commits.
- **UI strings never live in code**: use the keys in `src/i18n/locales/pt-BR.json` and `en.json`, always in both languages (a test ensures the keys match).
- **Commits** follow [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `docs:`, `ci:`, `chore:`, `refactor:`, `test:`.
- **Clean Code**: intention-revealing names, small functions, early returns, no dead or commented-out code. Comments only to explain the _why_.
- **TypeScript**: `strict`, no `any` (use `unknown` + Zod). Tauri calls go through the wrappers in `src/lib/tauri.ts`; state lives in Zustand stores; function components, one per file, with named exports.
- **Rust**: no `unwrap()`/`expect()` outside tests; errors via `AppError`; Windows-only code in `src-tauri/src/platform/windows/`, with a counterpart in `platform/fallback.rs`; every `unsafe` block has a comment explaining its invariant.

## License

By contributing, you agree that your contribution will be licensed under the project's license, the [GNU General Public License v3.0 or later](LICENSE), including the additional terms described in the [README](README.md#additional-terms-and-trademarks), and you confirm that you have the right to submit it under that license.
