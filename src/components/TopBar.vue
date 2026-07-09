<script setup lang="ts">
import { ref } from "vue";
import type { Grouping, SortKey } from "../types";
import { SORTS, sortDef } from "../lib/sort";

const query = defineModel<string>("query", { required: true });
const grouping = defineModel<Grouping>("grouping", { required: true });
const sort = defineModel<SortKey>("sort", { required: true });

defineProps<{
  contextTitle: string;
  resultCount: string;
}>();

const emit = defineEmits<{ reload: [] }>();

const groupOptions: { key: Grouping; label: string }[] = [
  { key: "none", label: "None" },
  { key: "project", label: "Project" },
  { key: "date", label: "Date" },
];

const sortOpen = ref(false);
function pickSort(key: SortKey) {
  sort.value = key;
  sortOpen.value = false;
}
</script>

<template>
  <div class="topbar">
    <div class="heading">
      <h1>{{ contextTitle }}</h1>
      <span class="result-count">{{ resultCount }}</span>
    </div>

    <div class="controls">
      <!-- search -->
      <div class="search">
        <svg width="15" height="15" viewBox="0 0 16 16" fill="none" class="search-icon">
          <circle cx="7" cy="7" r="4.5" stroke="currentColor" stroke-width="1.4" />
          <path d="M10.5 10.5 14 14" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
        <input v-model="query" placeholder="Search sessions by title…" />
        <button v-if="query" class="clear" title="Clear" @click="query = ''">×</button>
      </div>

      <!-- grouping -->
      <div class="group">
        <span class="group-label">Group</span>
        <div class="segmented">
          <button
            v-for="g in groupOptions"
            :key="g.key"
            :class="{ on: grouping === g.key }"
            @click="grouping = g.key"
          >
            {{ g.label }}
          </button>
        </div>
      </div>

      <!-- reload -->
      <button class="reload" title="Rescan sessions" @click="emit('reload')">
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

      <!-- sort -->
      <div class="sort">
        <button class="sort-btn" @click="sortOpen = !sortOpen">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" class="sort-icon">
            <path
              d="M3 4.5h10M4.5 8h7M6.5 11.5h3"
              stroke="currentColor"
              stroke-width="1.4"
              stroke-linecap="round"
            />
          </svg>
          <span class="sort-lead">Sort:</span> {{ sortDef(sort).label }}
          <span class="caret">▾</span>
        </button>
        <template v-if="sortOpen">
          <div class="overlay" @click="sortOpen = false"></div>
          <div class="menu">
            <button
              v-for="opt in SORTS"
              :key="opt.key"
              class="opt"
              :class="{ active: sort === opt.key }"
              @click="pickSort(opt.key)"
            >
              <span class="opt-text">
                <span class="opt-label">{{ opt.label }}</span>
                <span class="opt-sub">{{ opt.sub }}</span>
              </span>
              <span v-if="sort === opt.key" class="check">✓</span>
            </button>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.topbar {
  padding: 16px 22px 12px;
  border-bottom: 1px solid var(--bd);
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.heading {
  display: flex;
  align-items: baseline;
  gap: 10px;
}
h1 {
  margin: 0;
  font-size: 16px;
  font-weight: 650;
  letter-spacing: -0.3px;
}
.result-count {
  font-family: var(--mono);
  font-size: 12px;
  color: var(--faint);
}

.controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.search {
  position: relative;
  flex: 1;
  min-width: 0;
}
.search-icon {
  position: absolute;
  left: 11px;
  top: 50%;
  transform: translateY(-50%);
  opacity: 0.55;
}
.search input {
  width: 100%;
  height: 36px;
  padding: 0 32px 0 33px;
  border-radius: 9px;
  border: 1px solid var(--bd2);
  background: var(--panel2);
  color: var(--tx);
  font-size: 13.5px;
}
.clear {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 20px;
  height: 20px;
  border-radius: 5px;
  border: none;
  background: var(--hover);
  color: var(--dim);
  cursor: pointer;
  font-size: 13px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.group {
  display: flex;
  align-items: center;
  gap: 8px;
}
.group-label {
  font-size: 11.5px;
  color: var(--faint);
}
.segmented {
  display: flex;
  padding: 2px;
  gap: 2px;
  background: var(--panel2);
  border: 1px solid var(--bd2);
  border-radius: 9px;
}
.segmented button {
  padding: 6px 12px;
  border-radius: 7px;
  border: none;
  cursor: pointer;
  font-size: 12.5px;
  font-weight: 500;
  background: transparent;
  color: var(--dim);
}
.segmented button.on {
  background: var(--bg);
  color: var(--tx);
  font-weight: 600;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
}

.reload {
  height: 36px;
  width: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 9px;
  border: 1px solid var(--bd2);
  background: var(--panel2);
  color: var(--dim);
  cursor: pointer;
  flex: none;
}
.reload:hover {
  background: var(--hover);
  color: var(--tx);
}

.sort {
  position: relative;
}
.sort-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 36px;
  padding: 0 12px;
  border-radius: 9px;
  border: 1px solid var(--bd2);
  background: var(--panel2);
  color: var(--tx);
  cursor: pointer;
  font-size: 13px;
  white-space: nowrap;
}
.sort-btn:hover {
  background: var(--hover);
}
.sort-icon {
  opacity: 0.7;
}
.sort-lead {
  color: var(--dim);
}
.caret {
  color: var(--faint);
  font-size: 10px;
  margin-left: 2px;
}

.overlay {
  position: fixed;
  inset: 0;
  z-index: 20;
}
.menu {
  position: absolute;
  right: 0;
  top: 42px;
  z-index: 21;
  width: 296px;
  background: var(--panel2);
  border: 1px solid var(--bd2);
  border-radius: 12px;
  box-shadow: 0 24px 60px -18px rgba(0, 0, 0, 0.6);
  padding: 6px;
  max-height: 420px;
  overflow-y: auto;
}
.opt {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  width: 100%;
  text-align: left;
  padding: 9px 11px;
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
  gap: 2px;
  text-align: left;
  min-width: 0;
}
.opt-label {
  font-size: 13px;
  font-weight: 550;
}
.opt-sub {
  font-size: 11.5px;
  color: var(--faint);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.check {
  color: var(--acc);
  font-size: 14px;
  flex: none;
}
</style>
