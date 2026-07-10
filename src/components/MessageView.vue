<script setup lang="ts">
import { computed } from "vue";
import type { Block, Message } from "../types";
import { renderMarkdown } from "../lib/markdown";
import { lineDiff, type DiffLine } from "../lib/diff";

const props = defineProps<{ message: Message }>();

type ToolUse = Extract<Block, { kind: "toolUse" }>;

const DIFF_TOOLS = new Set(["Edit", "MultiEdit", "Write"]);

// A user turn that only carries tool_result blocks is tool feedback, not a real
// prompt — render it as result cards instead of a user bubble.
const isToolFeedback = computed(
  () =>
    props.message.role === "user" &&
    props.message.blocks.length > 0 &&
    props.message.blocks.every((b) => b.kind === "toolResult"),
);

const roleLabel = computed(() => (props.message.role === "user" ? "You" : "Claude"));

function fileName(p?: unknown): string {
  return typeof p === "string" ? p.split(/[\\/]/).pop() || p : "";
}

function toolSummary(b: ToolUse): string {
  const i = (b.input ?? {}) as Record<string, unknown>;
  if (b.name === "Bash") return String(i.description ?? i.command ?? "");
  if (i.file_path) return fileName(i.file_path);
  if (i.path) return fileName(i.path);
  if (i.pattern) return String(i.pattern);
  if (i.query) return String(i.query);
  if (i.command) return String(i.command);
  return "";
}

function isDiff(b: ToolUse): boolean {
  return DIFF_TOOLS.has(b.name);
}

function diffGroups(b: ToolUse): { file: string; lines: DiffLine[] }[] {
  const i = (b.input ?? {}) as Record<string, unknown>;
  if (b.name === "Write") {
    return [{ file: fileName(i.file_path), lines: lineDiff("", String(i.content ?? "")) }];
  }
  if (b.name === "MultiEdit") {
    const edits = Array.isArray(i.edits) ? i.edits : [];
    return edits.map((e: Record<string, unknown>) => ({
      file: fileName(i.file_path),
      lines: lineDiff(String(e.old_string ?? ""), String(e.new_string ?? "")),
    }));
  }
  return [{ file: fileName(i.file_path), lines: lineDiff(String(i.old_string ?? ""), String(i.new_string ?? "")) }];
}

function bashCommand(b: ToolUse): string {
  return String(((b.input ?? {}) as Record<string, unknown>).command ?? "");
}

function prettyInput(b: ToolUse): string {
  try {
    return JSON.stringify(b.input, null, 2);
  } catch {
    return String(b.input);
  }
}

function resultLineCount(text: string): number {
  return text ? text.split("\n").length : 0;
}
</script>

<template>
  <!-- tool feedback (user message of only tool_result blocks) -->
  <div v-if="isToolFeedback" class="feedback">
    <details v-for="(b, i) in message.blocks" :key="i" class="result" :class="{ error: (b as any).isError }">
      <summary>
        <span class="res-dot"></span>
        <span class="res-label">{{ (b as any).isError ? "Error" : "Result" }}</span>
        <span class="res-meta">{{ resultLineCount((b as any).text) }} lines</span>
      </summary>
      <pre class="res-body"><code>{{ (b as any).text }}</code></pre>
    </details>
  </div>

  <!-- normal message -->
  <div v-else class="msg" :class="message.role">
    <div class="rail">
      <span class="avatar" :class="message.role"></span>
      <span class="who">{{ roleLabel }}</span>
    </div>

    <div class="content">
      <template v-for="(b, i) in message.blocks" :key="i">
        <!-- text -->
        <div v-if="b.kind === 'text'" class="md" v-html="renderMarkdown(b.text)"></div>

        <!-- thinking -->
        <details v-else-if="b.kind === 'thinking'" class="thinking">
          <summary>💭 Thinking</summary>
          <div class="md" v-html="renderMarkdown(b.text)"></div>
        </details>

        <!-- tool call -->
        <div v-else-if="b.kind === 'toolUse'" class="tool">
          <div class="tool-head">
            <span class="tool-name">{{ b.name }}</span>
            <span v-if="toolSummary(b)" class="tool-arg">{{ toolSummary(b) }}</span>
          </div>

          <!-- diff for Edit / MultiEdit / Write -->
          <div v-if="isDiff(b)" class="diffs">
            <div v-for="(g, gi) in diffGroups(b)" :key="gi" class="diff">
              <div class="diff-line" v-for="(l, li) in g.lines" :key="li" :class="{
                add: l.sign === '+',
                del: l.sign === '-',
              }">
                <span class="sign">{{ l.sign }}</span><span class="dtext">{{ l.text }}</span>
              </div>
            </div>
          </div>

          <!-- bash command -->
          <pre v-else-if="b.name === 'Bash'" class="code"><code>{{ bashCommand(b) }}</code></pre>

          <!-- generic input -->
          <pre v-else class="code json"><code>{{ prettyInput(b) }}</code></pre>
        </div>

        <!-- inline tool_result (rare: assistant-side) -->
        <details v-else-if="b.kind === 'toolResult'" class="result" :class="{ error: b.isError }">
          <summary>
            <span class="res-dot"></span>
            <span class="res-label">{{ b.isError ? "Error" : "Result" }}</span>
            <span class="res-meta">{{ resultLineCount(b.text) }} lines</span>
          </summary>
          <pre class="res-body"><code>{{ b.text }}</code></pre>
        </details>
      </template>
    </div>
  </div>
