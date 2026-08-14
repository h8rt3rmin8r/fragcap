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
  cpSync,
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

// The legal disclaimer is single-sourced from the root README.md, exactly as
// the glossary is single-sourced from docs/glossary/. This step extracts the
// "## Disclaimer" section and emits its paragraphs as a generated TypeScript
// module the /disclaimer route imports, so the site copy cannot drift from the
// vetted README text (specification section 23.3 companion; issue #39).
const readmePath = join('..', 'README.md');
const disclaimerDir = join('app', '(home)', 'disclaimer');
const disclaimerFile = join(disclaimerDir, 'disclaimer.generated.ts');

function extractDisclaimer(readme) {
  const lines = readme.replace(/\r\n/g, '\n').split('\n');
  const start = lines.findIndex((l) => /^##\s+Disclaimer\s*$/.test(l));
  if (start === -1) return null;
  // Collect the section body until the next level-1 or level-2 heading.
  const body = [];
  for (let i = start + 1; i < lines.length; i++) {
    if (/^#{1,2}\s+/.test(lines[i])) break;
    body.push(lines[i]);
  }
  // Split into paragraphs on blank lines; join each paragraph's wrapped lines
  // into one string so the route renders one <p> per paragraph.
  const paragraphs = [];
  let current = [];
  for (const line of body) {
    if (line.trim() === '') {
      if (current.length) paragraphs.push(current.join(' ').trim());
      current = [];
    } else {
      current.push(line.trim());
    }
  }
  if (current.length) paragraphs.push(current.join(' ').trim());
  return paragraphs.filter((p) => p.length);
}

function writeDisclaimer() {
  if (!existsSync(readmePath)) {
    console.error(`prebuild: README ${readmePath} is missing`);
    process.exit(1);
  }
  const paragraphs = extractDisclaimer(readFileSync(readmePath, 'utf8'));
  if (!paragraphs || paragraphs.length === 0) {
    console.error('prebuild: README has no "## Disclaimer" section');
    process.exit(1);
  }
  mkdirSync(disclaimerDir, { recursive: true });
  const out = [
    '// SPDX-License-Identifier: Apache-2.0',
    '// Generated by scripts/prebuild.mjs from the root README.md "## Disclaimer"',
    '// section. Do not edit by hand; edit README.md, the single source of truth.',
    'export const disclaimerParagraphs: string[] = [',
    ...paragraphs.map((p) => `  ${JSON.stringify(p)},`),
    '];',
    '',
  ].join('\n');
  writeFileSync(disclaimerFile, out);
  console.log(
    `prebuild: rendered ${paragraphs.length} disclaimer paragraph(s) into ${disclaimerFile}`,
  );
}

// The workspace version, single-sourced from the root Cargo.toml so the site
// never hardcodes a release number (issue #45). Read from [workspace.package].
const cargoPath = join('..', 'Cargo.toml');
const versionFile = join('lib', 'version.generated.ts');

function extractVersion(cargo) {
  const lines = cargo.replace(/\r\n/g, '\n').split('\n');
  let inWorkspacePackage = false;
  for (const line of lines) {
    const section = line.match(/^\s*\[([^\]]+)\]\s*$/);
    if (section) {
      inWorkspacePackage = section[1].trim() === 'workspace.package';
      continue;
    }
    if (inWorkspacePackage) {
      const v = line.match(/^\s*version\s*=\s*"([^"]+)"/);
      if (v) return v[1];
    }
  }
  return null;
}

function writeVersion() {
  if (!existsSync(cargoPath)) {
    console.error(`prebuild: ${cargoPath} is missing`);
    process.exit(1);
  }
  const version = extractVersion(readFileSync(cargoPath, 'utf8'));
  if (!version) {
    console.error('prebuild: no [workspace.package] version in Cargo.toml');
    process.exit(1);
  }
  mkdirSync('lib', { recursive: true });
  const out = [
    '// SPDX-License-Identifier: Apache-2.0',
    '// Generated by scripts/prebuild.mjs from the root Cargo.toml',
    '// [workspace.package] version. Do not edit by hand.',
    `export const fragcapVersion = ${JSON.stringify(version)};`,
    '',
  ].join('\n');
  writeFileSync(versionFile, out);
  console.log(`prebuild: wrote version ${version} into ${versionFile}`);
}

