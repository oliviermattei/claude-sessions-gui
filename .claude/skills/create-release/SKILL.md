---
name: create-release
description: >
  Automatise la préparation d'une release pour claude-sessions. Sur "Create
  Release" (ou "prépare une release", "nouvelle version", "release x.y.z"),
  calcule le diff entre HEAD et le dernier tag poussé, propose un bump de
  version (semver), génère la section CHANGELOG au format Keep a Changelog, met à
  jour les liens de comparaison, puis (après validation) commit + tag + push. Le
  push du tag `v*` déclenche le workflow de build multi-plateforme.
---

# Create Release

Prépare et publie une release. Le repo suit **Keep a Changelog** + **SemVer** ;
le workflow `.github/workflows/build.yml` lit `CHANGELOG.md` **au moment du tag**
et utilise la section `## [version]` comme corps de la GitHub Release (au-dessus
des assets). Donc le CHANGELOG doit être commité **avant** le tag.

Exécute les étapes dans l'ordre. **Ne push jamais sans confirmation explicite**
de l'utilisateur (étape 6).

## 1. État du repo & dernier tag

```bash
git fetch --tags --quiet
git status --short                               # doit être clean (sinon prévenir)
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
echo "Dernier tag: ${LAST_TAG:-<aucun>}"
DATE=$(date +%F)                                 # ex 2026-08-01
```

- Si working tree pas clean → prévenir l'utilisateur, proposer de commit/stash avant.
- Si aucun tag → première release : range = tout l'historique (`RANGE=""` → `git log`), version proposée par défaut `0.1.0`.

## 2. Diff depuis le dernier tag

```bash
RANGE="${LAST_TAG:+$LAST_TAG..}HEAD"
git log $RANGE --no-merges --pretty=format:'%s|%h'
```

Parse chaque sujet en Conventional Commits `type(scope): desc`.

## 3. Proposer le bump de version (SemVer)

Depuis les types de commits :

| Signal dans les commits                          | Bump          |
| ------------------------------------------------ | ------------- |
| `!` ou `BREAKING CHANGE` (et version ≥ 1.0.0)    | **major**     |
| `!` ou `BREAKING CHANGE` (et version < 1.0.0)    | **minor** (convention pre-1.0) |
| au moins un `feat`                               | **minor**     |
| sinon (`fix`/`perf`/…)                           | **patch**     |

Calcule `NEW_VERSION` à partir de `LAST_TAG` (sans le `v`). **Propose-la à
l'utilisateur et laisse-le corriger** avant de continuer.

## 4. Classer les commits en sections Keep a Changelog

Mappe les types vers les sections, dans cet ordre : `Added`, `Changed`,
`Deprecated`, `Removed`, `Fixed`, `Security`.

| type commit                         | section    |
| ----------------------------------- | ---------- |
| `feat`                              | Added      |
| `fix`                               | Fixed      |
| `perf`, `refactor` (user-facing)    | Changed    |
| `revert`                            | Removed    |
| commit avec impact sécurité         | Security   |
| `docs`, `chore`, `ci`, `build`, `test`, `style` | **exclure** (non user-facing) |

Règles de rédaction :
- Une puce par changement, formulée pour l'utilisateur final (pas le message brut).
- Retire le préfixe `type(scope):`. Garde le scope entre parenthèses seulement s'il aide.
- Fusionne aussi les entrées déjà présentes sous `## [Unreleased]` de `CHANGELOG.md` (ne les perds pas).
- N'inclus pas de section vide.

## 5. Éditer `CHANGELOG.md`

Transforme le haut du fichier ainsi (garde un `## [Unreleased]` vide au-dessus) :

```markdown
## [Unreleased]

## [NEW_VERSION] - DATE

### Added
- …

### Fixed
- …
```

Puis mets à jour les **link definitions** en bas du fichier. Remplace la ligne
`[Unreleased]` et insère la nouvelle ligne de version juste au-dessus de
l'ancienne :

```markdown
[Unreleased]: https://github.com/oliviermattei/claude-sessions-gui/compare/vNEW_VERSION...HEAD
[NEW_VERSION]: https://github.com/oliviermattei/claude-sessions-gui/compare/vLAST_TAG...vNEW_VERSION
[LAST_TAG_sans_v]: …            # (les lignes existantes restent)
```

Pour la toute première release (pas de `LAST_TAG`), la ligne version pointe vers
le tag :
`[NEW_VERSION]: https://github.com/oliviermattei/claude-sessions-gui/releases/tag/vNEW_VERSION`.

**Montre le diff de `CHANGELOG.md` à l'utilisateur et demande validation.**
N'édite ni `package.json`, ni `Cargo.toml`, ni `tauri.conf.json` : le workflow
synchronise la version depuis le tag.

## 6. Commit, tag, push (APRÈS confirmation explicite)

Le push est une action externe irréversible. Récapitule (version, nb de commits,
sections) et demande une confirmation claire avant de lancer :

```bash
git add CHANGELOG.md
git commit -m "docs: changelog vNEW_VERSION"
git push
git tag vNEW_VERSION
git push origin vNEW_VERSION          # ← déclenche build.yml (release multi-OS)
```

Puis indique le lien de suivi :
`https://github.com/oliviermattei/claude-sessions-gui/actions/workflows/build.yml`
et la page release finale.

## Garde-fous

- Jamais de `git push` (branche ou tag) sans un "oui" explicite à l'étape 6.
- Refuse de tagger si le working tree n'est pas clean.
- Refuse un `NEW_VERSION` qui n'incrémente pas `LAST_TAG` (ou déjà taggé) — signale-le.
- Si le header de section ne correspond pas exactement à `## [NEW_VERSION]`, le
  workflow retombera sur un placeholder : vérifie l'exactitude du format.
