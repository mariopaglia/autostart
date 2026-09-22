## MODIFIED Requirements

### Requirement: Abertura ordenada com delay
Ao iniciar uma sessão, o sistema SHALL abrir os itens habilitados em duas etapas. Na primeira, SHALL processar, na ordem da lista, os itens que não aguardam o SimConnect. Na segunda, SHALL processar, na ordem da lista, os itens `app` com `waitForSimConnect = true`, somente depois que o SimConnect estiver disponível. Em ambas as etapas, o sistema SHALL aguardar o `delayMs` de cada item antes de abri-lo. Itens desabilitados SHALL ser ignorados sem gerar status.

#### Scenario: Três apps em sequência
- **WHEN** a sessão inicia com os itens A (delay 0), B (delay 800) e C (delay 2000), nenhum aguardando o SimConnect
- **THEN** A abre imediatamente, B cerca de 800 ms depois de A, e C cerca de 2000 ms depois de B

#### Scenario: Itens que aguardam o SimConnect vão para o fim
- **WHEN** a lista é A, B (aguarda o SimConnect), C e D (aguarda o SimConnect)
- **THEN** A e C abrem primeiro, nessa ordem, e B e D abrem depois, nessa ordem, quando o SimConnect estiver disponível

### Requirement: Execução do item
Um item `app` SHALL ser iniciado com seus `args` e `workingDir` (padrão: pasta do exe). Um item `url` SHALL ser aberto no navegador padrão. Falha ao iniciar SHALL marcar o item com status `error` e a mensagem da causa, sem interromper os itens seguintes.

#### Scenario: Executável removido
- **WHEN** o `exePath` de um item não existe mais
- **THEN** o item fica com status `error` ("executável não encontrado") e os próximos itens continuam sendo abertos

#### Scenario: Confirmação de execução
- **WHEN** um app é iniciado e, em até 10 segundos, o processo iniciado, um descendente dele ou um processo com o `processName` do item está em execução
- **THEN** o item passa para `running`; se nenhum desses processos existir no prazo, o item fica `error` com a mensagem "processo não detectado"

#### Scenario: Launcher que encerra logo após abrir o app
- **WHEN** o AutoStart inicia `Launcher.exe`, que abre `Volanta.exe` e encerra em 3 segundos
- **THEN** o item passa para `running`, pois um processo iniciado pelo launcher continua em execução

### Requirement: Testes manuais
O usuário SHALL poder executar "Testar abertura" e "Testar fechamento" para um perfil sem o simulador aberto, usando as mesmas regras da sessão real, com uma exceção: em "Testar abertura", os itens que aguardam o SimConnect SHALL abrir na segunda etapa sem aguardar o SimConnect, e a timeline SHALL registrar que a espera foi ignorada no teste. "Testar fechamento" SHALL fechar os itens abertos pelo último "Testar abertura" (ou, se não houver teste anterior, SHALL tratar todos os itens em execução como abertos pelo AutoStart, após confirmação do usuário). Os testes SHALL ficar indisponíveis enquanto o monitor estiver em `simRunning` ou `closing`.

#### Scenario: Testar abertura
- **WHEN** o usuário clica em "Testar abertura" com o monitor em `idle`
- **THEN** os itens habilitados abrem na ordem das duas etapas, com os status sendo atualizados nos cards

#### Scenario: Teste com item que aguarda o SimConnect
- **WHEN** o usuário testa a abertura de um perfil com um item que aguarda o SimConnect e o simulador está fechado
- **THEN** o item abre na segunda etapa sem esperar, e a timeline indica que a espera pelo SimConnect foi ignorada no teste

#### Scenario: Teste bloqueado durante voo
- **WHEN** o monitor está em `simRunning`
- **THEN** os botões de teste ficam desabilitados, com uma dica explicando o motivo

## ADDED Requirements

