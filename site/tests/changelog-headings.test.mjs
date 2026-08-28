// SPDX-License-Identifier: Apache-2.0
import assert from 'node:assert/strict';
import test from 'node:test';
import { normalizeChangelogHeadings } from '../scripts/changelog-headings.mjs';

test('normalizes siblings and descendants beneath the generated H2', () => {
  assert.deepEqual(
    normalizeChangelogHeadings([
      '### First',
      '##### Deep child',
      '#### Sibling of deep child',
      '### Second',
    ]),
    ['## First', '### Deep child', '### Sibling of deep child', '## Second'],
  );
});

test('does not interpret heading markers inside either fence style', () => {
  assert.deepEqual(
    normalizeChangelogHeadings([
      '```md',
      '###### literal',
      '```',
      '~~~text',
      '#### also literal',
      '~~~',
      '#### Authored heading',
    ]),
    [
      '```md',
      '###### literal',
      '```',
      '~~~text',
      '#### also literal',
      '~~~',
      '## Authored heading',
    ],
  );
});
