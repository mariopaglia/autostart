# AutoStart

[![CI](https://github.com/mariopaglia/autostart/actions/workflows/ci.yml/badge.svg)](https://github.com/mariopaglia/autostart/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/mariopaglia/autostart?label=release)](https://github.com/mariopaglia/autostart/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/mariopaglia/autostart/total)](https://github.com/mariopaglia/autostart/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Platform: Windows 10/11](https://img.shields.io/badge/platform-Windows%2010%2F11-0078d4)

Abre automaticamente os apps auxiliares do seu simulador de voo quando ele inicia e fecha tudo quando ele encerra.

> **English summary** — AutoStart is a free Windows tray app for virtual pilots. Pick a trigger process (e.g. `FlightSimulator2024.exe`), add your helper apps and websites (Volanta, Navigraph, SimBrief, SPAD.neXt…) and AutoStart launches them in order when the simulator starts and closes them when it exits. Install it from the [latest release](https://github.com/mariopaglia/autostart/releases/latest) (per-user, no admin rights). The app is available in Portuguese and English. Development instructions below work on Windows and macOS (macOS uses stubs for the Windows-only APIs).

## O que ele faz

- **Perfis**: cada perfil tem um gatilho (o processo do simulador) e uma lista de itens (apps ou URLs).
- **Abertura em ordem**: quando o gatilho aparece, os itens habilitados abrem na ordem da lista, com uma espera configurável antes de cada um. Apps que já estavam abertos são pulados.
- **Fechamento inteligente**: quando o simulador fecha, cada app é fechado normalmente (como clicar no X), à força, ou mantido aberto, conforme você escolher. Por padrão, só é fechado o que o próprio AutoStart abriu.
- **Discreto**: fica na bandeja do sistema, pode iniciar com o Windows e mostra um log de cada sessão.

## Instalação

1. Baixe o instalador `AutoStart_x.y.z_x64-setup.exe` no [último release](https://github.com/mariopaglia/autostart/releases/latest). Baixe **somente** por esse link oficial.
2. Execute o instalador. Ele instala só para o seu usuário, em `%LOCALAPPDATA%`, **sem pedir administrador**, e cria o atalho no menu Iniciar.
3. **Aviso do SmartScreen**: como o instalador ainda não tem assinatura de código, o Windows pode mostrar "O Windows protegeu o computador". Clique em **Mais informações → Executar assim mesmo**.

Para desinstalar, use **Configurações do Windows → Aplicativos → AutoStart → Desinstalar**.

### Atualizações

O AutoStart verifica novas versões ao iniciar (dá para desligar em Configurações) e pelo botão **Verificar atualizações agora**. Toda atualização é assinada e só é instalada se a assinatura conferir com a chave pública embutida no app.

## Como usar

1. Na primeira execução, um assistente cria o perfil de exemplo e pergunta qual simulador você usa (MSFS 2024, MSFS 2020, X-Plane 12 ou personalizado).
2. Na tela principal, use **Adicionar app** para escolher um `.exe` (nome, ícone e processo são preenchidos sozinhos) ou **Adicionar URL** para abrir um site.
3. Arraste os cards para definir a ordem. O teclado também funciona: foque a alça, aperte Espaço e use as setas.
4. Use **Testar abertura** e **Testar fechamento** para conferir tudo sem abrir o simulador.
5. O perfil **ativo** (marcado na lista, também trocável pela bandeja) é o que o AutoStart observa.

### Como descobrir o nome de um processo

O AutoStart identifica o simulador e os apps pelo **nome do executável** (ex.: `Volanta.exe`). Isso importa principalmente para apps que usam um _launcher_: você abre `Launcher.exe`, mas quem fica rodando é outro processo.

- **Pelo AutoStart**: no seletor de gatilho, a lista "Processos em execução" mostra tudo o que está aberto agora. Abra o app antes e procure por ele.
- **Pelo Gerenciador de Tarefas**: `Ctrl + Shift + Esc` → aba **Detalhes** → a coluna **Nome** mostra o executável.

No formulário do app, ajuste o campo **Nome do processo** para o processo que fica rodando.

### Importar e exportar perfis

No menu `…` de um perfil, **Exportar** salva um arquivo `.json` que pode ser compartilhado. **Importar perfil** (na barra lateral) valida o arquivo e cria uma cópia com identificadores novos. Se os caminhos dos apps forem diferentes no seu PC, edite os itens depois de importar (os cards mostram um aviso quando o executável não existe).

### Apps que precisam de administrador

- Marque **Executar como administrador** no item. O Windows vai pedir confirmação (UAC) na hora de abrir.
- Para **fechar** um app que roda como administrador, o AutoStart também precisa estar elevado. Nesse caso a tela principal mostra um aviso com o botão **Reiniciar como administrador**.
- Com o AutoStart elevado, todos os apps abertos por ele também herdam a elevação.
- O "Iniciar com o Windows" não inicia o AutoStart elevado. Se você precisa disso sempre, crie uma tarefa no **Agendador de Tarefas** com "Executar com privilégios mais altos" apontando para o `autostart.exe` com o argumento `--minimized`.

### Onde ficam os dados

| O quê                                  | Onde                                             |
| -------------------------------------- | ------------------------------------------------ |
| Perfis, configurações e última sessão  | `%APPDATA%\com.mariopaglia.autostart\`           |
| Logs técnicos (5 arquivos de até 5 MB) | `%LOCALAPPDATA%\com.mariopaglia.autostart\logs\` |

A tela de Configurações tem o botão **Abrir pasta de logs**. Os logs registram nomes e caminhos, mas nunca os argumentos dos apps. Ao relatar um problema, anexe o arquivo de log mais recente.

## Desenvolvimento

Stack: Tauri 2, React 19, TypeScript, Vite, Tailwind CSS 4 + shadcn/ui, Zustand, Zod e i18next no frontend; Rust com `sysinfo`, `windows`, `tokio` e `ts-rs` no backend. O planejamento fica em [`openspec/`](openspec/) e as convenções de código em [`CLAUDE.md`](CLAUDE.md).

### Pré-requisitos

- [Node.js 24 LTS](https://nodejs.org/) e pnpm (fixado em `package.json`): `corepack enable`
- [Rust estável](https://rustup.rs/)
- **Windows**: [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (carga "Desenvolvimento para desktop com C++") e o WebView2 (já vem no Windows 10/11 atualizado)
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)

Detalhes em [Tauri — Prerequisites](https://v2.tauri.app/start/prerequisites/).

### Rodando

```bash
pnpm install
pnpm tauri dev
```

A janela nasce oculta quando **Iniciar minimizado** está ligado (padrão). Nesse caso, clique no ícone da bandeja (ou da barra de menus, no macOS).

**No macOS**, as APIs exclusivas do Windows (ícone do executável, elevação, fechamento por janela) usam os stubs de `src-tauri/src/platform/fallback.rs`. O monitor funciona de ponta a ponta: use qualquer processo do Mac como gatilho (ex.: `TextEdit.exe`, já que o nome do gatilho precisa terminar em `.exe`; no Mac o AutoStart compara o nome sem a extensão).

### Verificações

```bash
pnpm typecheck && pnpm lint && pnpm test          # frontend
cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test   # Rust
```

`cargo test` também regenera os tipos TypeScript em `src/bindings/` a partir dos structs Rust (`pnpm bindings` faz só isso). O CI (`.github/workflows/ci.yml`, em `windows-latest`) roda tudo isso em cada push e falha se os bindings estiverem desatualizados.

### Build

```bash
pnpm tauri build --bundles nsis --config src-tauri/tauri.preview.conf.json
```

O instalador fica em `src-tauri/target/release/bundle/nsis/`. O `tauri.preview.conf.json` desliga os artefatos do updater, que exigem a chave privada. Sem precisar de toolchain local, o workflow **Preview build** (aba Actions → Run workflow, ou `gh workflow run preview-build.yml --ref <branch>`) gera o instalador e o `.exe` portátil como artifact.

## Publicação (mantenedor)

### Chaves do updater

Feito uma única vez. **Perder a chave privada impede para sempre as atualizações automáticas** das instalações existentes; guarde a chave e a senha num gerenciador de senhas.

1. Gere o par de chaves num terminal interativo (a senha é pedida duas vezes):

   ```bash
   pnpm tauri signer generate -w ~/.tauri/autostart.key
   chmod 600 ~/.tauri/autostart.key
   ```

2. Coloque o conteúdo de `~/.tauri/autostart.key.pub` em `plugins.updater.pubkey` no `src-tauri/tauri.conf.json`. A chave pública pode ficar no repositório.
3. Cadastre os secrets do repositório (a chave privada **nunca** entra no repositório):

   ```bash
   gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.tauri/autostart.key
   gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD   # cole a senha quando pedir
   ```

### Lançando uma versão

1. Atualize a mesma versão em `package.json`, `src-tauri/Cargo.toml` e `src-tauri/tauri.conf.json` e faça o commit no `main`.
2. Crie e envie a tag:

   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

3. O workflow **Release** (`.github/workflows/release.yml`) confere se a tag bate com a versão dos três arquivos (senão falha antes de publicar), builda no `windows-latest` e publica o release com o instalador, a assinatura `.sig` e o `latest.json`, que é o arquivo que os apps instalados consultam.

Para tirar uma versão do ar, apague o release no GitHub: o `latest.json` volta a ser o do release anterior.

### Assinatura de código (Azure Trusted Signing)

Preparada, mas desativada. Com ela, o SmartScreen para de alertar e o Windows mostra o editor do instalador.

1. Crie uma conta do [Azure Trusted Signing](https://learn.microsoft.com/azure/trusted-signing/) com um perfil de certificado e um App Registration com a função _Trusted Signing Certificate Profile Signer_.
2. Cadastre no repositório:
   - Secrets: `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID`
   - Variables: `AZURE_SIGNING_ENDPOINT` (ex.: `https://eus.codesigning.azure.net`), `AZURE_SIGNING_ACCOUNT`, `AZURE_CERTIFICATE_PROFILE`
3. No `release.yml`, descomente o passo **Install Azure Trusted Signing CLI** e as variáveis `AZURE_*` e `TAURI_CONFIG` do passo **Build and publish release**. O `TAURI_CONFIG` define o `bundle.windows.signCommand` usando o [`trusted-signing-cli`](https://github.com/Levminer/trusted-signing-cli):

   ```
   trusted-signing-cli -e <endpoint> -a <conta> -c <perfil> -d AutoStart %1
   ```

Os próximos releases saem com o instalador e o executável assinados.

## Comunidade

- **Contribuindo**: leia o [guia de contribuição](CONTRIBUTING.md) antes de abrir um pull request.
- **Bugs e ideias**: use as [issues](https://github.com/mariopaglia/autostart/issues/new/choose), com os modelos prontos.
- **Segurança**: relate vulnerabilidades de forma privada, conforme a [Política de Segurança](SECURITY.md).
- **Conduta**: todos os espaços do projeto seguem o [Código de Conduta](CODE_OF_CONDUCT.md).
- **Novidades**: cada versão está descrita no [CHANGELOG](CHANGELOG.md).

## Licença

[MIT](LICENSE) © Mario Paglia
