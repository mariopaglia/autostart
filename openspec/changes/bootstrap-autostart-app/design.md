## Context

Projeto greenfield (repositório `mariopaglia/autostart`, público). Motivação e escopo estão em `proposal.md`, e o comportamento está nas specs em `specs/`. Restrições que moldam o design:

- Alvo: Windows 10/11 x64. O desenvolvimento acontece em **macOS**, e o comportamento real é validado em uma máquina Windows e no CI (`windows-latest`).
- O mantenedor é sênior em TypeScript e não tem experiência com Rust. Por isso o Rust fica pequeno, plano e com poucas abstrações.
- Convenções: código 100% em inglês, Clean Code e comentários só quando necessários (ver `CLAUDE.md`).
- Entrega em 4 fases, com parada para validação ao fim de cada uma (ver `tasks.md`).

## Goals / Non-Goals

**Goals:**
- Rust com responsabilidades claras: monitor, orquestração de launch/close, persistência e WinAPI. Todo o restante fica no frontend.
- Contrato de tipos Rust ↔ TS verificado em tempo de compilação.
- Máquina de estados testável sem o SO.
- App compilando e com a UI utilizável no macOS para iterar rápido.

**Non-Goals:**
- Suporte oficial a macOS/Linux (os stubs existem só para desenvolvimento).
- Monitorar vários perfis ao mesmo tempo (só o perfil ativo é observado).
- Reduzir a elevação de itens abertos por um AutoStart elevado (herdam a elevação, ver Riscos).
- Integração com SimConnect/estado do voo, telemetria ou contas de usuário.
- Assinatura de código ativa (fica apenas preparada).

## Decisions

### D1. Versões principais (verificadas em 22/09/2026)

| Pacote | Versão | Observação |
|---|---|---|
| Tauri (`tauri` / `@tauri-apps/cli` / `@tauri-apps/api`) | 2.11.x | |
| React / React DOM | 19.3.x | Escolhido no lugar do 18: o shadcn v4 não usa `forwardRef` |
| Vite / `@vitejs/plugin-react` | 8.3.x / 6.1.x | |
| TypeScript | 5.9.x | Mais maduro no ecossistema (typescript-eslint, ts-rs) que o 7.x nativo |
| Tailwind CSS / `@tailwindcss/vite` | 4.3.x | Configuração CSS-first, sem `tailwind.config.js` |
| shadcn (CLI) | 4.x | Preset `radix-nova` (Radix + Lucide + Geist), base color neutral; `cn` vem do pacote oficial `cn` |
| lucide-react | 1.x | |
| Zustand | 5.0.x | |
| `@dnd-kit/core` / `@dnd-kit/sortable` | 6.3.x / 10.0.x | |
| Zod | 4.x | |
| i18next / react-i18next | 26.x / 17.x | |
| Vitest | 5.x | |
| Plugins Tauri (JS + Rust) | autostart 2.5, updater 2.12, dialog 2.7, opener 2.5, process 2.3, single-instance 2.4, log 2.9 | |
| `sysinfo` | 0.39.x | |
| `windows` | 0.62.x | Com features WinAPI específicas (ver D8) |
| `tokio` | 1.x | Via runtime do Tauri (`tauri::async_runtime`) |
| `serde` / `serde_json` | 1.x | |
| `thiserror` | 2.x | |
| `ts-rs` | 12.x | Gera tipos TS a partir do Rust |
| `uuid` | 1.x (v4) | |
| `image` | latest, só feature `png` | Codifica o ícone extraído |

Node 24 LTS e pnpm 10 (via `corepack enable`). No `.tool-versions`/`packageManager` fica a versão exata do pnpm.

### D2. Árvore de pastas

