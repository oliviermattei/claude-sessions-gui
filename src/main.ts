import { createApp } from "vue";
import "./styles/theme.css";
import App from "./App.vue";

// Tag the root with the host OS so CSS can pick a native window radius
// (see --app-radius in theme.css). The window is borderless + transparent,
// so the rounded corners are ours to draw per platform.
function detectOs(): "macos" | "windows" | "linux" {
  const ua = navigator.userAgent;
  if (/Mac/i.test(ua)) return "macos";
  if (/Win/i.test(ua)) return "windows";
  return "linux";
}
document.documentElement.dataset.os = detectOs();

createApp(App).mount("#app");
