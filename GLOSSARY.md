# Claude Sessions

A desktop app to browse, resume and tidy the Claude Code sessions stored on this machine, across every Claude configuration the user runs.

## Language

**Profile**:
One Claude Code configuration directory (what `CLAUDE_CONFIG_DIR` points to, `~/.claude` when unset) with the name the user knows it by; every Session belongs to exactly one Profile.
_Avoid_: Config, configuration, account, settings (the app's own settings are another thing), Space (the UI switches between Profiles, there is no separate set)

**Default Profile**:
The Profile at `~/.claude`, used by Claude Code when `CLAUDE_CONFIG_DIR` is unset; always present and never removed.
_Avoid_: Main profile, root config

**Active Profile**:
The one Profile whose Sessions and Projects the window shows; the user switches it from the Profile switcher, shown only when there are several Profiles.
_Avoid_: Current config, selected space

**Session**:
One Claude Code conversation, identified by its UUID, belonging to one Project of one Profile.
_Avoid_: Conversation, chat, transcript (the transcript is the file a Session is written to)

**Project**:
The working directory a Session runs in; the same Project can hold Sessions in several Profiles.
_Avoid_: Folder, repo, workspace

**Transcript**:
The `.jsonl` file a Session is written to; a Session also owns other files in its Profile (subagents, checkpoints, plan, prompt history lines), and they all travel with it.
_Avoid_: Log, session file

**Live Session**:
A Session currently open in a running Claude Code process; it cannot be moved.
_Avoid_: Open session, active session (the Active Profile is another thing)

**Move**:
Relocating a Session with everything it owns, either to another Project of its Profile or to the same Project in another Profile; all or nothing, never leaving a copy in both places.
_Avoid_: Transfer, migrate, copy
