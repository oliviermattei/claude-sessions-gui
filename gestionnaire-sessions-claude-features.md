# Gestionnaire de sessions Claude Code — Fonctionnalités par itération

Application desktop locale pour retrouver, consulter et relancer facilement ses sessions Claude Code (CLI).

Chaque itération est autonome et livrable seule : on peut s'arrêter à n'importe quel palier sans avoir un demi-produit.

---

## Itération 1 — MVP « Retrouver & relancer »

*À elle seule, cette version comble déjà le manque principal : c'est un outil utilisable tous les jours.*

### Découverte
- Scan de `~/.claude/projects/`, lecture des sessions locales
- Métadonnées de base par session : titre lisible, date de dernière activité, dossier/projet, nombre de messages

### Groupage (sélecteur)
- Aucun
- Par projet
- Par date

### Tri (sélecteur)
- **Recently Updated** — message le plus récent d'abord *(défaut)*
- **Latest Summary** — résumé le plus récent d'abord
- **Recently Modified** — fichier modifié le plus récemment d'abord
- **Recently Created** — session la plus récente d'abord
- **Most Messages** — plus grand nombre de messages d'abord
- **Title A-Z** — ordre alphabétique du titre
- **Oldest First** — session la plus ancienne d'abord

> Groupage et tri sont indépendants (ex. groupé par projet, trié par « Most Messages » à l'intérieur de chaque groupe).

### Recherche
- Recherche par titre de session, filtrage instantané de la liste au fil de la frappe

### Reprise
- Action « Reprendre » sur une session : `cd` vers le bon dossier puis `claude --resume <id>`

### Confort
- Mode sombre / clair

---

## Itération 2 — « Chercher »

*Objectif : ne plus jamais grep dans les JSONL.*

- Recherche full-text sur tous les projets (index rapide via `history.jsonl` + lecture des transcripts pour le détail)
- Filtres : par projet, par date, par durée
- Résultats affichant le projet, la date et l'extrait qui matche
- Renommage des sessions (mappé sur `--name` / `/rename`)

---

## Itération 3 — « Lire »

*Objectif : comprendre une session sans la relancer.*

- Visualiseur de transcript propre : messages, appels d'outils, diffs de code avec coloration syntaxique
- Panneau d'aperçu rapide (premiers / derniers messages) sans ouvrir la session complète
- Affichage des sous-agents (stockés à part, à côté de la session)
- Lecture via `getSessionMessages()` du SDK plutôt qu'un parsing maison, pour survivre aux changements de format JSONL

---

## Itération 4 — « Organiser »

*Objectif : dompter des centaines de sessions.*

- Tags et couleurs par projet
- Épinglage, favoris, archivage
- Notes personnelles attachées à une session
- Détection des sessions « abandonnées » (courtes, sans conclusion)
- Gestion des sessions dont le dossier a été déplacé ou supprimé

---

## Itération 5 — « Comprendre & confort »

*Objectif : la couche premium.*

- Dashboard : activité dans le temps, projets les plus actifs
- Stats par session : outils utilisés (Read / Edit / Bash…), durée, coût / tokens estimés
- Export Markdown / JSON (aligné sur `/export`)
- Reprise avec prompt pré-rempli (`claude -c "où en étions-nous ?"`)
- Fork d'une session comme point de départ d'une nouvelle
- Palette de commandes ⌘K + raccourcis clavier
- File-watching : rafraîchissement en temps réel quand une session tourne en parallèle
- Backups automatiques des transcripts

---

## Repères techniques

- Sessions stockées en JSONL : `~/.claude/projects/<encoded-cwd>/<session-id>.jsonl` (chemin du dossier avec chaque caractère non-alphanumérique remplacé par `-`)
- Index global des prompts : `~/.claude/history.jsonl`
- Reprise : `claude --resume <id>` / `claude -c` (dernière) / `claude --resume <name>`
- **Scoping par dossier** : il faut se placer dans le dossier d'origine avant de reprendre
- **Format JSONL instable** entre versions → privilégier le SDK (`listSessions()`, `getSessionMessages()`) au parsing direct
- Stack desktop recommandée : **Tauri** (lecture de fichiers locaux + lancement de commandes shell)
