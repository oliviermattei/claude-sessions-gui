<script setup lang="ts">
import { computed, ref } from "vue";
import type { Session, Grouping, SortKey } from "../types";
import { relTime, highlight, dateBucket, DATE_BUCKET_ORDER } from "../lib/format";
import { SORTS, sortDef } from "../lib/sort";
import { useTheme } from "../composables/useTheme";

const props = defineProps<{
  sessions: Session[];
  selectedId: string | null;
}>();

const query = defineModel<string>("query", { required: true });
const grouping = defineModel<Grouping>("grouping", { required: true });
const sort = defineModel<SortKey>("sort", { required: true });

const emit = defineEmits<{
  select: [id: string];
  resume: [session: Session];
  openSettings: [];
  reload: [];
}>();

const { theme, toggleTheme } = useTheme();
const themeLabel = () => (theme.value === "light" ? "Light" : "Dark");

const GROUPS: { key: Grouping; label: string }[] = [
  { key: "none", label: "None" },
  { key: "project", label: "Project" },
  { key: "date", label: "Date" },
];

// --- tree data -----------------------------------------------------------

interface Node {
  key: string;
  name: string;
  path?: string;
  sessions: Session[];
}

const searching = computed(() => !!query.value.trim());

const model = computed<{ flat: boolean; nodes: Node[] }>(() => {
  const q = query.value.trim().toLowerCase();
  const list = props.sessions
    .filter((s) => !q || s.title.toLowerCase().includes(q))
    .sort(sortDef(sort.value).cmp);

  if (grouping.value === "none") {
    return { flat: true, nodes: [{ key: "all", name: "", sessions: list }] };
  }

  const buckets = new Map<string, Session[]>();
  for (const s of list) {
    const k = grouping.value === "project" ? s.project : dateBucket(s.updated);
    (buckets.get(k) ?? buckets.set(k, []).get(k)!).push(s);
  }

  let keys = [...buckets.keys()];
  if (grouping.value === "date") {
    keys.sort((a, b) => DATE_BUCKET_ORDER.indexOf(a) - DATE_BUCKET_ORDER.indexOf(b));
  } else {
    keys.sort(
      (a, b) =>
        Math.max(...buckets.get(b)!.map((s) => s.updated)) -
        Math.max(...buckets.get(a)!.map((s) => s.updated)),
    );
  }

  const nodes = keys.map<Node>((k) => {
    const ss = buckets.get(k)!;
    return grouping.value === "project"
      ? { key: k, name: ss[0].projectName, path: k, sessions: ss }
      : { key: k, name: k, sessions: ss };
  });
  return { flat: false, nodes };
});

const total = computed(() =>
  model.value.nodes.reduce((sum, n) => sum + n.sessions.length, 0),
);

// --- accordion (one open at a time) -------------------------------------

const openKey = ref<string | null>(null);
const isOpen = (key: string) => searching.value || openKey.value === key;
function toggle(key: string) {
  if (searching.value) return; // expansion driven by the query while searching
  openKey.value = openKey.value === key ? null : key;
}

// --- options popover -----------------------------------------------------

const optionsOpen = ref(false);
function pickGroup(key: Grouping) {
  grouping.value = key;
}
function pickSort(key: SortKey) {
  sort.value = key;
  optionsOpen.value = false;
}
</script>

