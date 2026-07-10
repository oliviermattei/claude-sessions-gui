# Claude Sessions

[![CI](https://github.com/oliviermattei/claude-sessions-gui/actions/workflows/ci.yml/badge.svg)](https://github.com/oliviermattei/claude-sessions-gui/actions/workflows/ci.yml)
[![Release](https://github.com/oliviermattei/claude-sessions-gui/actions/workflows/build.yml/badge.svg)](https://github.com/oliviermattei/claude-sessions-gui/actions/workflows/build.yml)
[![Latest release](https://img.shields.io/github/v/release/oliviermattei/claude-sessions-gui?include_prereleases&sort=semver)](https://github.com/oliviermattei/claude-sessions-gui/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)](https://github.com/oliviermattei/claude-sessions-gui/releases/latest)

A lightweight local desktop app to browse, search and relaunch your Claude Code
sessions. Built with **Tauri 2** (Rust backend) + **Vue 3 / Vite** (TypeScript).

The app scans `~/.claude/projects/**/*.jsonl` read-only and never mutates your
transcripts.

## Features

- **Session list** with title, project folder, last activity and message count.
- **Search** by title (live, with in-title highlighting).
- **Grouping**: none · by project · by date (Today / This week / This month / Earlier).
- **Sorting**: Recently Updated · Recently Modified · Recently Created · Most
  Messages · Title A–Z · Oldest First.
  _(The prototype's "Latest Summary" was dropped — Claude transcripts carry no
  summary timestamp to sort on.)_
- **Dark / light** themes with a configurable accent color.
- **Configurable Resume launchers** (see below).

## Prerequisites

- Node 18+ and pnpm
- Rust toolchain (stable)
- Windows: WebView2 (preinstalled on Windows 10/11)

## Develop

```bash
pnpm install
pnpm tauri dev
```

## Build

```bash
pnpm tauri build
```

## How "Resume" works

Resume launchers are fully editable in **Settings** (gear icon, bottom-left).
Each launcher is a command template with two placeholders:

- `{cwd}` — the session's working directory
- `{id}` — the session UUID

Presets:

| Launcher         | Kind      | Template                                                              |
| ---------------- | --------- | -------------------------------------------------------------------- |
| Warp             | uri       | `warp://action/new_tab?path={cwd}`                                   |
| Windows Terminal | run       | `wt.exe -d "{cwd}" claude --resume {id}`                             |
| PowerShell       | run       | `start powershell -NoExit -Command "cd '{cwd}'; claude --resume {id}"` |
| Copy command     | clipboard | `claude --resume {id}`                                                |

Launcher kinds:

- **run** — executed via the OS shell (`cmd /C` on Windows) as a detached process.
- **uri** — opened via the OS protocol handler (e.g. `warp://`); `{cwd}`/`{id}` are
  percent-encoded. If the template omits `{id}`, the resume command is also copied
  to the clipboard.
- **clipboard** — copied to the clipboard instead of run.

Launchers are add/removable and categorized in **Settings → Launchers**.

### Note on Warp

Warp on Windows can open a tab in a folder via its URI scheme, but it currently
has **no reliable way to auto-run a command** from outside the app: inline
commands aren't supported by the URI scheme, and the `warp://launch/<config>`
YAML path silently drops `commands` when triggered externally
([warpdotdev/warp#9007](https://github.com/warpdotdev/warp/issues/9007)).

So the Warp preset opens a tab in the session folder **and** copies
`claude --resume <id>` to your clipboard — just paste and press Enter. Tweak the
template once Warp ships inline-command support.

## Project structure

```
src-tauri/src/lib.rs   # list_sessions + run_command commands
src/App.vue            # main list view orchestration
src/components/        # Sidebar, TopBar, SessionRow, EmptyState, Toast, SettingsModal
src/composables/       # useSessions, useSettings, useTheme
src/lib/               # sort defs + date/highlight helpers
src/styles/theme.css   # design tokens (dark/light)
```

## Toolchain note (Windows GNU)

The Rust lib is built as `crate-type = ["rlib"]` only. Under the
`x86_64-pc-windows-gnu` toolchain, building the Tauri lib as a `cdylib` trips
`ld`'s "export ordinal too large" limit; a plain rlib linked statically into the
binary avoids it. (Mobile targets, which need cdylib/staticlib, are not a goal
here.)

## Contributing

Contributions are welcome! Please read the
[contributing guide](./CONTRIBUTING.md) for setup, conventions and the PR
process, and note our [Code of Conduct](./CODE_OF_CONDUCT.md). Found a security
issue? See [SECURITY.md](./SECURITY.md) — please report it privately.

Changes are tracked in [CHANGELOG.md](./CHANGELOG.md).

## Support

If you find this useful, you can
<a href="https://buymeacoffee.com/oliviermattei" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" height="40"></a>

## License

[MIT](./LICENSE) © Olivier Mattei
