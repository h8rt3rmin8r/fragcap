// SPDX-License-Identifier: Apache-2.0
import type { Metadata } from 'next';
import { disclaimerParagraphs } from './disclaimer.generated';

// The legal disclaimer, published where visitors actually land rather than only
// in the repository (issue #39). The wording is the vetted "## Disclaimer"
// section of the root README.md, imported verbatim from a build-generated module
// (scripts/prebuild.mjs) so the site copy stays in sync and cannot be reworded
// here by accident. This route sits in the (home) group for the shared nav and
// footer chrome, and deliberately outside the docs sidebar: it is a legal
// notice, not part of the documentation.
export const metadata: Metadata = {
  title: 'Disclaimer',
};

export default function DisclaimerPage() {
  return (
    <main
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
        Disclaimer
      </h1>

      {disclaimerParagraphs.map((paragraph, i) => (
        <p key={i} style={{ lineHeight: 1.6 }}>
          {paragraph}
        </p>
      ))}
    </main>
  );
}
