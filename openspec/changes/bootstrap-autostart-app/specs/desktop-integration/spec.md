## Purpose

Faz o AutoStart se comportar como um utilitário de bandeja discreto: sempre disponível, com uma única instância, iniciando junto com o Windows se o usuário quiser.

## ADDED Requirements

### Requirement: Ícone na bandeja por estado
O app SHALL exibir um ícone na bandeja do sistema com variantes visuais distintas para `idle`, `simRunning`/`closing` e `paused`, e um tooltip com o nome do app, o perfil ativo e o estado atual.

#### Scenario: Simulador abre
- **WHEN** o monitor entra em `simRunning`
- **THEN** o ícone da bandeja muda para a variante "rodando" e o tooltip indica "Simulador em execução"

### Requirement: Menu da bandeja
O menu da bandeja SHALL conter: um submenu com os perfis (o ativo marcado) que permite trocar o perfil ativo, "Pausar monitoramento"/"Retomar monitoramento", "Abrir AutoStart" e "Sair". Os textos SHALL seguir o idioma configurado, e o menu SHALL refletir mudanças de perfis sem reiniciar o app.

#### Scenario: Trocar perfil pela bandeja
- **WHEN** o usuário escolhe outro perfil no submenu
- **THEN** ele vira o ativo, e a UI (se aberta) reflete a troca

#### Scenario: Clique no ícone
- **WHEN** o usuário clica com o botão esquerdo no ícone da bandeja
- **THEN** a janela principal é exibida e focada

### Requirement: Fechar janela minimiza para a bandeja
Fechar a janela principal SHALL escondê-la sem encerrar o app. Apenas "Sair" no menu da bandeja SHALL encerrar o processo. Sair durante `simRunning` SHALL NOT fechar os itens abertos.

#### Scenario: Fechar no X
- **WHEN** o usuário clica no X da janela
- **THEN** a janela some, o ícone continua na bandeja e o monitor segue ativo

### Requirement: Instância única
Iniciar o AutoStart quando ele já está em execução SHALL apenas exibir e focar a janela da instância existente.

#### Scenario: Duplo clique no atalho
- **WHEN** o usuário abre o AutoStart pelo menu Iniciar com o app já rodando na bandeja
- **THEN** nenhuma nova instância fica rodando e a janela existente é exibida em primeiro plano

### Requirement: Iniciar com o Windows
Quando `startWithWindows` estiver ativo, o app SHALL ser registrado para iniciar no login do usuário (sem elevação); quando desativado, o registro SHALL ser removido. O estado exibido SHALL refletir o registro real do sistema.

#### Scenario: Ativar autostart
- **WHEN** o usuário ativa "Iniciar com o Windows" e reinicia o computador
- **THEN** o AutoStart inicia no login, conforme a configuração `startMinimized`

### Requirement: Iniciar minimizado
Quando `startMinimized` estiver ativo, o app SHALL iniciar apenas na bandeja, sem exibir a janela. Na primeira execução (onboarding pendente), a janela SHALL ser exibida independentemente dessa configuração.

#### Scenario: Início minimizado
- **WHEN** o app inicia com `startMinimized = true` e o onboarding concluído
- **THEN** somente o ícone da bandeja aparece

### Requirement: Janela principal
A janela SHALL ser redimensionável, com tamanho inicial de 1000x680, tamanho mínimo de 800x560 e título "AutoStart".

#### Scenario: Redimensionar
- **WHEN** o usuário tenta reduzir a janela abaixo de 800x560
- **THEN** a janela para no tamanho mínimo
