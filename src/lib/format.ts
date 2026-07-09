// Date / text formatting helpers, ported from the prototype's support.js.

const MIN = 60_000;

/** Relative time label from an epoch-ms timestamp. */
export function relTime(ts: number, now: number = Date.now()): string {
  const min = Math.max(0, (now - ts) / MIN);
  if (min < 1) return "just now";
  if (min < 60) return Math.round(min) + "m ago";
  if (min < 24 * 60) return Math.round(min / 60) + "h ago";
  const d = Math.round(min / (60 * 24));
  if (d === 1) return "Yesterday";
  if (d < 7) return d + "d ago";
  return new Date(ts).toLocaleDateString("en-US", { month: "short", day: "numeric" });
}

/** Coarse bucket used for "Group by date". */
export function dateBucket(ts: number, now: number = Date.now()): string {
  const min = Math.max(0, (now - ts) / MIN);
  if (min < 24 * 60) return "Today";
  if (min < 7 * 24 * 60) return "This week";
  if (min < 30 * 24 * 60) return "This month";
  return "Earlier";
}

export const DATE_BUCKET_ORDER = ["Today", "This week", "This month", "Earlier"];

export interface TitleParts {
  pre: string;
  hit: string;
  post: string;
  hasHit: boolean;
}

/** Split a title around the first case-insensitive match of `query` for highlighting. */
export function highlight(title: string, query: string): TitleParts {
  const q = query.trim();
  if (!q) return { pre: title, hit: "", post: "", hasHit: false };
  const idx = title.toLowerCase().indexOf(q.toLowerCase());
  if (idx < 0) return { pre: title, hit: "", post: "", hasHit: false };
  return {
    pre: title.slice(0, idx),
    hit: title.slice(idx, idx + q.length),
    post: title.slice(idx + q.length),
    hasHit: true,
  };
}
