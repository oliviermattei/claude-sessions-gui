<script setup lang="ts">
import { useProfiles } from "../composables/useProfiles";
import { colorHex } from "../lib/colors";

// The bar switching the Active Profile. Profiles come from Settings (detected
// or added there), so there is no "+" here, and with a single Profile there is
// nothing to switch: the bar is not shown.
const { visibleProfiles, activeProfileId, setActive } = useProfiles();
</script>

<template>
  <div v-if="visibleProfiles.length > 1" class="switcher" role="tablist" aria-label="Profiles">
    <button
      v-for="p in visibleProfiles"
      :key="p.id"
      class="profile"
      :class="{ on: p.id === activeProfileId }"
      role="tab"
      :aria-selected="p.id === activeProfileId"
      :title="p.path"
      @click="setActive(p.id)"
    >
      <span class="dot" :style="{ background: colorHex(p.color) ?? 'var(--faint)' }"></span>
      <span class="name">{{ p.name }}</span>
    </button>
  </div>
</template>

<style scoped>
.switcher {
  display: flex;
  gap: 4px;
  padding: 8px 10px;
  border-top: 1px solid var(--bd);
  overflow-x: auto;
  flex: none;
  scrollbar-width: none;
}
.profile {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  max-width: 140px;
  padding: 5px 9px;
  border-radius: 7px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--dim);
  cursor: pointer;
  font-size: 12px;
  font-weight: 550;
  flex: none;
}
.profile:hover {
  background: var(--hover);
}
.profile.on {
  background: var(--sel);
  border-color: var(--bd2);
  color: var(--tx);
}
.dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex: none;
}
.name {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
