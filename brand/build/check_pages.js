/*
 * Page-fit check for brand-guide.html.
 *
 * Each .page is a fixed 8.5x11in box with overflow:hidden, so content that
 * runs long is silently cropped at the folio rather than reflowing onto the
 * next page. Eyeballing thumbnails does not catch it reliably.
 *
 * This asks the browser directly: for every .page, where does the lowest piece
 * of real content actually end, and does it clear the folio rule?
 *
 * Usage:  node build/check_pages.js build/brand-guide.html
 * Exits non-zero if any page overflows.
 */

const { chromium } = require('playwright');

(async () => {
  const file = process.argv[2];
  const browser = await chromium.launch();
  const page = await browser.newPage();
  await page.goto('file://' + require('path').resolve(file), { waitUntil: 'networkidle' });
  await page.evaluate(() => document.fonts.ready);
  await page.waitForTimeout(600);

  const report = await page.evaluate(() => {
    const out = [];
    document.querySelectorAll('.page').forEach((pg, i) => {
      const pageRect = pg.getBoundingClientRect();
      const folio = pg.querySelector('.folio');
      // With no folio (the cover) the limit is the page's bottom padding edge.
      const style = getComputedStyle(pg);
      const padBottom = parseFloat(style.paddingBottom) || 0;
      const limit = folio
        ? folio.getBoundingClientRect().top
        : pageRect.bottom - padBottom;

      let lowest = -Infinity;
      let culprit = null;
      pg.querySelectorAll('*').forEach((el) => {
        if (el.closest('.folio')) return;
        const s = getComputedStyle(el);
        if (s.display === 'none' || s.visibility === 'hidden') return;
        const r = el.getBoundingClientRect();
        if (r.height === 0 && r.width === 0) return;
        // ignore full-bleed decorative layers (the cover's gradient wash)
        if (Math.abs(r.height - pageRect.height) < 2) return;
        if (r.bottom > lowest) {
          lowest = r.bottom;
          culprit = (el.tagName.toLowerCase() +
            (el.className && typeof el.className === 'string'
              ? '.' + el.className.trim().split(/\s+/).join('.')
              : '')).slice(0, 48);
        }
      });

      out.push({
        page: i + 1,
        overflowPx: +(lowest - limit).toFixed(1),
        culprit,
      });
    });
    return out;
  });

  await browser.close();

  // Require real clearance, not a hairline. Pages are fixed-height with
  // overflow:hidden, so a page that only just fits today gets silently
  // cropped tomorrow when a word is added or a font metric shifts.
  const MIN_CLEARANCE = 8;
  let bad = 0;
  console.log('  page   clearance   lowest element');
  for (const r of report) {
    const over = r.overflowPx > -MIN_CLEARANCE;
    if (over) bad++;
    const gap = (-r.overflowPx).toFixed(1);
    console.log(
      '  %s   %s   %s',
      String(r.page).padStart(4),
      (over ? 'OVERFLOW +' + r.overflowPx.toFixed(1) + 'px' : gap + 'px').padStart(11),
      over ? r.culprit : ''
    );
  }
  console.log(bad
    ? `\n  ${bad} page(s) overflow or sit within ${MIN_CLEARANCE}px of the folio.`
    : `\n  All pages fit with at least ${MIN_CLEARANCE}px clearance.`);
  process.exit(bad ? 1 : 0);
})();
