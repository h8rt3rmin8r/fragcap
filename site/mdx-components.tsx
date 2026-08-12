import defaultComponents from 'fumadocs-ui/mdx';
import type { MDXComponents } from 'mdx/types';

// The MDX component set. The Callout renders the "why it matters here" note as a
// distinct visual element (specification section 22.4).
export function getMDXComponents(components?: MDXComponents): MDXComponents {
  return {
    ...defaultComponents,
    ...components,
  };
}
