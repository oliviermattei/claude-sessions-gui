// Shared types. `Session` mirrors the Rust `Session` struct (camelCase via serde).

export interface Session {
  id: string;
  title: string;
  project: string; // absolute cwd
  projectName: string; // last path segment
  messages: number;
  updated: number; // epoch ms
  created: number; // epoch ms
  modified: number; // epoch ms
  path: string;
}

export type Grouping = "none" | "project" | "date";

export type SortKey =
  | "recentUpdated"
  | "recentModified"
  | "recentCreated"
  | "mostMessages"
  | "titleAZ"
  | "oldestFirst";

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
