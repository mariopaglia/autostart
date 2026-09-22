## Context

Motivação e escopo em `proposal.md`; comportamento nos deltas em `specs/`. Estado atual relevante (v0.1.0):

- `launcher.rs` inicia apps normais com `std::process::Command` (flags detached + `raw_arg`) e apps elevados com `ShellExecuteExW("runas")` em `platform/windows/elevation.rs`. Nenhum dos caminhos devolve o PID ao chamador: a confirmação (`wait_for_process`) e o fechamento dependem só do `processName` do item.
- `monitor/runner.rs::launch_items` percorre os itens habilitados em uma única passada sequencial; a `Session` guarda uma cópia congelada do perfil, usada pelo fechamento (`close_targets`).
- `processes.rs` usa `sysinfo` (nomes, PIDs e caminhos de executável); `sysinfo` também expõe o PID pai e o horário de início de cada processo.
- O `AppState` é o dono dos perfis; o frontend recebe mudanças feitas pelo backend por eventos (`settings://changed`, usado pela bandeja).
- Restrições: Rust pequeno e plano, `unsafe` isolado em `platform/windows/`, stubs em `platform/fallback.rs` para o macOS, lógica em funções puras testáveis.

## Goals / Non-Goals

**Goals:**
- Um único ponto de lançamento na camada `platform`, que devolve o PID e aceita as opções "elevado" e "minimizado".
- Rastreamento e escolha do processo aprendido como funções puras, testáveis sem o SO.
- Detecção do SimConnect sem depender do SDK do MSFS.

**Non-Goals:**
- Conectar de fato ao SimConnect (ler dados do voo, estado da aeronave).
- Suportar configurações personalizadas do `SimConnect.xml` (pipe com outro nome ou apenas TCP); nesse caso a espera termina em erro por tempo limite, conforme a spec.
- Aprender o nome de processo de itens em modo manual ou de itens URL.
- Forçar a minimização de apps que restauram a própria janela depois do período de observação.

## Decisions

### D1. Lançamento unificado via `ShellExecuteExW`, devolvendo o PID
`platform::launch(exe, args, working_dir, LaunchOptions { elevated, minimized }) -> AppResult<u32>` substitui `launch_elevated` e o `Command` do `launcher.rs`. No Windows, usa `ShellExecuteExW` com `SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC`, verbo `runas` quando elevado (ou nenhum verbo, que equivale a "open"), `nShow = SW_SHOWMINNOACTIVE` quando minimizado (senão `SW_SHOWNORMAL`), e obtém o PID com `GetProcessId(hProcess)`, fechando o handle num guard com `Drop`. `lpParameters` recebe os `args` como string única, preservando as aspas do usuário como o `raw_arg` fazia.
- *Alternativa descartada:* manter o `Command` e usar `CommandExt::show_window`, que ainda é instável no Rust estável; ou chamar `CreateProcessW` diretamente, o que adicionaria um segundo bloco `unsafe` grande só para o caso minimizado.
- *Consequência:* um só caminho de lançamento no Windows e o PID disponível também para itens elevados. As flags `DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP` deixam de ser necessárias: o processo criado pelo shell não herda console nem grupo do AutoStart. Como isso troca um caminho já validado na v0.1.0, o checkpoint da fase repete o teste de abertura normal, com argumentos e pasta de trabalho.
- No macOS, o stub usa `Command::spawn` e devolve `child.id()`; `minimized` é ignorado.

### D2. Rastreamento da árvore de processos e escolha do nome (funções puras)
Novo módulo `process_tracker.rs`, sem acesso ao SO:
- `ProcessSample { pid, parent_pid, name, exe_path, start_time }` descreve um processo numa leitura do `sysinfo`.
- `expand_tracked(tracked: &HashSet<u32>, samples: &[ProcessSample]) -> HashSet<u32>` adiciona todo processo cujo pai já está no conjunto. O conjunto só cresce, então netos continuam rastreados mesmo quando o launcher intermediário já encerrou.
- `choose_process_name(root, tracked_alive, new_in_install_dir) -> Option<String>` aplica a prioridade da spec: o processo raiz vivo; senão o descendente vivo mais antigo (por `start_time`, desempate por nome); senão o processo novo mais antigo cujo executável está sob a pasta do `exePath` do item (comparação de caminho sem diferenciar maiúsculas); senão `None`.
- "Processo novo" é qualquer PID ausente no snapshot tirado imediatamente antes do lançamento.

