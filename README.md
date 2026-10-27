# Claude Sessions

[![CI](https://github.com/oliviermattei/claude-sessions-gui/actions/workflows/ci.yml/badge.svg)](https://github.com/oliviermattei/claude-sessions-gui/actions/workflows/ci.yml)
[![Release](https://github.com/oliviermattei/claude-sessions-gui/actions/workflows/build.yml/badge.svg)](https://github.com/oliviermattei/claude-sessions-gui/actions/workflows/build.yml)
[![Latest release](https://img.shields.io/github/v/release/oliviermattei/claude-sessions-gui?include_prereleases&sort=semver)](https://github.com/oliviermattei/claude-sessions-gui/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)](https://github.com/oliviermattei/claude-sessions-gui/releases/latest)

A lightweight local desktop app to browse, search and relaunch your Claude Code
sessions. Built with **Tauri 2** (Rust backend) + **Vue 3 / Vite** (TypeScript).

<p align="center">
  <a href="https://buymeacoffee.com/oliviermattei" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" height="48"></a>
</p>

<p align="center"><em>Free &amp; open source. If it saves you time, you can <a href="https://buymeacoffee.com/oliviermattei">buy me a coffee</a> ☕ — every little bit helps and keeps the project going.</em></p>

The app reads your transcripts under `<profile>/projects/**/*.jsonl`, for every
Claude config folder you use (`~/.claude`, `~/.claude-perso`…, see
[Profiles](#profiles)). Browsing,
search and sorting are strictly read-only. Only the explicit session actions
write, and never silently:

- **Rename** and **set color** _append_ Claude-native rows (`custom-title`,
  `agent-name`, `agent-color`) — the same rows Claude Code itself writes. Nothing
  in the existing transcript is rewritten or removed.
- **Move** (to another project, or to another profile) copies the session and
  everything it owns, rewriting the paths that point into what moved, then
  removes the original — all or nothing.
- **Delete** sends the `.jsonl` (and its sidecar folder) to the OS trash, so it
  stays recoverable.

The app never modifies transcript content in place and never touches a file you
didn't act on.

## Features

- **Session list** with title, project folder, last activity and message count.
- **Search** by title (live, with in-title highlighting).
- **Grouping**: none · by project · by date (Today / This week / This month / Earlier).
- **Sorting**: Recently Updated · Recently Modified · Recently Created · Most
  Messages · Title A–Z · Oldest First.
  _(The prototype's "Latest Summary" was dropped — Claude transcripts carry no
  summary timestamp to sort on.)_
- **Dark / light** themes with a configurable accent color.
- **Profiles**: one per Claude config folder, switched from the sidebar; move a
  session from one to another (see below).
- **Configurable Resume launchers** (see below).
- **Automatic updates**: installed apps check GitHub Releases at startup and
  offer to restart on a new version (signed and verified). The Windows
  _portable_ exe does not update itself — download the new one by hand.

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
Each launcher is a command template with these placeholders:

- `{cwd}` — the session's working directory
- `{id}` — the session UUID
- `{configDir}` — the session's profile folder (its `CLAUDE_CONFIG_DIR`)

A session of a non-default profile is resumed with `CLAUDE_CONFIG_DIR` set:
**run** launchers get it in their environment, and copied commands are prefixed
(`$env:CLAUDE_CONFIG_DIR='…'; ` in PowerShell by default, editable per profile).
Windows Terminal may open the new tab with the environment of an already open
window instead: if so, put `{configDir}` in the template, e.g.
`wt.exe -d "{cwd}" cmd /k "set CLAUDE_CONFIG_DIR={configDir}&& claude --resume {id}"`.

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

## Profiles

A profile is one Claude Code config folder — what `CLAUDE_CONFIG_DIR` points to,
`~/.claude` when it is unset. `~/.claude` and every `~/.claude-*` folder holding
a `projects/` folder are detected; **Settings → Profiles** renames, colours or
hides them, and adds any other folder. With two profiles or more, a switcher at
the bottom of the sidebar picks the one the window shows.

**Move to profile** (session context menu) moves a session to the same project in
another profile, with everything it owns there: transcript and sidecar folder
(in every project folder it has one), `/rewind` checkpoints (`file-history`),
`session-env`, its plan, its prompt-history rows and the pastes only they recall.
Paths pointing into what moved are rewritten
([ADR 0001](docs/adr/0001-move-rewrites-only-paths-to-moved-files.md)); the
source is deleted only once the copy is complete. A session open in a running
`claude` process is never moved.

## Project structure

```
src-tauri/src/lib.rs   # list_sessions + run_command commands
src-tauri/src/profiles.rs  # Claude config folders (detection, settings)
src-tauri/src/mover.rs     # moving a session between projects / profiles
src-tauri/src/retention.rs # keep sessions beyond Claude Code's cleanup
src/App.vue            # main list view orchestration
src/components/        # Sidebar, TopBar, SessionRow, EmptyState, Toast, SettingsModal
src/composables/       # useSessions, useProfiles, useSettings, useTheme
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

## License

[MIT](./LICENSE) © Olivier Mattei
