import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Profile } from "../types";

// Profiles = the Claude Code config dirs (`~/.claude`, `~/.claude-perso`, …).
// The window shows one at a time, the Active Profile, switched from the Profile
// switcher. The list itself lives in the backend (profiles.json), since the
// scan and the retention keep-alive need it too.

const STORAGE_ACTIVE = "cs.activeProfile";
const DEFAULT_ID = "default";

const profiles = ref<Profile[]>([]);
const activeId = ref<string>(localStorage.getItem(STORAGE_ACTIVE) || DEFAULT_ID);
let loading: Promise<void> | null = null;

async function load() {
  try {
    profiles.value = await invoke<Profile[]>("list_profiles");
  } catch {
    /* outside the Tauri runtime: no profiles */
  }
}

/** Profiles worth switching to: present on disk and not hidden by the user. */
const visibleProfiles = computed(() =>
  profiles.value.filter((p) => p.exists && (!p.hidden || p.isDefault)),
);

/** The Active Profile, falling back to the Default one when it is gone or hidden. */
const activeProfile = computed<Profile | null>(
  () =>
    visibleProfiles.value.find((p) => p.id === activeId.value) ??
    profiles.value.find((p) => p.isDefault) ??
    null,
);

/** Id the backend is asked about: the Active Profile's, once it is known. */
const activeProfileId = computed(() => activeProfile.value?.id ?? DEFAULT_ID);

function setActive(id: string) {
  activeId.value = id;
  localStorage.setItem(STORAGE_ACTIVE, id);
}

function byId(id: string): Profile | undefined {
  return profiles.value.find((p) => p.id === id);
}

async function update(p: Profile) {
  const saved = await invoke<Profile>("update_profile", {
    edit: {
      id: p.id,
      name: p.name,
      color: p.color ?? null,
      hidden: p.hidden,
      resumePrefix: p.resumePrefix ?? null,
    },
  });
  const i = profiles.value.findIndex((x) => x.id === saved.id);
  if (i >= 0) profiles.value[i] = saved;
}

async function add(path: string) {
  await invoke<Profile>("add_profile", { path });
  await load();
}

async function remove(id: string) {
  await invoke("remove_profile", { id });
  await load();
}

export function useProfiles() {
  return {
    profiles,
    visibleProfiles,
    activeProfile,
    activeProfileId,
    /** Resolves once the list has been read (first call triggers it). */
    ready: () => (loading ??= load()),
    reload: load,
    setActive,
    byId,
    update,
    add,
    remove,
  };
}
