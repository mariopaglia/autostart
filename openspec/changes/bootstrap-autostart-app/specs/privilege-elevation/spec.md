## Purpose

Suporta apps auxiliares que precisam rodar como administrador e deixa claro ao usuário quando o AutoStart precisa estar elevado para conseguir fechá-los.

## ADDED Requirements

### Requirement: Executar item como administrador
Um item com `runAsAdmin = true` SHALL ser iniciado com solicitação de elevação (prompt do UAC). Se o usuário negar o UAC, o item SHALL ficar com status `error` e a mensagem "elevação negada".

#### Scenario: UAC aceito
- **WHEN** a sessão abre um item com `runAsAdmin = true` e o usuário aceita o UAC
- **THEN** o app roda elevado e o item fica `running`

#### Scenario: UAC negado
- **WHEN** o usuário nega o UAC
- **THEN** o item fica `error` e os itens seguintes continuam sendo abertos

### Requirement: Detectar elevação do AutoStart
O sistema SHALL informar se o próprio AutoStart está rodando elevado. Quando o perfil ativo tiver um item habilitado com `runAsAdmin = true`, fechamento diferente de `keep` e o AutoStart não estiver elevado, a UI SHALL exibir um aviso persistente explicando que esse app pode não ser fechado, com a ação "Reiniciar como administrador". O AutoStart SHALL NOT exigir elevação por padrão.

#### Scenario: Aviso exibido
- **WHEN** o perfil ativo tem o SPAD.neXt com `runAsAdmin = true` e `onClose = graceful`, e o AutoStart não está elevado
- **THEN** a tela principal exibe o banner de aviso com o botão "Reiniciar como administrador"

### Requirement: Reiniciar como administrador
A ação "Reiniciar como administrador" SHALL abrir uma nova instância elevada (via UAC) e encerrar a instância atual somente depois que a nova for aceita. Se o UAC for negado, a instância atual SHALL continuar rodando e informar a falha.

#### Scenario: Reinício aceito
- **WHEN** o usuário clica em "Reiniciar como administrador" e aceita o UAC
- **THEN** a instância antiga encerra, a nova abre elevada e o aviso desaparece

#### Scenario: Reinício negado
- **WHEN** o usuário nega o UAC
- **THEN** a instância atual continua aberta e exibe "elevação cancelada"

### Requirement: Erro de permissão no fechamento
Se o encerramento de um processo falhar por falta de permissão, o item SHALL ficar com status `error` e a mensagem SHALL indicar que é necessário executar o AutoStart como administrador.

#### Scenario: Acesso negado ao fechar
- **WHEN** o AutoStart não elevado tenta encerrar um processo elevado
- **THEN** o item fica `error` com a mensagem "acesso negado: execute o AutoStart como administrador"
