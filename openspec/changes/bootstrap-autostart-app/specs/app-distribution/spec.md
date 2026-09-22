## Purpose

Permite que qualquer piloto da comunidade instale o AutoStart sem privilégios de administrador e receba atualizações automaticamente, com um processo de release reproduzível.

## ADDED Requirements

### Requirement: Instalador per-user
O projeto SHALL gerar um instalador NSIS para Windows x64 que instala o app por usuário (sem exigir administrador), cria atalho no menu Iniciar e oferece desinstalação pelo Windows. O instalador SHALL estar disponível em pt-BR e en.

#### Scenario: Instalação sem admin
- **WHEN** um usuário comum executa o instalador
- **THEN** o app é instalado em `%LOCALAPPDATA%` sem prompt do UAC e aparece no menu Iniciar

### Requirement: Auto update assinado
O app SHALL verificar atualizações no GitHub Releases do repositório público (automaticamente ao iniciar, se habilitado, e pelo botão manual). Com atualização disponível, SHALL exibir versão e notas e instalar somente após o aceite do usuário, reiniciando o app em seguida. Pacotes de atualização SHALL ser verificados com a chave pública do updater.

#### Scenario: Nova versão disponível
- **WHEN** existe um release mais recente e o usuário clica em "Atualizar"
- **THEN** o pacote é baixado, verificado, instalado, e o app reinicia na nova versão

#### Scenario: Sem conexão
- **WHEN** a verificação falha por falta de rede
- **THEN** a verificação automática falha em silêncio (apenas log) e a manual exibe mensagem de erro

#### Scenario: Assinatura inválida
- **WHEN** o pacote baixado não confere com a chave pública
- **THEN** a atualização é abortada e o erro é exibido e registrado

### Requirement: Pipeline de release
Criar uma tag `v*` no repositório SHALL disparar um workflow em `windows-latest` que builda o app, gera o instalador assinado para o updater e publica um GitHub Release com o instalador e o `latest.json`. A versão da tag SHALL corresponder à versão do app.

#### Scenario: Publicar versão
- **WHEN** o mantenedor faz push da tag `v0.1.0`
- **THEN** um release `v0.1.0` é publicado com o instalador `.exe`, a assinatura `.sig` e o `latest.json`

#### Scenario: Versão divergente
- **WHEN** a tag é `v0.2.0` mas a versão do app é `0.1.0`
- **THEN** o workflow falha antes de publicar

### Requirement: Verificação contínua
Pushes e pull requests SHALL disparar um workflow que roda typecheck, lint e testes do frontend, além de clippy e testes do Rust, em `windows-latest`.

#### Scenario: PR com erro de tipo
- **WHEN** um PR introduz um erro de TypeScript
- **THEN** o workflow falha

### Requirement: Assinatura de código preparada
O workflow e a configuração SHALL conter o passo de assinatura com Azure Trusted Signing, desativado e documentado, ativável apenas configurando secrets e descomentando o passo.

#### Scenario: Ativar assinatura
- **WHEN** o mantenedor segue a documentação e configura os secrets do Azure
- **THEN** os próximos releases saem com o instalador assinado

### Requirement: Documentação
O README SHALL cobrir: o que o app faz, instalação, como descobrir o nome de um processo (Gerenciador de Tarefas → Detalhes, e o seletor de processos do app), desenvolvimento (pré-requisitos, `pnpm tauri dev` no Windows e no macOS com stubs), build, geração das chaves do updater, secrets do GitHub, processo de release e ativação da assinatura.

#### Scenario: Novo contribuidor
- **WHEN** alguém clona o repositório e segue o README
- **THEN** consegue rodar o app em modo de desenvolvimento sem outras instruções
