// SPDX-License-Identifier: Apache-2.0
'use client';

import type { MouseEvent } from 'react';

export function SkipLink() {
  const activate = (event: MouseEvent<HTMLAnchorElement>) => {
    const target = document.getElementById('main-content');
    if (!target) return;

    event.preventDefault();
    window.history.pushState(null, '', '#main-content');
    target.focus({ preventScroll: true });
    target.scrollIntoView({ block: 'start' });
  };

  return (
    <a className="fc-skip-link" href="#main-content" onClick={activate}>
      Skip to main content
    </a>
  );
}
