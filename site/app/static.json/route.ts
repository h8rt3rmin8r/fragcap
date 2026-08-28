// SPDX-License-Identifier: Apache-2.0
import { source } from '@/lib/source';
import { createFromSource } from 'fumadocs-core/search/server';
import { insertPin, type ZBSearchPlugin } from 'zbsearch';

// Static search index (specification section 22, FR-009). With output: export
// there is no server to answer queries at request time, so the whole index is
// exported once as a static file the client downloads and searches in the
// browser. createFromSource indexes the content by heading, so each glossary
// term -- one term to a heading -- is an independent search result.
export const dynamic = 'force-static';
export const revalidate = false;

const currentCommandPins: ZBSearchPlugin = {
  name: 'current-command-pins',
  afterCreate(database) {
    for (const query of ['fragcap run', 'fragcap tap']) {
      insertPin(database, {
        id: `current-command-${query.replace(' ', '-')}`,
        conditions: [{ anchoring: 'is', pattern: query }],
        consequence: {
          promote: [{ doc_id: '/docs/reference/cli', position: 0 }],
        },
      });
    }
  },
};

export const { staticGET: GET } = createFromSource(source, {
  plugins: [currentCommandPins],
});