// The License page content, single-sourced from the root NOTICE so the site
// cannot drift from the vetted attribution text (issue #47). The full Apache-2.0
// text is long and unchanging, so the page links to it in the repository rather
// than embedding it; the project-specific NOTICE is what is shown in full.
const noticePath = join('..', 'NOTICE');
const licenseDir = join('app', '(home)', 'license');
const licenseFile = join(licenseDir, 'license.generated.ts');

function writeLicense() {
  if (!existsSync(noticePath)) {
    console.error(`prebuild: ${noticePath} is missing`);
    process.exit(1);
  }
  const notice = readFileSync(noticePath, 'utf8').replace(/\r\n/g, '\n').trimEnd();
  mkdirSync(licenseDir, { recursive: true });
  const out = [
    '// SPDX-License-Identifier: Apache-2.0',
    '// Generated by scripts/prebuild.mjs from the root NOTICE. Do not edit by',
    '// hand; edit NOTICE, the single source of truth.',
    'export const licenseName = "Apache-2.0";',
    `export const noticeText = ${JSON.stringify(notice)};`,
    '',
  ].join('\n');
  writeFileSync(licenseFile, out);
  console.log(`prebuild: wrote license notice into ${licenseFile}`);
}

// The Changelog pages, single-sourced from the root CHANGELOG.md (issues #49,
// #50). Each version becomes a sidebar group and each Keep a Changelog category
// under it (Added, Changed, Fixed, Decisions, ...) becomes its own page, mirroring
// how the glossary generates a multi-page group. Non-canonical subsections a
// release fragment introduced are demoted to sub-headings within the category
// they follow, so no content is lost. The generated tree is gitignored; the
// committed source is CHANGELOG.md, assembled by `cargo xtask changelog`.
const changelogPath = join('..', 'CHANGELOG.md');
const changelogDest = join('content', 'docs', 'changelog');

// Canonical Keep a Changelog categories, in the order the assembler buckets them
// (xtask/src/changelog.rs SECTION_ORDER).
const CHANGELOG_CATEGORIES = [
  'Highlights',
  'Added',
  'Changed',
  'Deprecated',
  'Removed',
  'Fixed',
  'Security',
  'Decisions',
];

function slugify(text) {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

// Escape the three characters MDX treats specially (`<`, `{`, `}`) everywhere
// except inside inline code spans and fenced code blocks, where they are already
// literal. CHANGELOG.md is authored as GitHub Markdown, not MDX, so a bare `<`
// would otherwise be read as a JSX tag.
//
// Inline-code state is tracked across lines, because a backtick span can wrap a
// soft line break (Markdown joins the two lines with a space): the ring command
// in the changelog does exactly this. Resetting the state per line would insert a
// stray backtick and escape the `<` inside the span. A fence boundary resets any
// dangling inline state.
function escapeMdx(text) {
  const escape = (ch) =>
    ({ '<': '&lt;', '{': '&#123;', '}': '&#125;' })[ch] ?? ch;
  let inFence = false;
  let inCode = false;
  const out = [];
  for (const line of text.split('\n')) {
    if (/^\s*```/.test(line)) {
      inFence = !inFence;
      inCode = false;
      out.push(line);
      continue;
    }
    if (inFence) {
      out.push(line);
      continue;
    }
    let rendered = '';
    for (const ch of line) {
      if (ch === '`') {
        inCode = !inCode;
        rendered += ch;
      } else if (!inCode && (ch === '<' || ch === '{' || ch === '}')) {
        rendered += escape(ch);
      } else {
        rendered += ch;
      }
    }
    out.push(rendered);
  }
  return out.join('\n');
}

