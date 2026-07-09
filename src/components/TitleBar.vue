<script setup lang="ts">
import { ref, onMounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

const win = getCurrentWindow();
const maximized = ref(false);

async function refreshMaximized() {
  maximized.value = await win.isMaximized();
}
onMounted(() => {
  refreshMaximized();
  win.onResized(refreshMaximized);
});

const minimize = () => win.minimize();
const toggleMax = async () => {
  await win.toggleMaximize();
  refreshMaximized();
};
const close = () => win.close();
</script>

<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="left" data-tauri-drag-region>
      <div class="glyph">&gt;_</div>
      <span class="name">Claude Sessions</span>
      <span class="ver">v0.1 · local</span>
    </div>

    <div class="controls">
      <button class="ctl" title="Minimize" @click="minimize">
        <svg width="11" height="11" viewBox="0 0 12 12"><rect x="1.5" y="5.5" width="9" height="1" fill="currentColor" /></svg>
      </button>
      <button class="ctl" title="Maximize" @click="toggleMax">
        <svg v-if="!maximized" width="11" height="11" viewBox="0 0 12 12" fill="none">
          <rect x="1.5" y="1.5" width="9" height="9" stroke="currentColor" stroke-width="1.1" />
        </svg>
        <svg v-else width="11" height="11" viewBox="0 0 12 12" fill="none">
          <rect x="1.5" y="3.2" width="7.3" height="7.3" stroke="currentColor" stroke-width="1.1" />
          <path d="M3.7 3.2V1.5h7.3v7.3H9.1" stroke="currentColor" stroke-width="1.1" />
        </svg>
      </button>
      <button class="ctl close" title="Close" @click="close">
        <svg width="11" height="11" viewBox="0 0 12 12" fill="none">
          <path d="M2 2l8 8M10 2l-8 8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  height: 36px;
  flex: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--panel);
  border-bottom: 1px solid var(--bd);
  padding-left: 12px;
  user-select: none;
}
.left {
  display: flex;
  align-items: center;
  gap: 9px;
  height: 100%;
  flex: 1;
  min-width: 0;
}
.glyph {
  width: 20px;
  height: 20px;
  border-radius: 6px;
  background: var(--acc);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--acc-tx);
  font: 600 10px/1 var(--mono);
  letter-spacing: -0.5px;
}
.name {
  font-size: 12.5px;
  font-weight: 600;
  letter-spacing: -0.2px;
  color: var(--tx);
}
.ver {
  font-size: 11px;
  color: var(--faint);
  font-family: var(--mono);
}

.controls {
  display: flex;
  align-items: stretch;
  height: 100%;
}
.ctl {
  width: 44px;
  height: 100%;
  border: none;
  background: transparent;
  color: var(--dim);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}
.ctl:hover {
  background: var(--hover);
  color: var(--tx);
}
.ctl.close:hover {
  background: #e04c3f;
  color: #fff;
}
</style>
