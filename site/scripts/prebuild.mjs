// SPDX-License-Identifier: Apache-2.0
// Render the single-source glossary into the Fumadocs content tree
// (specification section 22.4). The authored glossary lives once, under
// docs/glossary/, in kramdown-flavored Markdown that GitHub renders directly.
// This step copies each category page into content/docs/glossary/ as MDX,
// performing the two transforms kramdown carries that MDX does not:
//
//   1. The "{: .matters }" inline attribute list ahead of a blockquote becomes
//      a Fumadocs <Callout> titled "Why it matters here" (section 22.4 asks the
//      note to render as a distinct element, not an ordinary quote).
//   2. Relative sibling links "<slug>.md#<anchor>" become site routes
//      "/docs/glossary/<slug>#<anchor>", so a link that resolves on GitHub also
//      resolves as a page route here.
//
// The destination is generated, never committed: docs/glossary/ is the only
// copy under version control. The generated tree is gitignored, so a fresh
// checkout produces it at build time and the two cannot drift.
import {
  readdirSync,
  readFileSync,
  writeFileSync,
  rmSync,
  mkdirSync,
  existsSync,
} from 'node:fs';
import { join } from 'node:path';

const srcDir = join('..', 'docs', 'glossary');
const destDir = join('content', 'docs', 'glossary');

// One-line description per category page, used for the page's meta description
// and the card subtitle on the glossary index. The index page carries its own.
const descriptions = {
  'anti-cheat-and-security':
    'Terms for the security posture: what anti-cheat watches for, and the techniques fragcap will not use.',
  'capture-and-networking':
    'The vocabulary of packets, flows, and the capture pipeline that carries them.',
  'command-line-and-diagnostics':
    'The command-line surface and the diagnostics fragcap prints about its own run.',
  'file-and-wire-formats':
    'The output formats fragcap writes and the wire formats it reads.',
  'platform-and-distribution':
    'Windows platform surfaces, launchers, and how fragcap ships.',
  'process-and-attribution':
    'How fragcap recovers which process owns a flow: the socket table, the process tree, and the join between them.',
  'rust-and-tooling':
    'The Rust workspace vocabulary and the checks that gate a change.',
  'windows-internals':
    'The Windows facilities fragcap reads to attribute traffic without touching a target process.',
};

const indexDescription =
  'Every term across the category pages, one alphabetical list, each linking to its definition.';

// Sidebar order: the generated index first, then the eight category pages in
// the section-4.4 order rather than alphabetically by filename.
const order = [
  'index',
  'capture-and-networking',
  'process-and-attribution',
  'file-and-wire-formats',
  'command-line-and-diagnostics',
  'windows-internals',
  'platform-and-distribution',
  'anti-cheat-and-security',
  'rust-and-tooling',
];

function rewriteLinks(text) {
  // "<slug>.md" or "<slug>.md#<anchor>" -> "/docs/glossary/<slug>[#anchor]".
  return text.replace(
    /\]\(([a-z0-9-]+)\.md(#[a-z0-9.-]+)?\)/g,
    (_m, slug, anchor) => `](/docs/glossary/${slug}${anchor ?? ''})`,
  );
}

function transformMatters(lines) {
  const out = [];
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (line.trim() === '{: .matters }') {
      // The blockquote that follows becomes a Callout. Consume every
      // subsequent line that opens with ">" and strip the marker.
      const body = [];
      let j = i + 1;
      while (j < lines.length && /^\s*>/.test(lines[j])) {
        body.push(lines[j].replace(/^\s*>\s?/, ''));
        j++;
      }
      out.push('<Callout title="Why it matters here">');
      out.push('');
      out.push(...body);
      out.push('');
      out.push('</Callout>');
      i = j - 1;
      continue;
    }
    out.push(line);
  }
  return out;
}

function transform(slug, raw) {
  const isIndex = slug === 'index';
  const lines = raw.replace(/\r\n/g, '\n').split('\n');

  // Pull the H1 into the frontmatter title and drop it from the body; Fumadocs
  // renders the title from frontmatter, so a body H1 would double it.
  let title = slug;
  const h1 = lines.findIndex((l) => /^#\s+/.test(l));
  if (h1 !== -1) {
    title = lines[h1].replace(/^#\s+/, '').trim();
    lines.splice(h1, 1);
    // Drop a single leading blank line left behind by the removed heading.
    if (lines[h1] === '') lines.splice(h1, 1);
  }

  let body = transformMatters(lines).join('\n');
  body = rewriteLinks(body);
  // The generated index warns "do not edit by hand"; that instruction is for
  // docs/glossary/index.md, not this derived copy, so it is dropped by the H1
  // splice above only incidentally. Leave the body prose intact otherwise.

  const description = isIndex
    ? indexDescription
    : (descriptions[slug] ?? title);

  const frontmatter = [
    '---',
    `title: ${JSON.stringify(title)}`,
    `description: ${JSON.stringify(description)}`,
    '---',
    '',
    'import { Callout } from \'fumadocs-ui/components/callout\';',
    '',
    '',
  ].join('\n');

  return frontmatter + body.replace(/\n{3,}/g, '\n\n').trimEnd() + '\n';
}

function main() {
  if (!existsSync(srcDir)) {
    console.error(`prebuild: glossary source ${srcDir} is missing`);
    process.exit(1);
  }
  rmSync(destDir, { recursive: true, force: true });
  mkdirSync(destDir, { recursive: true });

  const files = readdirSync(srcDir).filter((f) => f.endsWith('.md'));
  let count = 0;
  for (const file of files) {
    const slug = file.replace(/\.md$/, '');
    const raw = readFileSync(join(srcDir, file), 'utf8');
    writeFileSync(join(destDir, `${slug}.mdx`), transform(slug, raw));
    count++;
  }

  // meta.json fixes the folder title and page order in the sidebar.
  const pages = order.filter((slug) =>
    files.includes(slug === 'index' ? 'index.md' : `${slug}.md`),
  );
  writeFileSync(
    join(destDir, 'meta.json'),
    JSON.stringify({ title: 'Glossary', pages }, null, 2) + '\n',
  );

  console.log(
    `prebuild: rendered ${count} glossary page(s) into ${destDir}`,
  );
}

main();
