import { ref } from "vue";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

// Self-update from the `latest.json` of the latest GitHub Release (endpoint and
// public key in tauri.conf.json, see docs/releasing.md). The plugin verifies the
// download's signature before installing; a failed check (offline, no release
// yet) is silent and simply retried later.
//
// Dev builds never check: they would be offered the published release.

export type UpdateState =
  | { status: "none" }
  | { status: "available"; version: string }
  | { status: "installing"; version: string; progress: number | null }
  | { status: "failed"; version: string; error: string };

// An app left open for days still hears about a new release.
const RECHECK_MS = 6 * 60 * 60 * 1000;

const state = ref<UpdateState>({ status: "none" });
const dismissed = ref(false);
let pending: Update | null = null;
let started = false;

async function lookForUpdate() {
  // Never swap the update out from under an install in progress.
  if (state.value.status === "installing") return;
  try {
    const update = await check();
    if (!update) return;
    if (pending?.version !== update.version) dismissed.value = false;
    pending = update;
    state.value = { status: "available", version: update.version };
  } catch {
    /* offline or no release published yet: try again on the next round */
  }
}

async function install() {
  const update = pending;
  if (!update || state.value.status === "installing") return;
  const version = update.version;
  state.value = { status: "installing", version, progress: null };

  let total = 0;
  let received = 0;
  try {
    await update.downloadAndInstall((event) => {
      if (event.event === "Started") {
        total = event.data.contentLength ?? 0;
      } else if (event.event === "Progress") {
        received += event.data.chunkLength;
        if (total > 0) {
          state.value = {
            status: "installing",
            version,
            progress: Math.min(1, received / total),
          };
        }
      }
    });
    // On Windows the NSIS installer has already quit the app by now; elsewhere
    // the new binary is in place and only needs a restart.
    await relaunch();
  } catch (e) {
    state.value = { status: "failed", version, error: String(e) };
  }
}

export function useUpdater() {
  if (!started && !import.meta.env.DEV) {
    started = true;
    void lookForUpdate();
    setInterval(() => void lookForUpdate(), RECHECK_MS);
  }

  return {
    state,
    dismissed,
    install,
    dismiss: () => (dismissed.value = true),
  };
}
