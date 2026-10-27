<script setup lang="ts">
import { computed, ref } from "vue";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { useSettings, DEFAULT_RESUME_PREFIX } from "../composables/useSettings";
import { useProfiles } from "../composables/useProfiles";
import { SESSION_COLORS, colorHex, colorLabel } from "../lib/colors";
import type { Profile } from "../types";
import { useTheme } from "../composables/useTheme";
import { useRetention } from "../composables/useRetention";
import { useSessions } from "../composables/useSessions";

defineEmits<{ close: [] }>();

const { launchers, defaultLauncherId, addLauncher, removeLauncher, resetLaunchers } =
  useSettings();
const { accent, theme, layout } = useTheme();
const { policy } = useRetention();
const { sessions } = useSessions();
const { profiles, update: updateProfile, add: addProfile, remove: removeProfile } =
  useProfiles();

const ACCENTS = ["#d97757", "#8a63d2", "#2a78d6", "#1f8a5b"];

type Section = "appearance" | "profiles" | "launchers" | "retention";
const section = ref<Section>("appearance");
const NAV: { key: Section; label: string }[] = [
  { key: "appearance", label: "Appearance" },
  { key: "profiles", label: "Profiles" },
  { key: "launchers", label: "Launchers" },
  { key: "retention", label: "Retention" },
];

const retainedCount = computed(() => sessions.value.filter((s) => s.retained).length);
const archivedCount = computed(() => sessions.value.filter((s) => s.archived).length);

const profileError = ref<string | null>(null);
const newProfilePath = ref("");

async function saveProfile(p: Profile) {
  profileError.value = null;
  try {
    await updateProfile(p);
  } catch (e) {
    profileError.value = String(e);
  }
}

async function onAddProfile() {
  const path = newProfilePath.value.trim();
  if (!path) return;
  profileError.value = null;
  try {
    await addProfile(path);
    newProfilePath.value = "";
  } catch (e) {
    profileError.value = String(e);
  }
}

async function onRemoveProfile(p: Profile) {
  profileError.value = null;
  try {
    await removeProfile(p.id);
  } catch (e) {
    profileError.value = String(e);
  }
}

async function revealVault() {
  if (!policy.value.vault) return;
  try {
    await revealItemInDir(policy.value.vault);
  } catch {
    /* nothing retained yet — the folder doesn't exist */
  }
}
</script>

