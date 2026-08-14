const { chromium } = require('playwright');
const { resolve } = require('node:path');
const { pathToFileURL } = require('node:url');
(async () => {
  const b = await chromium.launch();
  const p = await b.newPage();
  // Resolve to an absolute path first: a bare "build/brand-guide.html" would
  // otherwise form "file://build/..." where "build" is read as a hostname.
  const src = pathToFileURL(resolve(process.argv[2])).href;
  await p.goto(src, { waitUntil: 'networkidle' });
  await p.evaluate(() => document.fonts.ready);
  await p.waitForTimeout(1200);
  await p.pdf({ path: process.argv[3], width: '8.5in', height: '11in',
                printBackground: true, margin: {top:0,right:0,bottom:0,left:0},
                preferCSSPageSize: true });
  await b.close();
})();
