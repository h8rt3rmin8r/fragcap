// SPDX-License-Identifier: Apache-2.0

// This function is serialized into the audited browser page. Keep it free of
// module-scope dependencies so Playwright and equivalent page evaluators can
// execute it directly.
export function auditDocument() {
  const visible = (element) => {
    const rectangle = element.getBoundingClientRect();
    const style = getComputedStyle(element);
    return rectangle.width > 0
      && rectangle.height > 0
      && style.visibility !== 'hidden'
      && style.display !== 'none';
  };
  const nameOf = (element) => (
    element.getAttribute('aria-label')
    || element.getAttribute('title')
    || element.textContent
    || ''
  ).trim();
  const headings = [...document.querySelectorAll(
    'article h1, article h2, article h3, article h4, article h5, article h6',
  )].map((heading) => ({
    level: Number(heading.tagName[1]),
    text: (heading.textContent || '').trim().slice(0, 90),
  }));
  const headingSkips = headings.filter(
    (heading, index) => index > 0 && heading.level > headings[index - 1].level + 1,
  );
  const unnamedControls = [...document.querySelectorAll('button, input, select, textarea')]
    .filter(visible)
    .filter((element) => !nameOf(element)
      && !element.getAttribute('aria-labelledby')
      && !(element instanceof HTMLInputElement && element.labels && element.labels.length))
    .map((element) => element.outerHTML.slice(0, 160));
  const imagesMissingAlt = [...document.images]
    .filter(visible)
    .filter((image) => !image.hasAttribute('alt'))
    .map((image) => image.getAttribute('src'));
  const duplicateIds = [...document.querySelectorAll('[id]')]
    .map((element) => element.id)
    .filter((id, index, ids) => id && ids.indexOf(id) !== index)
    .filter((id, index, ids) => ids.indexOf(id) === index);
  const primary = document.querySelector('main, [role="main"]');
  const skip = document.querySelector('.fc-skip-link');
  const firstFocusable = [...document.querySelectorAll(
    'a[href], button, input, select, textarea, [tabindex]:not([tabindex="-1"])',
  )].filter(visible).find((element) => !element.hasAttribute('disabled'));
  const rootOverflow = document.documentElement.scrollWidth
    > document.documentElement.clientWidth + 2;
  const hiddenClipping = [...document.querySelectorAll('article *')]
    .filter(visible)
    .filter((element) => {
      const style = getComputedStyle(element);
      return element.scrollWidth > element.clientWidth + 2
        && (style.overflowX === 'hidden' || style.overflowX === 'clip')
        && !String(element.className).includes('truncate');
    })
    .slice(0, 8)
    .map((element) => ({
      tag: element.tagName,
      className: String(element.className).slice(0, 90),
      clientWidth: element.clientWidth,
      scrollWidth: element.scrollWidth,
    }));
  const complexContent = [...document.querySelectorAll('article pre, article table, article svg, article img')]
    .filter(visible)
    .map((element) => {
      const rectangle = element.getBoundingClientRect();
      const parent = element.parentElement;
      const style = getComputedStyle(element);
      const parentStyle = parent ? getComputedStyle(parent) : null;
      return {
        tag: element.tagName,
        clientWidth: element.clientWidth,
        scrollWidth: element.scrollWidth,
        left: Math.round(rectangle.left),
        right: Math.round(rectangle.right),
        viewport: innerWidth,
        overflow: style.overflowX,
        parentOverflow: parentStyle?.overflowX,
        parentClientWidth: parent?.clientWidth,
        parentScrollWidth: parent?.scrollWidth,
        ariaLabel: element.getAttribute('aria-label'),
        role: element.getAttribute('role'),
        alt: element.getAttribute('alt'),
      };
    });

  return {
    title: document.title,
    language: document.documentElement.lang,
    viewport: innerWidth,
    mainCount: document.querySelectorAll('main, [role="main"]').length,
    primaryId: primary?.id || null,
    skipHref: skip?.getAttribute('href') || null,
    firstFocusableClass: String(firstFocusable?.className || ''),
    h1: headings.filter((heading) => heading.level === 1),
    headingSkips,
    unnamedControls,
    imagesMissingAlt,
    duplicateIds,
    rootOverflow,
    rootClientWidth: document.documentElement.clientWidth,
    rootScrollWidth: document.documentElement.scrollWidth,
    hiddenClipping,
    complexContent,
    footerVisible: [...document.querySelectorAll('footer')].some(visible),
  };
}