<template>
  <div class="backdrop" @click.self="$emit('close')">
    <div class="modal">
      <header>
        <h2>Settings</h2>
        <button class="x" @click="$emit('close')">×</button>
      </header>

      <div class="pane">
        <!-- left category menu -->
        <nav class="nav">
          <button
            v-for="n in NAV"
            :key="n.key"
            class="nav-item"
            :class="{ on: section === n.key }"
            @click="section = n.key"
          >
            {{ n.label }}
          </button>
        </nav>

        <!-- right content -->
        <div class="content">
          <!-- APPEARANCE -->
          <template v-if="section === 'appearance'">
            <section>
              <h3>Theme</h3>
              <div class="segmented">
                <button :class="{ on: theme === 'dark' }" @click="theme = 'dark'">Dark</button>
                <button :class="{ on: theme === 'light' }" @click="theme = 'light'">Light</button>
              </div>
            </section>
            <section>
              <h3>Accent</h3>
              <div class="accents">
                <button
                  v-for="a in ACCENTS"
                  :key="a"
                  class="swatch"
                  :class="{ on: accent === a }"
                  :style="{ background: a }"
                  @click="accent = a"
                />
              </div>
            </section>
            <section>
              <h3>Layout</h3>
              <div class="segmented">
                <button
                  :class="{ on: layout === 'comfortable' }"
                  @click="layout = 'comfortable'"
                >
                  Comfortable
                </button>
                <button
                  :class="{ on: layout === 'compact' }"
                  @click="layout = 'compact'"
                >
                  Compact
                </button>
              </div>
              <p class="hint layout-hint">
                <strong>Comfortable</strong> keeps the two-pane view. <strong>Compact</strong>
                folds every project into a single accordion tree — click a project to reveal
                its sessions underneath.
              </p>
            </section>
          </template>

          <!-- PROFILES -->
          <template v-else-if="section === 'profiles'">
            <section>
              <h3>Claude profiles</h3>
              <p class="hint">
                Each profile is one Claude Code config folder (what
                <code>CLAUDE_CONFIG_DIR</code> points to). <code>~/.claude</code> and every
                <code>~/.claude-*</code> folder holding a <code>projects</code> folder are
                found automatically; switch between them at the bottom of the sidebar.
              </p>

              <div v-for="p in profiles" :key="p.id" class="launcher profile-card">
                <div class="row1">
                  <select
                    v-model="p.color"
                    class="kind color-pick"
                    :style="{ color: colorHex(p.color) ?? 'var(--dim)' }"
                    title="Color"
                    @change="saveProfile(p)"
                  >
                    <option :value="null">●  None</option>
                    <option v-for="c in SESSION_COLORS" :key="c" :value="c">
                      ●  {{ colorLabel(c) }}
                    </option>
                  </select>
                  <input
                    class="label-input"
                    v-model="p.name"
                    placeholder="Name"
                    @change="saveProfile(p)"
                  />
                  <label v-if="!p.isDefault" class="show" title="Show in the profile switcher">
                    <input
                      type="checkbox"
                      :checked="!p.hidden"
                      @change="
                        p.hidden = !($event.target as HTMLInputElement).checked;
                        saveProfile(p);
                      "
                    />
                    Show
                  </label>
                  <button
                    v-if="p.manual"
                    class="del"
                    title="Remove"
                    @click="onRemoveProfile(p)"
                  >
                    <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
                      <path
                        d="M3 4.5h10M6.5 4.5V3h3v1.5M5 4.5l.5 8h5l.5-8"
                        stroke="currentColor"
                        stroke-width="1.3"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      />
                    </svg>
                  </button>
                </div>
                <p class="hint path">
                  {{ p.path }}
                  <span v-if="p.isDefault" class="tag">default</span>
                  <span v-if="!p.exists" class="tag warn">missing</span>
                </p>
                <input
                  v-if="!p.isDefault"
                  class="template"
                  v-model="p.resumePrefix"
                  :placeholder="DEFAULT_RESUME_PREFIX"
                  spellcheck="false"
                  title="Prefix of a copied resume command"
                  @change="saveProfile(p)"
                />
              </div>

              <div class="add-row">
                <input
                  class="template"
                  v-model="newProfilePath"
                  placeholder="Add another folder, e.g. D:\claude-work"
                  spellcheck="false"
                  @keydown.enter="onAddProfile"
                />
                <button class="ghost" @click="onAddProfile">+ Add</button>
              </div>
              <p v-if="profileError" class="hint error">{{ profileError }}</p>
              <p class="hint">
                Copied resume commands of a non-default profile start with its prefix,
                <code>{configDir}</code> being its folder.
              </p>
            </section>
          </template>

          <!-- RETENTION -->
          <template v-else-if="section === 'retention'">
            <section>
              <h3>Automatic cleanup</h3>
              <p class="hint">
                Claude Code deletes any transcript left untouched for
                <strong>{{ policy.days }} days</strong
                >{{
                  policy.managed
                    ? " — set by managed settings, so it can't be raised locally."
                    : "."
                }}
                A session marked <strong>Keep beyond cleanup</strong> stays out of
                reach: its timestamp is refreshed on every launch of this app, and a
                copy is mirrored into a vault outside the profile's
                <code>projects</code> folder.
              </p>
              <p class="hint">
                Open this app at least once per period and the session never moves. If
                nobody does, the vault copy is what survives — restore it from the
                session's context menu to make it resumable again.
              </p>
            </section>

            <section>
              <div class="section-head">
                <h3>Vault</h3>
                <div class="head-actions">
                  <button class="ghost" :disabled="!policy.vault" @click="revealVault">
                    Reveal
                  </button>
                </div>
              </div>
              <p class="hint path">{{ policy.vault || "unavailable" }}</p>
              <p class="hint">
                {{ retainedCount }} session{{ retainedCount === 1 ? "" : "s" }} kept<template
                  v-if="archivedCount"
                >
                  · {{ archivedCount }} served from the vault</template
                >.
              </p>
            </section>
          </template>

          <!-- LAUNCHERS -->
          <template v-else>
            <section>
              <div class="section-head">
                <h3>Resume launchers</h3>
                <div class="head-actions">
                  <button class="ghost" @click="addLauncher()">+ Add</button>
                  <button class="ghost" @click="resetLaunchers()">Reset</button>
                </div>
              </div>
              <p class="hint">
                The selected launcher runs on <strong>Resume</strong>. Templates support
                <code>{cwd}</code>, <code>{id}</code> and <code>{configDir}</code> (the
                session's profile folder; <strong>Run</strong> launchers also get it as
                <code>CLAUDE_CONFIG_DIR</code>). Kinds:
                <strong>Run</strong> (shell command), <strong>URI</strong> (e.g.
                <code>warp://</code>), <strong>Clipboard</strong> (copy only).
              </p>

              <div v-for="l in launchers" :key="l.id" class="launcher">
                <label class="row1">
                  <input
                    type="radio"
                    name="default-launcher"
                    :value="l.id"
                    v-model="defaultLauncherId"
                    title="Set as default"
                  />
                  <input class="label-input" v-model="l.label" placeholder="Name" />
                  <select v-model="l.kind" class="kind">
                    <option value="run">Run</option>
                    <option value="uri">URI</option>
                    <option value="clipboard">Clipboard</option>
                  </select>
                  <button class="del" title="Delete" @click="removeLauncher(l.id)">
                    <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
                      <path
                        d="M3 4.5h10M6.5 4.5V3h3v1.5M5 4.5l.5 8h5l.5-8"
                        stroke="currentColor"
                        stroke-width="1.3"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      />
                    </svg>
                  </button>
                </label>
                <input class="template" v-model="l.template" spellcheck="false" />
              </div>

              <p v-if="!launchers.length" class="empty">
                No launchers. Add one to enable Resume.
              </p>
            </section>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.backdrop {
  position: absolute;
  inset: 0;
  z-index: 40;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px;
}
.modal {
  width: min(720px, 100%);
  height: min(520px, 100%);
  background: var(--panel2);
  border: 1px solid var(--bd2);
  border-radius: 14px;
  box-shadow: 0 30px 80px -20px rgba(0, 0, 0, 0.65);
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}
header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 18px;
  border-bottom: 1px solid var(--bd);
  flex: none;
}
h2 {
  margin: 0;
  font-size: 15px;
  font-weight: 650;
}
.x {
  width: 26px;
  height: 26px;
  border-radius: 7px;
  border: none;
  background: transparent;
  color: var(--dim);
  cursor: pointer;
  font-size: 17px;
}
.x:hover {
  background: var(--hover);
}

