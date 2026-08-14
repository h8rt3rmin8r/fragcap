// SPDX-License-Identifier: Apache-2.0
import { defineDocs, defineConfig } from 'fumadocs-mdx/config';

export const docs = defineDocs({
  dir: 'content/docs',
});

// Rewrite ```mermaid fences into a <Mermaid chart="..."> element before Shiki
// highlights them, so an ordinary fence (the same source GitHub renders) becomes
// the client renderer wired in mdx-components.tsx. A top-level walk is enough:
// mermaid fences are block-level `code` nodes. Kept dependency-free (no
// unist-util-visit) since the shape needed is this narrow.
function remarkMermaid() {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return (tree: any) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const walk = (node: any) => {
      if (!node || !Array.isArray(node.children)) return;
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      node.children = node.children.map((child: any) => {
        if (child && child.type === 'code' && child.lang === 'mermaid') {
          return {
            type: 'mdxJsxFlowElement',
            name: 'Mermaid',
            attributes: [
              { type: 'mdxJsxAttribute', name: 'chart', value: child.value },
            ],
            children: [],
          };
        }
        walk(child);
        return child;
      });
    };
    walk(tree);
  };
}

export default defineConfig({
  mdxOptions: {
    remarkPlugins: [remarkMermaid],
  },
});
