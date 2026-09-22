## Purpose

Define as telas e interações da interface do AutoStart: visual moderno estilo stream deck, gerenciamento visual de perfis e itens, onboarding, idiomas e temas.

## ADDED Requirements

### Requirement: Tela principal
A tela principal SHALL conter: uma sidebar com a lista de perfis (criar, renomear, duplicar, excluir, exportar, importar, marcar como ativo); uma barra superior com o status do monitor (Aguardando simulador / Simulador rodando / Fechando / Pausado), o seletor de gatilho do perfil exibido e os botões "Testar abertura" e "Testar fechamento"; e uma área central com os itens do perfil em cards.

#### Scenario: Status ao vivo
- **WHEN** o monitor muda de estado
- **THEN** a barra superior é atualizada sem recarregar a tela

### Requirement: Cards de itens
Cada card SHALL exibir ícone (ou ícone genérico/globo para URL), nome, caminho ou URL resumido, badges (Admin, delay em segundos, comportamento ao fechar), status da sessão atual quando houver e um toggle de habilitado. Os cards SHALL poder ser reordenados por arrastar e soltar (mouse e teclado). Clicar no card SHALL abrir a edição.

#### Scenario: Toggle desabilita
- **WHEN** o usuário desliga o toggle de um card
- **THEN** o card fica visualmente esmaecido e o item é ignorado na próxima abertura

#### Scenario: Reordenar via teclado
- **WHEN** o usuário foca a alça de arraste, pressiona espaço e usa as setas
- **THEN** o card muda de posição e a ordem é persistida

### Requirement: Estado vazio
Um perfil sem itens SHALL exibir um estado vazio com as ações "Adicionar app" e "Adicionar URL".

#### Scenario: Perfil novo
- **WHEN** o usuário cria um perfil
- **THEN** a área central mostra o estado vazio com as duas ações

### Requirement: Formulário de item
O formulário SHALL permitir adicionar/editar um item `app`, escolhendo o `.exe` pelo diálogo nativo (preenchendo nome, ícone e processo automaticamente), ou um item `url`. Campos de app: nome, caminho, argumentos, pasta de trabalho, nome do processo, delay, executar como admin, comportamento ao fechar. Campos de URL: nome, URL, delay. A validação SHALL usar os mesmos schemas da persistência e exibir erros por campo.

#### Scenario: Adicionar app pelo diálogo
- **WHEN** o usuário clica em "Adicionar app" e seleciona um `.exe`
- **THEN** nome, ícone e processo são preenchidos e podem ser ajustados antes de salvar

### Requirement: Tela de configurações
A tela de configurações SHALL expor: iniciar com o Windows, iniciar minimizado, timeout do fechamento gracioso, fechar apenas o que o AutoStart abriu, tema, idioma, verificar atualizações (automática ao iniciar e botão manual), abrir pasta de logs e seção "Sobre" (versão, link do repositório, licença).

#### Scenario: Alterar tema
- **WHEN** o usuário escolhe o tema `light`
- **THEN** a interface muda para o tema claro imediatamente

### Requirement: Internacionalização
Todos os textos visíveis ao usuário (UI, bandeja, notificações, mensagens de erro exibidas) SHALL vir de arquivos de tradução, com pt-BR como padrão e en como alternativa. Chaves sem tradução em en SHALL cair para pt-BR.

#### Scenario: Idioma inglês
- **WHEN** o idioma é `en`
- **THEN** nenhum texto em português aparece na UI

### Requirement: Temas
O app SHALL suportar os temas `dark` (padrão), `light` e `system`, este último seguindo o tema do Windows em tempo real.

#### Scenario: Tema do sistema
- **WHEN** o tema é `system` e o Windows muda para o modo claro
- **THEN** o app muda para o tema claro sem reiniciar

### Requirement: Onboarding
Na primeira execução, o sistema SHALL criar um perfil de exemplo "MSFS 2024" com gatilho `FlightSimulator2024.exe` e exibir um wizard curto (3 a 4 passos) explicando: como funciona o gatilho, a escolha do preset do simulador (MSFS 2024, MSFS 2020, X-Plane 12, personalizado), como adicionar apps e as opções de iniciar com o Windows. Concluir ou pular o wizard SHALL marcar `onboardingCompleted = true`.

#### Scenario: Primeira execução
- **WHEN** o app é aberto pela primeira vez
- **THEN** o wizard aparece sobre a tela principal e o perfil de exemplo existe

#### Scenario: Escolha de preset no wizard
- **WHEN** o usuário escolhe "X-Plane 12" no wizard
- **THEN** o gatilho do perfil de exemplo passa a ser `X-Plane.exe` e o nome vira "X-Plane 12"

#### Scenario: Execuções seguintes
- **WHEN** o app é aberto depois do onboarding concluído
- **THEN** o wizard não aparece

### Requirement: Visual e acessibilidade
A interface SHALL usar cantos arredondados, cards com ícone, animações sutis que respeitam `prefers-reduced-motion`, foco visível em todos os controles interativos e contraste AA nos dois temas.

#### Scenario: Movimento reduzido
- **WHEN** o Windows está com animações desativadas
- **THEN** as transições da UI são desativadas
