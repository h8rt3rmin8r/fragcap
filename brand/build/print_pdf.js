const { chromium } = require('playwright');
(async () => {
  const b = await chromium.launch();
  const p = await b.newPage();
  await p.goto('file://' + process.argv[2], { waitUntil: 'networkidle' });
  await p.evaluate(() => document.fonts.ready);
  await p.waitForTimeout(1200);
  await p.pdf({ path: process.argv[3], width: '8.5in', height: '11in',
                printBackground: true, margin: {top:0,right:0,bottom:0,left:0},
                preferCSSPageSize: true });
  await b.close();
})();
