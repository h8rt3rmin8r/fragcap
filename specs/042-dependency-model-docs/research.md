# Research: Slice 042

## R1. Mermaid rendering on the fumadocs static export

**Decision**: Render Mermaid with a client component. Add a `mermaid` npm
dependency, a `site/components/mermaid.tsx` client component that dynamically
imports `mermaid` in the browser and renders a `<pre class="mermaid">` source to
SVG, and map fenced `mermaid` code blocks to it through `site/mdx-components.tsx`
(override the `pre`/code handling so a ```mermaid fence routes to the component).

**Rationale**:
- The site ships as a static export (`next build` then a static `out/`), and the
  repo's constitution and CI keep the build browser-free. A build-time
  `rehype-mermaid` plugin rasterizes diagrams by launching Playwright/Chromium at
  build, which adds a browser dependency to every build and to CI. A client
  component moves rendering into the reader's browser and keeps the build a pure
  static export.
- Mermaid follows the page theme at render time, satisfying the light/dark
  requirement, if the component reads the active theme (fumadocs uses
  `next-themes`) and re-renders on theme change.
- The `mermaid` package must be imported dynamically inside a client effect
  (`await import('mermaid')`); importing it at module top level pulls a
  browser-only dependency into the server render and breaks the static export.

**Alternatives considered**:
- `rehype-mermaid` (build-time SVG): rejected for the browser-at-build cost above.
- Pre-rendering diagrams to committed SVGs by hand: rejected because it
  duplicates the source and drifts from the `mermaid` fences kept in the master
  specification; the single-source requirement wants one authored source.
- fumadocs' built-in support: fumadocs documents a client `Mermaid` component
  pattern rather than bundling a renderer, which is exactly this decision.

## R2. Diagrams valid on both GitHub and the site

**Decision**: Author the three diagrams as plain `mermaid` fences using only core
Mermaid syntax (flowchart/graph), no renderer-specific directives beyond an
optional `%%{init}%%` theme hint that both renderers tolerate.

**Rationale**: `docs/fragcap-specification.md` is read on GitHub, which renders
`mermaid` natively (the spec already carries four such fences). The site renders
the same fences through R1. Keeping to core syntax means one authored source
works on both surfaces (P-5, compatibility).

**Alternatives considered**: GitHub-only or site-only syntax extensions: rejected
because the same three diagrams must render on both.

## R3. Canonical home of the dependency model

**Decision**: `docs/glossary/platform-and-distribution.md` is the single source.
It already carries an `## npcap` entry; extend it (and add or extend Wireshark and
extcap entries) to state the required/recommended/optional tiers, matching the
`fragcap doctor` severities and the slice 040 taxonomy decision. README and
`getting-started.mdx` summarize and link.

**Rationale**: The glossary is already the single authored source that
`prebuild.mjs` renders into the site glossary tree; P-6 makes it the natural home
for dependency terminology. Restating the tiers in README or MDX would create the
exact drift this slice removes.

**Alternatives considered**: A standalone `docs/dependencies.md`: rejected, it
would compete with the glossary as a second home for the same terms.

## R4. The loopback correction

**Decision**: Remove "Support loopback traffic capture" as a user action from the
README install-option table and from the glossary npcap note; state that current
Npcap installs loopback automatically. Keep "WinPcap API-compatible mode" as a
real user-facing option.

**Rationale**: Modern Npcap (1.88 and later) installs loopback support
automatically and removed the checkbox; instructing a user to find and enable it
is a P-9 falsehood. The WinPcap-compatible mode checkbox still exists and still
matters, so it stays. This aligns the docs with what slice 040's doctor now
reports (three-valued loopback, undetermined preserved).

**Alternatives considered**: Leaving the row with a footnote: rejected, a user
action that no longer exists should not be presented as one.

## R5. The doctor verification step

**Decision**: Show real `fragcap doctor` output as a fenced code block, captured
from a real run, rather than a terminal screenshot.

**Rationale**: A code block is copyable, searchable, theme-aware, and does not
bind the docs to one machine's console rendering; it also stays legible when the
doctor output evolves. No doctor screenshot was provided, and slice 040 made the
human output deterministic and legible, which reproduces cleanly as text.

**Alternatives considered**: A terminal screenshot: rejected for the reasons
above and because none was captured.
