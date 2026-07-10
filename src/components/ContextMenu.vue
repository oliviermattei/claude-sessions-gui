<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from "vue";

export interface MenuItem {
  type?: "item" | "divider" | "header";
  key?: string;
  label?: string;
  sub?: string;
  danger?: boolean;
}

const props = defineProps<{ items: MenuItem[]; x: number; y: number }>();
const emit = defineEmits<{ select: [key: string]; close: [] }>();

const el = ref<HTMLElement | null>(null);
const pos = ref({ x: props.x, y: props.y });

function clamp() {
  const r = el.value?.getBoundingClientRect();
  if (!r) return;
  let x = props.x;
  let y = props.y;
  if (x + r.width > window.innerWidth) x = window.innerWidth - r.width - 8;
  if (y + r.height > window.innerHeight) y = window.innerHeight - r.height - 8;
  pos.value = { x: Math.max(8, x), y: Math.max(8, y) };
}

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") emit("close");
}

onMounted(() => {
  clamp();
  window.addEventListener("keydown", onKey);
});
onBeforeUnmount(() => window.removeEventListener("keydown", onKey));

function pick(it: MenuItem) {
  if (it.type === "divider" || it.type === "header" || !it.key) return;
  emit("select", it.key);
}
</script>

<template>
  <div class="cm-overlay" @click="emit('close')" @contextmenu.prevent="emit('close')">
    <div
      ref="el"
      class="cm"
      :style="{ left: pos.x + 'px', top: pos.y + 'px' }"
      @click.stop
      @contextmenu.prevent.stop
    >
      <template v-for="(it, i) in items" :key="i">
        <div v-if="it.type === 'divider'" class="cm-div"></div>
        <div v-else-if="it.type === 'header'" class="cm-head">{{ it.label }}</div>
        <button v-else class="cm-item" :class="{ danger: it.danger }" @click="pick(it)">
          <span class="cm-label">{{ it.label }}</span>
          <span v-if="it.sub" class="cm-sub">{{ it.sub }}</span>
        </button>
      </template>
    </div>
  </div>
</template>

<style scoped>
.cm-overlay {
  position: fixed;
  inset: 0;
  z-index: 60;
}
.cm {
  position: fixed;
  min-width: 200px;
  max-width: 280px;
  background: var(--panel2);
  border: 1px solid var(--bd2);
  border-radius: 10px;
  box-shadow: 0 24px 60px -18px rgba(0, 0, 0, 0.6);
  padding: 5px;
}
.cm-item {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
  text-align: left;
  padding: 7px 9px;
  border: none;
  border-radius: 7px;
  background: transparent;
  color: var(--tx);
  cursor: pointer;
  font-size: 13px;
}
.cm-item:hover {
  background: var(--hover);
}
.cm-item.danger {
  color: #e06c5b;
}
.cm-item.danger:hover {
  background: color-mix(in srgb, #e06c5b 16%, transparent);
}
.cm-label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.cm-sub {
  flex: none;
  font-family: var(--mono);
  font-size: 10.5px;
  color: var(--faint);
}
.cm-div {
  height: 1px;
  background: var(--bd);
  margin: 5px 6px;
}
.cm-head {
  padding: 6px 9px 3px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.5px;
  text-transform: uppercase;
  color: var(--faint);
}
</style>
