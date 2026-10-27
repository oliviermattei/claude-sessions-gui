# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Mises à jour automatiques** : l'app vérifie les GitHub Releases au démarrage
  puis toutes les 6 h, et propose « Restart to update » quand une nouvelle
  version est publiée. Le paquet est vérifié par signature avant installation.
  L'exe portable Windows ne se met pas à jour seul. Les installations en 0.3.0
  et avant doivent installer à la main la première version qui embarque
  l'updater.

### Fixed

- Le pied de page affiche la vraie version de l'app au lieu de « v0.1 » figé.
- Cliquer un lien dans un transcript ouvre désormais le navigateur système au
  lieu de remplacer l'app par la page (sans retour possible). Les liens non
  web (`javascript:`…) sont ignorés, et une Content Security Policy est en place.
- L'interface ne se fige plus pendant le scan des sessions : les commandes du
  backend tournent hors du thread principal.
- Déplacer une session vers un projet situé sur un autre disque ne perd plus
  ses sous-agents ; si leur copie échoue, le déplacement est annulé.
- « Resume » refuse un dossier ou un id de session contenant des caractères
  shell (guillemets, `&`, `|`, `$`…) qui pourraient injecter une commande, et
  propose « Copy command » à la place.

## [0.3.0] - 2026-08-18

### Added

- Action **« Keep beyond cleanup »** dans le menu contextuel : exempte une session
  du nettoyage automatique de Claude Code, qui supprime tout transcript dont la
  date de modification dépasse `cleanupPeriodDays` (14 jours en réglages managés),
  sidecar `subagents/*.jsonl` compris. Deux mécanismes appliqués ensemble : un
  **keep-alive** qui rafraîchit la date du transcript et de son sidecar au
  démarrage de l'app puis toutes les 6 h — la session reste en place et
  reprenable —, et une **copie d'archive** dans un coffre hors de
  `~/.claude/projects`, qui la préserve même si l'app n'est pas ouverte pendant
  une période entière. Une session qui n'existe plus que dans le coffre est
  listée comme archivée et peut être restaurée, ce qui se fait aussi
  automatiquement quand on la reprend. Libérer une session lui rend sa date
  d'origine : le nettoyage s'applique de nouveau comme si elle n'avait jamais
  été retenue.
- Panneau **Retention** dans les Réglages : `cleanupPeriodDays` effectif et
  emplacement du coffre.

### Changed

- Vue compacte : les métadonnées (messages, date) passent dans l'infobulle de la
  ligne pour laisser la place au titre, désormais tronqué en ellipse au lieu
  d'élargir la colonne.
- Le tampon keep-alive ne remonte jamais dans l'interface : la date de dernière
  activité réelle est conservée à part, et une modification que l'app n'a pas
  écrite est reconnue comme une vraie activité.
- Mise à jour des dépendances internes : Vite 6→8, Vue, `vue-tsc`,
  `@vitejs/plugin-vue`, `marked`, `serde`/`serde_json` (backend Rust). Aucun
  changement fonctionnel visible.

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

[Unreleased]: https://github.com/oliviermattei/claude-sessions-gui/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/oliviermattei/claude-sessions-gui/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/oliviermattei/claude-sessions-gui/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/oliviermattei/claude-sessions-gui/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/oliviermattei/claude-sessions-gui/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/oliviermattei/claude-sessions-gui/releases/tag/v0.1.0
