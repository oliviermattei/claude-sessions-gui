<script setup lang="ts">
import { computed } from "vue";
import type { Session } from "../types";
import { relTime, highlight } from "../lib/format";
import { colorHex } from "../lib/colors";
import RetainedIcon from "./RetainedIcon.vue";

const props = defineProps<{
  session: Session;
  selected: boolean;
  query: string;
  /** Retention window shown in the badge tooltip, e.g. "org policy · 14 days". */
  policyLabel?: string;
}>();

const emit = defineEmits<{
  select: [];
  resume: [];
  menu: [ev: MouseEvent];
}>();

const parts = computed(() => highlight(props.session.title, props.query));
const date = computed(() => relTime(props.session.updated));
const barColor = computed(() => colorHex(props.session.color));

const retainedHint = computed(() => {
  const window = props.policyLabel ? ` (${props.policyLabel})` : "";
  return props.session.archived
    ? `Archived — the transcript was cleaned up${window}; restore it to resume`
    : `Kept beyond the automatic cleanup${window}`;
});

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
        <span class="ttext">
          <span>{{ parts.pre }}</span
          ><mark v-if="parts.hasHit">{{ parts.hit }}</mark
          ><span>{{ parts.post }}</span>
        </span>
        <span
          v-if="session.retained"
          class="retained"
          :class="{ archived: session.archived }"
          :title="retainedHint"
        >
          <RetainedIcon />
        </span>
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
        <template v-if="session.archived">
          <span class="dot"></span>
          <span class="nowrap pill">archived</span>
        </template>
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
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 14.5px;
  font-weight: 560;
  letter-spacing: -0.2px;
  min-width: 0;
}
/* The text owns the ellipsis so the retention badge is never truncated away. */
.ttext {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.retained {
  display: flex;
  align-items: center;
  flex: none;
  color: var(--retain);
}
.retained.archived {
  color: var(--faint);
}
.pill {
  padding: 1px 6px;
  border-radius: 5px;
  border: 1px solid var(--bd2);
  font-size: 10.5px;
  font-weight: 600;
  letter-spacing: 0.03em;
  text-transform: uppercase;
  color: var(--faint);
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
