## MODIFIED Requirements

### Requirement: Eventos para a interface
O monitor SHALL emitir eventos para o frontend a cada mudança de estado do monitor, a cada mudança de status de item (`pending`, `waitingSimConnect`, `launching`, `running`, `skipped`, `closing`, `closed`, `error`) e a cada entrada de log. O estado atual SHALL também estar disponível sob demanda.

#### Scenario: Janela aberta depois do início
- **WHEN** a janela é aberta a partir da bandeja no meio de uma sessão
- **THEN** a UI consulta o estado atual e exibe o status do monitor e de cada item corretamente

#### Scenario: Item aguardando o SimConnect
- **WHEN** um item passa a aguardar o SimConnect durante a sessão
- **THEN** a UI recebe o status `waitingSimConnect` do item sem precisar recarregar
