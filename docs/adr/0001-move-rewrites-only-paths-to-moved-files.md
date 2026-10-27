# A Move rewrites only the paths that point to moved files

Transcripts embed absolute paths into their Profile's directory (tool-results, subagent transcript dirs, workflow scripts, plan file, the compaction note pointing at the transcript itself), so a Move to another Profile or Project has to rewrite them or a resumed Session reads files that no longer exist. We rewrite a path if and only if it points into something the Move relocates, in all three spellings Claude writes (`C:\\…` escaped, `C:/…`, `/c/…`) and on a path-segment boundary (`.claude` never matches `.claude-perso`). Every other path is left untouched, notably `file-history-snapshot.trackedFileBackups` keys and paths to `skills/`, `CLAUDE.md`, `settings.json` or `memory/`: they name real files that stay where they are, and rewriting them would make `/rewind` restore into the wrong place.

## Considered Options

- **Rewrite every occurrence of the source Profile directory**: simplest, but breaks `/rewind` and points at skills or settings that don't exist in the target.
- **Rewrite nothing**: keeps transcripts byte-identical, but a resumed Session follows dead paths to its own tool results and plan.
