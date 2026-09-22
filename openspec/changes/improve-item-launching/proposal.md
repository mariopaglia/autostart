## Why

Na v0.1.0, o usuário precisa acertar à mão o `processName` de apps que usam launcher (o `.exe` escolhido abre outro processo e encerra), abrir apps minimizados exige conhecer argumentos de linha de comando específicos de cada app, e addons que dependem do SimConnect falham se abrirem antes de o MSFS estar pronto. Ferramentas como o Addon Linker já resolvem esses pontos, e a comunidade espera o mesmo. Esta versão (v0.2.0) também serve como a primeira atualização real entregue pelo updater, validando de ponta a ponta a tarefa 4.2 da change `bootstrap-autostart-app`.

## What Changes

- **Nome do processo aprendido automaticamente**: ao abrir um item `app`, o AutoStart acompanha os processos que ele iniciou (o processo lançado e seus descendentes, com fallback para processos novos cujo executável está na mesma pasta de instalação). Quando o `.exe` iniciado encerra e o app real continua rodando, o nome desse processo é salvo no item. O usuário só informa nome e caminho do executável.
- **Modo do nome do processo**: cada item passa a ter `processNameMode` (`auto` | `manual`). No formulário, o campo "Nome do processo" sai da área principal para uma seção "Avançado", com a indicação "detectado automaticamente"; editar o campo muda o modo para `manual`, e é possível voltar para automático.
- **Abrir minimizado**: novo checkbox por item `app` (`startMinimized`). O app é iniciado com pedido de janela minimizada e, nos primeiros segundos, as janelas do processo são minimizadas pelo AutoStart. Apps que ignoram o pedido ou se restauram sozinhos são uma limitação documentada.
- **Aguardar o SimConnect**: novo checkbox por item `app` (`waitForSimConnect`). Na abertura da sessão, os itens sem a opção abrem primeiro, na ordem da lista; os itens marcados abrem depois, também na ordem, quando o SimConnect do MSFS estiver disponível. Se o SimConnect não ficar disponível no tempo limite, o item fica com erro. A opção só vale para gatilhos do MSFS 2020/2024 e fica desativada, com dica, para outros gatilhos.
- **Novo status de item** `waitingSimConnect`, exibido no card e na timeline.
- **Badges** novos nos cards: "Minimizado" e "SimConnect".
- Compatível com os perfis da v0.1.0: os campos novos têm valores padrão e perfis antigos continuam válidos (import/export incluídos).

## Capabilities

### New Capabilities

Nenhuma. As mudanças ampliam capabilities existentes.

### Modified Capabilities

- `launch-orchestration`: a abertura passa a ter duas etapas (itens comuns e itens que aguardam o SimConnect); a confirmação de execução passa a considerar a árvore de processos iniciada; novos requisitos de aprendizado do nome do processo, abertura minimizada e espera pelo SimConnect; os testes manuais não aguardam o SimConnect.
- `profile-management`: o item `app` ganha `processNameMode`, `startMinimized` e `waitForSimConnect`; o `processName` deixa de ser responsabilidade do usuário por padrão.
- `process-monitor`: os eventos de status de item passam a incluir `waitingSimConnect`.
- `app-interface`: o formulário de item ganha os checkboxes novos e a seção "Avançado"; os cards ganham os badges e o status novos.

## Impact

- **Rust**: `models.rs` (campos novos em `AppItem`, `ProcessNameMode`, status `WaitingSimConnect`), `launcher.rs` (rastreamento de processos, aprendizado, minimização), `monitor/runner.rs` (duas etapas de abertura), `processes.rs` (árvore de processos por PID pai e por pasta), `platform/windows/` (janela minimizada, minimizar janelas de um PID, detecção do named pipe do SimConnect, PID do processo elevado) e os stubs em `platform/fallback.rs`; persistência do nome aprendido pelo `AppState` com evento para a UI.
- **Frontend**: schemas Zod, `AppItemForm` (checkboxes e seção Avançado), `ItemCard`/`ItemStatusBadge` (badges e status), timeline, store de perfis (recebe o perfil atualizado pelo aprendizado) e traduções pt-BR/en.
- **Bindings** regenerados (`AppItem`, `ItemStatus`, `ProcessNameMode`).
- **Docs**: README (seções sobre nome do processo, abrir minimizado e SimConnect) e CHANGELOG da v0.2.0.
- **Dependência de ordem**: esta change altera requisitos criados pela `bootstrap-autostart-app`, que deve ser arquivada antes desta.
