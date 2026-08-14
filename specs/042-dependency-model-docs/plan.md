# Implementation Plan: Dependency-model docs, Mermaid diagrams, and install tutorial

**Branch**: `042-dependency-model-docs` | **Date**: 2026-08-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/042-dependency-model-docs/spec.md`

## Summary

Document fragcap's external-dependency model once, in the authored glossary, and
have the README and the site Getting Started page summarize and link to it;
correct the stale README and glossary loopback framing (current Npcap installs
loopback automatically); add a theme-aware Mermaid rendering component to the
fumadocs site and author three seed diagrams (pieces, runtime data flow,
acquisition and bundling) that also render on GitHub from the master
specification; and build an annotated install walkthrough in Getting Started from
five real installer screenshots, closing with a real `fragcap doctor` output
block. No Rust or behavior change.

## Technical Context

**Language/Version**: Documentation only. Markdown (`docs/**`, README), MDX and
TSX for the fumadocs site (`site/`), one client React component. No Rust change.

**Primary Dependencies**: fumadocs-ui/core/mdx 16.x (site), a new `mermaid`
npm dependency for the site's client renderer. GitHub's native Mermaid for the
master specification and README paths.

**Storage**: N/A. Static assets under `site/public/screenshots/`.

**Testing**: `cargo xtask ci` (includes the documentation linter and the repo
text lint: no em or en dashes, UTF-8, LF, SPDX where applicable) plus the site
static export build (`pnpm build`, which runs `prebuild.mjs` then `next build`).

**Target Platform**: The documentation site (static export to GitHub Pages) and
GitHub-rendered Markdown.

**Project Type**: Documentation and static site within the Rust workspace repo.

**Performance Goals**: N/A (docs). The site build must stay browser-free at build
time (constraint below).

**Constraints**: No em or en dashes anywhere including alt text (P-8); every new
term gets a glossary entry same-change (P-6); the dependency model is
single-sourced and cannot contradict `fragcap doctor` (P-9); diagrams must show
detection-only acquisition, never bundling (P-1); the generated site glossary
tree is never hand-edited; the site static export must not require a browser at
build time.

**Scale/Scope**: One glossary source extended, README + one MDX page edited, one
site component added, three diagrams authored in two places, five screenshots
vendored, one changelog fragment.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive Observation**: The acquisition/bundling diagram and prose state
  detection-only: fragcap never downloads, installs, or bundles npcap. Diagrams
  reinforce the posture rather than weakening it. PASS.
- **P-2 Core Platform-Neutral**: No crate changes; `fragcap-core` untouched. PASS.
- **P-3 Capture/Attribution Separate**: The data-flow diagram shows capture and
  attribution as distinct stages, matching the architecture of record. PASS.
- **P-4 No Silent Loss**: N/A (no capture path changes).
- **P-5 Compatibility Outranks Richness**: Diagrams use plain `mermaid` fences
  valid on both GitHub and the site renderer; no renderer-specific extensions. PASS.
- **P-6 Glossary First**: Any new term (for example the dependency tiers, or
  "Nmap Project" framing) is defined in `docs/glossary/` in this same change; the
  documentation linter enforces it. PASS by construction.
- **P-7 Wrappers Stay Thin**: N/A.
- **P-8 House Standards**: All added text is UTF-8, LF, dash-free including alt
  text; `.tsx`/`.mjs`/`.css` carry SPDX. The repo lint gates it. PASS.
- **P-9 The Instrument Does Not Lie**: The core correctness deliverable. The
  loopback framing is corrected to match current Npcap; the doctor output shown
  is real; the tier language matches the doctor severities. PASS.

No violations. Complexity Tracking not required.

## Project Structure

### Documentation (this feature)

```text
specs/042-dependency-model-docs/
├── plan.md              # This file
├── spec.md              # Feature spec
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (entities: tiers, diagrams, screenshots)
├── quickstart.md        # Phase 1 output (how to validate)
└── checklists/
    ├── requirements.md  # Spec quality
    └── docs.md          # Documentation quality gate
```

### Source (repository paths touched)

```text
docs/glossary/platform-and-distribution.md   # canonical dependency model; fix loopback framing
docs/glossary/capture-and-networking.md      # loopback entry: align with automatic-install reality (if stale)
docs/fragcap-specification.md                # add the three mermaid diagrams
README.md                                    # correct the npcap install-option table; Nmap/Wireshark-bundles framing; link the model
site/content/docs/getting-started.mdx        # dependency summary + link; annotated screenshot walkthrough; real doctor output
site/content/docs/architecture.mdx           # host the three mermaid diagrams on the site
site/mdx-components.tsx                       # wire the <Mermaid> client component
site/components/mermaid.tsx                   # NEW theme-aware client renderer (mermaid package)
site/package.json                            # add the mermaid dependency
site/public/screenshots/*.png                 # NEW: five vendored install screenshots
changelog.d/042-dependency-model-docs.*.md    # changelog fragment(s)
```

**Structure Decision**: Single-source the dependency model in the authored
glossary (`docs/glossary/platform-and-distribution.md`), the one place
`prebuild.mjs` renders the site glossary from. README and Getting Started
reference it. The three diagrams live on `architecture.mdx` (site) and in
`docs/fragcap-specification.md` (GitHub), authored once as identical `mermaid`
fences. Mermaid renders via a client component, not a build plugin, to keep the
static export browser-free.

## Complexity Tracking

No constitution violations; no entries.
