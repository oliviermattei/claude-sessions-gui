<script setup lang="ts">
defineProps<{
  message: string | null;
  /** Label of an optional button, e.g. "Open". */
  action?: string | null;
}>();
defineEmits<{ action: [] }>();
</script>

<template>
  <Transition name="toast">
    <div v-if="message" class="toast">
      <span class="pulse"></span>
      <span class="text">{{ message }}</span>
      <button v-if="action" class="action" @click="$emit('action')">{{ action }}</button>
    </div>
  </Transition>
</template>

<style scoped>
.toast {
  position: absolute;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 30;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 11px 16px;
  background: var(--panel2);
  border: 1px solid var(--bd2);
  border-radius: 10px;
  box-shadow: 0 20px 50px -16px rgba(0, 0, 0, 0.6);
  font-size: 13px;
  max-width: 70%;
}
.pulse {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--acc);
  box-shadow: 0 0 0 4px var(--acc-soft);
  flex: none;
}
.text {
  font-family: var(--mono);
  font-size: 12.5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.action {
  flex: none;
  border: 1px solid var(--bd2);
  background: transparent;
  color: var(--acc);
  border-radius: 7px;
  padding: 3px 9px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}
.action:hover {
  background: var(--hover);
}
.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(6px);
}
</style>