```
autostart/
├── .github/workflows/
│   ├── ci.yml                    # typecheck, lint, test, clippy, cargo test (push/PR)
│   └── release.yml               # tag v* → build + release + latest.json
├── CLAUDE.md
├── README.md
├── LICENSE                       # MIT
├── package.json
├── pnpm-lock.yaml
├── components.json               # shadcn
├── index.html
├── tsconfig.json / tsconfig.node.json
├── vite.config.ts
├── eslint.config.js
├── src/
│   ├── main.tsx
│   ├── App.tsx                   # layout + roteamento simples por estado (sem router)
│   ├── index.css                 # Tailwind 4 + tokens de tema do shadcn
│   ├── bindings/                 # GERADO pelo ts-rs (não editar)
│   ├── schemas/
│   │   ├── profile.ts            # Zod: Profile, LaunchItem (discriminated union), Trigger
│   │   ├── settings.ts
│   │   └── *.test.ts
│   ├── lib/
│   │   ├── tauri.ts              # wrappers tipados de invoke/listen
│   │   ├── trigger-presets.ts
│   │   └── utils.ts              # cn() do shadcn
│   ├── stores/
│   │   ├── profiles-store.ts
│   │   ├── settings-store.ts
│   │   └── monitor-store.ts      # estado do monitor, status dos itens, timeline
│   ├── i18n/
│   │   ├── index.ts
│   │   └── locales/{pt-BR,en}.json
│   ├── hooks/                    # use-monitor-events, use-theme
│   ├── components/
│   │   ├── ui/                   # gerado pelo shadcn
│   │   ├── layout/               # AppShell, Sidebar, TopBar
│   │   ├── profiles/             # ProfileList, ProfileMenu, TriggerSelector, ProcessPicker
│   │   ├── items/                # ItemList (dnd), ItemCard, ItemFormDialog, EmptyState
│   │   └── common/               # ElevationBanner, StatusBadge
│   └── screens/
│       ├── MainScreen.tsx
│       ├── LogsScreen.tsx
│       ├── SettingsScreen.tsx
│       └── OnboardingWizard.tsx
└── src-tauri/
    ├── Cargo.toml
    ├── build.rs
    ├── tauri.conf.json
    ├── capabilities/default.json
    ├── icons/                    # ícones do app + tray-idle / tray-running / tray-paused
    └── src/
        ├── main.rs               # só chama lib::run()
        ├── lib.rs                # builder: plugins, state, setup, comandos
        ├── error.rs              # AppError (thiserror) + Serialize
        ├── models.rs             # Profile, LaunchItem, Settings, MonitorSnapshot... (serde + ts-rs)
        ├── storage.rs            # leitura/escrita atômica de JSON, defaults, migração
        ├── commands.rs           # todos os #[tauri::command], finos, delegam aos módulos
        ├── monitor/
        │   ├── mod.rs            # loop de polling + emissão de eventos
        │   ├── state_machine.rs  # função pura de transição + testes
        │   └── session.rs        # snapshot do perfil, itens abertos/preexistentes, timeline
        ├── launcher.rs           # abrir itens (app/url/runas), aguardar processo
        ├── closer.rs             # fechar itens (graceful/force), em paralelo
        ├── processes.rs          # sysinfo: listar, procurar por nome
        ├── tray.rs               # ícone, menu, atualização por estado/idioma
        └── platform/
            ├── mod.rs            # API única re-exportando windows/ ou fallback
            ├── windows/
            │   ├── mod.rs
            │   ├── windows_close.rs   # EnumWindows + WM_CLOSE, TerminateProcess
            │   ├── elevation.rs       # is_elevated, ShellExecuteExW runas
            │   └── exe_info.rs        # ícone + VersionInfo
            └── fallback.rs       # stubs para macOS (kill via sysinfo, sem ícone, is_elevated=false)
```

### D3. Rust como dono dos dados; JSON próprio no lugar de `plugin-store`/`plugin-fs`
O monitor roda no Rust e precisa ler o perfil ativo e as configurações sem depender do frontend (a janela pode nem ter sido aberta). Por isso o Rust é a fonte de verdade: `storage.rs` lê e grava `profiles.json` e `settings.json` em `app_data_dir`, com `schemaVersion`, escrita atômica (grava em `*.tmp` e depois faz `rename`) e `#[serde(default)]` para campos novos. O estado em memória fica em `tauri::State<AppState>` com `Mutex`.
- *Alternativa descartada:* `tauri-plugin-store`. Ficaria com duas fontes de verdade (JS e Rust) e ainda exigiria sincronização.
- *Consequência:* `plugin-fs` e `plugin-store` saem da lista de dependências. Importar e exportar são comandos Rust que recebem o caminho escolhido no `plugin-dialog`.