function parseChangelogVersions(raw) {
  const lines = raw.replace(/\r\n/g, '\n').split('\n');
  const versions = [];
  let current = null;
  for (const line of lines) {
    const vm = line.match(/^##\s+\[([^\]]+)\]\s*(?:-\s*(.+))?\s*$/);
    if (vm) {
      current = { version: vm[1].trim(), date: (vm[2] || '').trim(), body: [] };
      versions.push(current);
      continue;
    }
    if (current) current.body.push(line);
  }
  return versions;
}

function bucketCategories(bodyLines) {
  const buckets = new Map();
  let key = null;
  for (const line of bodyLines) {
    const hm = line.match(/^###\s+(.+?)\s*$/);
    if (hm) {
      const heading = hm[1].trim();
      const canonical = CHANGELOG_CATEGORIES.find(
        (c) => c.toLowerCase() === heading.toLowerCase(),
      );
      if (canonical) {
        key = canonical;
        if (!buckets.has(key)) buckets.set(key, []);
        continue;
      }
      // A non-canonical subsection: keep it as a sub-heading of the current
      // category so its content survives.
      if (!key) {
        key = 'Notes';
        if (!buckets.has(key)) buckets.set(key, []);
      }
      buckets.get(key).push(`#### ${heading}`);
      continue;
    }
    if (key) buckets.get(key).push(line);
  }
  return buckets;
}

function writeChangelog() {
  if (!existsSync(changelogPath)) {
    console.error(`prebuild: ${changelogPath} is missing`);
    process.exit(1);
  }
  rmSync(changelogDest, { recursive: true, force: true });
  mkdirSync(changelogDest, { recursive: true });

  const versions = parseChangelogVersions(readFileSync(changelogPath, 'utf8'));
  const order = [...CHANGELOG_CATEGORIES, 'Notes'];
  const written = [];

  for (const v of versions) {
    const buckets = bucketCategories(v.body);
    const present = order.filter(
      (c) => buckets.has(c) && buckets.get(c).join('').trim().length,
    );
    if (!present.length) continue; // skip an empty version such as [Unreleased]

    const vslug = slugify(v.version);
    const vdir = join(changelogDest, vslug);
    mkdirSync(vdir, { recursive: true });

    const pages = [];
    for (const category of present) {
      const cslug = slugify(category);
      const body = escapeMdx(buckets.get(category).join('\n'))
        .replace(/\n{3,}/g, '\n\n')
        .trim();
      const frontmatter = [
        '---',
        `title: ${JSON.stringify(category)}`,
        `description: ${JSON.stringify(`${category} in fragcap ${v.version}.`)}`,
        '---',
        '',
        '',
      ].join('\n');
      writeFileSync(join(vdir, `${cslug}.mdx`), frontmatter + body + '\n');
      pages.push(cslug);
    }
    writeFileSync(
      join(vdir, 'meta.json'),
      JSON.stringify({ title: v.version, pages }, null, 2) + '\n',
    );
    written.push({ version: v.version, date: v.date, slug: vslug, first: pages[0] });
  }

  const indexLines = [
    '---',
    'title: Changelog',
    'description: "Notable changes to fragcap, by version and category."',
    '---',
    '',
    'All notable changes to fragcap, single-sourced from the release changelog.',
    'Each version below breaks out into its Keep a Changelog categories.',
    '',
    ...written.map(
      (v) =>
        `- **[${v.version}](/docs/changelog/${v.slug}/${v.first})**${
          v.date ? ` (${v.date})` : ''
        }`,
    ),
    '',
  ];
  writeFileSync(join(changelogDest, 'index.mdx'), indexLines.join('\n'));
  writeFileSync(
    join(changelogDest, 'meta.json'),
    JSON.stringify(
      { title: 'Changelog', pages: ['index', ...written.map((v) => v.slug)] },
      null,
      2,
    ) + '\n',
  );
  console.log(
    `prebuild: rendered ${written.length} changelog version(s) into ${changelogDest}`,
  );
}

// The Brand page data and assets, single-sourced from brand/ (issue #60). The
// palette and type roles are parsed from brand/tokens/*.css so the page cannot
// drift from the kit, and the logo, favicon, and brand-guide assets are copied
// into public/brand/ for the page to serve under the static export.
const brandDir = join('..', 'brand');
const brandGenDir = join('app', '(home)', 'brand');
const brandGenFile = join(brandGenDir, 'brand.generated.ts');
const brandPublicDir = join('public', 'brand');

// The subset of tokens the page presents as swatches, with a human label and a
// note on role. Keyed by the CSS custom property name in brand/tokens/colors.css.
// The canonical names are the --fc-* tokens, which hold the hex literals; the
// v1.0.0 --fragcap-* names are kept in the kit only as var() aliases of these,
// so the swatches read the --fc-* source directly.
const BRAND_SWATCHES = [
  ['--fc-signal-cyan', 'Signal Cyan', 'The single accent. Links, focus, emphasis.'],
  ['--fc-capture-orange', 'Capture Orange', 'Scarce. Genuine emphasis only, never the sole carrier of meaning.'],
  ['--fc-void', 'Void', 'The dark-first ground.'],
  ['--fc-surface', 'Surface', 'Raised panels on the void.'],
  ['--fc-line', 'Line', 'Borders and dividers.'],
  ['--fc-text', 'Text', 'Primary text on dark.'],
  ['--fc-text-muted', 'Text Muted', 'Secondary text and labels.'],
  ['--fc-fault', 'Fault', 'Failed capture or a hard error. Always paired with text or an icon.'],
  ['--fc-light-cyan', 'Light Cyan', 'The accent on light surfaces, for contrast.'],
];

function parseCssVars(css) {
  const vars = {};
  // Capture both the canonical --fc-* tokens and the legacy --fragcap-* aliases;
  // the swatch list reads the --fc-* names, which resolve to hex literals rather
  // than the var() indirection the --fragcap-* aliases now carry.
  for (const m of css.matchAll(/(--(?:fc|fragcap)-[a-z-]+)\s*:\s*([^;]+);/g)) {
    vars[m[1]] = m[2].trim();
  }
  return vars;
}

function writeBrand() {
  const colorsPath = join(brandDir, 'tokens', 'colors.css');
  if (!existsSync(colorsPath)) {
    console.error(`prebuild: ${colorsPath} is missing`);
    process.exit(1);
  }
  const colors = parseCssVars(readFileSync(colorsPath, 'utf8'));

  const palette = BRAND_SWATCHES.filter(([token]) => colors[token]).map(
    ([token, name, note]) => ({ token, name, hex: colors[token].toUpperCase(), note }),
  );

  mkdirSync(brandGenDir, { recursive: true });
  const out = [
    '// SPDX-License-Identifier: Apache-2.0',
    '// Generated by scripts/prebuild.mjs from brand/tokens/colors.css. Do not',
    '// edit by hand; edit the brand kit, the single source of truth.',
    'export interface BrandSwatch {',
    '  token: string;',
    '  name: string;',
    '  hex: string;',
    '  note: string;',
    '}',
    `export const brandPalette: BrandSwatch[] = ${JSON.stringify(palette, null, 2)};`,
    '',
  ].join('\n');
  writeFileSync(brandGenFile, out);

  // Copy the logo, favicon, and brand-guide assets into public/brand/ for the
  // page to serve. brand/logos/svg stays the source of truth.
  rmSync(brandPublicDir, { recursive: true, force: true });
  mkdirSync(brandPublicDir, { recursive: true });
  cpSync(join(brandDir, 'logos', 'svg'), join(brandPublicDir, 'logos'), {
    recursive: true,
  });
  cpSync(join(brandDir, 'favicons'), join(brandPublicDir, 'favicons'), {
    recursive: true,
  });
  const guide = join(brandDir, 'brand-guide.pdf');
  if (existsSync(guide)) {
    cpSync(guide, join(brandPublicDir, 'brand-guide.pdf'));
  }
  console.log(
    `prebuild: wrote ${palette.length} brand swatch(es) and copied assets into ${brandPublicDir}`,
  );
}

function main() {
  writeVersion();
  writeLicense();
  writeChangelog();
  writeBrand();
  writeDisclaimer();

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
