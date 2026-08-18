import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { RefreshReport, RetentionPolicy, Session } from "../types";

// Claude Code deletes any transcript whose mtime is older than
// `cleanupPeriodDays`. Retaining a session refreshes that mtime and mirrors the
// file into a vault outside the scanned tree — see src-tauri/src/retention.rs.
//
// The keep-alive runs once per app start (from `useSessions.load`) and then on
// this interval, so a window left open for weeks still keeps its sessions clear
// of the cutoff.
const KEEPALIVE_MS = 6 * 60 * 60 * 1000;

const policy = ref<RetentionPolicy>({ days: 30, managed: false, vault: "" });
let started = false;

/** Push every retained transcript's mtime past the cutoff. Best-effort. */
export async function keepAlive(): Promise<RefreshReport | null> {
  try {
    return await invoke<RefreshReport>("refresh_retained");
  } catch {
    return null;
  }
}

async function loadPolicy() {
  try {
    policy.value = await invoke<RetentionPolicy>("retention_policy");
  } catch {
    /* keep the 30-day default outside the Tauri runtime */
  }
}

export function useRetention() {
  if (!started) {
    started = true;
    void loadPolicy();
    setInterval(() => void keepAlive(), KEEPALIVE_MS);
  }

  /** Human-readable window, e.g. "14 days" / "org policy · 14 days". */
  function policyLabel(): string {
    const d = `${policy.value.days} day${policy.value.days > 1 ? "s" : ""}`;
    return policy.value.managed ? `org policy · ${d}` : d;
  }

  async function setRetained(s: Session, retained: boolean) {
    await invoke("set_session_retained", { path: s.path, retained });
  }

  /** Copy a vault-only session back into ~/.claude/projects. */
  async function restore(s: Session): Promise<string> {
    return invoke<string>("restore_retained", { id: s.id });
  }

  return { policy, policyLabel, setRetained, restore, keepAlive };
}
