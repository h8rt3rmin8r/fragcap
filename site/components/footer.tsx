// SPDX-License-Identifier: Apache-2.0
import Link from 'next/link';

// The site-wide footer. It renders once per page, but from two places so it
// sits in flow under each layout: the home group renders it after HomeLayout
// (app/(home)/layout.tsx), and docs pages render it inside the docs content
// column after the body (app/docs/[[...slug]]/page.tsx). A single body-level
// render does not work for docs, whose fumadocs layout forces a full-viewport
// grid that would push a body-level footer a full viewport below the content.
// The endorsement and the legal disclaimer link stay subordinate (Geist Mono,
// uppercase, low emphasis) per specification section 23.3 and
// docs/brand/README.md: the endorsement must not grow louder for being a link.
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
