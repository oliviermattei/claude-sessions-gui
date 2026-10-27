import { ref } from "vue";
import { getVersion } from "@tauri-apps/api/app";

// The running version, as Tauri reports it from tauri.conf.json — the same value
// the updater compares against `latest.json`. scripts/bump-version.mjs keeps it
// in sync with package.json and Cargo.toml.
const version = ref<string | null>(null);
let started = false;

export function useAppVersion() {
  if (!started) {
    started = true;
    getVersion()
      .then((v) => (version.value = v))
      .catch(() => {
        /* outside the Tauri runtime (vite preview): no version to show */
      });
  }
  return { version };
}
