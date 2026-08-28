// SPDX-License-Identifier: Apache-2.0
import { HomeLayout } from 'fumadocs-ui/layouts/home';
import type { ReactNode } from 'react';
import { baseOptions } from '@/lib/layout.shared';
import { Footer } from '@/components/footer';

// The footer renders here for the home group (home, brand, disclaimer, license)
// rather than at body level, because HomeLayout imposes no forced height: it and
// the footer are the two flex children of the 100vh body column, and the
// footer's marginTop:auto pins it to the bottom. Docs pages render their own
// footer inside the docs content column instead (see app/docs/[[...slug]]/page).
export default function Layout({ children }: { children: ReactNode }) {
  return (
    <>
      <HomeLayout id="main-content" tabIndex={-1} {...baseOptions()}>
        {children}
      </HomeLayout>
      <Footer />
    </>
  );
}
