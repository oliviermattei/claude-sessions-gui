<script setup lang="ts">
import { useTheme } from "../composables/useTheme";

defineProps<{
  projects: { name: string; path: string; count: number }[];
  activeProject: string | null;
  total: number;
}>();

const emit = defineEmits<{
  selectProject: [name: string | null];
  openSettings: [];
}>();

const { toggleTheme } = useTheme();
</script>

<template>
  <aside class="sidebar">
    <div class="all-wrap">
      <button
        class="all"
        :class="{ on: !activeProject }"
        @click="emit('selectProject', null)"
      >
        <span class="left"><span class="checkbox"></span>All sessions</span>
        <span class="count">{{ total }}</span>
      </button>
    </div>

    <div class="section-label">Projects</div>
    <div class="projects">
      <button
        v-for="p in projects"
        :key="p.path"
        class="project"
        :class="{ on: activeProject === p.path }"
        :title="p.path"
        @click="emit('selectProject', activeProject === p.path ? null : p.path)"
      >
        <span class="left">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" class="folder">
            <path
              d="M1.5 4.2c0-.66.54-1.2 1.2-1.2h3.1c.4 0 .78.2 1 .53l.5.75c.22.33.6.53 1 .53h4c.66 0 1.2.54 1.2 1.2v6.06c0 .66-.54 1.2-1.2 1.2H2.7c-.66 0-1.2-.54-1.2-1.2V4.2Z"
              stroke="currentColor"
              stroke-width="1.3"
            />
          </svg>
          <span class="pname">{{ p.name }}</span>
        </span>
        <span class="count">{{ p.count }}</span>
      </button>
    </div>

    <div class="footer">
      <span class="ver">v0.1 · local</span>
      <button class="icon-btn" title="Toggle theme" @click="toggleTheme">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
          <path
            d="M2 12s3.6-7 10-7 10 7 10 7-3.6 7-10 7-10-7-10-7Z"
            stroke="currentColor"
            stroke-width="1.7"
          />
          <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="1.7" />
        </svg>
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
.sidebar {
  background: var(--panel);
  border-right: 1px solid var(--bd);
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.all-wrap {
  padding: 14px 10px 8px;
}
.all {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  border-radius: 8px;
  border: none;
  cursor: pointer;
  font-size: 13px;
  font-weight: 550;
  background: transparent;
  color: var(--dim);
}
.all.on {
  background: var(--sel);
  color: var(--tx);
}
.checkbox {
  width: 15px;
  height: 15px;
  border-radius: 4px;
  border: 1.5px solid currentColor;
  opacity: 0.85;
}

.section-label {
  padding: 14px 18px 6px;
  font-size: 10.5px;
  font-weight: 600;
  letter-spacing: 0.7px;
  text-transform: uppercase;
  color: var(--faint);
}
.projects {
  padding: 0 10px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}
.project {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
  padding: 7px 10px;
  margin-bottom: 2px;
  border-radius: 8px;
  border: none;
  cursor: pointer;
  background: transparent;
  color: var(--dim);
}
.project:hover {
  background: var(--hover);
}
.project.on {
  background: var(--sel);
  color: var(--tx);
}
.left {
  display: flex;
  align-items: center;
  gap: 9px;
  min-width: 0;
}
.folder {
  flex: none;
  opacity: 0.75;
}
.pname {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: var(--mono);
  font-size: 12px;
}
.count {
  font-family: var(--mono);
  font-size: 11.5px;
  color: var(--faint);
  flex: none;
}

.footer {
  border-top: 1px solid var(--bd);
  padding: 10px;
  display: flex;
  align-items: center;
  gap: 8px;
}
.footer .icon-btn {
  flex: none;
}
.ver {
  margin-right: auto;
  min-width: 0;
  font-family: var(--mono);
  font-size: 10.5px;
  color: var(--faint);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.icon-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px;
  border-radius: 7px;
  border: 1px solid var(--bd2);
  background: transparent;
  color: var(--dim);
  cursor: pointer;
  font-size: 11.5px;
}
.icon-btn:hover {
  background: var(--hover);
  color: var(--tx);
}
</style>
