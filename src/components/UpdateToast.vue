<script setup lang="ts">
import { computed } from "vue";
import { useUpdater } from "../composables/useUpdater";

const { state, dismissed, install, dismiss } = useUpdater();

const visible = computed(() => state.value.status !== "none" && !dismissed.value);
const percent = computed(() =>
  state.value.status === "installing" && state.value.progress !== null
    ? `${Math.round(state.value.progress * 100)}%`
    : "",
);
</script>

<template>
  <Transition name="update">
    <div v-if="visible && state.status !== 'none'" class="update" role="status">
      <template v-if="state.status === 'available'">
        <div class="text">
          <span class="title">Update available</span>
          <span class="sub">Version {{ state.version }} is ready to install.</span>
        </div>
        <div class="actions">
          <button class="btn" @click="dismiss">Later</button>
          <button class="btn btn-primary" @click="install">Restart to update</button>
        </div>
      </template>

      <template v-else-if="state.status === 'installing'">
        <div class="text">
          <span class="title">Updating to {{ state.version }}…</span>
          <span class="sub">
            {{ percent ? `Downloading · ${percent}` : "Downloading" }} — the app restarts
            when done.
          </span>
        </div>
      </template>

      <template v-else>
        <div class="text">
          <span class="title">The update to {{ state.version }} failed</span>
          <span class="sub err" :title="state.error">{{ state.error }}</span>
        </div>
        <div class="actions">
          <button class="btn" @click="dismiss">Dismiss</button>
          <button class="btn btn-primary" @click="install">Retry</button>
        </div>
      </template>
    </div>
  </Transition>
</template>

<style scoped>
.update {
  position: absolute;
  right: 16px;
  bottom: 16px;
  z-index: 40;
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: min(320px, calc(100% - 32px));
  padding: 14px 16px;
  background: var(--panel2);
  border: 1px solid var(--bd2);
  border-radius: 10px;
  box-shadow: 0 20px 50px -16px rgba(0, 0, 0, 0.6);
  font-size: 13px;
}
.text {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}
.title {
  font-weight: 600;
}
.sub {
  color: var(--dim);
  font-size: 12px;
}
.err {
  font-family: var(--mono);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
.btn {
  padding: 6px 12px;
  border-radius: 7px;
  border: 1px solid var(--bd2);
  background: transparent;
  color: var(--dim);
  font-size: 12.5px;
  font-weight: 600;
  cursor: pointer;
}
.btn:hover {
  color: var(--tx);
  background: var(--hover);
}
.btn-primary {
  border-color: var(--acc);
  background: var(--acc);
  color: var(--acc-tx);
}
.btn-primary:hover {
  background: var(--acc);
  color: var(--acc-tx);
  filter: brightness(1.08);
}
.update-enter-active,
.update-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}
.update-enter-from,
.update-leave-to {
  opacity: 0;
  transform: translateY(6px);
}
</style>
