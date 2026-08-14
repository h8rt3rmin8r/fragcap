# Documentation Quality Checklist: Slice 042

**Purpose**: Gate the authored docs, diagrams, and tutorial against the
constitution and the single-sourcing constraint before commit.
**Created**: 2026-08-14
**Feature**: [spec.md](../spec.md)

## Single-sourcing (no drift)

- [x] The required/recommended/optional dependency model is defined in exactly
      one authored place (`docs/glossary/platform-and-distribution.md`).
- [x] README and `getting-started.mdx` summarize and link to that source rather
      than restating the tier definitions.
- [x] The tier wording matches the `fragcap doctor` severities and
      `changelog.d/dependency-taxonomy.decisions.md`.
- [x] No hand-edits to generated `site/content/docs/glossary/*.mdx`.

## Correctness

- [x] The README no longer lists "Support loopback traffic capture" as a user
      action (loopback is automatic on current Npcap).
- [x] The docs state Npcap is by the Nmap Project and that the Wireshark
      installer bundles Npcap.
- [x] The `fragcap doctor` output shown is real, not invented.
- [x] Diagram content matches the architecture of record (capture and
      attribution only; npcap detection-only, never bundled).

## Diagrams

- [x] A `mermaid` fence renders as a diagram in the site static export.
- [x] All three diagrams render in both light and dark site themes.
- [x] The same three diagrams appear as `mermaid` fences in
      `docs/fragcap-specification.md` and render on GitHub.

## Tutorial and assets

- [x] All five install screenshots are served from `site/public/screenshots/`
      and referenced as `/screenshots/*.png`.
- [x] Each screenshot has descriptive alt text and a step caption.
- [x] The walkthrough ends with a `fragcap doctor` verification step.

## Constitution compliance

- [x] No em or en dashes anywhere added, including image alt text.
- [x] Every new term has a glossary entry in this same change (P-6).
- [x] All added/edited text is UTF-8 without BOM and LF.
- [x] No Rust crate, CLI surface, or runtime behavior changed.
- [x] A `changelog.d/` fragment is added.
- [x] `cargo xtask ci` is green (includes the docs linter) and the site builds.
