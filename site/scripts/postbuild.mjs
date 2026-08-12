// SPDX-License-Identifier: Apache-2.0
// Write the static-export markers into the export root (specification section
// 22.2). Without .nojekyll the static host strips the framework asset directory
// and the site renders unstyled; CNAME binds the custom domain.
import { writeFileSync, existsSync, mkdirSync } from 'node:fs';

const out = 'out';
if (!existsSync(out)) {
  console.error('postbuild: export directory "out" is missing; did next build run with output export?');
  process.exit(1);
}

writeFileSync(`${out}/.nojekyll`, '');
writeFileSync(`${out}/CNAME`, 'fragcap.com\n');
console.log('postbuild: wrote out/.nojekyll and out/CNAME (fragcap.com)');
