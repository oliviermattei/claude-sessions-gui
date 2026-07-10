import { marked } from "marked";
import hljs from "highlight.js/lib/core";

// Register only the languages that show up in Claude transcripts — keeps the
// bundle a fraction of the full highlight.js build.
import bash from "highlight.js/lib/languages/bash";
import javascript from "highlight.js/lib/languages/javascript";
import typescript from "highlight.js/lib/languages/typescript";
import rust from "highlight.js/lib/languages/rust";
import python from "highlight.js/lib/languages/python";
import json from "highlight.js/lib/languages/json";
import xml from "highlight.js/lib/languages/xml";
import css from "highlight.js/lib/languages/css";
import yaml from "highlight.js/lib/languages/yaml";
import markdown from "highlight.js/lib/languages/markdown";
import diff from "highlight.js/lib/languages/diff";
import go from "highlight.js/lib/languages/go";
import sql from "highlight.js/lib/languages/sql";
import shell from "highlight.js/lib/languages/shell";
import plaintext from "highlight.js/lib/languages/plaintext";

for (const [name, lang] of Object.entries({
  bash,
  javascript,
  typescript,
  rust,
  python,
  json,
  xml,
  css,
  yaml,
  markdown,
  diff,
  go,
  sql,
  shell,
  plaintext,
})) {
  hljs.registerLanguage(name, lang);
}
hljs.registerAliases(["js"], { languageName: "javascript" });
hljs.registerAliases(["ts"], { languageName: "typescript" });
hljs.registerAliases(["py"], { languageName: "python" });
hljs.registerAliases(["html", "vue"], { languageName: "xml" });
hljs.registerAliases(["sh", "zsh"], { languageName: "bash" });
hljs.registerAliases(["yml"], { languageName: "yaml" });

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

const renderer = new marked.Renderer();

// Highlight fenced code blocks. Unknown languages fall back to auto-detection.
renderer.code = ({ text, lang }) => {
  const language = lang && hljs.getLanguage(lang) ? lang : "";
  const html = language
    ? hljs.highlight(text, { language }).value
    : hljs.highlightAuto(text).value;
  return `<pre class="code"><code class="hljs">${html}</code></pre>`;
};

// Drop raw HTML from transcript text so it can't inject markup into the viewer.
renderer.html = ({ text }) => escapeHtml(text);

marked.use({
  renderer,
  breaks: true,
  gfm: true,
});

/** Render trusted-but-untrusted transcript markdown to safe HTML (no raw tags). */
export function renderMarkdown(text: string): string {
  return marked.parse(text, { async: false }) as string;
}
