<script setup lang="ts">
import { computed } from "vue";
import type { Session } from "../types";
import { relTime, highlight } from "../lib/format";
import { colorHex } from "../lib/colors";

const props = defineProps<{
  session: Session;
  selected: boolean;
  query: string;
}>();

const emit = defineEmits<{
  select: [];
  resume: [];
  menu: [ev: MouseEvent];
}>();

const parts = computed(() => highlight(props.session.title, props.query));
const date = computed(() => relTime(props.session.updated));
const barColor = computed(() => colorHex(props.session.color));

function onResume(e: MouseEvent) {
  e.stopPropagation();
  emit("resume");
}
</script>

<template>
  <div
    class="row"
    :class="{ selected }"
    :title="session.project"
    @click="emit('select')"
    @dblclick="emit('resume')"
    @contextmenu.prevent="emit('menu', $event)"
  >
    <span v-if="barColor" class="cbar" :style="{ background: barColor }"></span>
    <div class="body">
      <div class="title">
        <span>{{ parts.pre }}</span
        ><mark v-if="parts.hasHit">{{ parts.hit }}</mark
        ><span>{{ parts.post }}</span>
      </div>
      <div class="meta">
        <span class="project">
          <svg width="12" height="12" viewBox="0 0 16 16" fill="none" class="folder">
            <path
              d="M1.5 4.2c0-.66.54-1.2 1.2-1.2h3.1c.4 0 .78.2 1 .53l.5.75c.22.33.6.53 1 .53h4c.66 0 1.2.54 1.2 1.2v6.06c0 .66-.54 1.2-1.2 1.2H2.7c-.66 0-1.2-.54-1.2-1.2V4.2Z"
              stroke="currentColor"
              stroke-width="1.3"
            />
          </svg>
          {{ session.projectName }}
        </span>
        <span class="dot"></span>
        <span class="nowrap">{{ date }}</span>
        <span class="dot"></span>
        <span class="nowrap">{{ session.messages }} messages</span>
      </div>
    </div>

    <button class="resume" @click="onResume">
      <svg width="11" height="11" viewBox="0 0 12 12" fill="currentColor">
        <path
          d="M2.5 1.6c0-.4.44-.65.78-.43l6.4 4.4c.3.2.3.66 0 .86l-6.4 4.4a.5.5 0 0 1-.78-.43V1.6Z"
        />
      </svg>
      Resume
    </button>
  </div>
</template>

<style scoped>
.row {
  position: relative;
  display: flex;
  align-items: center;
  gap: 14px;
  cursor: pointer;
  padding: 11px 12px;
  border-radius: 10px;
  background: transparent;
  box-shadow: none;
  transition: background 0.09s ease;
}
.cbar {
  position: absolute;
  left: 3px;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 26px;
  border-radius: 2px;
  flex: none;
}
.row:hover {
  background: var(--hover);
}
.row.selected {
  background: var(--sel);
  box-shadow: inset 2px 0 0 var(--acc);
}

.body {
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-width: 0;
  flex: 1;
}
.title {
  font-size: 14.5px;
  font-weight: 560;
  letter-spacing: -0.2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.meta {
  display: flex;
  align-items: center;
  gap: 9px;
  font-size: 12px;
  color: var(--dim);
  min-width: 0;
}
.project {
  display: flex;
  align-items: center;
  gap: 5px;
  font-family: var(--mono);
  color: var(--faint);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.folder {
  flex: none;
  opacity: 0.8;
}
.dot {
  width: 3px;
  height: 3px;
  border-radius: 50%;
  background: var(--faint);
  flex: none;
}
.nowrap {
  white-space: nowrap;
  flex: none;
}

.resume {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: none;
  padding: 7px 13px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 12.5px;
  font-weight: 600;
  border: 1px solid var(--bd2);
  background: transparent;
  color: var(--dim);
  transition: all 0.1s ease;
}
.resume svg {
  flex: none;
}
.row:hover .resume,
.row.selected .resume {
  border-color: var(--acc);
  background: var(--acc);
  color: var(--acc-tx);
}
</style>
