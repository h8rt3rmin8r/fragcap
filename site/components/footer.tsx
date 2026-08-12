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
          gap: '1.5rem',
          flexWrap: 'wrap',
          fontFamily: 'var(--font-mono)',
          fontSize: '0.75rem',
          textTransform: 'uppercase',
          letterSpacing: '0.05em',
          opacity: 0.6,
        }}
      >
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
        <Link href="/disclaimer">Disclaimer</Link>
      </div>
    </footer>
  );
}
