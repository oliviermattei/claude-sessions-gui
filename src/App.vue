<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import type { Grouping, SortKey, Session } from "./types";
import { useSessions } from "./composables/useSessions";
import { useSettings } from "./composables/useSettings";
import { sortDef } from "./lib/sort";
import { dateBucket, DATE_BUCKET_ORDER } from "./lib/format";
import TitleBar from "./components/TitleBar.vue";
import Sidebar from "./components/Sidebar.vue";
import TopBar from "./components/TopBar.vue";
import SessionRow from "./components/SessionRow.vue";
import EmptyState from "./components/EmptyState.vue";
import Toast from "./components/Toast.vue";
import SettingsModal from "./components/SettingsModal.vue";
import CompactView from "./components/CompactView.vue";
import TranscriptViewer from "./components/TranscriptViewer.vue";
import ContextMenu, { type MenuItem } from "./components/ContextMenu.vue";
import { useTheme } from "./composables/useTheme";
import { useTranscript } from "./composables/useTranscript";
import { revealItemInDir } from "@tauri-apps/plugin-opener";

const { sessions, loading, error, reload } = useSessions();
const { resume, launchers, defaultLauncher } = useSettings();
const { layout } = useTheme();
const {
  transcript,
  loading: transcriptLoading,
  error: transcriptError,
  load: loadTranscript,
} = useTranscript();

// getCurrentWindow() throws synchronously outside the Tauri runtime (plain vite
// preview / a browser), so guard it — otherwise setup throws and the app never
// mounts.
function safeWindow() {
  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
}

// Compact mode lets the window collapse down to just the tree pane (~30% of the
// default width); comfortable keeps the roomier two-pane minimum.
watch(
  layout,
  (mode) => {
    const min = mode === "compact" ? new LogicalSize(360, 560) : new LogicalSize(880, 560);
    safeWindow()
      ?.setMinSize(min)
      .catch(() => {});
  },
  { immediate: true },
);

const query = ref("");
const grouping = ref<Grouping>("none");
const sort = ref<SortKey>("recentUpdated");
// Compact view keeps its own group/sort so switching layouts doesn't disturb
// the comfortable defaults. Compact defaults to a project accordion.
const compactGrouping = ref<Grouping>("project");
const compactSort = ref<SortKey>("recentUpdated");
const projectFilter = ref<string | null>(null); // stores the project path
const selectedId = ref<string | null>(null);
const settingsOpen = ref(false);

// The transcript viewer follows the current selection.
const selectedSession = computed(
  () => sessions.value.find((s) => s.id === selectedId.value) ?? null,
);
watch(selectedSession, (s) => loadTranscript(s), { immediate: true });

// In compact mode the window itself is the widget: it collapses to just the
// sidebar width when no transcript is open, and grows to fit the viewer when one
// is. Comfortable mode just makes sure the window isn't stuck at a compact width.
async function fitWindow() {
  const win = safeWindow();
  if (!win) return;
  // Logical (CSS) px straight from the DOM — no innerSize()/scaleFactor() reads,
  // which can silently fail and leave the window stuck at a wide width.
  const w = window.innerWidth;
  const h = window.innerHeight;
  try {
    // The settings modal needs a fixed footprint; widen the window to fit it
    // instead of letting the compact width squeeze it.
    if (settingsOpen.value) {
      if (w < 780) await win.setSize(new LogicalSize(820, h));
      return;
    }
    if (layout.value === "compact") {
      if (selectedSession.value) {
        if (w < 700) await win.setSize(new LogicalSize(1040, h));
      } else {
        await win.setSize(new LogicalSize(360, h));
      }
    } else if (w < 900) {
      await win.setSize(new LogicalSize(1200, h));
    }
  } catch {
    /* no-op outside the Tauri runtime */
  }
}
watch([layout, selectedSession, settingsOpen], fitWindow, { immediate: true });

const toast = ref<string | null>(null);
let toastTimer: ReturnType<typeof setTimeout> | undefined;
function fireToast(msg: string) {
  toast.value = msg;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (toast.value = null), 2400);
}

// --- derived data ---------------------------------------------------------

const projectList = computed(() => {
  const map = new Map<
    string,
    { name: string; path: string; count: number; recent: number }
  >();
  for (const s of sessions.value) {
    const e = map.get(s.project) ?? {
      name: s.projectName,
      path: s.project,
      count: 0,
      recent: 0,
    };
    e.count++;
    e.recent = Math.max(e.recent, s.updated);
    map.set(s.project, e);
  }
  return [...map.values()].sort((a, b) => b.recent - a.recent);
});

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase();
  return sessions.value.filter(
    (s) =>
      (!projectFilter.value || s.project === projectFilter.value) &&
      (!q || s.title.toLowerCase().includes(q)),
  );
});

