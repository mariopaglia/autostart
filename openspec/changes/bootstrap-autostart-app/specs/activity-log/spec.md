## Purpose

Dá visibilidade ao que o AutoStart fez em cada sessão e mantém um histórico técnico em disco para diagnóstico de problemas relatados pela comunidade.

## ADDED Requirements

### Requirement: Timeline da última sessão
O sistema SHALL registrar, para a sessão atual ou a mais recente (real ou de teste), uma timeline de eventos com horário, item, tipo (sessão iniciada, aberto, pulado, fechado graciosamente, encerrado à força, mantido, erro, sessão encerrada) e mensagem. A timeline da última sessão SHALL sobreviver a um reinício do app.

#### Scenario: Visualizar sessão
- **WHEN** o usuário abre a tela de Logs depois de um voo
- **THEN** vê em ordem cronológica o que abriu, o que foi pulado, o que fechou e os erros

#### Scenario: Após reiniciar o app
- **WHEN** o AutoStart é reiniciado depois de uma sessão
- **THEN** a tela de Logs ainda exibe a timeline daquela sessão

#### Scenario: Atualização ao vivo
- **WHEN** a tela de Logs está aberta durante uma sessão
- **THEN** novos eventos aparecem sem recarregar

### Requirement: Log em arquivo rotativo
O sistema SHALL gravar logs técnicos em arquivo no diretório de logs do app, com rotação por tamanho (máx. 5 MB por arquivo, mantendo os 5 arquivos mais recentes). A tela de configurações SHALL oferecer a ação de abrir a pasta de logs.

#### Scenario: Abrir pasta de logs
- **WHEN** o usuário clica em "Abrir pasta de logs"
- **THEN** o Explorer abre a pasta que contém os arquivos `.log`

### Requirement: Sem dados sensíveis
Os logs SHALL NOT conter conteúdo de argumentos marcados como sensíveis; o sistema SHALL registrar caminhos e nomes de processos, mas SHALL omitir os `args` dos itens nos arquivos de log.

#### Scenario: Args com token
- **WHEN** um item tem `args = "--token abc123"`
- **THEN** o arquivo de log registra a abertura do item sem o valor dos argumentos
