import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Session } from "../types";
import { keepAlive } from "./useRetention";

const sessions = ref<Session[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
let loadedOnce = false;

async function load() {
  loading.value = true;
  error.value = null;
  try {
    // Keep-alive first: retained transcripts get a fresh mtime (and a fresh
    // vault copy when they changed) before we read the list back, so the app
    // never lists a session it was about to rescue.
    await keepAlive();
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
