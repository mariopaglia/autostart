## Purpose

Permite ao piloto organizar seus programas auxiliares em perfis, cada um associado a um processo gatilho (simulador), com uma lista ordenada de apps e URLs a abrir e fechar.

## ADDED Requirements

### Requirement: Estrutura de perfil
O sistema SHALL armazenar perfis com `id` (UUID), `name` (não vazio), `trigger` (`processName` terminando em `.exe` e `label`), `items` (lista ordenada de itens) e `enabled` (boolean).

#### Scenario: Criar perfil
- **WHEN** o usuário cria um perfil chamado "Live MSFS 2024" com gatilho `FlightSimulator2024.exe`
- **THEN** o perfil é persistido com um UUID novo, `enabled = true` e lista de itens vazia

#### Scenario: Nome inválido
- **WHEN** o usuário tenta salvar um perfil com nome vazio ou só com espaços
- **THEN** o sistema rejeita o salvamento e exibe erro de validação no campo

### Requirement: Itens do tipo app
Um item `app` SHALL conter `id`, `name`, `exePath`, `args` opcional, `workingDir` opcional, `processName`, `iconBase64` opcional, `delayMs` (padrão 800, entre 0 e 60000), `runAsAdmin` (padrão false), `onClose` (`graceful` | `force` | `keep`, padrão `graceful`) e `enabled` (padrão true). O `processName` SHALL ser editável pelo usuário para cobrir apps cujo launcher dispara um processo com outro nome.

#### Scenario: processName divergente do exe
- **WHEN** o usuário adiciona `C:\Apps\Volanta\Launcher.exe` e altera o `processName` para `Volanta.exe`
- **THEN** a detecção de preexistência e o fechamento usam `Volanta.exe`

### Requirement: Itens do tipo URL
Um item `url` SHALL conter `id`, `name`, `url` (apenas `http` ou `https`), `delayMs` e `enabled`. Itens URL SHALL NOT ser fechados pelo sistema.

#### Scenario: URL inválida
- **WHEN** o usuário informa `ftp://exemplo.com` ou texto que não é URL
- **THEN** o formulário rejeita o valor com mensagem de validação

### Requirement: Operações sobre perfis
O sistema SHALL permitir criar, renomear, duplicar e excluir perfis, além de reordenar, habilitar/desabilitar, editar e remover itens. Duplicar SHALL gerar novos UUIDs para o perfil e todos os itens e adicionar o sufixo " (cópia)"/" (copy)" conforme o idioma.

#### Scenario: Reordenar itens
- **WHEN** o usuário arrasta o terceiro item para a primeira posição
- **THEN** a nova ordem é persistida e passa a ser usada na próxima abertura

#### Scenario: Excluir o perfil ativo
- **WHEN** o usuário exclui o perfil ativo e existem outros perfis
- **THEN** o primeiro perfil restante vira o ativo

#### Scenario: Excluir o último perfil
- **WHEN** o usuário tenta excluir o único perfil existente
- **THEN** o sistema impede a exclusão e informa que deve existir ao menos um perfil

### Requirement: Perfil ativo
Exatamente um perfil SHALL estar ativo por vez, e apenas o perfil ativo é considerado pelo monitor. Um perfil ativo com `enabled = false` SHALL fazer o monitor ignorar o gatilho.

#### Scenario: Trocar perfil ativo
- **WHEN** o usuário seleciona outro perfil como ativo (pela UI ou pela bandeja)
- **THEN** `activeProfileId` é persistido e o monitor passa a observar o gatilho do novo perfil

### Requirement: Persistência durável
Perfis e configurações SHALL ser persistidos em arquivos JSON no diretório de dados do app, com campo de versão de schema e escrita atômica, de modo que uma queda durante a gravação não corrompa o arquivo anterior.

#### Scenario: Reinício do app
- **WHEN** o app é encerrado e aberto novamente
- **THEN** perfis, ordem dos itens, perfil ativo e configurações são restaurados sem alterações

#### Scenario: Arquivo corrompido
- **WHEN** o arquivo de perfis existe mas não é um JSON válido
- **THEN** o sistema renomeia o arquivo para `.corrupt-<timestamp>`, inicia com o perfil de exemplo e registra um erro no log

### Requirement: Exportar e importar perfil
O sistema SHALL exportar um perfil para um arquivo `.json` escolhido pelo usuário e importar perfis de arquivos `.json` validados pelo mesmo schema. A importação SHALL gerar novos UUIDs e SHALL NOT sobrescrever perfis existentes.

#### Scenario: Importação válida
- **WHEN** o usuário importa um arquivo exportado por outro piloto
- **THEN** um novo perfil aparece na sidebar com os itens do arquivo

#### Scenario: Importação inválida
- **WHEN** o arquivo não segue o schema (campo obrigatório ausente, tipo errado)
- **THEN** nada é salvo e o usuário vê uma mensagem indicando o campo inválido

#### Scenario: Caminhos inexistentes após importar
- **WHEN** um item importado aponta para um `exePath` que não existe nesta máquina
- **THEN** o card do item exibe um aviso de "executável não encontrado"
