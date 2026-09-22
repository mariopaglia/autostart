# Política de Segurança

> **English** — Please do not report security issues in public issues. Use GitHub's [private vulnerability reporting](https://github.com/mariopaglia/autostart/security/advisories/new). Only the latest release receives fixes.

## Versões suportadas

Apenas a versão mais recente publicada em [Releases](https://github.com/mariopaglia/autostart/releases/latest) recebe correções de segurança. O próprio AutoStart avisa quando há uma versão nova.

## Como relatar uma vulnerabilidade

**Não abra uma issue pública.** Use o [relato privado de vulnerabilidades](https://github.com/mariopaglia/autostart/security/advisories/new) do GitHub (aba **Security → Report a vulnerability**) e inclua:

- a versão afetada e o Windows usado;
- a descrição do problema e o impacto;
- os passos para reproduzir ou uma prova de conceito.

Você recebe uma resposta inicial em até 7 dias. Depois de confirmada, a correção é publicada num novo release, e o relato é divulgado com os devidos créditos, a menos que você prefira anonimato.

## O que conta como vulnerabilidade

Exemplos do que nos interessa:

- contornar a verificação de assinatura das atualizações;
- fazer o AutoStart executar um programa que o usuário não configurou;
- escalar privilégios através do AutoStart;
- vazar dados dos perfis ou argumentos dos apps.

## Como o projeto se protege

- **Atualizações assinadas**: todo pacote de atualização é assinado com uma chave privada que fica fora do repositório (nos secrets do GitHub e com o mantenedor). O app só instala pacotes cuja assinatura confere com a chave pública embutida nele.
- **Releases só pelo pipeline**: os instaladores oficiais são gerados pelo workflow [`release.yml`](.github/workflows/release.yml) a partir de uma tag no repositório oficial.
- **Sem elevação por padrão**: o app instala e roda sem privilégios de administrador. A elevação só acontece quando o usuário pede.
- **Download oficial**: baixe o AutoStart somente em [github.com/mariopaglia/autostart/releases](https://github.com/mariopaglia/autostart/releases).
