## Purpose

Abre os itens do perfil na ordem configurada quando a sessão começa e fecha, ao fim da sessão, apenas o que deve ser fechado, identificando processos por nome para lidar com launchers e updaters.

## ADDED Requirements

### Requirement: Abertura ordenada com delay
Ao iniciar uma sessão, o sistema SHALL processar os itens habilitados do perfil na ordem da lista, aguardando o `delayMs` de cada item antes de abri-lo. Itens desabilitados SHALL ser ignorados sem gerar status.

#### Scenario: Três apps em sequência
- **WHEN** a sessão inicia com os itens A (delay 0), B (delay 800) e C (delay 2000)
- **THEN** A abre imediatamente, B cerca de 800 ms depois de A, e C cerca de 2000 ms depois de B

### Requirement: Detecção de app preexistente
Antes de abrir um item `app`, o sistema SHALL verificar se já existe processo com o `processName` do item. Se existir, o item SHALL NOT ser aberto e SHALL receber o status `skipped` com o motivo "preexistente".

#### Scenario: App já aberto
- **WHEN** o Navigraph Charts já está em execução no início da sessão
- **THEN** ele não é aberto de novo e o card mostra "Já estava aberto"

### Requirement: Execução do item
Um item `app` SHALL ser iniciado com seus `args` e `workingDir` (padrão: pasta do exe). Um item `url` SHALL ser aberto no navegador padrão. Falha ao iniciar SHALL marcar o item com status `error` e a mensagem da causa, sem interromper os itens seguintes.

#### Scenario: Executável removido
- **WHEN** o `exePath` de um item não existe mais
- **THEN** o item fica com status `error` ("executável não encontrado") e os próximos itens continuam sendo abertos

#### Scenario: Confirmação de execução
- **WHEN** um app é iniciado e, em até 10 segundos, surge um processo com o `processName` do item
- **THEN** o item passa para `running`; se nenhum processo surgir no prazo, o item fica `error` com a mensagem "processo não detectado"

### Requirement: Registro dos itens abertos pelo AutoStart
A sessão SHALL registrar quais itens foram abertos pelo AutoStart e quais eram preexistentes.

#### Scenario: Consultar sessão
- **WHEN** a sessão está em andamento
- **THEN** o estado exposto informa, por item, se foi aberto pelo AutoStart ou era preexistente

### Requirement: Regras de fechamento
Ao fim da sessão, o sistema SHALL fechar os itens `app` com `onClose` diferente de `keep`. Quando `closeOnlyIfLaunchedByApp` estiver ativo, SHALL fechar somente os itens abertos pelo AutoStart na sessão. O fechamento SHALL alcançar todos os processos com o `processName` do item, não apenas o PID originalmente iniciado.

#### Scenario: Launcher que troca de processo
- **WHEN** o AutoStart iniciou `Launcher.exe`, que encerrou e deixou `Volanta.exe` rodando (processName do item = `Volanta.exe`)
- **THEN** no fim da sessão o `Volanta.exe` é fechado

#### Scenario: App preexistente preservado
- **WHEN** `closeOnlyIfLaunchedByApp` está ativo e o SPAD.neXt já estava aberto antes do simulador
- **THEN** o SPAD.neXt continua aberto após o simulador fechar

#### Scenario: Item keep
- **WHEN** um item tem `onClose = keep`
- **THEN** ele nunca é fechado pelo AutoStart

### Requirement: Fechamento gracioso
Para `onClose = graceful`, o sistema SHALL pedir o fechamento de todas as janelas de nível superior dos processos do item e aguardar até `gracefulTimeoutMs`. Se algum processo ainda existir após o prazo, SHALL encerrá-lo à força. O status final SHALL ser `closed`, ou `error` se o encerramento falhar.

#### Scenario: App fecha sozinho
- **WHEN** o app responde ao pedido de fechamento em 1 segundo
- **THEN** o item fica `closed` sem encerramento forçado, e o log registra "fechado graciosamente"

#### Scenario: App trava no fechamento
- **WHEN** o app exibe um diálogo "Deseja salvar?" e não encerra em `gracefulTimeoutMs`
- **THEN** o processo é encerrado à força e o log registra "encerrado à força após timeout"

### Requirement: Fechamento forçado
Para `onClose = force`, o sistema SHALL encerrar imediatamente todos os processos com o `processName` do item.

#### Scenario: Force
- **WHEN** a sessão termina com um item `force` em execução
- **THEN** o processo é encerrado sem aguardar e o item fica `closed`

### Requirement: Fechamento em paralelo com limite de tempo
Os itens SHALL ser fechados em paralelo, e o estado `closing` SHALL terminar em no máximo `gracefulTimeoutMs` + 5 segundos.

#### Scenario: Vários apps graciosos
- **WHEN** cinco apps graciosos são fechados com timeout de 5000 ms
- **THEN** o monitor volta para `idle` em até 10 segundos

### Requirement: Testes manuais
O usuário SHALL poder executar "Testar abertura" e "Testar fechamento" para um perfil sem o simulador aberto, usando exatamente as mesmas regras da sessão real. "Testar fechamento" SHALL fechar os itens abertos pelo último "Testar abertura" (ou, se não houver teste anterior, SHALL tratar todos os itens em execução como abertos pelo AutoStart, após confirmação do usuário). Os testes SHALL ficar indisponíveis enquanto o monitor estiver em `simRunning` ou `closing`.

#### Scenario: Testar abertura
- **WHEN** o usuário clica em "Testar abertura" com o monitor em `idle`
- **THEN** os itens habilitados abrem na ordem, com os status sendo atualizados nos cards

#### Scenario: Teste bloqueado durante voo
- **WHEN** o monitor está em `simRunning`
- **THEN** os botões de teste ficam desabilitados, com uma dica explicando o motivo
