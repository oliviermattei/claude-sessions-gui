import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { RefreshReport, RetentionPolicy, Session } from "../types";
import { useProfiles } from "./useProfiles";

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

const { activeProfileId } = useProfiles();

/** Push every retained transcript's mtime past the cutoff, in every Profile. Best-effort. */
export async function keepAlive(): Promise<RefreshReport | null> {
  try {
    return await invoke<RefreshReport>("refresh_retained");
  } catch {
    return null;
  }
}

async function loadPolicy() {
  try {
    policy.value = await invoke<RetentionPolicy>("retention_policy", {
      profile: activeProfileId.value,
    });
  } catch {
    /* keep the 30-day default outside the Tauri runtime */
  }
}

export function useRetention() {
  if (!started) {
    started = true;
    void loadPolicy();
    // Each Profile has its own cleanup period.
    watch(activeProfileId, () => void loadPolicy());
    setInterval(() => void keepAlive(), KEEPALIVE_MS);
  }

  /** Human-readable window, e.g. "14 days" / "org policy · 14 days". */
  function policyLabel(): string {
    const d = `${policy.value.days} day${policy.value.days > 1 ? "s" : ""}`;
    return policy.value.managed ? `org policy · ${d}` : d;
  }

  async function setRetained(s: Session, retained: boolean) {
    await invoke("set_session_retained", { profile: s.profile, path: s.path, retained });
  }

  /** Copy a vault-only session back into its Profile's projects. */
  async function restore(s: Session): Promise<string> {
    return invoke<string>("restore_retained", { profile: s.profile, id: s.id });
  }

  return { policy, policyLabel, setRetained, restore, keepAlive };
}
