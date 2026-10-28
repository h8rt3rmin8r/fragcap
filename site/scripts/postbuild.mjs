// SPDX-License-Identifier: Apache-2.0
// Write the static-export markers into the export root (specification section
// 22.2). Without .nojekyll the static host strips the framework asset directory
// and the site renders unstyled; CNAME binds the custom domain.
import { writeFileSync, readFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const out = 'out';
if (!existsSync(out)) {
  console.error('postbuild: export directory "out" is missing; did next build run with output export?');
  process.exit(1);
}

// A source glossary heading is useful only when its published HTML contains
// the anchor that same-page links actually use. Keep this check on exported
// bytes, where MDX heading transforms and the static route are both visible.
const glossaryCandidates = [
  join(out, 'docs', 'glossary', 'capture-and-networking.html'),
  join(out, 'docs', 'glossary', 'capture-and-networking', 'index.html'),
];
const glossaryPath = glossaryCandidates.find((path) => existsSync(path));
if (!glossaryPath) {
  throw new Error('postbuild: exported Capture and networking glossary is missing');
}
const glossaryHtml = readFileSync(glossaryPath, 'utf8');
const resourceHeading = /<h[2-6]\b[^>]*\bid=["']resource-obligation["']/i.test(glossaryHtml);
const resourceLinks = [
  ...glossaryHtml.matchAll(
    /<a\b[^>]*\bhref=["'](?:#|\/docs\/glossary\/capture-and-networking#)resource-obligation["']/gi,
  ),
].length;
if (!resourceHeading || resourceLinks < 2) {
  throw new Error(
    `postbuild: Resource obligation heading or links missing in exported glossary (heading=${resourceHeading}, links=${resourceLinks})`,
  );
}

writeFileSync(`${out}/.nojekyll`, '');
writeFileSync(`${out}/CNAME`, 'fragcap.com\n');
console.log('postbuild: verified glossary resource anchor and wrote static hosting markers');