.pane {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 160px 1fr;
}
.nav {
  border-right: 1px solid var(--bd);
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.nav-item {
  text-align: left;
  padding: 8px 10px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--dim);
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
}
.nav-item:hover {
  background: var(--hover);
}
.nav-item.on {
  background: var(--sel);
  color: var(--tx);
  font-weight: 600;
}

.content {
  padding: 18px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 24px;
}
section h3 {
  margin: 0 0 10px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.6px;
  text-transform: uppercase;
  color: var(--faint);
}
.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.head-actions {
  display: flex;
  gap: 6px;
}
.ghost {
  border: 1px solid var(--bd2);
  background: transparent;
  color: var(--dim);
  border-radius: 7px;
  padding: 4px 9px;
  font-size: 11.5px;
  cursor: pointer;
}
.ghost:hover {
  background: var(--hover);
  color: var(--tx);
}
.hint {
  margin: 0 0 12px;
  font-size: 12px;
  color: var(--dim);
  line-height: 1.5;
}
.layout-hint {
  margin: 10px 0 0;
}
/* Vault location: long, breakable, and copy-pasteable. */
.hint.path {
  font-family: var(--mono);
  font-size: 11.5px;
  color: var(--faint);
  overflow-wrap: anywhere;
  user-select: text;
}
.ghost:disabled {
  opacity: 0.45;
  cursor: default;
}
.hint code {
  font-family: var(--mono);
  font-size: 11.5px;
  background: var(--hover);
  padding: 1px 5px;
  border-radius: 4px;
}

