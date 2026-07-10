# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] - 2026-07-10

### Fixed

- Build de l'AppImage **linux-arm64** : installation de `xdg-utils` (manquant sur
  l'image CI arm64) qui bloquait l'empaquetage. L'installeur arm64 est de nouveau
  produit et attaché à la release.

## [0.2.1] - 2026-07-10

### Changed

- Mise à jour des dépendances internes : `dirs` 5→6 (backend Rust), `vue-tsc`
  2→3, `@vitejs/plugin-vue` 5→6, et actions GitHub CI (checkout, setup-node,
  pnpm/action-setup, upload-artifact). Aucun changement fonctionnel visible.

## [0.2.0] - 2026-07-10

### Added

- Menu contextuel sur les sessions (clic droit) avec sous-menus.
- Renommer une session.
- Personnaliser la couleur d'accent d'une session.
- Déplacer une session vers un autre projet.
- Supprimer une session (mise à la corbeille).
- Open-source project scaffolding: `LICENSE` (MIT), `CONTRIBUTING.md`,
  `CODE_OF_CONDUCT.md`, `SECURITY.md`, issue/PR templates, CI workflow,
  Dependabot, and funding config.

## [0.1.0] - 2026-07-10

### Added

- Session list with title, project folder, last activity and message count.
- Live search by title with in-title highlighting.
- Grouping: none · by project · by date (Today / This week / This month /
  Earlier).
- Sorting: Recently Updated · Recently Modified · Recently Created · Most
  Messages · Title A–Z · Oldest First.
- Dark / light themes with a configurable accent color.
- Configurable Resume launchers (`run` / `uri` / `clipboard`) with presets for
  Warp, Windows Terminal, PowerShell, and clipboard copy.
- Multi-platform release workflow producing installers for Windows, macOS
  (arm64 + x64) and Linux (x86_64 + arm64).

[Unreleased]: https://github.com/oliviermattei/claude-sessions-gui/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/oliviermattei/claude-sessions-gui/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/oliviermattei/claude-sessions-gui/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/oliviermattei/claude-sessions-gui/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/oliviermattei/claude-sessions-gui/releases/tag/v0.1.0
