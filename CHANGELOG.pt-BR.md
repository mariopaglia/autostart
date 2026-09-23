# Changelog

Todas as mudanças relevantes do projeto são documentadas neste arquivo. Esta é a tradução do [CHANGELOG.md](CHANGELOG.md), usada nas notas de atualização exibidas no app em português.

O formato segue o [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/) e o projeto adota o [Versionamento Semântico](https://semver.org/lang/pt-BR/spec/v2.0.0.html).

## [Unreleased]

### Adicionado

- "Ver novidades", ao lado da versão em Configurações → Sobre, mostra o histórico completo de versões, no idioma do app e sem precisar de internet.

### Corrigido

- A janela de atualização agora mostra as novidades formatadas (títulos, listas, negrito) em vez do Markdown cru, e no idioma do app.

## [0.3.1] - 2026-09-23

### Alterado

- Configurações de fechamento mais claras: "Esperar após o simulador fechar" é a opção principal, e o prazo para um app fechar sozinho agora é definido em segundos, explica que só vale para os apps em "Fechar normalmente" e fica em "Avançado".
- O AutoStart agora é licenciado sob a GNU GPL v3.0 ou posterior (antes era MIT). O instalador e a seção "Sobre" das Configurações mostram a licença, e forks precisam usar outro nome e ícone.

### Corrigido

- Clicar em um perfil na barra lateral estando em Logs ou Configurações agora abre o perfil na tela Perfis, e ele só fica destacado enquanto essa tela está aberta.

## [0.3.0] - 2026-09-23

### Adicionado

- "Adicionar app" agora abre uma lista com os apps instalados no computador (Menu Iniciar e área de trabalho) e os apps abertos no momento, com busca; não é mais preciso procurar o `.exe` nas pastas.
- Ferramentas populares de simulação (Navigraph, SimBrief, Little Navmap, Volanta, vPilot, FSUIPC, SPAD.neXt, GSX e outras) aparecem sugeridas no topo da lista de apps instalados.
- Arrastar e soltar: solte um atalho de app, um `.exe` ou um atalho de site (`.url`) na janela para adicioná-lo; soltar vários arquivos adiciona todos de uma vez.
- Apps que já estão no perfil aparecem marcados na lista, e o formulário avisa quando o mesmo executável é adicionado duas vezes.
- Todos os perfis habilitados passam a ser monitorados: abra qualquer simulador e o perfil dele entra em ação, sem precisar trocar de perfil antes. Só um perfil habilitado pode monitorar cada simulador; habilitar um desliga o outro e avisa você.
- Um perfil pode ter vários simuladores como gatilho, por exemplo MSFS 2020 e MSFS 2024 compartilhando os mesmos apps.
- Os apps são fechados 60 segundos depois que o simulador fecha (configurável; 0 fecha na hora). Se o simulador voltar nesse tempo, por exemplo após um crash para a área de trabalho, nada é fechado. Uma contagem regressiva oferece "Fechar agora" e "Manter apps abertos", também no menu da bandeja.
- Notificações do Windows quando um app não pôde ser aberto, o SimConnect não ficou disponível, a contagem para fechar começou ou um app que travou foi reaberto. Elas podem ser desligadas nas Configurações.
- Opção "Reabrir se travar" por app: durante o voo, um app que trava é aberto de novo (até 3 vezes por sessão); apps que você mesmo fecha não são reabertos.

### Alterado

- O "perfil ativo" deixou de existir: o botão ao lado do nome de cada perfil (e o menu de perfis da bandeja) agora decide quais perfis são monitorados. Na primeira abertura após a atualização, perfis que compartilhavam um simulador com o antigo perfil ativo são desligados.
- Perfis e configurações salvos por esta versão não podem ser lidos por versões anteriores.

## [0.2.0] - 2026-09-22

### Adicionado

- O nome do processo de cada app é detectado automaticamente, inclusive em apps que usam um launcher; a configuração manual continua disponível em "Avançado".
- Opção "Abrir minimizado" por app.
- Opção "Aguardar o SimConnect" por app (MSFS 2020 e 2024): esses itens abrem depois dos outros, quando o simulador passa a aceitar conexões de addons.

### Alterado

- Todos os apps agora são abertos pelo shell do Windows, o que permite acompanhar os processos que eles iniciam e abri-los minimizados.

## [0.1.0] - 2026-09-22

Primeira versão pública.

### Adicionado

- Perfis com um gatilho (o processo do simulador) e itens de app ou URL, com modelos prontos para MSFS 2024, MSFS 2020 e X-Plane 12.
- Abertura em ordem com espera configurável, detecção de apps que já estão rodando e confirmação do processo iniciado.
- Por item: fechar normalmente, forçar encerramento ou manter aberto, em paralelo e com prazo; opção de fechar apenas o que o AutoStart abriu.
- Suporte a launchers: o fechamento alcança todos os processos com o nome configurado.
- Execução de itens como administrador e reinício do AutoStart com privilégios elevados.
- Botões para testar a abertura e o fechamento sem o simulador.
- Ícone na bandeja com menu de perfis, pausa do monitoramento, instância única e opção de iniciar com o Windows.
- Tela de logs com a linha do tempo da última sessão, além de logs técnicos em arquivo com rotação.
- Importação e exportação de perfis em JSON.
- Interface em português (Brasil) e inglês, com temas claro, escuro e do sistema.
- Assistente de primeira execução.
- Instalador por usuário (sem precisar de administrador) e atualizações automáticas assinadas.

[Unreleased]: https://github.com/mariopaglia/autostart/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/mariopaglia/autostart/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/mariopaglia/autostart/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/mariopaglia/autostart/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/mariopaglia/autostart/releases/tag/v0.1.0
