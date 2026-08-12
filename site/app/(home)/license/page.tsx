// SPDX-License-Identifier: Apache-2.0
import type { Metadata } from 'next';
import Link from 'next/link';
import { licenseName, noticeText } from './license.generated';

// The licensing summary, published where visitors land rather than only in the
// repository (issue #47). fragcap is licensed under Apache-2.0; the project
// NOTICE is imported verbatim from a build-generated module (scripts/prebuild.mjs)
// so the site copy stays in sync with the repository and cannot be reworded here.
// This route sits in the (home) group for the shared nav and footer chrome, and
// outside the docs sidebar: it is a legal notice, not part of the documentation.
export const metadata: Metadata = {
  title: 'License',
};

const repo = 'https://github.com/h8rt3rmin8r/fragcap';

export default function LicensePage() {
  return (
    <main
      className="fc-page"
      style={{
        maxWidth: '48rem',
        margin: '0 auto',
        padding: '4rem 1.5rem',
        display: 'flex',
        flexDirection: 'column',
        gap: '1.5rem',
      }}
    >
      <h1
        style={{
          fontFamily: 'var(--font-display)',
          fontSize: '2rem',
          fontWeight: 600,
          lineHeight: 1.2,
        }}
      >
        License
      </h1>

      <p style={{ lineHeight: 1.6 }}>
        fragcap is licensed under the{' '}
        <a href="https://www.apache.org/licenses/LICENSE-2.0">
          {licenseName} License
        </a>
        . Every source file carries an SPDX identifier, and each crate declares{' '}
        <code>license = &quot;Apache-2.0&quot;</code>.
      </p>

      <p style={{ lineHeight: 1.6 }}>
        The choice of Apache-2.0 alone, rather than the Rust ecosystem&apos;s
        conventional <code>MIT OR Apache-2.0</code> dual license, is deliberate:
        the patent grant and the explicit NOTICE requirement suit a tool that sits
        close to anti-cheat and platform boundaries. The full license text and the
        NOTICE live in the repository, at{' '}
        <Link href={`${repo}/blob/main/LICENSE`}>LICENSE</Link> and{' '}
        <Link href={`${repo}/blob/main/NOTICE`}>NOTICE</Link>.
      </p>

      <h2
        style={{
          fontFamily: 'var(--font-display)',
          fontSize: '1.25rem',
          fontWeight: 600,
          marginTop: '0.5rem',
        }}
      >
        NOTICE
      </h2>

      <pre
        className="fd-codeblock"
        style={{
          fontFamily: 'var(--font-mono)',
          fontSize: '0.8rem',
          lineHeight: 1.6,
          padding: '1rem 1.25rem',
          borderRadius: '0.5rem',
          overflowX: 'auto',
          whiteSpace: 'pre-wrap',
        }}
      >
        <code>{noticeText}</code>
      </pre>
    </main>
  );
}
