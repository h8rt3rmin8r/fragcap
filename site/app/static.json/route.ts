import { source } from '@/lib/source';
import { createFromSource } from 'fumadocs-core/search/server';

// Static search index (specification section 22, FR-009). With output: export
// there is no server to answer queries at request time, so the whole index is
// exported once as a static file the client downloads and searches in the
// browser. createFromSource indexes the content by heading, so each glossary
// term -- one term to a heading -- is an independent search result.
export const dynamic = 'force-static';
export const revalidate = false;

export const { staticGET: GET } = createFromSource(source);
