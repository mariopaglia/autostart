# Como contribuir

Obrigado pelo interesse em melhorar o AutoStart! Este guia explica como relatar problemas, sugerir ideias e enviar código.

> **English** — Contributions are welcome in English or Portuguese. Open an issue before large changes, follow the conventions below (English code, Conventional Commits, all checks green) and send a pull request against `main`.

Ao participar, você concorda em seguir o nosso [Código de Conduta](CODE_OF_CONDUCT.md).

## Relatando um bug

1. Confira se já existe uma [issue](https://github.com/mariopaglia/autostart/issues) sobre o assunto.
2. Abra uma issue com o modelo **Relatar um bug** e informe:
   - versão do AutoStart (Configurações → Sobre) e do Windows;
   - simulador e apps envolvidos;
   - passos para reproduzir, o que aconteceu e o que você esperava;
   - o arquivo de log mais recente (Configurações → **Abrir pasta de logs**). Os logs não contêm os argumentos dos apps, mas revise antes de anexar.

**Falhas de segurança não devem ser relatadas em issues públicas.** Siga a [Política de Segurança](SECURITY.md).

## Sugerindo uma funcionalidade

Abra uma issue com o modelo **Sugerir uma funcionalidade** descrevendo o problema que ela resolve. Mudanças grandes são planejadas antes em [`openspec/`](openspec/) (propostas, specs e tarefas), então vale discutir na issue antes de começar a codificar.

## Enviando código

### Ambiente

Siga a seção [Desenvolvimento](README.md#desenvolvimento) do README. O app roda no Windows e no macOS (com stubs para as APIs exclusivas do Windows), mas o comportamento real precisa ser validado no Windows.

### Fluxo

1. Faça um fork e crie uma branch a partir do `main` (ex.: `fix/close-timeout`, `feat/profile-icons`).
2. Faça mudanças pequenas e focadas, com testes quando houver lógica nova.
3. Rode todas as verificações:

   ```bash
   pnpm typecheck && pnpm lint && pnpm format:check && pnpm test
   cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
   ```

4. Se alterou structs Rust compartilhados com o frontend, inclua os arquivos regenerados em `src/bindings/` (o CI falha se estiverem desatualizados).
5. Abra o pull request preenchendo o modelo. O CI roda em `windows-latest` e precisa passar.

### Convenções

As regras completas estão em [`CLAUDE.md`](CLAUDE.md). Em resumo:

- **Código 100% em inglês**: identificadores, comentários, mensagens de log e commits.
- **Textos da interface nunca ficam no código**: use as chaves de `src/i18n/locales/pt-BR.json` e `en.json`, sempre nos dois idiomas (um teste garante que as chaves batem).
- **Commits** no padrão [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `docs:`, `ci:`, `chore:`, `refactor:`, `test:`.
- **Clean Code**: nomes que revelam a intenção, funções pequenas, retornos antecipados, sem código morto ou comentado. Comentários só para explicar o _porquê_.
- **TypeScript**: `strict`, sem `any` (use `unknown` + Zod). Chamadas ao Tauri passam pelos wrappers de `src/lib/tauri.ts`; estado em stores Zustand; componentes de função, um por arquivo, com export nomeado.
- **Rust**: sem `unwrap()`/`expect()` fora dos testes; erros via `AppError`; código exclusivo do Windows em `src-tauri/src/platform/windows/`, com equivalente em `platform/fallback.rs`; todo bloco `unsafe` com um comentário explicando a invariante.

## Licença

Ao contribuir, você concorda que sua contribuição será licenciada sob a [Licença MIT](LICENSE) do projeto.
