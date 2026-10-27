// Shared types. `Session` mirrors the Rust `Session` struct (camelCase via serde).

export interface Session {
  id: string;
  title: string;
  project: string; // absolute cwd
  projectName: string; // last path segment
  messages: number;
  updated: number; // epoch ms
  created: number; // epoch ms
  modified: number; // epoch ms — last real activity, never the keep-alive stamp
  path: string;
  profile: string; // id of the Profile it belongs to
  color?: string | null; // Claude "agent-color" (red/blue/…) or null
  retained: boolean; // exempted from Claude Code's automatic cleanup
  archived: boolean; // cleaned up already; `path` points into our vault copy
}

/** One Claude Code config dir (`CLAUDE_CONFIG_DIR`). Mirrors the Rust `Profile`. */
export interface Profile {
  id: string;
  name: string;
  path: string;
  color?: string | null; // one of SESSION_COLORS, or null
  hidden: boolean;
  /** Added by hand (removable) rather than detected. */
  manual: boolean;
  /** `~/.claude`, what Claude uses when CLAUDE_CONFIG_DIR is unset. */
  isDefault: boolean;
  /** False when its folder no longer exists. */
  exists: boolean;
  /** Overrides the shell prefix of a copied resume command; `{configDir}` placeholder. */
  resumePrefix?: string | null;
}

/** Result of `move_session`. */
export type MoveOutcome =
  | { status: "moved"; path: string; warnings: string[] }
  /** The target Profile already holds this exact session. */
  | { status: "duplicate" };

/** Effective `cleanupPeriodDays` and where retained copies are kept. */
export interface RetentionPolicy {
  days: number;
  /** Set by managed/org settings — the user cannot raise it locally. */
  managed: boolean;
  vault: string;
}

export interface RefreshReport {
  refreshed: number;
  archived: number;
  missing: number;
}

export type Grouping = "none" | "project" | "date";

export type SortKey =
  | "recentUpdated"
  | "recentModified"
  | "recentCreated"
  | "mostMessages"
  | "titleAZ"
  | "oldestFirst";

// --- Transcript (iteration 3: "Lire") --------------------------------------

/** One content block inside a message. `kind` mirrors the Rust `Block` enum tag. */
export type Block =
  | { kind: "text"; text: string }
  | { kind: "thinking"; text: string }
  | { kind: "toolUse"; id: string; name: string; input: Record<string, unknown> }
  | { kind: "toolResult"; toolUseId: string; text: string; isError: boolean };

export interface Message {
  role: "user" | "assistant";
  ts: number; // epoch ms (0 if missing)
  blocks: Block[];
}

export interface Subagent {
  id: string;
  messages: Message[];
}

export interface Transcript {
  messages: Message[];
  subagents: Subagent[];
}

/** A "Resume" launcher: an editable command template with {cwd} and {id} placeholders. */
export interface Launcher {
  id: string;
  label: string;
  /**
   * - "run": spawn the template as a shell command via the backend.
   * - "uri": open the template as a URI via the OS handler (e.g. warp://…);
   *   {cwd}/{id} are percent-encoded. If the template omits {id}, the resume
   *   command is also copied to the clipboard.
   * - "clipboard": copy the template to the clipboard instead of running it.
   */
  kind: "run" | "uri" | "clipboard";
  /** Command / URI template, e.g. "wt.exe -d \"{cwd}\" claude --resume {id}". */
  template: string;
}
