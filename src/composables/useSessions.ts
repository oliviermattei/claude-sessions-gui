import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Session } from "../types";

const sessions = ref<Session[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
let loadedOnce = false;

async function load() {
  loading.value = true;
  error.value = null;
  try {
    sessions.value = await invoke<Session[]>("list_sessions");
    loadedOnce = true;
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

export function useSessions() {
  if (!loadedOnce && !loading.value) void load();
  return { sessions, loading, error, reload: load };
}
