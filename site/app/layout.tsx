// SPDX-License-Identifier: Apache-2.0
import './global.css';
import { RootProvider } from 'fumadocs-ui/provider/next';
import type { ReactNode } from 'react';
import type { Metadata, Viewport } from 'next';
import { SkipLink } from '@/components/skip-link';

const url = 'https://fragcap.com';
const description =
  'Passive, process-attributed Capture and explicit, target-scoped Deep Capture for Windows game traffic.';

export const metadata: Metadata = {
  metadataBase: new URL(url),
  title: {
    default: 'fragcap',
    template: '%s | fragcap',
  },
  description,
  applicationName: 'fragcap',
  manifest: '/site.webmanifest',
  icons: {
    icon: [
      { url: '/favicon.svg', type: 'image/svg+xml' },
      { url: '/favicon-32x32.png', sizes: '32x32', type: 'image/png' },
      { url: '/favicon-16x16.png', sizes: '16x16', type: 'image/png' },
    ],
    shortcut: '/favicon.ico',
    apple: '/apple-touch-icon.png',
  },
  openGraph: {
    type: 'website',
    url,
    title: 'fragcap',
    description,
    siteName: 'fragcap',
    images: [{ url: '/social-preview.png', width: 1280, height: 640 }],
  },
  twitter: {
    card: 'summary_large_image',
    title: 'fragcap',
    description,
    images: ['/social-preview.png'],
  },
};

export const viewport: Viewport = {
  themeColor: '#050708',
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body style={{ display: 'flex', flexDirection: 'column', minHeight: '100vh' }}>
        <SkipLink />
        <RootProvider
          search={{ options: { type: 'static', api: '/static.json' } }}
        >
          {children}
        </RootProvider>
      </body>
    </html>
  );
}
