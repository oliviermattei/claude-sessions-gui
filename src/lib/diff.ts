export interface DiffLine {
  sign: " " | "-" | "+";
  text: string;
}

/**
 * Cheap line diff: trim the shared leading/trailing lines, mark the rest as
 * removed (old) then added (new). Good enough to visualise an Edit's before/after
 * without pulling in a full LCS library.
 */
export function lineDiff(oldStr: string, newStr: string): DiffLine[] {
  const a = oldStr.split("\n");
  const b = newStr.split("\n");

  let start = 0;
  while (start < a.length && start < b.length && a[start] === b[start]) start++;

  let endA = a.length;
  let endB = b.length;
  while (endA > start && endB > start && a[endA - 1] === b[endB - 1]) {
    endA--;
    endB--;
  }

  const out: DiffLine[] = [];
  for (let i = 0; i < start; i++) out.push({ sign: " ", text: a[i] });
  for (let i = start; i < endA; i++) out.push({ sign: "-", text: a[i] });
  for (let i = start; i < endB; i++) out.push({ sign: "+", text: b[i] });
  for (let i = endA; i < a.length; i++) out.push({ sign: " ", text: a[i] });
  return out;
}