<template>
  <aside class="compact">
    <header class="head">
      <div class="brand">
        <span class="title">CLAUDE SESSIONS</span>
        <span class="count">{{ total }}</span>
      </div>
      <div class="actions">
        <button class="icon-btn" title="Rescan sessions" @click="emit('reload')">
          <svg width="15" height="15" viewBox="0 0 16 16" fill="none">
            <path
              d="M13 8a5 5 0 1 1-1.46-3.54M13 2.5V5h-2.5"
              stroke="currentColor"
              stroke-width="1.4"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
        <div class="opt-wrap">
          <button
            class="icon-btn"
            :class="{ on: optionsOpen }"
            title="Group & sort"
            @click="optionsOpen = !optionsOpen"
          >
            <svg width="15" height="15" viewBox="0 0 16 16" fill="none">
              <path
                d="M2.5 4.5h11M4.5 8h7M6.5 11.5h3"
                stroke="currentColor"
                stroke-width="1.4"
                stroke-linecap="round"
              />
            </svg>
          </button>
          <template v-if="optionsOpen">
            <div class="overlay" @click="optionsOpen = false"></div>
            <div class="menu">
              <div class="menu-label">Group</div>
              <div class="segmented">
                <button
                  v-for="g in GROUPS"
                  :key="g.key"
                  :class="{ on: grouping === g.key }"
                  @click="pickGroup(g.key)"
                >
                  {{ g.label }}
                </button>
              </div>
              <div class="menu-label">Sort</div>
              <button
                v-for="opt in SORTS"
                :key="opt.key"
                class="opt"
                :class="{ active: sort === opt.key }"
                @click="pickSort(opt.key)"
              >
                <span class="opt-text">
                  <span class="opt-title">{{ opt.label }}</span>
                  <span class="opt-sub">{{ opt.sub }}</span>
                </span>
                <span v-if="sort === opt.key" class="check">✓</span>
              </button>
            </div>
          </template>
        </div>
      </div>
    </header>

    <div class="search">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none" class="search-icon">
        <circle cx="7" cy="7" r="4.5" stroke="currentColor" stroke-width="1.4" />
        <path d="M10.5 10.5 14 14" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      </svg>
      <input v-model="query" placeholder="Search sessions…" />
      <button v-if="query" class="clear" title="Clear" @click="query = ''">×</button>
    </div>

    <div class="tree">
      <p v-if="!total" class="empty">
        {{ searching ? "No sessions match your search." : "No sessions yet." }}
      </p>

      <!-- flat list (grouping = none) -->
      <template v-if="model.flat">
        <div
          v-for="s in model.nodes[0].sessions"
          :key="s.id"
          class="leaf flat"
          :class="{ selected: selectedId === s.id }"
          :title="s.project"
          @click="emit('select', s.id)"
          @dblclick="emit('resume', s)"
        >
          <span class="glyph">{}</span>
          <span class="ltitle">
            <template v-if="highlight(s.title, query).hasHit">
              {{ highlight(s.title, query).pre
              }}<mark>{{ highlight(s.title, query).hit }}</mark
              >{{ highlight(s.title, query).post }}
            </template>
            <template v-else>{{ s.title }}</template>
          </span>
          <span class="lmeta">{{ s.messages }} · {{ relTime(s.updated) }}</span>
          <button class="resume" title="Resume" @click.stop="emit('resume', s)">
            <svg width="10" height="10" viewBox="0 0 12 12" fill="currentColor">
              <path d="M2.5 1.6c0-.4.44-.65.78-.43l6.4 4.4c.3.2.3.66 0 .86l-6.4 4.4a.5.5 0 0 1-.78-.43V1.6Z" />
            </svg>
          </button>
        </div>
      </template>

      <!-- accordion (grouping = project | date) -->
      <template v-else v-for="node in model.nodes" :key="node.key">
        <button
          class="proj"
          :class="{ open: isOpen(node.key) }"
          :title="node.path"
          @click="toggle(node.key)"
        >
          <svg class="chevron" width="12" height="12" viewBox="0 0 16 16" fill="none">
            <path d="M6 4l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
          <span class="pname">{{ node.name }}</span>
          <span class="pcount">{{ node.sessions.length }} sessions</span>
        </button>

        <div v-if="isOpen(node.key)" class="children">
          <div
            v-for="s in node.sessions"
            :key="s.id"
            class="leaf"
            :class="{ selected: selectedId === s.id }"
            @click="emit('select', s.id)"
            @dblclick="emit('resume', s)"
          >
            <span class="glyph">{}</span>
            <span class="ltitle">
              <template v-if="highlight(s.title, query).hasHit">
                {{ highlight(s.title, query).pre
                }}<mark>{{ highlight(s.title, query).hit }}</mark
                >{{ highlight(s.title, query).post }}
              </template>
              <template v-else>{{ s.title }}</template>
            </span>
            <span class="lmeta">{{ s.messages }} · {{ relTime(s.updated) }}</span>
            <button class="resume" title="Resume" @click.stop="emit('resume', s)">
              <svg width="10" height="10" viewBox="0 0 12 12" fill="currentColor">
                <path d="M2.5 1.6c0-.4.44-.65.78-.43l6.4 4.4c.3.2.3.66 0 .86l-6.4 4.4a.5.5 0 0 1-.78-.43V1.6Z" />
              </svg>
            </button>
          </div>
        </div>
      </template>
    </div>

    <div class="footer">
      <button class="theme" title="Toggle theme" @click="toggleTheme">
        <span class="swatch"></span>{{ themeLabel() }}
      </button>
      <button class="icon-btn" title="Settings" @click="emit('openSettings')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="1.6" />
          <path
            d="M19.4 13a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z"
            stroke="currentColor"
            stroke-width="1.6"
          />
        </svg>
      </button>
    </div>
  </aside>
</template>

<style scoped>
.compact {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--panel);
}

/* header ---------------------------------------------------------------- */
.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 12px 14px 8px;
  flex: none;
}
.brand {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}
.title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.8px;
  color: var(--dim);
}
.count {
  font-family: var(--mono);
  font-size: 11px;
  color: var(--faint);
}
.actions {
  display: flex;
  gap: 2px;
  flex: none;
}
.icon-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 7px;
  border: none;
  background: transparent;
  color: var(--dim);
  cursor: pointer;
}
.icon-btn:hover,
.icon-btn.on {
  background: var(--hover);
  color: var(--tx);
}

