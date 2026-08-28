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

test('only closes on a plain same-character fence at least as long as its opener', () => {
  assert.deepEqual(
    normalizeChangelogHeadings([
      '````md',
      '```not-a-close',
      '### literal after suffix',
      '```',
      '#### literal after short marker',
      '~~~~',
      '##### literal after other marker',
      '    `````',
      '###### literal after indented marker',
      '`````',
      '#### Authored heading',
    ]),
    [
      '````md',
      '```not-a-close',
      '### literal after suffix',
      '```',
      '#### literal after short marker',
      '~~~~',
      '##### literal after other marker',
      '    `````',
      '###### literal after indented marker',
      '`````',
      '## Authored heading',
    ],
  );
});

test('normalizes headings indented up to three spaces but not indented code', () => {
  assert.deepEqual(
    normalizeChangelogHeadings([
      ' ### One space',
      '   ##### Three-space child',
      '    #### Indented code',
      '```bad`info',
      '  #### Heading after invalid backtick opener',
    ]),
    [
      ' ## One space',
      '   ### Three-space child',
      '    #### Indented code',
      '```bad`info',
      '  ### Heading after invalid backtick opener',
    ],
  );
});
