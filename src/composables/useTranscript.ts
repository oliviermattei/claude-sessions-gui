import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Session, Transcript } from "../types";

// Transcripts are immutable once written, so cache by file path for the session.
const cache = new Map<string, Transcript>();

export function useTranscript() {
  const transcript = ref<Transcript | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);
  let token = 0;

  async function load(session: Session | null) {
    if (!session) {
      transcript.value = null;
      error.value = null;
      loading.value = false;
      return;
    }

    const cached = cache.get(session.path);
    if (cached) {
      transcript.value = cached;
      error.value = null;
      loading.value = false;
      return;
    }

    const mine = ++token; // guard against out-of-order responses
    loading.value = true;
    error.value = null;
    try {
      const t = await invoke<Transcript>("get_session_messages", {
        path: session.path,
      });
      cache.set(session.path, t);
      if (mine === token) transcript.value = t;
    } catch (e) {
      if (mine === token) {
        error.value = String(e);
        transcript.value = null;
      }
    } finally {
      if (mine === token) loading.value = false;
    }
  }

  return { transcript, loading, error, load };
}