.segmented {
  display: inline-flex;
  padding: 2px;
  gap: 2px;
  background: var(--bg);
  border: 1px solid var(--bd2);
  border-radius: 9px;
}
.segmented button {
  padding: 6px 16px;
  border-radius: 7px;
  border: none;
  cursor: pointer;
  font-size: 12.5px;
  background: transparent;
  color: var(--dim);
}
.segmented button.on {
  background: var(--sel);
  color: var(--tx);
  font-weight: 600;
}

.accents {
  display: flex;
  gap: 10px;
}
.accents .swatch {
  width: 26px;
  height: 26px;
  border-radius: 8px;
  border: 2px solid transparent;
  cursor: pointer;
  outline: 1px solid var(--bd2);
}
.accents .swatch.on {
  border-color: var(--tx);
}

.launcher {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px;
  border: 1px solid var(--bd);
  border-radius: 10px;
  margin-bottom: 8px;
}
.row1 {
  display: flex;
  align-items: center;
  gap: 10px;
}
.row1 input[type="radio"] {
  accent-color: var(--acc);
  flex: none;
}
.label-input {
  flex: 1;
  min-width: 0;
  height: 30px;
  padding: 0 9px;
  border-radius: 7px;
  border: 1px solid var(--bd2);
  background: var(--bg);
  color: var(--tx);
  font-size: 13px;
  font-weight: 550;
}
.kind {
  height: 30px;
  border-radius: 7px;
  border: 1px solid var(--bd2);
  background: var(--bg);
  color: var(--tx);
  font-size: 12px;
  padding: 0 6px;
}
.del {
  flex: none;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 7px;
  border: 1px solid var(--bd2);
  background: transparent;
  color: var(--dim);
  cursor: pointer;
}
.del:hover {
  background: #e04c3f;
  border-color: #e04c3f;
  color: #fff;
}
.template {
  height: 32px;
  padding: 0 10px;
  border-radius: 7px;
  border: 1px solid var(--bd2);
  background: var(--bg);
  color: var(--tx);
  font-family: var(--mono);
  font-size: 12px;
}
.empty {
  font-size: 12.5px;
  color: var(--faint);
}

.profile-card .hint.path {
  margin: 0;
}
.color-pick {
  flex: none;
  width: 92px;
}
.color-pick option {
  color: var(--tx);
}
.show {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  color: var(--dim);
  flex: none;
  cursor: pointer;
}
.show input {
  accent-color: var(--acc);
}
.tag {
  margin-left: 6px;
  padding: 0 6px;
  border-radius: 5px;
  border: 1px solid var(--bd2);
  font-family: inherit;
  font-size: 10.5px;
  color: var(--dim);
}
.tag.warn,
.hint.error {
  color: #e04c3f;
}
.add-row {
  display: flex;
  gap: 8px;
  margin: 4px 0 10px;
}
.add-row .template {
  flex: 1;
  min-width: 0;
}
</style>
