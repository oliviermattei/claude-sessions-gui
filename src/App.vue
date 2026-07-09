<script setup lang="ts">
import { ref, computed } from "vue";
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

const { sessions, loading, error, reload } = useSessions();
const { resume } = useSettings();

const query = ref("");
const grouping = ref<Grouping>("none");
const sort = ref<SortKey>("recentUpdated");
const projectFilter = ref<string | null>(null); // stores the project path
const selectedId = ref<string | null>(null);
const settingsOpen = ref(false);

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

async function onResume(s: Session) {
  selectedId.value = s.id;
  try {
    const status = await resume(s);
    fireToast(status);
  } catch (e) {
    fireToast(`Failed: ${e}`);
  }
}
</script>

<template>
  <div class="app">
    <TitleBar />

    <div class="workspace">
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
            />
          </div>
        </template>
      </div>

        <Toast :message="toast" />
      </main>
    </div>

    <SettingsModal v-if="settingsOpen" @close="settingsOpen = false" />
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
}
.workspace {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 244px 1fr;
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
