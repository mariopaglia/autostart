## Purpose

Reduz o trabalho manual ao cadastrar itens e gatilhos: extrai automaticamente dados do executável escolhido e lista os processos em execução para seleção.

## ADDED Requirements

### Requirement: Inspecionar executável
Dado o caminho de um `.exe`, o sistema SHALL retornar `processName` (nome do arquivo), `productName` (da informação de versão do arquivo, com fallback para `FileDescription` e depois para o nome do arquivo sem extensão) e `iconBase64` (PNG 32x32 ou maior em base64, opcional).

#### Scenario: Executável com metadados
- **WHEN** o usuário seleciona `C:\Program Files\Navigraph\Charts\Navigraph Charts.exe`
- **THEN** o formulário é preenchido com nome "Navigraph Charts", processo `Navigraph Charts.exe` e o ícone do app

#### Scenario: Executável sem ícone ou versão
- **WHEN** o exe não tem ícone nem recurso de versão
- **THEN** o nome vem do arquivo, `iconBase64` fica vazio e o card mostra um ícone genérico

#### Scenario: Arquivo inválido
- **WHEN** o caminho não existe ou não é um `.exe`
- **THEN** o comando retorna erro tipado e o formulário exibe a mensagem

### Requirement: Listar processos em execução
O sistema SHALL listar os processos em execução com nome e caminho do executável (quando acessível), sem duplicatas por nome e em ordem alfabética, para o usuário escolher o processo gatilho.

#### Scenario: Escolher gatilho da lista
- **WHEN** o usuário abre o seletor de gatilho com o X-Plane aberto e busca "x-plane"
- **THEN** `X-Plane.exe` aparece na lista e, ao selecioná-lo, vira o `processName` do gatilho

### Requirement: Presets de gatilho
O sistema SHALL oferecer os presets MSFS 2024 (`FlightSimulator2024.exe`), MSFS 2020 (`FlightSimulator.exe`), X-Plane 12 (`X-Plane.exe`) e "processo personalizado" (digitado ou escolhido na lista de processos).

#### Scenario: Selecionar preset
- **WHEN** o usuário escolhe o preset "MSFS 2020"
- **THEN** o gatilho vira `{ processName: "FlightSimulator.exe", label: "MSFS 2020" }`
