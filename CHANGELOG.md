# Changelog

Todas as mudanças relevantes do projeto são registradas aqui.

O formato segue o [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/) e o projeto usa [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [Não lançado]

## [0.2.0] - 2026-09-22

### Adicionado

- O nome do processo de cada app é detectado automaticamente, inclusive em apps que usam launcher; a configuração manual continua disponível em "Avançado".
- Opção "Abrir minimizado" por app.
- Opção "Aguardar o SimConnect" por app (MSFS 2020 e 2024): esses itens abrem depois dos demais, quando o simulador aceita conexões de addons.

### Alterado

- Todos os apps passam a ser iniciados pelo shell do Windows, o que permite acompanhar os processos iniciados e abrir minimizado.

## [0.1.0] - 2026-09-22

Primeira versão pública.

### Adicionado

- Perfis com gatilho (processo do simulador) e itens do tipo app ou URL, com presets para MSFS 2024, MSFS 2020 e X-Plane 12.
- Abertura em ordem com espera configurável, detecção de apps já abertos e confirmação do processo iniciado.
- Fechamento gracioso, forçado ou "manter aberto" por item, em paralelo e com tempo limite; opção de fechar só o que o AutoStart abriu.
- Suporte a launchers: o fechamento alcança todos os processos com o nome configurado.
- Execução de itens como administrador e reinício do AutoStart elevado.
- Botões para testar a abertura e o fechamento sem o simulador.
- Ícone na bandeja com menu de perfis, pausa do monitoramento, instância única e opção de iniciar com o Windows.
- Tela de logs com a linha do tempo da última sessão e logs técnicos em arquivo rotativo.
- Importação e exportação de perfis em JSON.
- Interface em português (Brasil) e inglês, com temas claro, escuro e do sistema.
- Assistente de primeira execução.
- Instalador por usuário (sem administrador) e atualização automática assinada.

[Não lançado]: https://github.com/mariopaglia/autostart/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/mariopaglia/autostart/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/mariopaglia/autostart/releases/tag/v0.1.0