### D4. Contrato de tipos: ts-rs + Zod `satisfies`
Os structs em `models.rs` derivam `TS` (ts-rs), com `#[serde(rename_all = "camelCase")]` e `LaunchItem` como `#[serde(tag = "type", rename_all = "lowercase")]`. `cargo test` gera `src/bindings/*.ts`. Os schemas Zod em `src/schemas/` são declarados como `satisfies z.ZodType<Profile>`. Se Rust e TS divergirem, o `pnpm typecheck` falha.
- *Alternativa descartada:* tauri-specta, que gera também os comandos, mas adiciona mais peças móveis e macros para quem está aprendendo Rust.

### D5. Import/export com dupla validação
`import_profile(path)` lê o arquivo e devolve `serde_json::Value` bruto. O frontend valida com Zod (mensagens de erro por campo, traduzidas), gera UUIDs novos e chama `save_profile`, que desserializa com serde (segunda barreira). `export_profile(id, path)` serializa o perfil completo, incluindo `iconBase64`, para os cards aparecerem com ícone na máquina de quem importa, mesmo antes de o caminho ser corrigido.

### D6. Monitor: loop simples + máquina de estados pura
- Uma task `tauri::async_runtime::spawn` com `tokio::time::interval(2s)` e `MissedTickBehavior::Skip`.
- A cada tick: `System::refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing())` (só nomes e PIDs), e verifica se o gatilho existe.
- `state_machine::next(state, Input) -> (State, Option<Action>)` é pura. `Input` pode ser `TriggerSeen`, `TriggerMissing`, `ClosingFinished`, `Pause` ou `Resume`. `Action` pode ser `StartSession` ou `EndSession`. O contador de ausências (2 ticks) fica dentro do estado `SimRunning { missed_ticks }`. Testes unitários cobrem todos os cenários da spec `process-monitor`.
- Abertura e fechamento rodam em tasks separadas, para não travar o loop. O loop recebe `ClosingFinished` por um `tokio::sync::mpsc`.
- Comandos da UI e da bandeja (pause/resume, test_launch/test_close) conversam com o monitor pelo mesmo canal (`MonitorCommand`), evitando locks cruzados.
- Eventos: `monitor://state` (`MonitorSnapshot`), `monitor://item-status` (`{ itemId, status, message? }`) e `monitor://log` (`TimelineEntry`). `get_monitor_state` devolve o snapshot atual.

### D7. Launch
- App normal: `std::process::Command` com `CommandExt::raw_arg(args)` (os args chegam como string única, sem parsing nosso), `current_dir(workingDir ou pasta do exe)` e `creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)`, para o filho não morrer junto com o AutoStart.
- `runAsAdmin`: `ShellExecuteExW` com `lpVerb = "runas"`. Se o usuário negar o UAC, a chamada retorna `ERROR_CANCELLED`, que mapeamos para `AppError::ElevationDenied`.
- URL: `tauri_plugin_opener::OpenerExt::open_url`.
- Confirmação: depois de iniciar, faz polling do `processName` a cada 500 ms por até 10 s → `running`, ou `error` ("processo não detectado").
- Tudo sequencial, com `tokio::time::sleep(delayMs)` antes de cada item.

### D8. Close (WinAPI)
- Encontra os PIDs pelo `processName` com `sysinfo`.
- Graceful: `EnumWindows` → `GetWindowThreadProcessId` → `PostMessageW(hwnd, WM_CLOSE)` para **todas** as janelas top-level do PID, inclusive as ocultas. É o mesmo que o `taskkill` sem `/F` faz, e cobre apps de bandeja como o JoyToKey. Depois faz polling a cada 250 ms até `gracefulTimeoutMs`. Os PIDs que sobrarem vão para o force.
- Force: `OpenProcess(PROCESS_TERMINATE)` → `TerminateProcess`. `ERROR_ACCESS_DENIED` vira `AppError::AccessDenied` (mensagem da spec `privilege-elevation`).
- Os itens são fechados em paralelo (`join_all`), com um timeout global de `gracefulTimeoutMs + 5s`.
- Features do crate `windows`: `Win32_Foundation`, `Win32_UI_WindowsAndMessaging`, `Win32_System_Threading`, `Win32_Security`, `Win32_UI_Shell`, `Win32_Storage_FileSystem`, `Win32_Graphics_Gdi`.
- Todo `unsafe` fica isolado em `platform/windows/`, com funções seguras por fora e um comentário curto de invariante em cada bloco.

