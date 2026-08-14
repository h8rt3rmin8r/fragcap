// SPDX-License-Identifier: Apache-2.0
import './global.css';
import { RootProvider } from 'fumadocs-ui/provider/next';
import type { ReactNode } from 'react';
import type { Metadata, Viewport } from 'next';

const url = 'https://fragcap.com';

export const metadata: Metadata = {
  metadataBase: new URL(url),
  title: {
    default: 'fragcap',
    template: '%s | fragcap',
  },
  description:
    'Passive, process-attributed network capture for Windows game clients.',
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
    description:
      'Passive, process-attributed network capture for Windows game clients.',
    siteName: 'fragcap',
    images: [{ url: '/social-preview.png', width: 1280, height: 640 }],
  },
  twitter: {
    card: 'summary_large_image',
    title: 'fragcap',
    description:
      'Passive, process-attributed network capture for Windows game clients.',
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
        <RootProvider
          search={{ options: { type: 'static', api: '/static.json' } }}
        >
          {children}
        </RootProvider>
      </body>
    </html>
  );
}
