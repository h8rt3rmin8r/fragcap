// SPDX-License-Identifier: Apache-2.0

// Generated changelog pages already render their title as H1 and category as
// H2. Preserve source nesting below that category without allowing a skipped
// rank, including when historical source headings jump more than one level.
export function normalizeChangelogHeadings(lines) {
  const normalized = [];
  const parents = [];
  let fence = null;

  for (const line of lines) {
    if (fence) {
      const closing = line.match(/^ {0,3}(`{3,}|~{3,})[ \t]*$/);
      if (
        closing
        && closing[1][0] === fence.marker
        && closing[1].length >= fence.length
      ) {
        fence = null;
      }
      normalized.push(line);
      continue;
    }

    const opening = line.match(/^ {0,3}(`{3,}|~{3,})(.*)$/);
    if (opening) {
      const marker = opening[1][0];
      const validInfo = marker !== '`' || !opening[2].includes('`');
      if (validInfo) fence = { marker, length: opening[1].length };
      normalized.push(line);
      continue;
    }

    const heading = line.match(/^( {0,3})(#{1,6})(\s+.+)$/);
    if (!heading) {
      normalized.push(line);
      continue;
    }

    const sourceLevel = heading[2].length;
    while (parents.length && parents.at(-1).sourceLevel >= sourceLevel) {
      parents.pop();
    }
    const outputLevel = Math.min(6, (parents.at(-1)?.outputLevel ?? 1) + 1);
    parents.push({ sourceLevel, outputLevel });
    normalized.push(`${heading[1]}${'#'.repeat(outputLevel)}${heading[3]}`);
  }

  return normalized;
}
