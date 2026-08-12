import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';

// Shared chrome for both the home and docs layouts: the wordmark, the nav, and
// the repository link. Kept minimal per specification section 23.1.
export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <span style={{ fontFamily: 'var(--font-display)', fontWeight: 600 }}>
          fragcap
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
    ],
    githubUrl: 'https://github.com/h8rt3rmin8r/fragcap',
  };
}
