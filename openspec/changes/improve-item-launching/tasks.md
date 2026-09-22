<!--
Fases: cada fase termina com um CHECKPOINT. Pare, apresente o resumo ao mantenedor e só continue após a validação.
Convenções: código em inglês, Clean Code, comentários só quando necessários (ver CLAUDE.md).
Branch: feat/improve-item-launching a partir do main.
-->

## 1. Fase 1 — Modelo e lançamento unificado

- [x] 1.1 Adicionar em `models.rs` o `ProcessNameMode` (`auto` | `manual`), os campos `process_name_mode`, `start_minimized` e `wait_for_sim_connect` em `AppItem` (com `#[serde(default)]`), `ItemStatus::WaitingSimConnect`, `TimelineKind::SimConnectReady`/`SimConnectWaitSkipped` (o `AppError::SimConnectUnavailable` entra na 3.3, onde passa a ser usado, para o `clippy -D warnings` do CI não falhar antes). Verificar com teste de desserialização de um item da v0.1.0 (sem os campos novos) e com `cargo test` regenerando os bindings
- [x] 1.2 Atualizar os schemas Zod (`appItemSchema` com os campos novos e defaults) e os tipos derivados no frontend. Verificar com testes Vitest de um perfil da v0.1.0 aceito com os defaults e com `pnpm typecheck` passando contra os bindings
- [x] 1.3 Criar `platform::launch(exe, args, working_dir, LaunchOptions) -> AppResult<u32>` (design D1): `ShellExecuteExW` com `SEE_MASK_NOCLOSEPROCESS`, verbo `runas` quando elevado, `SW_SHOWMINNOACTIVE` quando minimizado e PID via `GetProcessId`, com o handle num guard `Drop`; stub no `fallback.rs` com `Command::spawn`. Remover `launch_elevated`/`configure_launch` e usar a função nova no `launcher.rs`. Verificar com `cargo clippy -D warnings` no macOS e no CI (Windows)
<!-- Checkpoints intermediários dispensados pelo mantenedor em 22/09/2026: implementar direto e validar no build final. -->
- [x] 1.4 CHECKPOINT Fase 1: gerar o build de preview e pedir ao mantenedor para validar no Windows que a abertura normal, com argumentos, com pasta de trabalho e como administrador (UAC aceito e negado) continua igual à v0.1.0

## 2. Fase 2 — Aprendizado do nome do processo

- [x] 2.1 Criar `process_tracker.rs` com `ProcessSample`, `expand_tracked` e `choose_process_name` como funções puras (design D2). Verificar com testes unitários: raiz viva, launcher que encerra com filho, neto após a morte do intermediário, fallback pela pasta de instalação, empate por horário de início e ausência de candidato
- [x] 2.2 Expor em `processes.rs` a leitura de `ProcessSample` (PID, PID pai, nome, caminho, início) a partir do `sysinfo`. Verificar com teste que a leitura inclui o próprio processo de teste com o PID pai correto
- [x] 2.3 Reescrever a confirmação de execução no `launcher.rs` para considerar o PID raiz, os descendentes rastreados e o `processName` (10 s), e iniciar a task de aprendizado de 30 s para itens em modo `auto`. Verificar com `cargo test` e, no macOS, com um item apontando para um script que abre outro app e encerra
- [x] 2.4 Aplicar o nome aprendido (design D3): `Session::learn_process_name`, `AppState::learn_process_name` (só em modo `auto` e se o item ainda existir), gravação em disco e evento `profiles://changed`; entrada na timeline com o nome aprendido. Verificar com testes unitários de `Session` e `AppState` (modo manual preservado, item removido ignorado)
- [x] 2.5 Tratar `profiles://changed` no frontend (wrapper em `src/lib/tauri.ts`, hook e upsert na `profiles-store`). Verificar com teste Vitest da store e observando o formulário atualizar sem recarregar após um "Testar abertura"
- [x] 2.6 Mover a pasta de trabalho e o nome do processo para a seção "Avançado" (componente `collapsible` do shadcn), com a indicação "detectado automaticamente", edição mudando para `manual` e o botão "Detectar automaticamente". Textos em pt-BR e en. Verificar adicionando um app sem abrir a seção e alternando entre os modos
- [x] 2.7 CHECKPOINT Fase 2: build de preview e checklist no Windows: app sem launcher mantém o nome; launcher real (ex.: Volanta ou outro que troque de processo) aprende o nome certo pelo "Testar abertura" e fecha no "Testar fechamento"; modo manual preservado; timeline mostra o aprendizado

