// SPDX-License-Identifier: Apache-2.0
import { docs } from '@/.source/server';
import { loader } from 'fumadocs-core/source';

// The content source for the documentation tree, served under /docs.
export const source = loader({
  baseUrl: '/docs',
  source: docs.toFumadocsSource(),
});
