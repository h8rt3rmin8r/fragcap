// SPDX-License-Identifier: Apache-2.0
import Link from 'next/link';

// The site-wide footer, rendered once at the body level (app/layout.tsx) so it
// reaches every page, home and docs alike. Fumadocs' layouts carry no footer
// slot, so the endorsement and the legal disclaimer link live here rather than
// in per-page markup. Both stay subordinate (Geist Mono, uppercase, low
// emphasis) per specification section 23.3 and docs/brand/README.md: the
// endorsement must not grow louder for being a link.
export function Footer() {
  return (
    <footer
      style={{
        marginTop: 'auto',
        borderTop: '1px solid var(--color-fd-border)',
        padding: '1.5rem',
      }}
    >
      <div
        style={{
          maxWidth: '48rem',
          margin: '0 auto',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          gap: '1.5rem 2.5rem',
          flexWrap: 'wrap',
          fontFamily: 'var(--font-mono)',
          fontSize: '0.75rem',
          textTransform: 'uppercase',
          letterSpacing: '0.05em',
          opacity: 0.6,
        }}
      >
        {/*
         * The endorsement is static text and stays visually apart from the
         * interactive links (issue #48): it anchors the left, the links group on
         * the right, so the two never read as one run. The endorsement stays
         * subordinate and must not grow louder for sitting beside links.
         */}
        <span>
          A{' '}
          <a
            href="https://shruggie.tech"
            target="_blank"
            rel="noopener noreferrer"
          >
            ShruggieTech
          </a>{' '}
          project
        </span>
        <nav style={{ display: 'flex', gap: '1.5rem', flexWrap: 'wrap' }}>
          <Link href="/disclaimer">Disclaimer</Link>
          <Link href="/license">License</Link>
          <Link href="/brand">Brand</Link>
        </nav>
      </div>
    </footer>
  );
}
