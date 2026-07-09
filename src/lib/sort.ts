import type { Session, SortKey } from "../types";

export interface SortDef {
  key: SortKey;
  label: string;
  sub: string;
  /** Comparator: negative if a should come before b. */
  cmp: (a: Session, b: Session) => number;
}

// "Latest Summary" from the prototype is dropped: Claude transcripts carry no
// summary timestamp to sort on. The remaining six map to real data.
export const SORTS: SortDef[] = [
  {
    key: "recentUpdated",
    label: "Recently Updated",
    sub: "Most recent message first",
    cmp: (a, b) => b.updated - a.updated,
  },
  {
    key: "recentModified",
    label: "Recently Modified",
    sub: "Most recently changed file first",
    cmp: (a, b) => b.modified - a.modified,
  },
  {
    key: "recentCreated",
    label: "Recently Created",
    sub: "Newest session first",
    cmp: (a, b) => b.created - a.created,
  },
  {
    key: "mostMessages",
    label: "Most Messages",
    sub: "Highest message count first",
    cmp: (a, b) => b.messages - a.messages,
  },
  {
    key: "titleAZ",
    label: "Title A–Z",
    sub: "Alphabetical by title",
    cmp: (a, b) => a.title.toLowerCase().localeCompare(b.title.toLowerCase()),
  },
  {
    key: "oldestFirst",
    label: "Oldest First",
    sub: "Oldest session first",
    cmp: (a, b) => a.created - b.created,
  },
];

export function sortDef(key: SortKey): SortDef {
  return SORTS.find((s) => s.key === key) ?? SORTS[0];
}
