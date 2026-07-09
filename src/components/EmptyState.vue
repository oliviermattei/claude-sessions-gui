<script setup lang="ts">
defineProps<{
  title: string;
  message: string;
  showClear: boolean;
}>();
defineEmits<{ clear: [] }>();
</script>

<template>
  <div class="empty">
    <div class="glyph">&gt;_</div>
    <div class="text">
      <span class="title">{{ title }}</span>
      <span class="msg">{{ message }}</span>
    </div>
    <button v-if="showClear" class="clear-btn" @click="$emit('clear')">Clear search</button>
  </div>
</template>

<style scoped>
.empty {
  height: 100%;
  min-height: 340px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  gap: 14px;
  padding: 40px;
}
.glyph {
  width: 56px;
  height: 56px;
  border-radius: 14px;
  border: 1.5px dashed var(--bd2);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--faint);
  font: 600 20px/1 var(--mono);
}
.text {
  display: flex;
  flex-direction: column;
  gap: 5px;
  max-width: 340px;
}
.title {
  font-size: 15px;
  font-weight: 600;
}
.msg {
  font-size: 13px;
  color: var(--dim);
  line-height: 1.5;
}
.clear-btn {
  margin-top: 2px;
  padding: 8px 15px;
  border-radius: 8px;
  border: 1px solid var(--bd2);
  background: transparent;
  color: var(--tx);
  cursor: pointer;
  font-size: 13px;
}
.clear-btn:hover {
  background: var(--hover);
}
</style>
