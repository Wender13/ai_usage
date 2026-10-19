<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="AI Usage icon" width="96" height="96">
</p>

<h1 align="center">AI Usage</h1>

<p align="center">
  Desktop monitor for your Claude usage limits, with notifications and an always-visible overlay.
</p>

---

AI Usage runs in the background from the system tray. It periodically checks how much of your Claude limits you have used and alerts you according to rules you define. It tracks the 5-hour session and the weekly window. Alerts can arrive as system notifications, in a compact overlay window, or both. The app is available in **English** and **Portuguese**.

> **Status:** the **Claude** provider is complete. The **Gemini/Antigravity** provider and **launch at login** are still in progress (see the [Roadmap](#roadmap)).

## Contents

- [Features](#features)
- [How it works](#how-it-works)
- [Requirements](#requirements)
- [Installation](#installation)
- [Usage](#usage)
- [Linux: GNOME, Wayland and the tray](#linux-gnome-wayland-and-the-tray)
- [Data and privacy](#data-and-privacy)
- [Architecture](#architecture)
- [Development](#development)
- [Troubleshooting](#troubleshooting)
- [Roadmap](#roadmap)

## Features

- **Usage status:** progress bar, percentage used and time until reset for each limit window (5-hour session and weekly).
- **Configurable alert rules:**
  - **Usage threshold:** alerts when a window goes past X%, at most once per reset cycle.
  - **Periodic summary:** a usage summary every N minutes.
  - **Before reset:** alerts N minutes before a window resets.
- **Channels per rule:** system notification, overlay or both.
- **Compact overlay:** borderless, transparent, always-on-top window with the percentages.
  - Six preset positions, free positioning by dragging, and a position lock.
  - Adjustable background opacity.
  - Choice of what to show: provider name, window name, percentage and time until reset, plus each window individually.
- **Tray:** open the window, turn the overlay on or off, refresh now and quit.
- **Command line:** `--toggle` and `--overlay`, to control the overlay from a keyboard shortcut.
- **English and Portuguese:** follows the system language, with a manual choice in the settings.
- **Single instance:** opening the app again just focuses the running instance, with no duplicate tray icon or notifications.

## How it works

### Where the numbers come from

The app does not compute the limits; Anthropic's servers do. On each check, the app:

1. Reads Claude Code's login token from `~/.claude/.credentials.json`.
2. Calls `GET https://api.anthropic.com/api/oauth/usage`, the same endpoint used by Claude Code's `/usage` command.
3. Receives, for each limit window, the percentage used and the reset time:

   ```json
   "five_hour": { "utilization": 43.0, "resets_at": "2026-10-19T01:00:00.42+00:00" },
   "seven_day": { "utilization": 40.0, "resets_at": "2026-10-22T22:00:00.42+00:00" }
   ```

   - **`utilization`** is the percentage already used, from 0 to 100. What is left is 100 minus that value. The API only reports percentages, not messages or tokens.
   - **`resets_at`** is when the window resets. The time left is computed by the UI (`resets_at − now`) and refreshed every 30 seconds.

Usage is counted per **account**: messages on claude.ai, in the desktop app and in Claude Code all add to the same percentage.

> The endpoint is not publicly documented. The parser is therefore tolerant: it ignores unknown fields and null values, and if the format changes, it shows a clear error instead of crashing.

### Token

- The credentials file is **read on every check** and **never written**. When Claude Code renews the token, the app picks up the new one automatically.
- If the token has already expired (`expiresAt`), the app skips the request and shows "Claude token expired. Open Claude Code to renew it."
- The token is never logged, displayed or sent anywhere else.

### Periodic check

Checks run in the Rust core, whether or not the window is open. The default interval is 3 minutes, with a minimum of 1, and it can be configured. "Refresh" (in the window or the tray) forces an immediate check.

If a check fails (network, expired token, etc.), the screen keeps the last valid values, dimmed, next to the error message.

### Rules

- **Reset cycle.** Each window has a cycle identified by its `resets_at`. The API shifts this time by about a second on every response, so reset times less than 10 minutes apart count as the same cycle.
- **Threshold.** Fires when usage crosses the limit (`previous < X ≤ current`), at most once per cycle. The first reading of a cycle counts as a crossing: if the app starts with usage already above the limit, you are alerted.
- **Persistence.** The cycle in which each rule fired is saved, so restarting the app does not repeat alerts from the current cycle.
- **Editing.** Editing, enabling or disabling a rule re-arms it, so it may fire again in the current cycle.
- **Before reset.** Windows at 0% usage are skipped, since their reset changes nothing.
- **Precision.** "Periodic summary" and "before reset" rules are evaluated on each check, so they are as precise as the check interval.

## Requirements

| Requirement | Version / notes |
|---|---|
| Operating system | Linux (tested on Fedora 44 with GNOME 50 on Wayland). Windows and Android are planned. |
| [Claude Code](https://docs.claude.com/en/docs/claude-code) | Installed and logged in (this creates `~/.claude/.credentials.json`) |

To build from source you also need:

| Requirement | Version / notes |
|---|---|
| Rust | Stable, with edition 2024 support (1.85 or newer) |
| Node.js | `^20.19` or `>= 22.12` (required by Vite 8) |
| pnpm | Frontend package manager |
| Tauri system dependencies | See below |

### System dependencies (Linux)

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

The official, up-to-date list is in the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

**Tray on GNOME (Fedora):** GNOME does not show tray icons without an extension.

```sh
sudo dnf install gnome-shell-extension-appindicator
gnome-extensions enable appindicatorsupport@rgcjonas.gmail.com
```

Then log out and back in. Ubuntu ships with this extension enabled.

### Main libraries

| Layer | Libraries |
|---|---|
| Core (Rust) | `tauri` 2, `tauri-plugin-notification`, `tauri-plugin-store`, `tauri-plugin-single-instance`, `reqwest` (rustls TLS), `tokio`, `chrono`, `serde`, `thiserror`, `uuid`, `sys-locale` |
| UI | Vue 3 (Composition API, `<script setup>`), TypeScript, Vite 8, `@tauri-apps/api` |

## Installation

### From a release

Download the package for your distribution from the [Releases](https://github.com/Wender13/ai_usage/releases) page:

| Package | Install |
|---|---|
| `.rpm` (Fedora) | `sudo dnf install ./ai-usage-0.1.0-1.x86_64.rpm` |
| `.deb` (Ubuntu 20.04+ / Debian 11+) | `sudo apt install ./ai-usage_0.1.0_amd64.deb` |
| `.AppImage` (any distribution) | `chmod +x ai-usage_0.1.0_amd64.AppImage && ./ai-usage_0.1.0_amd64.AppImage` |

The `.rpm` and `.deb` install the `ai_usage` command and a menu entry. The AppImage bundles its libraries and runs without installing. To verify a download, put `SHA256SUMS` next to it and run `sha256sum -c --ignore-missing SHA256SUMS`.

### From source

```sh
git clone git@github.com:Wender13/ai_usage.git
cd ai_usage
pnpm install
```

**Development** (reloads the UI and rebuilds the Rust core on save):

```sh
pnpm tauri dev
```

**Production build** (packages are written to `src-tauri/target/release/bundle/`):

```sh
pnpm tauri build
```

## Usage

### Main window

Closing the window only hides it; the app keeps running in the tray. To exit, use **Quit** in the tray menu. The window has four areas:

- **Status:** one card per provider, with each limit window (percentage, bar and "resets in 2h 10min (10:00 PM)"). The bar turns yellow from 70% and red from 90%.
- **Rules:** a list with an automatic description (e.g. "When Claude · Session (5h) reaches 80%"), a checkbox to enable or disable, **Edit** and **Delete**. **New rule** opens the form.
- **Settings:** check interval in minutes, and language.
- **Overlay:** all overlay options, described below.

### Creating rules

In the form, choose:

| Field | Options |
|---|---|
| Type | Usage threshold · Periodic summary · Before reset |
| Provider | Any · Claude |
| Window | Any · Session (5h) · Weekly (7 days) |
| Threshold (%) / minutes | Depends on the type |
| Channel | Notification · Overlay · Notification and overlay |

Some examples:

- **Alert when the session goes past 80%:** Usage threshold, Claude, Session (5h), 80%, Notification.
- **Reminder to use what is left before the weekly reset:** Before reset, Claude, Weekly, 120 minutes.
- **Follow consumption during the day:** Periodic summary, every 60 minutes.

With "Any", the rule applies to every matching window, and each window is tracked separately.

### Overlay

Turn it on in **Overlay → Show overlay**, from the tray or from the command line. Options:

| Option | Effect |
|---|---|
| Position | Top or bottom, each left, center or right, inside the monitor's work area. With a bottom anchor, the overlay grows upwards. |
| Dragging | Drag the overlay anywhere: the position is saved and restored on start. Choosing a position from the list snaps back to it. |
| Lock position | Prevents dragging. If something moves the window, it returns to the saved place when it reappears. |
| Background opacity | From 0% (text only) to 100% |
| Information shown | Provider name, window name (5h, 7d), percentage, time until reset |
| Windows shown | Turns each window on or off (e.g. show only the weekly one) |
| Use XWayland | Linux only; see [below](#linux-gnome-wayland-and-the-tray) |

Alerts from overlay-channel rules appear as a banner at the top of the overlay. Click to dismiss; when there are more, "+N more alert(s)" is shown. If the overlay is off, it appears only while there are alerts.

### Tray

The menu has **Open window**, **Overlay** (on/off), **Refresh now** and **Quit**. On Linux the tray does not receive clicks, so every action is in the menu.

### Command line

| Command | App not running | App running |
|---|---|---|
| `ai_usage` | Opens the main window | Focuses the main window |
| `ai_usage --overlay` | Starts with only the tray and the overlay | Turns the overlay on |
| `ai_usage --toggle` | Starts with only the tray and the overlay on | Turns the overlay on or off |
| `ai_usage --hidden` | Starts with only the tray | Nothing |

With the app running, the command does not start a second copy: it forwards the flag to the running instance and exits (about 0.1 s). When several flags are given, the one that does the most wins: `--toggle`, then `--overlay`, then `--hidden`.

### Keyboard shortcut for the overlay (GNOME)

1. Open **Settings → Keyboard → View and Customize Shortcuts → Custom Shortcuts → +**.
2. Use `ai_usage --toggle` as the command (for a source build, the full path, e.g. `/path/to/ai_usage/src-tauri/target/release/ai_usage --toggle`).
3. Choose the key combination (e.g. <kbd>Super</kbd>+<kbd>Shift</kbd>+<kbd>U</kbd>).

### Language

The app follows the system language: Portuguese for any `pt` locale, English otherwise. To choose manually, use **Settings → Language** (Automatic, English or Português). The change applies at once to the UI, the tray, notifications and error messages, with no restart. Dates and times follow your system locale.

## Linux: GNOME, Wayland and the tray

**Overlay on GNOME with Wayland.** GNOME (Mutter) ignores "always on top" and positioning for regular windows on Wayland. In that case, the overlay shows up, but not on top and wherever the compositor places it. The app also cannot tell where the window is, so drags in that mode are not saved. There are two ways to fix this:

- **From the app (recommended):** check **Overlay → Use XWayland** and restart the app. On start, it sets `GDK_BACKEND=x11` and runs through XWayland, where position, lock and always-on-top work. The option only takes effect in a Wayland session and when you have not set `GDK_BACKEND` yourself.
- **Manually:**

  ```sh
  GDK_BACKEND=x11 pnpm tauri dev
  ```

Through XWayland, the main window also runs on X11 and may look slightly blurry with fractional scaling. Fullscreen applications (videos, F11) still stay above the overlay.

**Notifications** are the main channel on GNOME Wayland and work with no extra setup.

**Tray:** see the AppIndicator extension setup in [Requirements](#system-dependencies-linux). Without it, the tray does not show up.

## Data and privacy

- **Network:** the only external connection is to `api.anthropic.com`, with your own token.
- **Credentials:** read only, on each check. The app never copies or stores them.
- **Settings:** stored in a local file in the app data directory:

  ```
  ~/.local/share/dev.jr.aiusage/ai-usage.json
  ```

  | Key | Contents |
  |---|---|
  | `settings` | Check interval, overlay state and preferences, saved position, XWayland option, language |
  | `rules` | Rule list |
  | `ruleState` | For each rule and window, the reset cycle in which it already fired |

  If the file is damaged, the app starts with defaults instead of crashing. An unreadable rule is skipped without discarding the others.

## Architecture

The Rust core is the single source of truth: checks, rules, persistence, notifications, overlay and tray. The UI only calls commands and listens to events.

```
ai_usage/
├─ index.html, overlay.html     # one Vite entry per window
├─ src/                         # UI (Vue 3 + TypeScript)
│  ├─ main/App.vue              # main window
│  ├─ overlay/OverlayApp.vue    # overlay
│  ├─ components/               # ProviderCard, UsageBar, RuleList, RuleForm,
│  │                            # SettingsPanel, OverlaySettings
│  ├─ composables/              # useUsage, useRules, useSettings, useOverlayAlerts, ...
│  ├─ i18n/                     # en.ts (reference), pt.ts, t()
│  └─ lib/                      # types.ts (mirrors the Rust serde output), api.ts, events.ts, format.ts
└─ src-tauri/
   ├─ icons/app-icon.svg        # source of every icon
   └─ src/
      ├─ lib.rs                 # startup, plugins, commands
      ├─ domain.rs, error.rs    # UsageWindow, ProviderStatus, ProviderError
      ├─ providers/             # UsageProvider trait + ClaudeOAuthProvider
      ├─ poller.rs              # periodic check and "refresh now"
      ├─ rules/                 # rule model and engine (pure function + tests)
      ├─ store.rs, settings.rs  # persistence and settings
      ├─ alerts.rs              # dispatch to notification and overlay
      ├─ overlay.rs             # window, alert queue, placement
      ├─ tray.rs, windows.rs    # tray and window behavior
      ├─ i18n.rs                # language detection and core texts
      └─ cli.rs                 # command-line flags
```

### Data model

```rust
UsageWindow { provider, kind: rolling_5h | rolling_7d, utilization /* 0–100 */, resets_at }

trait UsageProvider {
    fn id(&self) -> ProviderId;
    async fn fetch(&self) -> Result<Vec<UsageWindow>, ProviderError>;
}
```

New providers (such as Gemini) only need to implement `UsageProvider`.

### Commands and events

| Command | Description |
|---|---|
| `get_snapshot`, `refresh_now` | Current usage; force a check |
| `get_settings`, `update_settings` | Settings; changes only the check interval |
| `list_rules`, `create_rule`, `update_rule`, `delete_rule` | Rule CRUD |
| `set_overlay_visible`, `update_overlay_prefs`, `set_overlay_anchor` | Overlay |
| `get_overlay_alerts`, `dismiss_overlay_alert` | Overlay alert queue |
| `set_use_xwayland` | XWayland option (applies after a restart) |
| `get_language`, `set_language` | Current language and locale; manual choice |

| Event | Payload |
|---|---|
| `usage://updated` | `UsageSnapshot`, after each check |
| `settings://changed` | `Settings`, after any change (including from the tray) |
| `overlay://alerts` | Overlay alert queue |
| `language://changed` | Language and locale, after a language change |

## Development

```sh
# Core tests (rule engine, Claude parser, settings, overlay, CLI, i18n)
cd src-tauri && cargo test

# Rust lint and formatting
cargo clippy --all-targets
cargo fmt --check

# Type checking and frontend build
cd .. && pnpm build
```

The rule engine is a pure function (`rules::engine::evaluate`): it takes the rules, the windows, the state, the current time and the language, and returns the alerts. This makes threshold crossing, cycle reset, reset-time jitter and restarts testable without real time.

**Translations:** the UI texts live in `src/i18n/en.ts` (the reference) and `src/i18n/pt.ts`. TypeScript rejects a language file that is missing a key. The core texts (notifications, errors, tray) live in `src-tauri/src/i18n.rs`, where exhaustive `match` blocks reject a missing translation at compile time.

**Icons:** edit `src-tauri/icons/app-icon.svg` and regenerate every size with:

```sh
pnpm tauri icon src-tauri/icons/app-icon.svg
```

**Commits** follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat`, `fix`, `refactor`, `docs`...), in English, one subject per commit, with a body explaining what changed and why.

## Troubleshooting

| Symptom | Likely cause and fix |
|---|---|
| "Claude Code credentials not found" | Claude Code is not installed or not logged in. Run `claude` and log in. |
| "Claude token expired" or "refused (HTTP 401)" | Only Claude Code renews the token, which happens when it is used. Open and use Claude Code, then click **Refresh**. |
| "Usage endpoint rate limit reached (HTTP 429)" | Too many checks in a row. The app retries on the next check; increase the interval if it happens often. |
| The tray icon does not show up | The GNOME AppIndicator extension is missing; see [Requirements](#system-dependencies-linux). |
| The overlay does not stay on top or ignores its position | Wayland session. Enable **Use XWayland** and **restart the app**. |
| The overlay opens stuck to the top-left corner | Invalid saved position. Choose a position in the **Position** list. |
| Notifications show the name "ai_usage" | In development, GNOME uses the executable name. An installed build uses the app's desktop entry name. |

## Roadmap

- [x] Claude provider, status screen and periodic check
- [x] Rule engine, notifications and persistence
- [x] Overlay with position, lock, opacity and display options
- [x] Command line (`--toggle`, `--overlay`) and XWayland option
- [x] English and Portuguese
- [ ] Gemini/Antigravity provider
- [ ] Launch at login
- [ ] Windows
- [ ] Android
