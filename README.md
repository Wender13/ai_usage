<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="Ícone do AI Usage" width="96" height="96">
</p>

<h1 align="center">AI Usage</h1>

<p align="center">
  Monitor de desktop para os limites de uso do Claude, com avisos por notificação e um overlay sempre visível.
</p>

---

O AI Usage roda em segundo plano, no tray do sistema. Ele consulta periodicamente quanto dos seus limites do Claude você já usou e avisa conforme regras que você define. Os limites acompanhados são a sessão de 5 horas e a janela semanal. Os avisos podem vir como notificação do sistema, numa janela compacta de overlay ou nos dois.

> **Status:** o provider do **Claude** está completo. O provider do **Gemini/Antigravity** e a **inicialização automática com o sistema** ainda estão em desenvolvimento (ver [Roadmap](#roadmap)).

## Sumário

- [Funcionalidades](#funcionalidades)
- [Como funciona](#como-funciona)
- [Requisitos](#requisitos)
- [Instalação e execução](#instalação-e-execução)
- [Uso](#uso)
- [Linux: GNOME, Wayland e tray](#linux-gnome-wayland-e-tray)
- [Dados e privacidade](#dados-e-privacidade)
- [Arquitetura](#arquitetura)
- [Desenvolvimento](#desenvolvimento)
- [Solução de problemas](#solução-de-problemas)
- [Roadmap](#roadmap)

## Funcionalidades

- **Status do uso:** barra de progresso, percentual usado e tempo até o reset de cada janela de limite (sessão de 5h e semanal).
- **Regras de aviso configuráveis:**
  - **Limiar de uso:** avisa quando o uso de uma janela passa de X%, no máximo uma vez por ciclo de reset.
  - **Resumo periódico:** um resumo do uso a cada N minutos.
  - **Antes do reset:** avisa N minutos antes de uma janela reiniciar.
- **Canais por regra:** notificação do sistema, overlay ou ambos.
- **Overlay compacto:** janela sem bordas, transparente e sempre no topo, com os percentuais.
  - Posição padrão em seis pontos da tela, posição livre por arraste e trava de posição.
  - Opacidade do fundo ajustável.
  - Escolha do que exibir: nome do provider, nome da janela, percentual e tempo até o reset, além de cada janela individualmente.
- **Tray:** abrir a janela, ligar ou desligar o overlay, forçar atualização e sair.
- **Linha de comando:** `--toggle` e `--overlay`, para controlar o overlay por um atalho de teclado.
- **Instância única:** abrir o app de novo apenas foca a instância que já está rodando, sem duplicar o tray nem as notificações.

## Como funciona

### De onde vêm os números

O app não calcula o limite: quem calcula são os servidores da Anthropic. A cada consulta, o app faz o seguinte:

1. Lê o token de login do Claude Code em `~/.claude/.credentials.json`.
2. Chama `GET https://api.anthropic.com/api/oauth/usage`, o mesmo endpoint usado pelo comando `/usage` do Claude Code.
3. Recebe, para cada janela de limite, o percentual usado e o momento do reset:

   ```json
   "five_hour": { "utilization": 43.0, "resets_at": "2026-10-19T01:00:00.42+00:00" },
   "seven_day": { "utilization": 40.0, "resets_at": "2026-10-22T22:00:00.42+00:00" }
   ```

   - **`utilization`** é o percentual já usado, de 0 a 100. O que resta é 100 menos esse valor. A API informa apenas percentuais, não mensagens nem tokens.
   - **`resets_at`** é quando a janela zera. O tempo restante é calculado na interface (`resets_at − agora`) e atualizado a cada 30 segundos.

O uso é contado por **conta**: mensagens no claude.ai, no app de desktop e no Claude Code entram todas no mesmo percentual.

> O endpoint não é documentado publicamente. Por isso, o leitor tolera mudanças: ignora campos desconhecidos e valores nulos e, se o formato mudar, mostra um erro claro em vez de travar.

### Token

- O arquivo de credenciais é **relido a cada consulta** e **nunca escrito**. Quando o Claude Code renova o token, o app passa a usar o novo automaticamente.
- Se o token já expirou (`expiresAt`), o app nem faz a requisição: mostra "Token do Claude expirado. Abra o Claude Code para renová-lo."
- O token nunca é registrado em log, exibido ou enviado a outro destino.

### Consulta periódica

A consulta roda no núcleo em Rust, independente de a janela estar aberta. O intervalo padrão é de 3 minutos, com mínimo de 1, e pode ser configurado. "Atualizar agora" (na janela ou no tray) força uma consulta imediata.

Se uma consulta falhar (rede, token expirado etc.), a tela mantém os últimos valores válidos, esmaecidos, ao lado da mensagem de erro.

### Regras

- **Ciclo de reset.** Cada janela tem um ciclo identificado pelo seu `resets_at`. A API varia esse horário em cerca de 1 segundo a cada resposta, então horários a menos de 10 minutos de distância contam como o mesmo ciclo.
- **Limiar.** Dispara quando o uso cruza o limite (`anterior < X ≤ atual`), no máximo uma vez por ciclo. A primeira leitura de um ciclo conta como cruzamento: se o app abrir com o uso já acima do limite, você é avisado.
- **Persistência.** O ciclo em que cada regra disparou fica salvo. Reiniciar o app não repete avisos do ciclo atual.
- **Edição.** Editar, ativar ou desativar uma regra a "rearma", e ela pode disparar de novo no ciclo atual.
- **Antes do reset.** Ignora janelas com uso 0%, cujo reset não muda nada.
- **Precisão.** As regras "resumo periódico" e "antes do reset" são avaliadas a cada consulta, então a precisão delas é igual ao intervalo de consulta.

## Requisitos

| Requisito | Versão / observação |
|---|---|
| Sistema | Linux (testado no Fedora 44 com GNOME 50 em Wayland). Windows e Android estão planejados. |
| [Claude Code](https://docs.claude.com/en/docs/claude-code) | Instalado e com login feito (cria `~/.claude/.credentials.json`) |
| Rust | Estável, com suporte à edition 2024 (1.85 ou mais recente) |
| Node.js | `^20.19` ou `>= 22.12` (exigência do Vite 8) |
| pnpm | Gerenciador de pacotes do front |
| Dependências de sistema do Tauri | Ver abaixo |

### Dependências de sistema (Linux)

**Fedora:**

```sh
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file \
  libappindicator-gtk3-devel librsvg2-devel
sudo dnf group install c-development
```

**Ubuntu / Debian:**

```sh
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

A lista oficial e atualizada está nos [pré-requisitos do Tauri](https://v2.tauri.app/start/prerequisites/).

**Tray no GNOME (Fedora):** o GNOME não mostra ícones de tray sem uma extensão.

```sh
sudo dnf install gnome-shell-extension-appindicator
gnome-extensions enable appindicatorsupport@rgcjonas.gmail.com
```

Depois disso, saia e entre de novo na sessão. O Ubuntu já vem com essa extensão ativa.

### Principais bibliotecas

| Camada | Bibliotecas |
|---|---|
| Núcleo (Rust) | `tauri` 2, `tauri-plugin-notification`, `tauri-plugin-store`, `tauri-plugin-single-instance`, `reqwest` (TLS via rustls), `tokio`, `chrono`, `serde`, `thiserror`, `uuid` |
| Interface | Vue 3 (Composition API, `<script setup>`), TypeScript, Vite 8, `@tauri-apps/api` |

## Instalação e execução

```sh
git clone git@github.com:Wender13/ai_usage.git
cd ai_usage
pnpm install
```

**Desenvolvimento** (recarrega o front e recompila o Rust ao salvar):

```sh
pnpm tauri dev
```

**Build de produção** (gera os pacotes em `src-tauri/target/release/bundle/`):

```sh
pnpm tauri build
```

## Uso

### Janela principal

Fechar a janela apenas a esconde: o app continua no tray. Para sair, use **Sair** no menu do tray. A janela tem quatro áreas:

- **Status:** um card por provider, com cada janela de limite (percentual, barra e "reinicia em 2h 10min (22:00)"). A barra fica amarela a partir de 70% e vermelha a partir de 90%.
- **Regras:** lista com descrição automática (ex.: "Ao atingir 80% em Claude · Sessão (5h)"), caixa para ativar ou desativar, **Editar** e **Excluir**. **Nova regra** abre o formulário.
- **Configurações:** intervalo de consulta, em minutos.
- **Overlay:** todas as opções do overlay, descritas abaixo.

### Criando regras

No formulário, escolha:

| Campo | Opções |
|---|---|
| Tipo | Limiar de uso · Resumo periódico · Antes do reset |
| Provider | Qualquer · Claude |
| Janela | Qualquer · Sessão (5h) · Semanal (7 dias) |
| Limiar (%) / minutos | Depende do tipo |
| Canal | Notificação · Overlay · Notificação e overlay |

Alguns exemplos:

- **Avisar quando a sessão passar de 80%:** Limiar de uso, Claude, Sessão (5h), 80%, Notificação.
- **Lembrar de usar o que sobrou antes do reset semanal:** Antes do reset, Claude, Semanal, 120 minutos.
- **Acompanhar o consumo durante o dia:** Resumo periódico, a cada 60 minutos.

Com "Qualquer", a regra vale para todas as janelas correspondentes, e cada janela é acompanhada separadamente.

### Overlay

Ligue em **Overlay → Mostrar overlay**, pelo tray ou pela linha de comando. Opções:

| Opção | Efeito |
|---|---|
| Posição | Superior ou inferior, cada um à esquerda, no centro ou à direita, na área útil do monitor. Com âncora inferior, o overlay cresce para cima. |
| Arrastar | Arraste o overlay para qualquer lugar: a posição é salva e restaurada ao reabrir. Escolher uma posição da lista volta para ela. |
| Travar posição | Impede arrastar. Se algo mover a janela, ela volta ao lugar salvo quando reaparecer. |
| Opacidade do fundo | De 0% (só o texto) a 100% |
| Informações exibidas | Nome do provider, nome da janela (5h, 7d), percentual, tempo até o reset |
| Janelas exibidas | Liga ou desliga cada janela (ex.: mostrar só a semanal) |
| Usar XWayland | Só no Linux; ver [abaixo](#linux-gnome-wayland-e-tray) |

Alertas de regras com canal overlay aparecem como um banner no topo do overlay. Clique para dispensar; se houver mais de um, aparece "+N alerta(s)". Se o overlay estiver desligado, ele aparece só enquanto houver alertas.

### Tray

O menu tem: **Abrir janela**, **Overlay** (liga e desliga), **Atualizar agora** e **Sair**. No Linux, o tray não recebe cliques, então todas as ações ficam no menu.

### Linha de comando

| Comando | Com o app fechado | Com o app rodando |
|---|---|---|
| `ai_usage` | Abre a janela principal | Foca a janela principal |
| `ai_usage --overlay` | Inicia só com tray e overlay | Liga o overlay |
| `ai_usage --toggle` | Inicia só com tray e overlay ligado | Liga ou desliga o overlay |
| `ai_usage --hidden` | Inicia só no tray | Nada |

Com o app rodando, o comando não abre uma segunda cópia: ele repassa a flag para a instância existente e encerra (cerca de 0,1 s). Quando várias flags são passadas, vale a de efeito maior: `--toggle`, depois `--overlay`, depois `--hidden`.

### Atalho de teclado para o overlay (GNOME)

1. Abra **Configurações → Teclado → Ver e personalizar atalhos → Atalhos personalizados → +**.
2. Use como comando o caminho do binário com `--toggle`. Em desenvolvimento, por exemplo:

   ```
   /caminho/para/ai_usage/src-tauri/target/debug/ai_usage --toggle
   ```

3. Escolha a combinação de teclas (ex.: <kbd>Super</kbd>+<kbd>Shift</kbd>+<kbd>U</kbd>).

## Linux: GNOME, Wayland e tray

**Overlay no GNOME em Wayland.** O GNOME (Mutter) ignora "sempre no topo" e posicionamento para janelas comuns em Wayland. Nessa situação, o overlay aparece, mas sem ficar no topo e na posição que o compositor escolher. O app também não consegue saber onde a janela está, então arrastes nesse modo não são salvos. Há duas formas de resolver:

- **Pelo app (recomendado):** marque **Overlay → Usar XWayland** e reinicie o app. Na inicialização, ele define `GDK_BACKEND=x11` e passa a rodar via XWayland, onde posição, trava e sempre no topo funcionam. A opção só tem efeito numa sessão Wayland e quando `GDK_BACKEND` não foi definido por você.
- **Manualmente:**

  ```sh
  GDK_BACKEND=x11 pnpm tauri dev
  ```

Via XWayland, a janela principal também roda em X11 e pode ficar levemente borrada com escala fracionária. Aplicativos em tela cheia (vídeos, F11) ficam acima do overlay mesmo assim.

**Notificações** são o canal principal no GNOME Wayland e funcionam sem configuração extra.

**Tray:** veja a instalação da extensão AppIndicator em [Requisitos](#dependências-de-sistema-linux). Sem ela, o tray não aparece.

## Dados e privacidade

- **Rede:** a única conexão externa é com `api.anthropic.com`, usando o seu próprio token.
- **Credenciais:** só leitura, a cada consulta. Nada é copiado nem salvo pelo app.
- **Configurações:** ficam em um arquivo local, no diretório de dados do app:

  ```
  ~/.local/share/dev.jr.aiusage/ai-usage.json
  ```

  | Chave | Conteúdo |
  |---|---|
  | `settings` | Intervalo de consulta, estado e preferências do overlay, posição salva, opção de XWayland |
  | `rules` | Lista de regras |
  | `ruleState` | Para cada regra e janela, o ciclo de reset em que ela já disparou |

  Se o arquivo estiver danificado, o app inicia com os padrões em vez de travar. Uma regra ilegível é ignorada sem descartar as outras.

## Arquitetura

O núcleo em Rust é a fonte única da verdade: consulta, regras, persistência, notificações, overlay e tray. A interface apenas chama comandos e escuta eventos.

```
ai_usage/
├─ index.html, overlay.html     # uma entrada do Vite por janela
├─ src/                         # interface (Vue 3 + TypeScript)
│  ├─ main/App.vue              # janela principal
│  ├─ overlay/OverlayApp.vue    # overlay
│  ├─ components/               # ProviderCard, UsageBar, RuleList, RuleForm,
│  │                            # SettingsPanel, OverlaySettings
│  ├─ composables/              # useUsage, useRules, useSettings, useOverlayAlerts, ...
│  └─ lib/                      # types.ts (espelha o serde do Rust), api.ts, events.ts, format.ts
└─ src-tauri/
   ├─ icons/app-icon.svg        # fonte de todos os ícones
   └─ src/
      ├─ lib.rs                 # inicialização, plugins, comandos
      ├─ domain.rs, error.rs    # UsageWindow, ProviderStatus, ProviderError
      ├─ providers/             # trait UsageProvider + ClaudeOAuthProvider
      ├─ poller.rs              # consulta periódica e "atualizar agora"
      ├─ rules/                 # modelo e motor de regras (função pura + testes)
      ├─ store.rs, settings.rs  # persistência e configurações
      ├─ alerts.rs              # despacho para notificação e overlay
      ├─ overlay.rs             # janela, fila de alertas, posicionamento
      ├─ tray.rs, windows.rs    # tray e comportamento das janelas
      └─ cli.rs                 # flags de linha de comando
```

### Modelo de dados

```rust
UsageWindow { provider, kind: rolling_5h | rolling_7d, utilization /* 0–100 */, resets_at }

trait UsageProvider {
    fn id(&self) -> ProviderId;
    async fn fetch(&self) -> Result<Vec<UsageWindow>, ProviderError>;
}
```

Novos providers (como o Gemini) só precisam implementar `UsageProvider`.

### Comandos e eventos

| Comando | Descrição |
|---|---|
| `get_snapshot`, `refresh_now` | Estado atual do uso; forçar consulta |
| `get_settings`, `update_settings` | Configurações; altera só o intervalo de consulta |
| `list_rules`, `create_rule`, `update_rule`, `delete_rule` | CRUD de regras |
| `set_overlay_visible`, `update_overlay_prefs`, `set_overlay_anchor` | Overlay |
| `get_overlay_alerts`, `dismiss_overlay_alert` | Fila de alertas do overlay |
| `set_use_xwayland` | Opção de XWayland (vale após reiniciar) |

| Evento | Payload |
|---|---|
| `usage://updated` | `UsageSnapshot`, após cada consulta |
| `settings://changed` | `Settings`, após qualquer mudança (inclusive pelo tray) |
| `overlay://alerts` | Fila de alertas do overlay |

## Desenvolvimento

```sh
# Testes do núcleo (motor de regras, parser do Claude, configurações, overlay, CLI)
cd src-tauri && cargo test

# Lint e formatação do Rust
cargo clippy --all-targets
cargo fmt --check

# Checagem de tipos e build do front
cd .. && pnpm build
```

O motor de regras é uma função pura (`rules::engine::evaluate`): recebe as regras, as janelas, o estado e o horário atual, e devolve os alertas. Isso permite testar cruzamento de limiar, reset de ciclo, variação do horário de reset e reinício sem depender de tempo real.

**Ícones:** edite `src-tauri/icons/app-icon.svg` e regenere todos os tamanhos com:

```sh
pnpm tauri icon src-tauri/icons/app-icon.svg
```

**Commits** seguem [Conventional Commits](https://www.conventionalcommits.org/) (`feat`, `fix`, `refactor`, `docs`...), em inglês, um assunto por commit, com corpo explicando o que mudou e por quê.

## Solução de problemas

| Sintoma | Causa provável e solução |
|---|---|
| "Credenciais do Claude Code não encontradas" | O Claude Code não está instalado ou sem login. Rode `claude` e faça login. |
| "Token do Claude expirado" ou "recusado (HTTP 401)" | Só o Claude Code renova o token, o que acontece quando ele é usado. Abra e use o Claude Code, depois clique em **Atualizar**. |
| "Limite de consultas … (HTTP 429)" | Muitas consultas seguidas. O app tenta de novo na próxima consulta; aumente o intervalo se acontecer com frequência. |
| O ícone do tray não aparece | Falta a extensão AppIndicator do GNOME; ver [Requisitos](#dependências-de-sistema-linux). |
| O overlay não fica por cima ou não respeita a posição | Sessão Wayland. Ative **Usar XWayland** e **reinicie o app**. |
| O overlay abre colado no canto superior esquerdo | Posição salva inválida. Escolha uma posição na lista **Posição**. |
| Notificações aparecem com o nome "ai_usage" | Em desenvolvimento, o GNOME usa o nome do executável. Em um build instalado, usa o nome do atalho do app. |

## Roadmap

- [x] Provider do Claude, tela de status e consulta periódica
- [x] Motor de regras, notificações e persistência
- [x] Overlay com posição, trava, opacidade e escolha do que exibir
- [x] Linha de comando (`--toggle`, `--overlay`) e opção de XWayland
- [ ] Provider do Gemini/Antigravity
- [ ] Inicialização automática com o sistema
- [ ] Windows
- [ ] Android
