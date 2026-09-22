## MODIFIED Requirements

### Requirement: Cards de itens
Cada card SHALL exibir ícone (ou ícone genérico/globo para URL), nome, caminho ou URL resumido, badges (Admin, Minimizado, SimConnect, delay em segundos, comportamento ao fechar), status da sessão atual quando houver (incluindo "Aguardando SimConnect") e um toggle de habilitado. Os cards SHALL poder ser reordenados por arrastar e soltar (mouse e teclado). Clicar no card SHALL abrir a edição.

#### Scenario: Toggle desabilita
- **WHEN** o usuário desliga o toggle de um card
- **THEN** o card fica visualmente esmaecido e o item é ignorado na próxima abertura

#### Scenario: Reordenar via teclado
- **WHEN** o usuário foca a alça de arraste, pressiona espaço e usa as setas
- **THEN** o card muda de posição e a ordem é persistida

#### Scenario: Badges das opções de abertura
- **WHEN** um item tem `startMinimized = true` e `waitForSimConnect = true`
- **THEN** o card exibe os badges "Minimizado" e "SimConnect"

### Requirement: Formulário de item
O formulário SHALL permitir adicionar/editar um item `app`, escolhendo o `.exe` pelo diálogo nativo (preenchendo nome, ícone e processo automaticamente), ou um item `url`. Campos principais de app: nome, caminho, argumentos, delay, executar como admin, abrir minimizado, aguardar o SimConnect e comportamento ao fechar. Uma seção "Avançado", recolhida por padrão, SHALL conter a pasta de trabalho e o nome do processo, indicando quando o nome é detectado automaticamente e oferecendo a ação de voltar ao modo automático quando estiver em modo manual. A opção "Aguardar o SimConnect" SHALL ficar desativada, com uma dica explicando o motivo, quando o gatilho do perfil não for o MSFS 2020 ou 2024. Campos de URL: nome, URL, delay. A validação SHALL usar os mesmos schemas da persistência e exibir erros por campo.

#### Scenario: Adicionar app pelo diálogo
- **WHEN** o usuário clica em "Adicionar app" e seleciona um `.exe`
- **THEN** nome, ícone e processo são preenchidos, e o usuário pode salvar sem abrir a seção Avançado

#### Scenario: Nome detectado automaticamente
- **WHEN** o usuário abre a seção Avançado de um item em modo automático
- **THEN** o campo do nome do processo mostra o nome atual com a indicação "detectado automaticamente"

#### Scenario: Voltar ao modo automático
- **WHEN** o item está em modo manual e o usuário clica em "Detectar automaticamente"
- **THEN** o item volta para `processNameMode = auto` e o nome passa a ser mantido pelo sistema

#### Scenario: SimConnect indisponível para o gatilho
- **WHEN** o perfil tem gatilho `X-Plane.exe` e o usuário abre o formulário de um app
- **THEN** a opção "Aguardar o SimConnect" aparece desativada com a dica de que ela só vale para o MSFS 2020/2024
