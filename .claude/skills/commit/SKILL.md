---
name: commit
description: >
  Crée un commit avec un message structuré comme le CHANGELOG. Sur "Claude
  Commit" (ou "commit ça", "fais un commit", "/claude-commit"), analyse le diff,
  rédige un titre court (Conventional Commits, < 150 caractères) puis un corps au
  format Keep a Changelog (### Added / Changed / Fixed / …) listant les
  modifications, montre le message pour validation, et commit.
---

# Claude Commit

Produit un commit dont le message reprend la structure du `CHANGELOG.md` :
titre court en tête, puis les changements groupés en sections Keep a Changelog.

Exécute dans l'ordre. **Ne commit qu'après validation** du message par
l'utilisateur.

## 1. Inspecter les changements

```bash
git status --short
git diff --stat HEAD
git diff HEAD          # contenu complet (staged + unstaged) pour comprendre le QUOI
```

Détermine le périmètre à committer :
- Si des fichiers sont **déjà staged**, committe ceux-là (ne touche pas au reste).
- Sinon, propose de tout stager (`git add -A`) et **confirme** avant.
- Working tree vide → rien à faire, préviens l'utilisateur.

## 2. Rédiger le titre (< 150 caractères)

Format **Conventional Commits** : `type(scope): résumé`.

- Types : `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`.
- `scope` optionnel = zone touchée (ex `sidebar`, `ci`, `release`).
- Résumé à l'impératif, concis. Le tout **strictement sous 150 caractères**
  (vise plutôt ~50-72 quand c'est possible).
- Choisis le `type` du changement dominant si le diff en mélange plusieurs.
- `!` après le type/scope si breaking change (ex `feat(api)!: ...`).

## 3. Rédiger le corps (structure CHANGELOG)

Ligne vide après le titre, puis les changements en sections Keep a Changelog,
dans cet ordre, en n'incluant que les sections non vides :

```
### Added
- Nouveau : …

### Changed
- Modifié : …

### Fixed
- Corrigé : …

### Removed
- Supprimé : …
```

Mapping intention → section : ajout de fonctionnalité → **Added** ; modif de
comportement/refactor visible/perf → **Changed** ; correction de bug → **Fixed** ;
suppression → **Removed** ; dépréciation → **Deprecated** ; sécurité → **Security**.

Règles :
- Une puce par modification notable. Vise l'exhaustivité (à peu près tous les
  changements du diff), mais regroupe les détails triviaux d'un même fichier.
- Formulé pour un lecteur (pas « edited file X »). Mentionne le fichier/zone si utile.
- Pas de section vide, pas de remplissage.
- Si le changement est vraiment trivial (une ligne, un typo), un titre seul suffit — pas de corps forcé.

## 4. Montrer & committer (après validation)

Affiche le message complet à l'utilisateur. Après son accord, committe via un
fichier de message (préserve le multi-ligne et les `#`) :

```bash
git commit -F - <<'EOF'
type(scope): résumé court

### Added
- …

### Fixed
- …
EOF
```

Puis affiche le hash + `git log -1 --stat` pour confirmer. **Ne push pas** (hors
scope de ce skill).

## Garde-fous

- Titre toujours < 150 caractères — tronque/reformule si dépassé.
- Ne stage pas au-delà de ce que l'utilisateur a confirmé.
- N'invente pas de changements absents du diff ; le corps décrit le diff réel.
- Pas de push, pas de tag (voir le skill `create-release` pour publier).
