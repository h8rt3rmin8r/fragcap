// SPDX-License-Identifier: Apache-2.0
import defaultComponents from 'fumadocs-ui/mdx';
import type { MDXComponents } from 'mdx/types';
import { Mermaid } from './components/mermaid';

// The MDX component set. The Callout renders the "why it matters here" note as a
// distinct visual element (specification section 22.4). Mermaid renders diagrams
// from ```mermaid fences: a remark transform (source.config.ts) rewrites each
// fence to a <Mermaid> element that this map resolves to the client renderer.
export function getMDXComponents(components?: MDXComponents): MDXComponents {
  return {
    ...defaultComponents,
    Mermaid,
    ...components,
  };
}
