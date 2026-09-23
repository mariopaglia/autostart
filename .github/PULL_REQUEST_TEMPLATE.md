## What changes

<!-- Describe the change and why. Reference the issue: "Closes #123". -->

## How it was tested

<!-- Automated tests, and what was validated manually on Windows (and/or macOS). -->

## Checklist

- [ ] `pnpm typecheck && pnpm lint && pnpm format:check && pnpm test` pass
- [ ] `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` pass
- [ ] Bindings in `src/bindings/` regenerated, if Rust structs changed
- [ ] New UI strings added to both `pt-BR.json` and `en.json`
- [ ] Code in English and commits following Conventional Commits
- [ ] `CHANGELOG.md` and `CHANGELOG.pt-BR.md` updated under `## [Unreleased]`, if the change affects users
