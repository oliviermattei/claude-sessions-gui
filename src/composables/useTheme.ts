import { ref, watchEffect } from "vue";

type Theme = "dark" | "light";
type Layout = "comfortable" | "compact";

const STORAGE_THEME = "cs.theme";
const STORAGE_ACCENT = "cs.accent";
const STORAGE_LAYOUT = "cs.layout";

const theme = ref<Theme>((localStorage.getItem(STORAGE_THEME) as Theme) || "dark");
const accent = ref<string>(localStorage.getItem(STORAGE_ACCENT) || "#d97757");
const layout = ref<Layout>(
  (localStorage.getItem(STORAGE_LAYOUT) as Layout) || "comfortable",
);

// Apply to the document root and persist whenever any of these change.
watchEffect(() => {
  document.documentElement.setAttribute("data-theme", theme.value);
  document.documentElement.style.setProperty("--acc", accent.value);
  localStorage.setItem(STORAGE_THEME, theme.value);
  localStorage.setItem(STORAGE_ACCENT, accent.value);
  localStorage.setItem(STORAGE_LAYOUT, layout.value);
});

export function useTheme() {
  const toggleTheme = () => {
    theme.value = theme.value === "dark" ? "light" : "dark";
  };
  return { theme, accent, layout, toggleTheme };
}