</template>

<style scoped>
.msg {
  display: grid;
  grid-template-columns: 74px 1fr;
  gap: 10px;
  padding: 12px 0;
}
.rail {
  display: flex;
  align-items: center;
  gap: 7px;
  padding-top: 2px;
}
.avatar {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex: none;
}
.avatar.user {
  background: var(--dim);
}
.avatar.assistant {
  background: var(--acc);
}
.who {
  font-size: 11px;
  font-weight: 650;
  color: var(--dim);
}
.content {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

/* markdown ------------------------------------------------------------- */
.md {
  font-size: 13.5px;
  line-height: 1.62;
  color: var(--tx);
  word-break: break-word;
}
.md :deep(p) {
  margin: 0 0 8px;
}
.md :deep(p:last-child) {
  margin-bottom: 0;
}
.md :deep(ul),
.md :deep(ol) {
  margin: 4px 0 8px;
  padding-left: 20px;
}
.md :deep(li) {
  margin: 2px 0;
}
.md :deep(a) {
  color: var(--acc);
}
.md :deep(code) {
  font-family: var(--mono);
  font-size: 12px;
  background: var(--hover);
  padding: 1px 5px;
  border-radius: 4px;
}
.md :deep(pre.code) {
  margin: 8px 0;
}
.md :deep(pre.code code) {
  background: none;
  padding: 0;
}
.md :deep(h1),
.md :deep(h2),
.md :deep(h3) {
  font-size: 14px;
  margin: 12px 0 6px;
}
.md :deep(blockquote) {
  margin: 6px 0;
  padding-left: 12px;
  border-left: 2px solid var(--bd2);
  color: var(--dim);
}
.md :deep(table) {
  border-collapse: collapse;
  margin: 8px 0;
  font-size: 12.5px;
}
.md :deep(th),
.md :deep(td) {
  border: 1px solid var(--bd);
  padding: 4px 9px;
}

/* shared code block ---------------------------------------------------- */
.code {
  margin: 0;
  padding: 10px 12px;
  background: var(--panel2);
  border: 1px solid var(--bd);
  border-radius: 8px;
  overflow-x: auto;
  font-family: var(--mono);
  font-size: 12px;
  line-height: 1.5;
}
.code code {
  font-family: inherit;
}

/* thinking ------------------------------------------------------------- */
.thinking {
  border-left: 2px solid var(--bd2);
  padding-left: 10px;
}
.thinking > summary {
  cursor: pointer;
  font-size: 11.5px;
  color: var(--faint);
  list-style: none;
  user-select: none;
}
.thinking[open] > summary {
  margin-bottom: 6px;
}
.thinking .md {
  color: var(--dim);
  font-size: 12.5px;
}

/* tool call ------------------------------------------------------------ */
.tool {
  border: 1px solid var(--bd);
  border-radius: 9px;
  overflow: hidden;
  background: var(--panel);
}
.tool-head {
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding: 7px 11px;
  background: var(--panel2);
  border-bottom: 1px solid var(--bd);
}
.tool-name {
  font-family: var(--mono);
  font-size: 12px;
  font-weight: 650;
  color: var(--acc);
  flex: none;
}
.tool-arg {
  font-family: var(--mono);
  font-size: 11.5px;
  color: var(--dim);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tool .code {
  border: none;
  border-radius: 0;
  background: transparent;
}

/* diff ----------------------------------------------------------------- */
.diffs {
  font-family: var(--mono);
  font-size: 12px;
  line-height: 1.5;
  overflow-x: auto;
}
.diff + .diff {
  border-top: 1px dashed var(--bd);
}
.diff-line {
  display: flex;
  white-space: pre;
  padding: 0 10px;
}
.diff-line .sign {
  width: 14px;
  flex: none;
  color: var(--faint);
  user-select: none;
}
.diff-line.add {
  background: var(--diff-add-bg);
}
.diff-line.add .sign,
.diff-line.add .dtext {
  color: var(--diff-add-tx);
}
.diff-line.del {
  background: var(--diff-del-bg);
}
.diff-line.del .sign,
.diff-line.del .dtext {
  color: var(--diff-del-tx);
}

/* tool result ---------------------------------------------------------- */
.feedback {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 2px 0 2px 84px;
}
.result {
  border: 1px solid var(--bd);
  border-radius: 8px;
  background: var(--panel);
}
.result.error {
  border-color: color-mix(in srgb, #e06c5b 40%, var(--bd));
}
.result > summary {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  padding: 7px 11px;
  font-size: 12px;
  color: var(--dim);
  list-style: none;
  user-select: none;
}
.result > summary::-webkit-details-marker {
  display: none;
}
.res-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--faint);
  flex: none;
}
.result.error .res-dot {
  background: #e06c5b;
}
.res-label {
  font-weight: 600;
}
.res-meta {
  font-family: var(--mono);
  font-size: 11px;
  color: var(--faint);
}
.res-body {
  margin: 0;
  padding: 10px 12px;
  border-top: 1px solid var(--bd);
  max-height: 340px;
  overflow: auto;
  font-family: var(--mono);
  font-size: 11.5px;
  line-height: 1.5;
  color: var(--dim);
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
