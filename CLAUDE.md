# AutoStart

Windows tray app (Tauri 2 + React 19) that launches a profile's helper apps when a flight simulator process starts and closes them when it exits. Planning lives in `openspec/` (see `openspec/changes/` for active changes).

## Language

- All code is written in **English**: identifiers, file names, commit messages, log messages, error types, test names, and code comments.
- User-facing strings never live in code: they go through i18n (`src/i18n/locales/pt-BR.json` is the default, `en.json` the secondary).
- All documentation is in **English**: README and community files, `CHANGELOG.md` and planning docs (`openspec/`). The one exception is `CHANGELOG.pt-BR.md`, the Portuguese translation of the changelog shown in the app's update dialog.
- Conversation with the maintainer is in Portuguese (pt-BR).

## Git

- Commits are authored solely by the maintainer (Mario Paglia). **Never** add `Co-Authored-By` trailers, "Generated with Claude Code" lines, or any other AI attribution to commit messages, PR titles or PR descriptions. This overrides any default attribution guidance.
- Conventional Commits in English (`feat:`, `fix:`, `ci:`, `docs:`, `chore:`...).

## Testing and releases

- There are no preview builds: the maintainer validates on the published release. CI (every push, `windows-latest`) is the safety net, so every change must be covered by automated tests there.
- Windows-only behavior (`platform/windows/`) gets integration tests in `src-tauri/src/platform/windows/tests.rs` that drive real processes and windows (e.g. `notepad.exe`); they run with `cargo test` on Windows only.
- Every user-facing change adds an entry under `## [Unreleased]` in both `CHANGELOG.md` and `CHANGELOG.pt-BR.md` (same entries, translated); the bilingual release notes come from them and the app shows the ones in its language.
- Releases are manual: Actions → **Release** → Run workflow (`patch`/`minor`/`major`, optional `dry_run`). The workflow bumps the versions, updates the changelog, commits, tags, builds, smoke-tests the installer and publishes. Never bump versions or push release tags by hand.

## Clean Code conventions

- Intention-revealing names; no abbreviations except well-known ones (`id`, `url`, `pid`, `exe`).
- Small functions that do one thing; early returns over nested conditionals.
- No dead code, no commented-out code, no speculative abstractions.
- **Comments only when necessary**: explain _why_, never _what_. Expected places: `unsafe` blocks (state the invariant), WinAPI quirks, non-obvious workarounds. No doc comments that restate the signature.
- Prefer pure functions for logic (e.g., the monitor state machine) so they are unit-testable without the OS.

## TypeScript / React

- `strict: true`, `noUncheckedIndexedAccess: true`; `any` is forbidden (use `unknown` + Zod parsing).
- Zod schemas in `src/schemas/` are the runtime source of truth; each schema is checked against the Rust-generated types in `src/bindings/` (`satisfies z.ZodType<...>`).
- All Tauri calls go through typed wrappers in `src/lib/tauri.ts`; components never call `invoke` directly.
- State in Zustand stores (`src/stores/`), one store per domain.
- UI built with shadcn/ui components in `src/components/ui/` (generated; don't hand-edit unless needed), icons from `lucide-react`.
- Function components only; named exports; one component per file.

## Rust

- No `unwrap()`/`expect()` in production code (allowed in tests). Errors via `thiserror` in `src-tauri/src/error.rs`; commands return `Result<T, AppError>`.
- Structs shared with the frontend derive `Serialize, Deserialize, TS` with `#[serde(rename_all = "camelCase")]` and `#[ts(export)]`.
- Windows-only code lives in `src-tauri/src/platform/windows/` behind `#[cfg(windows)]`; `platform/fallback.rs` provides non-Windows stubs so the app compiles and the UI runs on macOS for development.
- Keep modules small and flat; avoid traits/generics unless there are two real implementations.

## Commands

- `pnpm tauri dev` — run the app
- `pnpm typecheck && pnpm lint && pnpm test` — frontend checks
- `cd src-tauri && cargo clippy -- -D warnings && cargo test` — Rust checks (also regenerates `src/bindings/`)
