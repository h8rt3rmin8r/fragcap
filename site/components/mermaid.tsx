// SPDX-License-Identifier: Apache-2.0
'use client';

// Client-side Mermaid renderer. The site is a static export and the repo keeps
// the build browser-free, so diagrams are not rasterized at build time (that
// would pull a headless browser into the build and CI). Instead the mermaid
// package is imported dynamically in the browser and renders the chart source to
// SVG after hydration, following the active theme. Authors write ordinary
// ```mermaid fences; a remark transform (source.config.ts) routes each fence to
// this component, so the same fenced source renders here and on GitHub.

import { useEffect, useId, useState } from 'react';

// Follow the site theme without depending on next-themes (a transitive, not
// direct, dependency): fumadocs toggles a `dark` class on the document element.
function useIsDark(): boolean {
  const [dark, setDark] = useState(false);
  useEffect(() => {
    const el = document.documentElement;
    const read = () => setDark(el.classList.contains('dark'));
    read();
    const observer = new MutationObserver(read);
    observer.observe(el, { attributes: true, attributeFilter: ['class'] });
    return () => observer.disconnect();
  }, []);
  return dark;
}

export function Mermaid({ chart }: { chart: string }) {
  const reactId = useId();
  const isDark = useIsDark();
  const [svg, setSvg] = useState('');

  useEffect(() => {
    let active = true;
    void (async () => {
      const { default: mermaid } = await import('mermaid');
      mermaid.initialize({
        startOnLoad: false,
        securityLevel: 'strict',
        theme: isDark ? 'dark' : 'default',
        fontFamily: 'inherit',
      });
      // mermaid.render needs a DOM-id-safe string; useId carries colons.
      const renderId = 'mermaid-' + reactId.replace(/[^a-zA-Z0-9]/g, '');
      try {
        const result = await mermaid.render(renderId, chart);
        if (active) setSvg(result.svg);
      } catch (error) {
        if (active) {
          setSvg('');
          console.error('mermaid render failed', error);
        }
      }
    })();
    return () => {
      active = false;
    };
  }, [chart, isDark, reactId]);

  // Before hydration renders the SVG, show the source so the diagram content is
  // never invisible (and remains legible if JavaScript is unavailable).
  if (!svg) {
    return (
      <pre className="mermaid" aria-label="diagram source">
        <code>{chart}</code>
      </pre>
    );
  }

  // No role="img" on the wrapper: that would make the injected SVG descendants
  // presentational and leave the diagram an unnamed image. Left as a plain
  // container, mermaid's own SVG carries the semantics (its aria-roledescription
  // and the node label text), so a screen reader can read the diagram content.
  return (
    <div
      className="mermaid"
      style={{ display: 'flex', justifyContent: 'center' }}
      dangerouslySetInnerHTML={{ __html: svg }}
    />
  );
}
