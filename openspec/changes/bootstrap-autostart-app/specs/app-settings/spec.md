## Purpose

Define as configurações globais do AutoStart, seus valores padrão e como elas influenciam o comportamento do app.

## ADDED Requirements

### Requirement: Configurações e valores padrão
O sistema SHALL persistir as configurações `activeProfileId`, `startWithWindows` (padrão false), `startMinimized` (padrão true), `gracefulTimeoutMs` (padrão 5000, entre 1000 e 60000), `closeOnlyIfLaunchedByApp` (padrão true), `theme` (`system` | `light` | `dark`, padrão `dark`), `language` (`pt-BR` | `en`, padrão `pt-BR`), `onboardingCompleted` (padrão false) e `checkUpdatesOnStartup` (padrão true).

#### Scenario: Primeira execução
- **WHEN** o app inicia sem arquivo de configurações
- **THEN** as configurações são criadas com os valores padrão

#### Scenario: Campo novo em versão futura
- **WHEN** o arquivo de configurações salvo por uma versão anterior não tem um campo novo
- **THEN** o campo ausente assume o valor padrão e os demais são preservados

### Requirement: Validação das configurações
O sistema MUST rejeitar configurações fora dos limites (ex.: `gracefulTimeoutMs` < 1000) tanto na UI quanto no backend.

#### Scenario: Timeout inválido
- **WHEN** o usuário informa 200 ms de timeout gracioso
- **THEN** o campo exibe erro e o valor não é salvo

### Requirement: Aplicação imediata
Alterações de configuração SHALL entrar em vigor sem reiniciar o app (tema, idioma, timeout, autostart, regra de fechamento).

#### Scenario: Troca de idioma
- **WHEN** o usuário muda o idioma para `en`
- **THEN** a interface e o menu da bandeja passam a exibir textos em inglês imediatamente