O `launcher.rs` faz o polling (a cada 500 ms): até 10 s para a confirmação de execução (qualquer PID rastreado vivo ou o `processName` em execução) e até 30 s para o aprendizado. O aprendizado roda numa task separada por item, para não atrasar a abertura dos itens seguintes.
- *Alternativa descartada:* Job Objects do Windows (seguem a árvore de forma nativa), porque exigem criar o processo suspenso e mais `unsafe`, e não pegam apps que o launcher abre pelo shell. A heurística da pasta de instalação cobre esse caso.

### D3. Aplicação do nome aprendido
Quando `choose_process_name` devolve um nome diferente do atual e o item está em `ProcessNameMode::Auto`:
1. `Session::learn_process_name(item_id, name)` atualiza a cópia do perfil da sessão, e o fechamento dessa sessão já usa o nome novo.
2. `AppState::learn_process_name(profile_id, item_id, name)` atualiza o perfil persistido, somente se o item ainda existir e ainda estiver em modo `Auto` (o usuário pode ter editado o item no meio da sessão), e grava em disco.
3. O backend emite `profiles://changed` com o `Profile` atualizado; a `profiles-store` faz upsert, e o formulário e o card refletem o nome sem recarregar.
- O aprendizado também roda em "Testar abertura", o que permite ao usuário configurar um launcher sem abrir o simulador.

### D4. Minimização após a abertura
`platform::minimize_new_windows(pids, already_minimized: &mut HashSet<isize>)` enumera as janelas de nível superior visíveis (`EnumWindows` + `IsWindowVisible`) dos PIDs rastreados e aplica `ShowWindow(SW_SHOWMINNOACTIVE)` a cada janela ainda não minimizada por nós, registrando o handle. O launcher chama essa função a cada 250 ms durante 10 s depois que o item fica `running`. Como cada janela é minimizada só uma vez, se o usuário restaurar a janela nesse período o AutoStart não briga com ele. `SW_SHOWMINNOACTIVE` em vez de `SW_MINIMIZE`, porque este último ativa a próxima janela e pode roubar o foco do simulador.
- No macOS, stub vazio.

### D5. Detecção do SimConnect por named pipe
O servidor SimConnect do MSFS abre o named pipe `\\.\pipe\Microsoft Flight Simulator\SimConnect` quando está pronto para conexões. `platform::is_simconnect_available() -> bool` chama `WaitNamedPipeW(name, 1)`: sucesso ou `ERROR_SEM_TIMEOUT` (o pipe existe, mas todas as instâncias estão ocupadas) significam disponível; `ERROR_FILE_NOT_FOUND` significa indisponível. O nome do pipe fica numa constante única.
- *Alternativa descartada:* usar o SDK do SimConnect (`SimConnect.dll` e `SimConnect_Open`). Dá a confirmação mais forte, mas exige redistribuir a DLL do SDK, FFI adicional e acoplamento à versão do MSFS, para um ganho pequeno sobre a existência do pipe.
- `supports_simconnect(trigger_process_name) -> bool` (função pura) aceita `FlightSimulator.exe` e `FlightSimulator2024.exe`, sem diferenciar maiúsculas. O frontend espelha a lista em `trigger-presets.ts` (`SIMCONNECT_TRIGGERS`) para desativar o checkbox.
- No macOS, o stub devolve `true`, para que o fluxo da segunda etapa possa ser exercitado em desenvolvimento.

### D6. Abertura em duas etapas no runner
`runner::launch_items` divide os itens habilitados em `immediate` e `deferred` (`waitForSimConnect && item é app`), preservando a ordem relativa (função pura `split_launch_phases`, testável). Depois da primeira etapa:
- Sessão real com gatilho que suporta SimConnect: todos os `deferred` recebem `ItemStatus::WaitingSimConnect`; o runner verifica a cada 2 s por até 10 min. Com o SimConnect disponível, registra `TimelineKind::SimConnectReady` e abre os `deferred` em ordem, com seus delays. No tempo limite, cada item em espera recebe `error` com `AppError::SimConnectUnavailable`.
- Sessão de teste: registra `TimelineKind::SimConnectWaitSkipped` e abre os `deferred` sem esperar.
- Gatilho sem suporte: abre os `deferred` sem esperar.
- O fim da sessão já aborta a task de abertura (`abort(self.launch_task)`), o que interrompe a espera e impede a abertura dos itens em espera, sem código novo.
- Prazo de 10 min, e não 5: o MSFS 2024 pode levar vários minutos até o menu principal em máquinas modestas, e um prazo curto geraria erros falsos.