const sorted = computed(() => [...filtered.value].sort(sortDef(sort.value).cmp));

interface Group {
  key: string;
  header: string;
  hasHeader: boolean;
  count: number;
  sessions: Session[];
}

const groups = computed<Group[]>(() => {
  const list = sorted.value;
  if (grouping.value === "none") {
    return [{ key: "all", header: "", hasHeader: false, count: list.length, sessions: list }];
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
    // projects ordered by most recent activity within
    keys.sort(
      (a, b) =>
        Math.max(...buckets.get(b)!.map((s) => s.updated)) -
        Math.max(...buckets.get(a)!.map((s) => s.updated)),
    );
  }

  return keys.map((k) => ({
    key: k,
    header: k,
    hasHeader: true,
    count: buckets.get(k)!.length,
    sessions: buckets.get(k)!,
  }));
});

const total = computed(() => sessions.value.length);
const shown = computed(() => filtered.value.length);
const contextTitle = computed(() =>
  projectFilter.value
    ? projectList.value.find((p) => p.path === projectFilter.value)?.name ?? "Sessions"
    : "All sessions",
);
const resultCount = computed(() =>
  shown.value === total.value
    ? `${total.value} sessions`
    : `${shown.value} of ${total.value}`,
);

const isEmpty = computed(() => !loading.value && shown.value === 0);
const emptyByQuery = computed(() => isEmpty.value && !!query.value.trim());
const emptyTitle = computed(() =>
  emptyByQuery.value ? `No sessions match “${query.value.trim()}”` : "No sessions here yet",
);
const emptyMsg = computed(() =>
  emptyByQuery.value
    ? "Try a different search term, or clear the filter to see everything."
    : "Start a session in your terminal with the Claude CLI and it will show up here automatically.",
);

// --- actions --------------------------------------------------------------

async function onResume(s: Session, launcher?: (typeof launchers.value)[number]) {
  selectedId.value = s.id;
  try {
    const status = await resume(s, launcher);
    fireToast(status);
  } catch (e) {
    fireToast(`Failed: ${e}`);
  }
}

// --- context menu ---------------------------------------------------------

const menu = ref<{ session: Session; x: number; y: number } | null>(null);

function openMenu(s: Session, e: MouseEvent) {
  menu.value = { session: s, x: e.clientX, y: e.clientY };
}

// Kill the webview's native right-click menu (Inspect Element, Reload, …) app-wide,
// keeping only our own context menu. Editable fields keep theirs (paste/select).
onMounted(() => {
  window.addEventListener("contextmenu", (e) => {
    const t = e.target as HTMLElement | null;
    if (t?.closest("input, textarea, [contenteditable='true']")) return;
    e.preventDefault();
  });
});

const menuItems = computed<MenuItem[]>(() => {
  const items: MenuItem[] = [
    { type: "item", key: "open", label: "Open transcript" },
    { type: "item", key: "resume", label: "Resume", sub: defaultLauncher.value?.label },
  ];
  if (launchers.value.length) {
    items.push({ type: "header", label: "Resume in" });
    for (const l of launchers.value) {
      items.push({ type: "item", key: `launch:${l.id}`, label: l.label, sub: l.kind });
    }
  }
  items.push(
    { type: "divider" },
    { type: "item", key: "copy-cmd", label: "Copy resume command" },
    { type: "item", key: "copy-id", label: "Copy session ID" },
    { type: "item", key: "copy-path", label: "Copy project path" },
    { type: "divider" },
    { type: "item", key: "reveal", label: "Reveal in Finder" },
  );
  return items;
});

async function onMenuAction(key: string) {
  const s = menu.value?.session;
  menu.value = null;
  if (!s) return;

  if (key === "open") {
    selectedId.value = s.id;
    return;
  }
  if (key === "resume") {
    onResume(s);
    return;
  }
  if (key.startsWith("launch:")) {
    const l = launchers.value.find((x) => x.id === key.slice("launch:".length));
    onResume(s, l);
    return;
  }
  try {
    if (key === "copy-cmd") {
      await navigator.clipboard.writeText(`claude --resume ${s.id}`);
      fireToast("Copied resume command");
    } else if (key === "copy-id") {
      await navigator.clipboard.writeText(s.id);
      fireToast("Copied session ID");
    } else if (key === "copy-path") {
      await navigator.clipboard.writeText(s.project);
      fireToast("Copied project path");
    } else if (key === "reveal") {
      await revealItemInDir(s.path);
    }
  } catch (e) {
    fireToast(`Failed: ${e}`);
  }
}
</script>

