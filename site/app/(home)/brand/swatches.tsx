// SPDX-License-Identifier: Apache-2.0
'use client';
import { useState } from 'react';
import type { BrandSwatch } from './brand.generated';

// The palette grid. A client island because copy-to-clipboard needs the browser;
// the swatch data itself is generated at build time from brand/tokens/colors.css
// (scripts/prebuild.mjs), so the colors cannot drift from the kit.
export function Swatches({ palette }: { palette: BrandSwatch[] }) {
  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(14rem, 1fr))',
        gap: '1rem',
      }}
    >
      {palette.map((swatch) => (
        <Swatch key={swatch.token} swatch={swatch} />
      ))}
    </div>
  );
}

function Swatch({ swatch }: { swatch: BrandSwatch }) {
  const [copied, setCopied] = useState(false);

  function copy() {
    navigator.clipboard?.writeText(swatch.hex).then(
      () => {
        setCopied(true);
        setTimeout(() => setCopied(false), 1200);
      },
      () => {},
    );
  }

  return (
    <button
      type="button"
      onClick={copy}
      title={`Copy ${swatch.hex}`}
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: '0.5rem',
        padding: 0,
        border: '1px solid var(--color-fd-border)',
        borderRadius: '0.5rem',
        background: 'transparent',
        textAlign: 'left',
        cursor: 'pointer',
        overflow: 'hidden',
        color: 'inherit',
        font: 'inherit',
      }}
    >
      <span
        aria-hidden
        style={{ height: '3.5rem', background: swatch.hex, display: 'block' }}
      />
      <span style={{ display: 'flex', flexDirection: 'column', gap: '0.15rem', padding: '0 0.75rem 0.75rem' }}>
        <span style={{ fontWeight: 500 }}>{swatch.name}</span>
        <span
          style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '0.8rem',
            opacity: 0.85,
          }}
        >
          {copied ? 'copied' : swatch.hex}
        </span>
        <span style={{ fontSize: '0.8rem', opacity: 0.6, lineHeight: 1.4 }}>
          {swatch.note}
        </span>
      </span>
    </button>
  );
}
