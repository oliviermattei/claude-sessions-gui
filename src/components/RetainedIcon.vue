<script setup lang="ts">
// Marks a session as exempted from Claude Code's automatic cleanup: a clock with
// a restore arrow — the mechanism is literally a timestamp being wound forward.
// Used in both the comfortable row and the compact leaf, so the glyph lives in
// one place.
withDefaults(defineProps<{ size?: number }>(), { size: 13 });
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="currentColor"
    aria-hidden="true"
  >
    <path
      fill-rule="evenodd"
      clip-rule="evenodd"
      d="M3.515 3.515A12 12 0 0 1 24 12c0 4.68-2.303 7.433-4.532 9.468L21 23l-1 1h-5v-5l1-1l1.697 1.697C19.751 17.837 21.5 15.702 21.5 12a9.5 9.5 0 1 0-9.5 9.5V24A12 12 0 0 1 3.515 3.515M10.25 12V5h3.5v5.25H17v3.5h-6.75z"
    />
  </svg>
</template>

<style scoped>
svg {
  flex: none;
}
</style>
