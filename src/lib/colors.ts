// Claude's session accent colors (the `agent-color` row in a transcript).
// Order mirrors Claude's own palette: [red|blue|green|yellow|purple|orange|pink|cyan].

export const SESSION_COLORS = [
  "red",
  "blue",
  "green",
  "yellow",
  "purple",
  "orange",
  "pink",
  "cyan",
] as const;

export type SessionColor = (typeof SESSION_COLORS)[number] | "default";

// Hexes tuned to read on both light and dark backgrounds.
const HEX: Record<string, string> = {
  red: "#e0625b",
  blue: "#5b8fe0",
  green: "#54b881",
  yellow: "#d8b34a",
  purple: "#a97fe0",
  orange: "#e0955b",
  pink: "#e07fc0",
  cyan: "#4fb8c9",
};

/** Resolve a color name to a CSS hex, or null for "default"/unknown/empty. */
export function colorHex(c?: string | null): string | null {
  if (!c || c === "default") return null;
  return HEX[c] ?? null;
}

/** "pink" → "Pink" for menu labels. */
export function colorLabel(c: string): string {
  return c.charAt(0).toUpperCase() + c.slice(1);
}
