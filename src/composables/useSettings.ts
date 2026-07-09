import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { Launcher, Session } from "../types";

const STORAGE_LAUNCHERS = "cs.launchers";
const STORAGE_DEFAULT = "cs.defaultLauncher";

// Presets. Templates use {cwd} and {id}; "run" templates go through `cmd /C`.
const DEFAULT_LAUNCHERS: Launcher[] = [
  {
    id: "warp",
    label: "Warp",
    kind: "uri",
    // Warp's URI scheme opens a new tab at the folder (path is percent-encoded).
    // Warp can't auto-run a command from a URI, so the resume command is copied
    // to the clipboard — just paste and press Enter.
    template: "warp://action/new_tab?path={cwd}",
  },
  {
    id: "wt",
    label: "Windows Terminal",
    kind: "run",
    template: 'wt.exe -d "{cwd}" claude --resume {id}',
  },
  {
    id: "powershell",
    label: "PowerShell",
    kind: "run",
    template:
      "start powershell -NoExit -Command \"cd '{cwd}'; claude --resume {id}\"",
  },
  {
    id: "clipboard",
    label: "Copy command",
    kind: "clipboard",
    template: "claude --resume {id}",
  },
];

function loadLaunchers(): Launcher[] {
  try {
    const raw = localStorage.getItem(STORAGE_LAUNCHERS);
    if (raw) {
      const parsed = JSON.parse(raw) as Launcher[];
      if (Array.isArray(parsed) && parsed.length) return parsed;
    }
  } catch {
    /* fall through to defaults */
  }
  return structuredClone(DEFAULT_LAUNCHERS);
}

const launchers = ref<Launcher[]>(loadLaunchers());
const defaultLauncherId = ref<string>(
  localStorage.getItem(STORAGE_DEFAULT) || "warp",
);

watch(
  launchers,
  (v) => localStorage.setItem(STORAGE_LAUNCHERS, JSON.stringify(v)),
  { deep: true },
);
watch(defaultLauncherId, (v) => localStorage.setItem(STORAGE_DEFAULT, v));

/** Plain substitution (for run / clipboard). */
function fill(template: string, s: Session): string {
  return template.replaceAll("{cwd}", s.project).replaceAll("{id}", s.id);
}

/** Percent-encoded substitution (for uri). */
function fillUri(template: string, s: Session): string {
  return template
    .replaceAll("{cwd}", encodeURIComponent(s.project))
    .replaceAll("{id}", encodeURIComponent(s.id));
}

export function useSettings() {
  const defaultLauncher = computed(
    () =>
      launchers.value.find((l) => l.id === defaultLauncherId.value) ??
      launchers.value[0],
  );

  /** Execute a launcher against a session. Returns a short status string for the toast. */
  async function resume(s: Session, launcher?: Launcher): Promise<string> {
    const l = launcher ?? defaultLauncher.value;

    if (l.kind === "clipboard") {
      const command = fill(l.template, s);
      await navigator.clipboard.writeText(command);
      return `Copied  ·  ${command}`;
    }

    if (l.kind === "uri") {
      await openUrl(fillUri(l.template, s));
      // A URI can open a folder but not run a command; hand over the resume
      // command via the clipboard when the template doesn't embed {id}.
      if (!l.template.includes("{id}")) {
        try {
          await navigator.clipboard.writeText(`claude --resume ${s.id}`);
        } catch {
          /* clipboard is best-effort */
        }
        return `${l.label}  ·  command copied — paste & Enter`;
      }
      return `${l.label}  ·  ${s.projectName}`;
    }

    // run
    await invoke("run_command", { command: fill(l.template, s), cwd: s.project });
    return `${l.label}  ·  ${s.projectName}`;
  }

  function addLauncher() {
    const l: Launcher = {
      id: crypto.randomUUID(),
      label: "New launcher",
      kind: "run",
      template: 'wt.exe -d "{cwd}" claude --resume {id}',
    };
    launchers.value.push(l);
    return l;
  }

  function removeLauncher(id: string) {
    const idx = launchers.value.findIndex((l) => l.id === id);
    if (idx < 0) return;
    launchers.value.splice(idx, 1);
    if (defaultLauncherId.value === id) {
      defaultLauncherId.value = launchers.value[0]?.id ?? "";
    }
  }

  function resetLaunchers() {
    launchers.value = structuredClone(DEFAULT_LAUNCHERS);
    defaultLauncherId.value = "warp";
  }

  return {
    launchers,
    defaultLauncherId,
    defaultLauncher,
    resume,
    addLauncher,
    removeLauncher,
    resetLaunchers,
    preview: (l: Launcher, s: Session) => fill(l.template, s),
  };
}
