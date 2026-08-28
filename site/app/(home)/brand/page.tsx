// SPDX-License-Identifier: Apache-2.0
import type { Metadata } from 'next';
import { brandPalette } from './brand.generated';
import { Swatches } from './swatches';

// The brand page (issue #60): the web counterpart of brand/brand-guide.pdf. The
// palette is generated from brand/tokens/colors.css and the logo, favicon, and
// guide assets are copied into public/brand/ by scripts/prebuild.mjs, so the page
// cannot drift from the kit. It sits in the (home) group for the shared chrome
// and outside the docs sidebar. brand/logos/svg is the source of truth for marks.
export const metadata: Metadata = {
  title: 'Brand',
};

const logos = '/brand/logos';

const sectionTitle = {
  fontFamily: 'var(--font-display)',
  fontSize: '1.35rem',
  fontWeight: 600,
  margin: 0,
} as const;

const downloadLink = {
  fontFamily: 'var(--font-mono)',
  fontSize: '0.8rem',
} as const;

export default function BrandPage() {
  return (
    <div
      className="fc-page"
      style={{
        maxWidth: '52rem',
        margin: '0 auto',
        padding: '4rem 1.5rem',
        display: 'flex',
        flexDirection: 'column',
        gap: '2.5rem',
      }}
    >
      <header style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
        <h1
          style={{
            fontFamily: 'var(--font-display)',
            fontSize: '2rem',
            fontWeight: 600,
            lineHeight: 1.2,
            margin: 0,
          }}
        >
          Brand
        </h1>
        <p style={{ lineHeight: 1.6, margin: 0 }}>
          The fragcap identity: an independent{' '}
          <a href="https://shruggie.tech">ShruggieTech</a> sub-brand. This page is
          the web companion to the brand guide; every color and asset here is
          single-sourced from the kit under <code>brand/</code>.
        </p>
      </header>

      <section style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
        <h2 style={sectionTitle}>Logo</h2>
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '2rem',
            flexWrap: 'wrap',
            padding: '2rem',
            border: '1px solid var(--color-fd-border)',
            borderRadius: '0.5rem',
          }}
        >
          <img
            src={`${logos}/fragcap-mark-color.svg`}
            alt="fragcap mark"
            style={{ height: '4rem', width: 'auto' }}
          />
          <img
            src={`${logos}/fragcap-horizontal-white.svg`}
            alt="fragcap horizontal lockup"
            className="fc-wordmark-dark"
            style={{ height: '2.5rem', width: 'auto' }}
          />
          <img
            src={`${logos}/fragcap-horizontal-dark.svg`}
            alt=""
            className="fc-wordmark-light"
            style={{ height: '2.5rem', width: 'auto' }}
          />
        </div>
        <p style={{ lineHeight: 1.6, margin: 0, opacity: 0.75 }}>
          The mark carries full color; the horizontal and stacked lockups ship in
          light, dark, and white variants for their background. The SVG sources are
          canonical.
        </p>
        <nav style={{ display: 'flex', gap: '1.25rem', flexWrap: 'wrap' }}>
          <a style={downloadLink} href={`${logos}/fragcap-mark-color.svg`} download>
            Mark (SVG)
          </a>
          <a style={downloadLink} href={`${logos}/fragcap-horizontal-white.svg`} download>
            Horizontal (SVG)
          </a>
          <a style={downloadLink} href={`${logos}/fragcap-stacked-dark.svg`} download>
            Stacked (SVG)
          </a>
          <a style={downloadLink} href={`${logos}/fragcap-wordmark-white.svg`} download>
            Wordmark (SVG)
          </a>
        </nav>
      </section>

      <section style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
        <h2 style={sectionTitle}>Color</h2>
        <p style={{ lineHeight: 1.6, margin: 0, opacity: 0.75 }}>
          Dark-first: a neutral ground, Signal Cyan as the single accent, Capture
          Orange kept scarce. Click a swatch to copy its hex.
        </p>
        <Swatches palette={brandPalette} />
      </section>

      <section style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
        <h2 style={sectionTitle}>Typography</h2>
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            gap: '1.25rem',
            padding: '1.5rem',
            border: '1px solid var(--color-fd-border)',
            borderRadius: '0.5rem',
          }}
        >
          <div>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem', textTransform: 'uppercase', letterSpacing: '0.08em', opacity: 0.6 }}>
              Space Grotesk / Display
            </div>
            <div style={{ fontFamily: 'var(--font-display)', fontSize: '2rem', fontWeight: 600 }}>
              fragcap
            </div>
          </div>
          <div>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem', textTransform: 'uppercase', letterSpacing: '0.08em', opacity: 0.6 }}>
              Geist / Body
            </div>
            <div style={{ fontFamily: 'var(--font-body)', fontSize: '1.05rem', lineHeight: 1.6 }}>
              Passive, process-attributed network capture for Windows.
            </div>
          </div>
          <div>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem', textTransform: 'uppercase', letterSpacing: '0.08em', opacity: 0.6 }}>
              Geist Mono / Code and interface
            </div>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: '1rem' }}>
              fragcap capture --target eso --out capture.fcapng
            </div>
          </div>
        </div>
      </section>

      <section style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
        <h2 style={sectionTitle}>Favicons</h2>
        <div style={{ display: 'flex', alignItems: 'center', gap: '1.5rem', flexWrap: 'wrap' }}>
          <img src="/brand/favicons/favicon.svg" alt="favicon" style={{ height: '3rem', width: '3rem' }} />
          <img src="/brand/favicons/favicon-48x48.png" alt="" style={{ height: '2rem', width: '2rem' }} />
          <img src="/brand/favicons/favicon-32x32.png" alt="" style={{ height: '1.5rem', width: '1.5rem' }} />
          <img src="/brand/favicons/apple-touch-icon.png" alt="apple touch icon" style={{ height: '3rem', width: '3rem', borderRadius: '0.5rem' }} />
        </div>
      </section>

      <section style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
        <h2 style={sectionTitle}>Usage</h2>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(15rem, 1fr))', gap: '1.5rem' }}>
          <div>
            <h3 style={{ fontSize: '1rem', margin: '0 0 0.5rem' }}>Do</h3>
            <ul style={{ margin: 0, paddingLeft: '1.1rem', lineHeight: 1.6, opacity: 0.85 }}>
              <li>Keep Signal Cyan as the single accent.</li>
              <li>Use the SVG sources; let them scale.</li>
              <li>Give the mark clear space on any background.</li>
              <li>Pair the lockup with its matching background variant.</li>
            </ul>
          </div>
          <div>
            <h3 style={{ fontSize: '1rem', margin: '0 0 0.5rem' }}>Don&apos;t</h3>
            <ul style={{ margin: 0, paddingLeft: '1.1rem', lineHeight: 1.6, opacity: 0.85 }}>
              <li>Recolor or restroke the marks.</li>
              <li>Make Capture Orange the sole carrier of meaning.</li>
              <li>Grow the ShruggieTech endorsement louder than the fragcap mark.</li>
              <li>Rasterize the logo where an SVG will serve.</li>
            </ul>
          </div>
        </div>
      </section>

      <section style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
        <h2 style={sectionTitle}>Download</h2>
        <nav style={{ display: 'flex', gap: '1.25rem', flexWrap: 'wrap' }}>
          <a style={downloadLink} href="/brand/brand-guide.pdf" download>
            Brand guide (PDF)
          </a>
          <a style={downloadLink} href="/brand/favicons/favicon.svg" download>
            Favicon (SVG)
          </a>
        </nav>
      </section>
    </div>
  );
}
