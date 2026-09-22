## Why

Antes de voar ou fazer live, pilotos virtuais abrem manualmente vários programas auxiliares (Navigraph Charts, Volanta, JoyToKey, SPAD.neXt, PilotUI, sites etc.) e, ao fechar o simulador, esses programas continuam abertos consumindo recursos. O AutoStart elimina esse ritual: fica na bandeja, detecta quando o simulador abre ou fecha e sobe/derruba o "kit" do piloto automaticamente, sem mudar a forma como ele abre o simulador. O app é gratuito e será distribuído para a comunidade do canal do YouTube do mantenedor.

## What Changes

Projeto greenfield. Esta change cria o aplicativo completo, versão 0.1.0:

- App desktop Windows em Tauri 2 (React 19 + TypeScript + Vite, Tailwind 4 + shadcn/ui, Zustand, dnd-kit, Zod, i18next) com núcleo em Rust enxuto.
- **Perfis** com um processo gatilho e uma lista ordenada de itens (apps `.exe` ou URLs), com persistência em JSON no app data dir, além de importação e exportação validadas por schema.
- **Monitor de processos** em segundo plano (polling a cada 2s) com máquina de estados `Idle → SimRunning → Closing → Idle`, pausável.
- **Abertura orquestrada** dos itens (ordem, delay, detecção de app já aberto, execução como administrador) e **fechamento** gracioso (WM_CLOSE → timeout → TerminateProcess) ou forçado, identificando processos por nome.
- **Inspeção de executáveis**: ícone, nome do produto e nome do processo extraídos do `.exe`; lista de processos em execução para escolher o gatilho.
- **Integração com o sistema**: bandeja com ícone por estado e menu, fechar janela = minimizar para a bandeja, instância única, iniciar com o Windows, iniciar minimizado, detecção de elevação com opção de reiniciar como administrador.
- **UI**: tela principal estilo stream deck (sidebar de perfis, cards arrastáveis, barra de status do monitor, testar abertura/fechamento), formulário de item, timeline de logs da última sessão, configurações, onboarding na primeira execução, i18n pt-BR/en e tema claro/escuro/sistema.
- **Distribuição**: instalador NSIS per-user, auto update via GitHub Releases (repo público `mariopaglia/autostart`), workflow do GitHub Actions disparado por tag `v*`, passo de assinatura Azure Trusted Signing preparado e comentado, README.
- Convenções de código (inglês, Clean Code, poucos comentários) registradas em `CLAUDE.md`.

## Capabilities

### New Capabilities
- `profile-management`: perfis, itens de abertura (app/URL), perfil ativo, persistência, importação/exportação com validação.
- `app-settings`: configurações globais persistidas e seus valores padrão.
- `process-monitor`: detecção do processo gatilho, máquina de estados, pausa e eventos de estado.
- `launch-orchestration`: abertura ordenada dos itens, detecção de preexistentes, registro do que foi aberto, fechamento gracioso/forçado, testes manuais de abertura/fechamento.
- `executable-inspection`: extração de ícone/metadados de `.exe` e listagem de processos em execução.
- `privilege-elevation`: execução de itens como administrador, detecção de elevação e reinício elevado.
- `desktop-integration`: bandeja, instância única, fechar para a bandeja, iniciar com o Windows, iniciar minimizado.
- `app-interface`: telas (principal, formulário de item, logs, configurações), onboarding, i18n e tema.
- `activity-log`: timeline da última sessão e log em arquivo rotativo.
- `app-distribution`: instalador, auto update, pipeline de release e documentação.

### Modified Capabilities
<!-- Nenhuma: não há specs existentes. -->

## Impact

- **Código novo**: `src/` (frontend), `src-tauri/` (Rust), `.github/workflows/`, `README.md`, `CLAUDE.md`.
- **Dependências**: Tauri 2 e plugins (tray via core, autostart, updater, dialog, opener, single-instance, log, process); crates `sysinfo`, `windows`, `serde`, `tokio`, `thiserror`, `ts-rs`, `uuid`, `image`.
- **Sistemas externos**: GitHub Releases (endpoint do updater, precisa ser público), GitHub Actions (secrets `TAURI_SIGNING_PRIVATE_KEY` e `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`), opcionalmente Azure Trusted Signing.
- **Plataforma**: alvo exclusivo Windows 10/11 x64. O desenvolvimento acontece no macOS com stubs e a validação real é feita no Windows.
- **Desvios do pedido original** (detalhados no design.md): React 19 no lugar do 18, persistência em JSON via Rust no lugar de `plugin-store`/`plugin-fs`, e comentários mínimos no código.
