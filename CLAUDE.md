# AutoStart

Windows tray app (Tauri 2 + React 19) that launches a profile's helper apps when a flight simulator process starts and closes them when it exits. Planning lives in `openspec/` (see `openspec/changes/` for active changes).

## Language

- All code is written in **English**: identifiers, file names, commit messages, log messages, error types, test names, and code comments.
- User-facing strings never live in code: they go through i18n (`src/i18n/locales/pt-BR.json` is the default, `en.json` the secondary).
- Conversation with the maintainer and planning docs (`openspec/`) are in Portuguese (pt-BR).

## Clean Code conventions

- Intention-revealing names; no abbreviations except well-known ones (`id`, `url`, `pid`, `exe`).
- Small functions that do one thing; early returns over nested conditionals.
- No dead code, no commented-out code, no speculative abstractions.
- **Comments only when necessary**: explain *why*, never *what*. Expected places: `unsafe` blocks (state the invariant), WinAPI quirks, non-obvious workarounds. No doc comments that restate the signature.
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