### D7. Modelo e compatibilidade
- `AppItem` ganha `process_name_mode: ProcessNameMode` (`auto` | `manual`, `#[serde(default)]` = `Auto`), `start_minimized: bool` e `wait_for_simconnect: bool` (`#[serde(default)]` = false). `ItemStatus` ganha `WaitingSimConnect`; `TimelineKind` ganha `SimConnectReady` e `SimConnectWaitSkipped`; `ErrorKind`/`AppError` ganham `SimConnectUnavailable`. Os bindings são regenerados, e os schemas Zod ganham os mesmos campos com `.default(...)`, o que mantém válidos o import de arquivos da v0.1.0 e o carregamento de `profiles.json` antigos.
- `schemaVersion` continua 1: só há campos novos com valor padrão. Em um downgrade para a v0.1.0, os campos desconhecidos são ignorados pelo serde e descartados pelo Zod, sem erro.

### D8. Formulário
- `AppItemForm`: "Abrir minimizado" e "Aguardar o SimConnect" usam o mesmo controle de "Executar como administrador" (`Switch` com rótulo), para manter a consistência visual.
- Nova seção "Avançado" com o componente `collapsible` do shadcn (adicionado via CLI), contendo a pasta de trabalho e o nome do processo. Editar o nome define `processNameMode = manual`; o botão "Detectar automaticamente" define `auto` e restaura o nome a partir do executável.
- O formulário recebe o gatilho do perfil para decidir se o SimConnect é suportado.

## Risks / Trade-offs

- [Troca do `Command` pelo `ShellExecuteExW` pode mudar o comportamento de apps que já funcionavam] → Mesmo tratamento de `args` (string crua) e `workingDir`. O checkpoint repete o teste de abertura normal da v0.1.0, com argumentos e pasta de trabalho.
- [Heurística escolhe o processo errado (ex.: o launcher abre um updater e o app)] → Prioridade por descendente mais antigo, escopo restrito à pasta de instalação, e o modo manual como saída. O nome aprendido aparece no formulário, e a timeline registra o aprendizado.
- [Processos novos alheios na mesma pasta (ex.: dois apps no mesmo diretório)] → A heurística da pasta só é usada quando não há descendente vivo, e apenas com processos que surgiram depois do lançamento.
- [Nome do pipe diferente no MSFS 2024, ou `SimConnect.xml` personalizado] → Constante única, fácil de ajustar. O checkpoint valida o nome no MSFS 2020 e no 2024 antes do release. Configuração personalizada resulta em erro claro por tempo limite.
- [SimConnect disponível não significa "aeronave carregada"] → A spec promete apenas "disponível para conexões", que é o que os addons precisam para iniciar. A limitação fica documentada no README.
- [Apps que se restauram após 10 s ou ignoram `nShow`] → Limitação aceita pela spec e documentada.
- [Aprendizado grava no perfil enquanto o usuário edita o mesmo item] → O `AppState` só aplica o nome se o item ainda estiver em modo `auto`, e o save do usuário, que envia o item inteiro, prevalece por ser a gravação mais recente.

## Migration Plan

1. Implementar e validar no Windows (checkpoints em `tasks.md`).
2. Arquivar a change `bootstrap-autostart-app`, pré-requisito para arquivar esta, que modifica os requisitos dela.
3. Subir a versão para 0.2.0 em `package.json`, `Cargo.toml` e `tauri.conf.json`, atualizar o CHANGELOG, fazer o merge no `main` e enviar a tag `v0.2.0`.
4. Com a v0.1.0 instalada, validar a atualização para a v0.2.0 pelo app (tarefa 4.2 da change `bootstrap-autostart-app`).
- Rollback: despublicar o release v0.2.0 faz o `latest.json` voltar à v0.1.0. Os perfis gravados pela v0.2.0 continuam legíveis pela v0.1.0 (D7).
