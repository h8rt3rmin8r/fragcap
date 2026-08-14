# Quickstart: validate slice 042

This slice ships documentation, three diagrams, and an install walkthrough. It is
validated by building the site, reading it, and running the repo gates.

## Prerequisites

- Node and pnpm for the site build (`cd site`).
- Rust toolchain for `cargo xtask ci`.

## Validate

1. Repo gates (docs linter, text lint, everything):

   ```bash
   cargo xtask ci
   ```

   Expect green. The documentation check enforces a glossary entry for every
   term and the no-dash / UTF-8 / LF rules, including image alt text.

2. Site build and static export:

   ```bash
   cd site && pnpm build
   ```

   Expect success. `prebuild.mjs` regenerates the glossary tree from
   `docs/glossary/`; `next build` compiles; the export writes `out/`.

3. Read the built site:
   - `/docs/architecture`: the three diagrams render as diagrams, in both light
     and dark theme.
   - `/docs/getting-started`: the dependency model is summarized with a link to
     the glossary; the install walkthrough shows all five screenshots with
     captions; the final step shows real `fragcap doctor` output.

4. Read on GitHub: `docs/fragcap-specification.md` renders the same three
   diagrams.

## Expected outcomes (maps to Success Criteria)

- SC-001: the dependency tiers are reachable from one canonical glossary page,
  linked from README and Getting Started.
- SC-002: five-step walkthrough with labeled screenshots, ending in doctor.
- SC-003 / SC-004: diagrams render on the site and on GitHub.
- SC-005: README no longer lists the loopback option as a user action; Nmap and
  Wireshark-bundles framing present.
- SC-006: `cargo xtask ci` green.
- SC-007: tier wording matches `fragcap doctor` severities.