<template>
  <div class="app">
    <TitleBar />

    <div
      v-if="layout === 'compact'"
      class="workspace compact-workspace"
      :class="{ 'has-viewer': selectedSession }"
    >
      <CompactView
        v-model:query="query"
        v-model:grouping="compactGrouping"
        v-model:sort="compactSort"
        :sessions="sessions"
        :selected-id="selectedId"
        @select="selectedId = $event"
        @resume="onResume"
        @menu="openMenu"
        @reload="reload"
        @open-settings="settingsOpen = true"
      />
      <div v-if="selectedSession" class="viz-pane">
        <TranscriptViewer
          :session="selectedSession"
          :transcript="transcript"
          :loading="transcriptLoading"
          :error="transcriptError"
          @resume="onResume(selectedSession)"
          @close="selectedId = null"
        />
      </div>
    </div>

    <div v-else class="workspace" :class="{ 'has-viewer': selectedSession }">
      <Sidebar
        :projects="projectList"
        :active-project="projectFilter"
        :total="total"
        @select-project="projectFilter = $event"
        @open-settings="settingsOpen = true"
      />

      <main class="main">
      <TopBar
        v-model:query="query"
        v-model:grouping="grouping"
        v-model:sort="sort"
        :context-title="contextTitle"
        :result-count="resultCount"
        @reload="reload"
      />

      <div class="list">
        <div v-if="loading" class="status">Scanning sessions…</div>
        <div v-else-if="error" class="status error">{{ error }}</div>

        <EmptyState
          v-else-if="isEmpty"
          :title="emptyTitle"
          :message="emptyMsg"
          :show-clear="emptyByQuery"
          @clear="query = ''"
        />

        <template v-else>
          <div v-for="grp in groups" :key="grp.key" class="group">
            <div v-if="grp.hasHeader" class="group-header">
              <span class="gh-title">{{ grp.header }}</span>
              <span class="gh-count">{{ grp.count }}</span>
              <div class="gh-line"></div>
            </div>
            <SessionRow
              v-for="s in grp.sessions"
              :key="s.id"
              :session="s"
              :selected="selectedId === s.id"
              :query="query"
              @select="selectedId = s.id"
              @resume="onResume(s)"
              @menu="openMenu(s, $event)"
            />
          </div>
        </template>
      </div>

        <Toast :message="toast" />
      </main>

      <div v-if="selectedSession" class="viz-pane">
        <TranscriptViewer
          :session="selectedSession"
          :transcript="transcript"
          :loading="transcriptLoading"
          :error="transcriptError"
          @resume="onResume(selectedSession)"
          @close="selectedId = null"
        />
      </div>
    </div>

    <SettingsModal v-if="settingsOpen" @close="settingsOpen = false" />

    <ContextMenu
      v-if="menu"
      :items="menuItems"
      :x="menu.x"
      :y="menu.y"
      @select="onMenuAction"
      @close="menu = null"
    />
  </div>
</template>

<style scoped>
.app {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--bg);
  color: var(--tx);
  overflow: hidden;
  position: relative;
  border-radius: 16px;
}
.workspace {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 264px 1fr;
}
.workspace.has-viewer {
  grid-template-columns: 264px minmax(300px, 430px) 1fr;
}
.compact-workspace {
  grid-template-columns: 1fr;
}
.compact-workspace.has-viewer {
  grid-template-columns: 320px 1fr;
}
.viz-pane {
  min-width: 0;
  min-height: 0;
  border-left: 1px solid var(--bd);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}
.viz-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  color: var(--faint);
  text-align: center;
  padding: 24px;
}
.viz-empty span {
  font-size: 14px;
  font-weight: 550;
  color: var(--dim);
}
.viz-empty small {
  font-size: 12px;
}
.main {
  display: flex;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
  position: relative;
}
.list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 8px 14px 24px;
}
.status {
  padding: 40px;
  text-align: center;
  color: var(--dim);
  font-size: 13px;
}
.status.error {
  color: #e06c5b;
  font-family: var(--mono);
  font-size: 12px;
  white-space: pre-wrap;
}

.group {
  margin-top: 14px;
}
.group-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 4px 12px 8px;
  position: sticky;
  top: -8px;
  background: var(--bg);
  z-index: 2;
}
.gh-title {
  font-size: 12px;
  font-weight: 650;
  letter-spacing: 0.2px;
  color: var(--dim);
}
.gh-count {
  font-family: var(--mono);
  font-size: 11px;
  color: var(--faint);
}
.gh-line {
  flex: 1;
  height: 1px;
  background: var(--bd);
}
</style>