/* options popover ------------------------------------------------------- */
.opt-wrap {
  position: relative;
}
.overlay {
  position: fixed;
  inset: 0;
  z-index: 20;
}
.menu {
  position: absolute;
  right: 0;
  top: 34px;
  z-index: 21;
  width: 250px;
  background: var(--panel2);
  border: 1px solid var(--bd2);
  border-radius: 12px;
  box-shadow: 0 24px 60px -18px rgba(0, 0, 0, 0.6);
  padding: 8px;
}
.menu-label {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.6px;
  text-transform: uppercase;
  color: var(--faint);
  padding: 6px 6px 5px;
}
.menu .segmented {
  display: flex;
  padding: 2px;
  gap: 2px;
  background: var(--bg);
  border: 1px solid var(--bd2);
  border-radius: 9px;
  margin-bottom: 4px;
}
.menu .segmented button {
  flex: 1;
  padding: 5px 0;
  border-radius: 7px;
  border: none;
  cursor: pointer;
  font-size: 12px;
  background: transparent;
  color: var(--dim);
}
.menu .segmented button.on {
  background: var(--sel);
  color: var(--tx);
  font-weight: 600;
}
.opt {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  width: 100%;
  text-align: left;
  padding: 7px 8px;
  border-radius: 8px;
  border: none;
  cursor: pointer;
  background: transparent;
  color: var(--tx);
}
.opt:hover {
  background: var(--hover);
}
.opt.active {
  background: var(--acc-soft);
}
.opt-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}
.opt-title {
  font-size: 12.5px;
  font-weight: 550;
}
.opt-sub {
  font-size: 11px;
  color: var(--faint);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.check {
  color: var(--acc);
  font-size: 13px;
  flex: none;
}

/* search ---------------------------------------------------------------- */
.search {
  position: relative;
  padding: 0 12px 8px;
  flex: none;
}
.search-icon {
  position: absolute;
  left: 22px;
  top: 50%;
  transform: translateY(-60%);
  opacity: 0.55;
}
.search input {
  width: 100%;
  height: 32px;
  padding: 0 30px 0 32px;
  border-radius: 8px;
  border: 1px solid var(--bd2);
  background: var(--panel2);
  color: var(--tx);
  font-size: 13px;
}
.clear {
  position: absolute;
  right: 20px;
  top: 50%;
  transform: translateY(-60%);
  width: 18px;
  height: 18px;
  border-radius: 5px;
  border: none;
  background: var(--hover);
  color: var(--dim);
  cursor: pointer;
  font-size: 12px;
  line-height: 1;
}

/* tree ------------------------------------------------------------------ */
.tree {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 2px 8px 16px;
}
.empty {
  padding: 24px 12px;
  font-size: 12.5px;
  color: var(--faint);
  text-align: center;
}

.proj {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 8px;
  border: none;
  border-radius: 7px;
  background: transparent;
  color: var(--tx);
  cursor: pointer;
  text-align: left;
}
.proj:hover {
  background: var(--hover);
}
.chevron {
  flex: none;
  color: var(--faint);
  transition: transform 0.12s ease;
}
.proj.open .chevron {
  transform: rotate(90deg);
  color: var(--dim);
}
.pname {
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: var(--mono);
  font-size: 12.5px;
  font-weight: 550;
}
.pcount {
  flex: none;
  font-size: 11px;
  color: var(--faint);
}

.children {
  margin: 1px 0 3px;
}
.leaf {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px 5px 14px;
  margin-left: 13px;
  border-left: 1px solid var(--bd);
  border-radius: 0 7px 7px 0;
  cursor: pointer;
}
.leaf.flat {
  margin-left: 0;
  padding-left: 8px;
  border-left: none;
  border-radius: 7px;
}
.leaf:hover {
  background: var(--hover);
}
.leaf.selected {
  background: var(--sel);
  box-shadow: inset 2px 0 0 var(--acc);
  border-left-color: transparent;
}
.glyph {
  flex: none;
  font-family: var(--mono);
  font-size: 11px;
  color: var(--acc);
  opacity: 0.8;
}
.ltitle {
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 13px;
  color: var(--tx);
}
.lmeta {
  flex: none;
  font-family: var(--mono);
  font-size: 10.5px;
  color: var(--faint);
}
.resume {
  flex: none;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  border: 1px solid var(--bd2);
  background: transparent;
  color: var(--dim);
  cursor: pointer;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.1s ease;
}
.leaf:hover .resume,
.leaf.selected .resume {
  opacity: 1;
  pointer-events: auto;
}
.resume:hover {
  border-color: var(--acc);
  background: var(--acc);
  color: var(--acc-tx);
}

/* footer ---------------------------------------------------------------- */
.footer {
  border-top: 1px solid var(--bd);
  padding: 10px 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  flex: none;
}
.theme {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 9px;
  border-radius: 7px;
  border: 1px solid var(--bd2);
  background: transparent;
  color: var(--dim);
  cursor: pointer;
  font-size: 11.5px;
}
.theme:hover {
  background: var(--hover);
}
.swatch {
  width: 11px;
  height: 11px;
  border-radius: 50%;
  background: var(--acc);
  display: inline-block;
}
.footer .icon-btn {
  border: 1px solid var(--bd2);
}
</style>