### D9. Inspeção de exe
- Ícone: `SHGetFileInfoW(SHGFI_ICON | SHGFI_LARGEICON)` (ou `PrivateExtractIconsW` a 48px) → `GetIconInfo` + `GetDIBits` → buffer BGRA → RGBA → `image` PNG → base64. `DestroyIcon` e `DeleteObject` ficam em guards com `Drop`.
- Metadados: `GetFileVersionInfoSizeW`/`GetFileVersionInfoW`/`VerQueryValueW` lendo a tradução em `\VarFileInfo\Translation`, depois `ProductName` → `FileDescription` → stem do arquivo.
- No macOS, o stub devolve só o nome do arquivo.

### D10. Elevação e reinício elevado × instância única
- `is_elevated`: `OpenProcessToken` + `GetTokenInformation(TokenElevation)`.
- Problema: a nova instância elevada esbarraria no `single-instance` e só focaria a antiga. Solução: `relaunch_as_admin` chama `ShellExecuteExW("runas", current_exe, "--wait-for-pid <pid>")`. Se der certo, a antiga chama `app.exit(0)`. A nova, no `main` e **antes** de construir o Tauri, detecta `--wait-for-pid` e espera o PID antigo morrer (timeout de 10 s). Só então registra o single-instance. Se o UAC for negado, a antiga continua viva e mostra o erro.

### D11. Bandeja e janela
- `TrayIconBuilder` (core do Tauri 2, feature `tray-icon`) com três ícones embutidos (`include_bytes!`). O `tray.rs` expõe `refresh(app)`, chamado quando mudam estado, perfis ou idioma, e reconstrói o menu (é pequeno, então reconstruir sai mais simples que atualizar item a item).
- Os textos do menu da bandeja vêm de uma tabela pequena em Rust (`tray_labels(language)`), porque o menu existe antes de a webview carregar. São as únicas strings de UI fora do i18n do frontend, e ficam num único lugar.
- `on_window_event(CloseRequested)` → `api.prevent_close()` + `window.hide()`.
- A janela nasce com `visible: false`. No `setup`, ela é exibida se `!startMinimized || !onboardingCompleted`. O autostart registra o argumento `--minimized` (tratado da mesma forma).

### D12. Frontend
- Sem router: `App.tsx` alterna entre as telas `main | logs | settings` com um estado do Zustand. O onboarding é um `Dialog` sobre a tela principal.
- Stores: `profiles-store` (CRUD otimista + `invoke`), `settings-store` e `monitor-store` (alimentado pelo hook `useMonitorEvents`, que faz `get_monitor_state` na montagem e depois `listen`).
- Formulários com `react-hook-form` + `@hookform/resolvers/zod` e os componentes `Field` do shadcn v4 (substituto do antigo `Form`) via `Controller`, reaproveitando os schemas.
- dnd-kit: `DndContext` + `SortableContext` + `KeyboardSensor` com `sortableKeyboardCoordinates`, para atender o requisito de acessibilidade.
- Tema: classe `dark` no `<html>`, com `matchMedia('(prefers-color-scheme: dark)')` quando o tema é `system`.
- i18n: `i18next` com `fallbackLng: 'pt-BR'`. A troca de idioma chama `save_settings`, que dispara `tray::refresh`.

### D13. Logs
- `tauri-plugin-log` com targets `LogDir` + `Stdout`, `RotationStrategy::KeepSome(5)` e `max_file_size(5 MB)`. Os `args` nunca são logados.
- A timeline da sessão (`Vec<TimelineEntry>`) vive em `session.rs`, é emitida ao vivo e gravada em `last-session.json` ao fim de cada sessão ou teste.

