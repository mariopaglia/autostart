## MODIFIED Requirements

### Requirement: Itens do tipo app
Um item `app` SHALL conter `id`, `name`, `exePath`, `args` opcional, `workingDir` opcional, `processName`, `processNameMode` (`auto` | `manual`, padrão `auto`), `iconBase64` opcional, `delayMs` (padrão 800, entre 0 e 60000), `runAsAdmin` (padrão false), `startMinimized` (padrão false), `waitForSimConnect` (padrão false), `onClose` (`graceful` | `force` | `keep`, padrão `graceful`) e `enabled` (padrão true). O `processName` SHALL ser preenchido inicialmente com o nome do executável e, no modo `auto`, SHALL ser mantido pelo sistema. O usuário SHALL poder editá-lo, o que muda o modo para `manual`, e SHALL poder voltar ao modo `auto`. Itens salvos por versões anteriores, sem os campos novos, SHALL ser carregados com os valores padrão, exceto o `processNameMode` de itens salvos localmente, que SHALL ser `manual` quando o `processName` diferir do nome do executável (sem diferenciar maiúsculas), pois nesse caso o nome foi definido pelo usuário.

#### Scenario: processName divergente do exe
- **WHEN** o usuário adiciona `C:\Apps\Volanta\Launcher.exe` e altera o `processName` para `Volanta.exe`
- **THEN** o item passa para `processNameMode = manual`, e a detecção de preexistência e o fechamento usam `Volanta.exe`

#### Scenario: Novo item em modo automático
- **WHEN** o usuário adiciona `C:\Apps\SPAD\Spad.exe` sem abrir a seção Avançado
- **THEN** o item é salvo com `processName = Spad.exe` e `processNameMode = auto`

#### Scenario: Perfil da versão anterior
- **WHEN** o app carrega um perfil salvo pela v0.1.0, cujos itens não têm `processNameMode`, `startMinimized` nem `waitForSimConnect`
- **THEN** os itens são carregados com `startMinimized = false` e `waitForSimConnect = false`, sem perda dos demais campos

#### Scenario: Nome definido pelo usuário na versão anterior
- **WHEN** um item salvo pela v0.1.0 aponta para `Launcher.exe` com `processName = Volanta.exe`
- **THEN** o item é carregado com `processNameMode = manual`, e o AutoStart não altera esse nome

#### Scenario: Nome igual ao executável na versão anterior
- **WHEN** um item salvo pela v0.1.0 aponta para `Spad.exe` com `processName = Spad.exe`
- **THEN** o item é carregado com `processNameMode = auto`

#### Scenario: Importar perfil antigo
- **WHEN** o usuário importa um arquivo exportado pela v0.1.0
- **THEN** a importação é aceita e os campos novos assumem os valores padrão