## 3. Fase 3 — Abrir minimizado e aguardar o SimConnect

- [x] 3.1 Implementar `platform::minimize_new_windows` (design D4) com stub no macOS e chamar a cada 250 ms por 10 s após o item ficar `running` quando `startMinimized = true`. Verificar com `cargo clippy` e, no Windows, na checagem da fase
- [x] 3.2 Implementar `platform::is_simconnect_available` (design D5, `WaitNamedPipeW` com o nome do pipe numa constante) com stub no macOS, e `supports_simconnect(trigger)` como função pura. Verificar com testes unitários de `supports_simconnect` (MSFS 2020, MSFS 2024, maiúsculas, X-Plane)
- [x] 3.3 Implementar a abertura em duas etapas no `runner.rs` (design D6), com o `AppError`/`ErrorKind::SimConnectUnavailable`: `split_launch_phases` pura, status `waitingSimConnect`, polling de 2 s com prazo de 10 min, erro por tempo limite, pulo da espera no teste e gatilhos sem suporte. Verificar com testes unitários de `split_launch_phases` e, no macOS, com "Testar abertura" registrando `simConnectWaitSkipped` na timeline
- [x] 3.4 Adicionar ao formulário os switches "Abrir minimizado" e "Aguardar o SimConnect" (desativado com dica quando o gatilho não é MSFS 2020/2024, via `SIMCONNECT_TRIGGERS`), os badges "Minimizado" e "SimConnect" nos cards, o status "Aguardando SimConnect" no `ItemStatusBadge` e os tipos novos na timeline, em pt-BR e en. Verificar com o teste de paridade de chaves de i18n e visualmente no macOS
- [x] 3.5 Atualizar README (nome do processo automático, abrir minimizado e suas limitações, aguardar o SimConnect e suas limitações) e a seção "Não lançado" do CHANGELOG. Verificar com `pnpm format:check`
- [x] 3.6 CHECKPOINT Fase 3: build de preview e checklist no Windows: item minimizado sem roubar o foco; item que só abre na bandeja fica `running`; com MSFS 2020 e/ou 2024, itens com SimConnect aguardam e abrem após o menu principal (confirmando o nome do pipe); fechar o simulador durante a espera não abre os itens; gatilho X-Plane desativa a opção

## 4. Fase 4 — Release v0.2.0 e teste do updater

<!-- 4.1: pendência registrada em 22/09/2026. A bootstrap-autostart-app só pode ser arquivada depois do teste do updater com esta v0.2.0 (tarefa 4.2 dela); arquivar a bootstrap antes desta. -->
- [x] 4.1 Arquivar a change `bootstrap-autostart-app` (pré-requisito para arquivar esta) quando suas tarefas restantes permitirem, ou registrar a pendência. Verificar com `openspec validate improve-item-launching --strict` sem avisos de arquivamento
- [ ] 4.2 Subir a versão para 0.2.0 em `package.json`, `src-tauri/Cargo.toml` e `src-tauri/tauri.conf.json`, datar o CHANGELOG, fazer o merge no `main` e, com a confirmação do mantenedor, enviar a tag `v0.2.0`. Verificar o release publicado com `AutoStart-Setup.exe`, `.sig` e um `latest.json` apontando para `AutoStart-Setup.exe`
- [ ] 4.3 CHECKPOINT Fase 4: com a v0.1.0 instalada no Windows, o mantenedor usa "Verificar atualizações agora", aceita a v0.2.0 e confirma o download, a verificação da assinatura, a reinstalação e o reinício na nova versão com os perfis preservados (fecha também a tarefa 4.2 da change `bootstrap-autostart-app`)
