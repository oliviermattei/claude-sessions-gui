# Contributing to Claude Sessions

Thanks for taking the time to contribute! This document explains how to set up
the project, the conventions we follow, and how to get a change merged.

By participating in this project you agree to abide by our
[Code of Conduct](./CODE_OF_CONDUCT.md).

## Table of contents

- [Ways to contribute](#ways-to-contribute)
- [Development setup](#development-setup)
- [Project layout](#project-layout)
- [Coding conventions](#coding-conventions)
- [Commit messages](#commit-messages)
- [Pull requests](#pull-requests)
- [Releasing](#releasing)

## Ways to contribute

- **Report a bug** — open a [bug report](https://github.com/oliviermattei/claude-sessions-gui/issues/new?template=bug_report.yml).
- **Request a feature** — open a [feature request](https://github.com/oliviermattei/claude-sessions-gui/issues/new?template=feature_request.yml).
- **Improve docs** — typo fixes and clarifications are always welcome.
- **Send code** — pick an open issue (comment first so we don't duplicate work),
  or open one describing what you plan to build before writing a large change.

For anything non-trivial, please open an issue first so we can agree on the
approach before you invest time.

## Development setup

### Prerequisites

- **Node 18+** and **pnpm** (this repo pins `pnpm@10` via the `packageManager`
  field — run with Corepack or install pnpm globally)
- **Rust** stable toolchain (`rustup toolchain install stable`)
- Platform dependencies for Tauri 2 — see the
  [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/).
  - **Linux**: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`
  - **Windows**: WebView2 (preinstalled on Windows 10/11)
  - **macOS**: Xcode Command Line Tools

### Run

```bash
pnpm install
pnpm tauri dev      # hot-reloading desktop app
```

### Build

```bash
pnpm tauri build    # produces installers under src-tauri/target/release/bundle
```

### Checks before you push

```bash
pnpm build                     # vue-tsc typecheck + vite build (frontend)
cargo fmt --manifest-path src-tauri/Cargo.toml --all
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

CI runs the same checks on every pull request, so running them locally saves a
round-trip.

## Project layout

```
src-tauri/src/lib.rs   # list_sessions + run_command Tauri commands (Rust backend)
src/App.vue            # main list view orchestration
src/components/        # Sidebar, TopBar, SessionRow, EmptyState, Toast, SettingsModal
src/composables/       # useSessions, useSettings, useTheme
src/lib/               # sort defs + date/highlight helpers
src/styles/theme.css   # design tokens (dark/light)
```

## Coding conventions

- **Vue**: Composition API with `<script lang="ts" setup>`.
- **TypeScript**: keep `pnpm build` (which runs `vue-tsc --noEmit`) green — no
  type errors.
- **Rust**: format with `cargo fmt`; keep `cargo clippy` warning-free.
- **No silent mutation**: browsing/search/sort stay read-only. Writes only happen
  from explicit user actions — rename/color _append_ Claude-native rows, move
  rewrites into a copy then removes the original, delete goes to the OS trash
  (recoverable). Never modify transcript content in place, and never touch a file
  the user didn't act on. Any change touching the filesystem must preserve this.

## Commit messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<optional scope>): <short summary>
```

Common types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`, `ci`,
`chore`. Example: `feat(sidebar): add per-project session count`.

## Pull requests

1. Fork the repo and create a branch from `master`.
2. Make your change, keeping commits focused.
3. Ensure the checks above pass.
4. Open a PR against `master` and fill in the template.
5. Link any related issue (`Closes #123`).

A maintainer will review; please be responsive to feedback. Small, focused PRs
get merged fastest.

## Releasing

Releases are automated. Maintainers only. Steps:

1. **Update the changelog.** In [`CHANGELOG.md`](./CHANGELOG.md), move the
   `## [Unreleased]` items under a new dated section header matching the version
   you're about to tag:

   ```markdown
   ## [0.2.0] - 2026-08-01
   ```

   Leave a fresh, empty `## [Unreleased]` above it, and add a `[0.2.0]:` compare
   link at the bottom. The release workflow extracts the section whose header
   matches the tag version verbatim, so the header **must** read `## [0.2.0]`.

2. **Tag and push.** Pushing a `v*` tag (e.g. `v0.2.0`) triggers
   [`.github/workflows/build.yml`](./.github/workflows/build.yml), which:
   - extracts the `## [0.2.0]` section from `CHANGELOG.md` and uses it as the
     GitHub Release body (so the changelog appears right above the downloadable
     assets), with auto-generated commit notes appended below it;
   - builds installers for Windows, macOS (arm64 + x64) and Linux
     (x86_64 + arm64) and attaches them to that release.

If no `## [<version>]` section exists for the tag, the release is still created
but its body falls back to a "no changelog entry" placeholder — so don't skip
step 1.