### Requirement: Aprendizado do nome do processo
Para itens `app` com `processNameMode = auto`, o sistema SHALL identificar qual processo representa o app depois de iniciá-lo e SHALL salvar esse nome no `processName` do item. Se o processo iniciado continuar em execução até o fim da janela de observação (30 segundos), o nome aprendido SHALL ser o dele. Se ele encerrar antes, o nome aprendido SHALL ser o de um processo que ele iniciou (direta ou indiretamente) e que continua rodando; na ausência de descendentes, SHALL ser o de um processo que surgiu após a abertura com executável dentro da pasta de instalação do item. Não havendo candidato, o `processName` SHALL permanecer inalterado. O nome aprendido SHALL valer imediatamente para o fechamento da sessão em andamento e SHALL ser persistido no perfil, refletindo na interface sem reiniciar. Itens com `processNameMode = manual` SHALL NOT ter o `processName` alterado.

#### Scenario: App sem launcher
- **WHEN** o AutoStart abre `C:\Apps\SPAD\Spad.exe` e ele continua rodando
- **THEN** o `processName` do item permanece `Spad.exe`

#### Scenario: Launcher que troca de processo
- **WHEN** o item aponta para `C:\Apps\Volanta\Launcher.exe` em modo automático, e o launcher abre `Volanta.exe` e encerra
- **THEN** o `processName` do item passa a ser `Volanta.exe`, o fechamento da sessão fecha o `Volanta.exe`, e o formulário do item mostra `Volanta.exe` como detectado automaticamente

#### Scenario: App aberto fora da árvore de processos
- **WHEN** o launcher em `C:\Apps\Foo\` encerra e o app real `C:\Apps\Foo\bin\Foo.exe` surge sem ser descendente dele
- **THEN** o `processName` do item passa a ser `Foo.exe`

#### Scenario: Modo manual preservado
- **WHEN** o item tem `processNameMode = manual` com `processName = Volanta.exe`
- **THEN** o sistema nunca altera o `processName` desse item

### Requirement: Abertura minimizada
Um item `app` com `startMinimized = true` SHALL ser iniciado com pedido ao Windows de janela minimizada, sem roubar o foco. Além disso, durante os primeiros 10 segundos após o item ficar `running`, o sistema SHALL minimizar as janelas visíveis de nível superior que surgirem nos processos do item. Apps que ignoram o pedido ou restauram a própria janela depois desse período SHALL NOT gerar erro.

#### Scenario: App minimizado
- **WHEN** a sessão abre um item com `startMinimized = true`
- **THEN** a janela do app fica minimizada na barra de tarefas e o foco permanece onde estava

#### Scenario: App que abre só na bandeja
- **WHEN** um item com `startMinimized = true` não cria janela visível
- **THEN** o item fica `running` normalmente, sem erro

### Requirement: Espera pelo SimConnect
Um item `app` com `waitForSimConnect = true` SHALL ser aberto somente quando o SimConnect do simulador estiver disponível para conexões. Enquanto espera, o item SHALL ter o status `waitingSimConnect`. O sistema SHALL verificar a disponibilidade a cada 2 segundos por até 10 minutos a partir do início da segunda etapa; esgotado o prazo, os itens ainda em espera SHALL receber o status `error` com a mensagem "SimConnect não ficou disponível". Se a sessão terminar durante a espera, os itens em espera SHALL NOT ser abertos. A opção SHALL ser respeitada apenas quando o gatilho do perfil for o MSFS 2020 (`FlightSimulator.exe`) ou o MSFS 2024 (`FlightSimulator2024.exe`); para outros gatilhos, o item SHALL abrir na segunda etapa sem aguardar.

#### Scenario: SimConnect fica disponível
- **WHEN** o MSFS 2024 inicia e o item Volanta tem `waitForSimConnect = true`
- **THEN** o card mostra "Aguardando SimConnect" e o Volanta abre assim que o SimConnect fica disponível

#### Scenario: SimConnect não fica disponível
- **WHEN** o SimConnect não fica disponível em 10 minutos
- **THEN** os itens em espera ficam `error` com "SimConnect não ficou disponível" e a timeline registra o erro

#### Scenario: Simulador fechado durante a espera
- **WHEN** o simulador é fechado enquanto um item aguarda o SimConnect
- **THEN** o item não é aberto e a sessão segue para o fechamento normalmente

#### Scenario: Gatilho que não é o MSFS
- **WHEN** o perfil tem gatilho `X-Plane.exe` e um item com `waitForSimConnect = true`
- **THEN** o item abre na segunda etapa sem aguardar o SimConnect
