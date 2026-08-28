// SPDX-License-Identifier: Apache-2.0

// Generated changelog pages already render their title as H1 and category as
// H2. Preserve source nesting below that category without allowing a skipped
// rank, including when historical source headings jump more than one level.
export function normalizeChangelogHeadings(lines) {
  const normalized = [];
  const parents = [];
  let fence = null;

  for (const line of lines) {
    const fenceMarker = line.match(/^\s*(`{3,}|~{3,})/);
    if (fenceMarker) {
      const marker = fenceMarker[1][0];
      fence = fence === marker ? null : (fence ?? marker);
      normalized.push(line);
      continue;
    }
    if (fence) {
      normalized.push(line);
      continue;
    }

    const heading = line.match(/^(#{1,6})(\s+.+)$/);
    if (!heading) {
      normalized.push(line);
      continue;
    }

    const sourceLevel = heading[1].length;
    while (parents.length && parents.at(-1).sourceLevel >= sourceLevel) {
      parents.pop();
    }
    const outputLevel = Math.min(6, (parents.at(-1)?.outputLevel ?? 1) + 1);
    parents.push({ sourceLevel, outputLevel });
    normalized.push(`${'#'.repeat(outputLevel)}${heading[2]}`);
  }

  return normalized;
}
