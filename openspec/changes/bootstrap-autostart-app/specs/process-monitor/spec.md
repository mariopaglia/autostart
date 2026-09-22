## Purpose

Observa continuamente os processos do sistema para detectar quando o simulador (processo gatilho do perfil ativo) inicia ou encerra, disparando a abertura e o fechamento dos itens.

## ADDED Requirements

### Requirement: Detecção do gatilho por nome
O monitor SHALL verificar a cada 2 segundos se existe algum processo cujo nome de executável corresponda ao `trigger.processName` do perfil ativo, com comparação sem diferenciar maiúsculas de minúsculas.

#### Scenario: Simulador inicia
- **WHEN** o estado é `idle` e surge um processo `flightsimulator2024.exe`, com gatilho configurado como `FlightSimulator2024.exe`
- **THEN** em até 2 segundos o estado muda para `simRunning` e a abertura dos itens começa

### Requirement: Máquina de estados
O monitor SHALL operar com os estados `idle`, `simRunning`, `closing` e `paused`, com as transições: `idle → simRunning` (gatilho detectado), `simRunning → closing` (gatilho ausente em 2 verificações consecutivas), `closing → idle` (fechamento concluído), qualquer estado → `paused` (usuário pausa) e `paused → idle` (usuário retoma). Enquanto estiver em `closing`, o monitor SHALL NOT iniciar nova abertura.

#### Scenario: Simulador encerra
- **WHEN** o estado é `simRunning` e o gatilho está ausente em duas verificações seguidas
- **THEN** o estado muda para `closing`, os itens são fechados conforme as regras e o estado volta para `idle`

#### Scenario: Oscilação momentânea
- **WHEN** o gatilho some em uma verificação e reaparece na seguinte
- **THEN** o estado permanece `simRunning` e nada é fechado

#### Scenario: Simulador reaberto durante o fechamento
- **WHEN** o gatilho reaparece enquanto o estado é `closing`
- **THEN** o fechamento termina, o estado vai para `idle` e, na verificação seguinte, entra em `simRunning` e abre os itens novamente

### Requirement: Pausa do monitoramento
O usuário SHALL poder pausar e retomar o monitoramento pela bandeja e pela UI. Pausar durante `simRunning` SHALL NOT fechar os itens abertos, e a sessão em andamento SHALL ser descartada.

#### Scenario: Pausar com simulador aberto
- **WHEN** o usuário pausa com o simulador aberto e depois fecha o simulador
- **THEN** nenhum item é fechado

### Requirement: App iniciado com o simulador já aberto
Se o gatilho já estiver em execução quando o AutoStart iniciar (ou quando o monitoramento for retomado), o monitor SHALL entrar em `simRunning` e executar a abertura normalmente, com itens já em execução marcados como preexistentes.

#### Scenario: AutoStart aberto depois do simulador
- **WHEN** o AutoStart inicia com o MSFS já rodando e o Volanta já aberto
- **THEN** o Volanta é marcado como `skipped` (preexistente) e os demais itens habilitados são abertos

### Requirement: Snapshot do perfil na sessão
A sessão SHALL usar uma cópia do perfil ativo tirada no momento da transição para `simRunning`. Edições no perfil ou a troca de perfil ativo durante a sessão SHALL NOT afetar o fechamento dessa sessão.

#### Scenario: Troca de perfil durante o voo
- **WHEN** o usuário troca o perfil ativo com o simulador aberto
- **THEN** ao fechar o simulador, são fechados os itens do perfil que abriu a sessão

### Requirement: Eventos para a interface
O monitor SHALL emitir eventos para o frontend a cada mudança de estado do monitor, a cada mudança de status de item (`pending`, `launching`, `running`, `skipped`, `closing`, `closed`, `error`) e a cada entrada de log. O estado atual SHALL também estar disponível sob demanda.

#### Scenario: Janela aberta depois do início
- **WHEN** a janela é aberta a partir da bandeja no meio de uma sessão
- **THEN** a UI consulta o estado atual e exibe o status do monitor e de cada item corretamente
