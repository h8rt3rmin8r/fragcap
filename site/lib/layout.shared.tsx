// SPDX-License-Identifier: Apache-2.0
import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';

// Shared chrome for both the home and docs layouts: the wordmark, the nav, and
// the repository link. Kept minimal per specification section 23.1.
//
// The nav title is the vendored wordmark rather than plain text (issue #41):
// the white variant on the dark void, the cyan variant on light surfaces. Both
// images ship and CSS shows one per theme (see .fc-wordmark-* in global.css);
// only the theme-appropriate one carries alt text so the accessible name is not
// duplicated. Raw <img> is correct here: the static export sets images
// unoptimized, so next/image would add nothing.
export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <span
          style={{ display: 'inline-flex', alignItems: 'center' }}
        >
          <img
            src="/logos/fragcap-wordmark-white.svg"
            alt="fragcap"
            className="fc-wordmark-dark"
            style={{ height: '1.25rem', width: 'auto' }}
          />
          <img
            src="/logos/fragcap-wordmark-cyan.svg"
            alt=""
            className="fc-wordmark-light"
            style={{ height: '1.25rem', width: 'auto' }}
          />
        </span>
      ),
    },
    links: [
      {
        text: 'Documentation',
        url: '/docs/getting-started',
      },
      {
        text: 'Glossary',
        url: '/docs/glossary',
      },
      {
        text: 'Changelog',
        url: '/docs/changelog',
      },
    ],
    githubUrl: 'https://github.com/h8rt3rmin8r/fragcap',
  };
}