### D14. Distribuição
- `tauri.conf.json`: `bundle.targets = ["nsis"]`, `windows.nsis.installMode = "currentUser"`, `languages = ["PortugueseBR", "English"]`, `createUpdaterArtifacts = true`, e updater com `endpoints = ["https://github.com/mariopaglia/autostart/releases/latest/download/latest.json"]` e `pubkey` gerada por `pnpm tauri signer generate`.
- `release.yml`: `on: push: tags: ['v*']`, `windows-latest`, pnpm + Rust cache, passo que compara a tag com a `version` do `tauri.conf.json` e `tauri-apps/tauri-action` com `tagName`/`releaseName`, publicando o `latest.json`. Secrets: `TAURI_SIGNING_PRIVATE_KEY` e `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
- Azure Trusted Signing: `bundle.windows.signCommand` com `trusted-signing-cli` documentado no README e passo `azure/login` + variáveis (`AZURE_CLIENT_ID`, `AZURE_TENANT_ID`, `AZURE_CLIENT_SECRET`, endpoint, account, certificate profile) comentados no workflow.
- Identifier: `com.mariopaglia.autostart`. Licença: MIT.

### D16. Builds de preview para validação no Windows
O mantenedor valida no Windows como usuário final, sem toolchain instalada. O workflow `preview-build.yml` roda em todo push fora da `main` e gera um artifact do GitHub Actions (retido por 14 dias) com o instalador NSIS e o `.exe` portátil, sem assinatura e sem artefatos do updater. Os checkpoints de cada fase são validados com esse artifact. Ele é independente do `release.yml` (D14), que continua sendo o único caminho para publicar versões.

### D15. Desenvolvimento no macOS
`platform/fallback.rs` implementa a mesma API: close via `sysinfo::Process::kill_with(Signal::Term)` e depois `kill()`, ícone `None`, `is_elevated = false`, `runas` retorna erro "não suportado". O `sysinfo` funciona no macOS, então dá para testar o monitor de ponta a ponta usando um processo qualquer como gatilho (ex.: `TextEdit`). O `CommandExt::raw_arg` e as `creation_flags` ficam atrás de `#[cfg(windows)]`.

## Risks / Trade-offs

- [O AutoStart elevado abre todos os itens elevados (herança de token)] → Documentar no README e no tooltip do banner. Reduzir a elevação via `explorer.exe`/token do shell fica como melhoria futura.
- [O autostart do Windows não inicia apps elevados] → Quem precisa de elevação precisa reiniciar como admin manualmente ou usar o Agendador de Tarefas (documentado, não automatizado).
- [Launchers com nome de processo imprevisível] → `processName` editável e seletor a partir dos processos em execução. O README ensina a descobrir o nome.
- [Processos com o mesmo nome que não foram abertos pelo AutoStart (ex.: um segundo `chrome.exe`)] → O fechamento é por nome, por decisão explícita. Com `closeOnlyIfLaunchedByApp`, só age se o item não era preexistente. Itens URL nunca são fechados.
- [Polling de 2 s + 2 ticks = até ~4-6 s para detectar o fechamento] → Aceitável para o caso de uso e evita falsos positivos quando o simulador se reinicia.
- [SmartScreen alerta em instaladores não assinados] → Documentar ("Mais informações → Executar assim mesmo"). Assinatura Azure preparada.
- [Não dá para testar WinAPI no Mac] → CI em `windows-latest` roda clippy e testes. Cada fase termina com um checklist de validação manual no Windows.
- [ts-rs gera arquivos só ao rodar `cargo test`] → Script `pnpm bindings` e verificação no CI (`git diff --exit-code src/bindings`).

## Migration Plan

Não se aplica (primeira versão). O rollback de releases é feito despublicando o release no GitHub: o `latest.json` volta a apontar para o release anterior.

## Open Questions

- Artes finais do ícone do app e dos ícones de bandeja: começam com placeholders gerados por `pnpm tauri icon` e o mantenedor troca depois.
- Texto de licença/créditos na seção "Sobre" (MIT assumido).
