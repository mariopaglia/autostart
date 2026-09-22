## O que muda / What changes

<!-- Descreva a mudança e o motivo. Referencie a issue: "Closes #123". -->

## Como foi testado / How it was tested

<!-- Testes automatizados, e o que foi validado manualmente no Windows (e/ou macOS). -->

## Checklist

- [ ] `pnpm typecheck && pnpm lint && pnpm format:check && pnpm test` passam
- [ ] `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` passam
- [ ] Bindings em `src/bindings/` regenerados, se structs Rust mudaram
- [ ] Textos novos da interface em `pt-BR.json` e `en.json`
- [ ] Código em inglês e commits no padrão Conventional Commits
- [ ] `CHANGELOG.md` atualizado, se a mudança afeta o usuário
