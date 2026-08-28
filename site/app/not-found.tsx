// SPDX-License-Identifier: Apache-2.0
import Link from 'next/link';

export default function NotFound() {
  return (
    <main
      id="main-content"
      tabIndex={-1}
      className="fc-page"
      style={{
        width: '100%',
        maxWidth: '42rem',
        minHeight: '100vh',
        margin: '0 auto',
        padding: 'clamp(3rem, 12vh, 7rem) 1.25rem',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-start',
        gap: '1.5rem',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
        <img
          src="/logos/fragcap-mark-color.svg"
          alt=""
          style={{ width: '2.5rem', height: '2.5rem' }}
        />
        <img
          src="/logos/fragcap-wordmark-white.svg"
          alt="fragcap"
          className="fc-wordmark-dark"
          style={{ width: 'auto', height: '1.6rem' }}
        />
        <img
          src="/logos/fragcap-wordmark-cyan.svg"
          alt=""
          className="fc-wordmark-light"
          style={{ width: 'auto', height: '1.6rem' }}
        />
      </div>

      <p
        aria-hidden="true"
        style={{
          margin: 0,
          fontFamily: 'var(--font-mono)',
          color: 'var(--color-fd-primary)',
          fontWeight: 600,
        }}
      >
        404
      </p>
      <h1 style={{ margin: 0, fontFamily: 'var(--font-display)', fontSize: 'clamp(2rem, 8vw, 3.5rem)' }}>
        Page not found
      </h1>
      <p style={{ margin: 0, maxWidth: '36rem', fontSize: '1.125rem', lineHeight: 1.6 }}>
        The fragcap page at this address does not exist. Return home or continue
        with the current setup and capture guidance.
      </p>

      <nav
        aria-label="Page recovery"
        style={{ display: 'flex', flexWrap: 'wrap', gap: '0.75rem' }}
      >
        <Link
          href="/"
          style={{
            borderRadius: '0.4rem',
            padding: '0.7rem 1rem',
            background: 'var(--color-fd-primary)',
            color: 'var(--color-fd-primary-foreground)',
            fontWeight: 600,
          }}
        >
          Go home
        </Link>
        <Link
          href="/docs/getting-started"
          style={{
            border: '1px solid var(--color-fd-border)',
            borderRadius: '0.4rem',
            padding: '0.7rem 1rem',
            fontWeight: 600,
          }}
        >
          Getting started
        </Link>
      </nav>
    </main>
  );
}
