<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import type { Session, Transcript } from "../types";
import { relTime } from "../lib/format";
import MessageView from "./MessageView.vue";

const props = defineProps<{
  session: Session | null;
  transcript: Transcript | null;
  loading: boolean;
  error: string | null;
}>();

const emit = defineEmits<{ resume: []; close: [] }>();

// Jump to the latest message when a transcript finishes loading — the last
// exchange is what you usually want to see first.
const body = ref<HTMLElement | null>(null);
watch(
  () => [props.session?.id, props.loading] as const,
  async ([, loading]) => {
    if (loading || !props.transcript) return;
    await nextTick();
    if (body.value) body.value.scrollTop = body.value.scrollHeight;
  },
  { immediate: true },
);

// Collapse subagents by default; remember which are open.
const openAgents = ref(new Set<string>());
function toggleAgent(id: string) {
  const next = new Set(openAgents.value);
  next.has(id) ? next.delete(id) : next.add(id);
  openAgents.value = next;
}
// Reset expansion when the session changes.
watch(
  () => props.session?.id,
  () => (openAgents.value = new Set()),
);
</script>

<template>
  <section class="viewer">
    <!-- nothing selected -->
    <div v-if="!session" class="placeholder">
      <span class="ph-title">Select a session</span>
      <small>Pick a session on the left to read its transcript.</small>
    </div>

    <template v-else>
      <header class="vhead">
        <div class="vtitle" :title="session.title">{{ session.title }}</div>
        <div class="vmeta">
          <span class="proj">{{ session.projectName }}</span>
          <span class="dot"></span>
          <span>{{ relTime(session.updated) }}</span>
          <span class="dot"></span>
          <span>{{ session.messages }} messages</span>
        </div>
        <div class="vactions">
          <button class="resume" @click="emit('resume')">
            <svg width="11" height="11" viewBox="0 0 12 12" fill="currentColor">
              <path d="M2.5 1.6c0-.4.44-.65.78-.43l6.4 4.4c.3.2.3.66 0 .86l-6.4 4.4a.5.5 0 0 1-.78-.43V1.6Z" />
            </svg>
            Resume
          </button>
          <button class="close" title="Close transcript" @click="emit('close')">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
          </button>
        </div>
      </header>

      <div ref="body" class="vbody">
        <div v-if="loading" class="status">Reading transcript…</div>
        <div v-else-if="error" class="status err">{{ error }}</div>
        <div v-else-if="!transcript || !transcript.messages.length" class="status">
          No readable messages in this session.
        </div>

        <template v-else>
          <MessageView v-for="(m, i) in transcript.messages" :key="i" :message="m" />

          <!-- subagents -->
          <div v-if="transcript.subagents.length" class="agents">
            <div class="agents-label">
              Subagents
              <span class="agents-count">{{ transcript.subagents.length }}</span>
            </div>
            <div v-for="a in transcript.subagents" :key="a.id" class="agent">
              <button class="agent-head" :class="{ open: openAgents.has(a.id) }" @click="toggleAgent(a.id)">
                <svg class="chevron" width="12" height="12" viewBox="0 0 16 16" fill="none">
                  <path d="M6 4l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
                <span class="agent-id">agent · {{ a.id.replace(/^agent-/, "").slice(0, 10) }}</span>
                <span class="agent-count">{{ a.messages.length }} messages</span>
              </button>
              <div v-if="openAgents.has(a.id)" class="agent-body">
                <MessageView v-for="(m, i) in a.messages" :key="i" :message="m" />
              </div>
            </div>
          </div>
        </template>
      </div>
    </template>
  </section>
</template>

<style scoped>
.viewer {
  height: 100%;
  min-height: 0;
  min-width: 0;
  display: flex;
  flex-direction: column;
  background: var(--bg);
}

/* placeholder ---------------------------------------------------------- */
.placeholder {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  color: var(--faint);
  text-align: center;
  padding: 24px;
}
.ph-title {
  font-size: 14px;
  font-weight: 550;
  color: var(--dim);
}
.placeholder small {
  font-size: 12px;
}

/* header --------------------------------------------------------------- */
.vhead {
  flex: none;
  padding: 14px 20px 12px;
  border-bottom: 1px solid var(--bd);
  display: grid;
  grid-template-columns: 1fr auto;
  grid-template-rows: auto auto;
  column-gap: 12px;
  row-gap: 4px;
  align-items: center;
}
.vtitle {
  font-size: 15px;
  font-weight: 650;
  letter-spacing: -0.2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.vmeta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--dim);
}
.proj {
  font-family: var(--mono);
  color: var(--faint);
}
.dot {
  width: 3px;
  height: 3px;
  border-radius: 50%;
  background: var(--faint);
}
.vactions {
  grid-row: 1 / 3;
  grid-column: 2;
  display: flex;
  align-items: center;
  gap: 8px;
}
.resume {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 12.5px;
  font-weight: 600;
  border: 1px solid var(--bd2);
  background: transparent;
  color: var(--dim);
}
.resume:hover {
  border-color: var(--acc);
  background: var(--acc);
  color: var(--acc-tx);
}
.close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 8px;
  border: 1px solid var(--bd2);
  background: transparent;
  color: var(--dim);
  cursor: pointer;
  flex: none;
}
.close:hover {
  background: var(--hover);
  color: var(--tx);
}

/* body ----------------------------------------------------------------- */
.vbody {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 8px 20px 48px;
}
.status {
  padding: 40px;
  text-align: center;
  color: var(--dim);
  font-size: 13px;
}
.status.err {
  color: #e06c5b;
  font-family: var(--mono);
  font-size: 12px;
  white-space: pre-wrap;
}

/* subagents ------------------------------------------------------------ */
.agents {
  margin-top: 20px;
  border-top: 1px solid var(--bd);
  padding-top: 12px;
}
.agents-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 10.5px;
  font-weight: 600;
  letter-spacing: 0.6px;
  text-transform: uppercase;
  color: var(--faint);
  padding: 0 2px 8px;
}
.agents-count {
  font-family: var(--mono);
}
.agent {
  border: 1px solid var(--bd);
  border-radius: 9px;
  margin-bottom: 8px;
  overflow: hidden;
}
.agent-head {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 9px 11px;
  border: none;
  background: var(--panel);
  color: var(--tx);
  cursor: pointer;
  text-align: left;
}
.agent-head:hover {
  background: var(--hover);
}
.chevron {
  flex: none;
  color: var(--faint);
  transition: transform 0.12s ease;
}
.agent-head.open .chevron {
  transform: rotate(90deg);
}
.agent-id {
  flex: 1;
  font-family: var(--mono);
  font-size: 12px;
  font-weight: 550;
}
.agent-count {
  font-size: 11px;
  color: var(--faint);
}
.agent-body {
  padding: 4px 14px 8px;
  border-top: 1px solid var(--bd);
  background: var(--bg);
}
</style>
