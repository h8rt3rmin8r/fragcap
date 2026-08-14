# Tasks: Dependency-model docs, Mermaid diagrams, and install tutorial

**Feature**: 042-dependency-model-docs | **Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

Organized by user story. This is a documentation and static-site slice: "tests"
are the repo gates (`cargo xtask ci`, including the documentation linter) and the
site build, not unit tests. No Rust or behavior change.

## Phase 1: Setup

- [ ] T001 Vendor the five minified install screenshots into `site/public/screenshots/` (01_wireshark_choose-components_extcap.png, 02_wireshark_choose-install-location.png, 03_wireshark_packet-capture_install-npcap.png, 04_wireshark_npcap-setup.png, 05_wireshark_npcap-setup_options.png) from the provided minified set.
- [ ] T002 Add the `mermaid` dependency to `site/package.json` and install it (`cd site && pnpm add mermaid`), confirming `pnpm-lock.yaml` updates.

## Phase 2: Foundational (blocking prerequisites)

- [ ] T003 Create the theme-aware client renderer `site/components/mermaid.tsx` (SPDX header): a `'use client'` component that dynamically imports `mermaid` in an effect, initializes it to follow the active theme (next-themes), renders the fenced source to SVG, and re-renders on theme change.
- [ ] T004 Wire the component into `site/mdx-components.tsx`: route ```mermaid fenced code blocks to `<Mermaid>` (override `pre`/code handling so a mermaid fence renders as a diagram), keeping all other MDX components intact and the SPDX header.
- [ ] T005 Verify a throwaway ```mermaid fence renders as a diagram in `pnpm build` output before authoring real diagrams (smoke test of T003/T004), then remove the throwaway.

## Phase 3: User Story 1 - Newcomer installs prerequisites and verifies (P1)

**Goal**: A complete, illustrated, verifiable install-to-doctor path on Getting Started.

**Independent test**: Build the site; Getting Started names the three tiers with what each provides, shows five labeled screenshots across the install steps, and ends with real `fragcap doctor` output.

- [ ] T006 [US1] Capture real `fragcap doctor` output for the verification step: build the CLI and run `fragcap doctor` (or reuse the slice 040 golden `crates/fragcap-cli/tests/goldens/doctor-ready.txt` if it is the current human output), saving the exact text to paste into the tutorial.
- [ ] T007 [US1] Rewrite `site/content/docs/getting-started.mdx` "Install the capture driver" section into an annotated walkthrough: reference `/screenshots/01..05*.png` in order, each with dash-free alt text and a step caption; correct the loopback framing (loopback is automatic on current Npcap; do not tell the reader to enable it); keep WinPcap-compatible mode as the real option; add the "Npcap is by the Nmap Project" and "Wireshark bundles Npcap" framing.
- [ ] T008 [US1] Add a short dependency-model summary near the top of `getting-started.mdx` (required npcap / recommended Wireshark / optional extcap) that links to the glossary source rather than restating the tier definitions.
- [ ] T009 [US1] Replace the "Verify the install" section's bare command with the real `fragcap doctor` output block from T006, framed as the success check.

## Phase 4: User Story 3 - Dependency model stated once, cannot drift (P2)

**Goal**: One canonical source for the tiers; README and Getting Started link to it; wording matches doctor severities.

**Independent test**: The tier definitions live only in the glossary; README and Getting Started reference it; wording matches the doctor severities and the slice 040 decision.

- [ ] T010 [US3] Extend `docs/glossary/platform-and-distribution.md`: state the required/recommended/optional model in the npcap, Wireshark, and extcap entries (add a Wireshark or extcap entry if absent), matching the `fragcap doctor` severities and `changelog.d/dependency-taxonomy.decisions.md`. Add a glossary entry for any new term (P-6).
- [ ] T011 [US3] Correct the stale loopback framing in `docs/glossary/platform-and-distribution.md` (and `capture-and-networking.md` loopback entry if it repeats the "option is required" claim): loopback installs automatically on current Npcap.
- [ ] T012 [US3] Correct the README install-option table in `README.md`: remove "Support loopback traffic capture" as a user action, keep WinPcap-compatible mode, add the Nmap Project and Wireshark-bundles-Npcap framing, and link the dependency model to the glossary rather than restating the tiers.

## Phase 5: User Story 2 - Diagrams explain the pieces and data flow (P2)

**Goal**: Three seed diagrams render on the site and on GitHub.

**Independent test**: `architecture.mdx` renders three diagrams in both themes; `docs/fragcap-specification.md` renders the same three on GitHub.

- [ ] T013 [P] [US2] Author the three ```mermaid diagrams (pieces; runtime data flow with the extcap path; acquisition/bundling detection-only) into `site/content/docs/architecture.mdx`, using core Mermaid syntax valid on both renderers.
- [ ] T014 [P] [US2] Add the same three ```mermaid fences to `docs/fragcap-specification.md` in the appropriate architecture section, identical sources to T013.
- [ ] T015 [US2] Build the site and confirm all three diagrams render as diagrams (not code) in light and dark themes; confirm the sources render on GitHub (visually inspect the Markdown preview).

## Phase 6: Polish & Cross-Cutting

- [ ] T016 Add a `changelog.d/042-dependency-model-docs.added.md` (and a `.changed.md` for the README/glossary correction) fragment describing the docs, diagrams, and tutorial.
- [ ] T017 Run `cargo xtask ci` in the foreground: fix any documentation-linter finding (missing glossary entry, em/en dash including alt text, UTF-8/LF, SPDX on the new .tsx).
- [ ] T018 Run `cd site && pnpm build` in the foreground: confirm the static export succeeds, all three diagrams and five screenshots render, and no broken asset paths.
- [ ] T019 Walk the `checklists/docs.md` gate and check every box; confirm no Rust crate, CLI surface, or runtime behavior changed (`git diff --stat` touches only docs/site/changelog/specs).

## Dependencies & order

- Phase 1 (T001, T002) before Phase 2.
- Phase 2 (T003, T004, T005) blocks Phase 5 (diagrams need the renderer). It does not block Phase 3/4 (prose/screenshots), which can proceed in parallel with Phase 2.
- Phase 3 (US1) depends on T001 (screenshots) and T006 (doctor output). T008 depends on T010 existing to link to.
- Phase 4 (US3) is largely independent; T012 links to T010's glossary anchor.
- Phase 5 (US2) depends on Phase 2. T013 and T014 are [P] (different files, same authored source).
- Phase 6 gates the commit.

## Parallel opportunities

- T013 and T014 can be authored in parallel (paste the same three sources into two files).
- Phase 3/4 prose work can proceed while Phase 2 renderer work is in progress.

## MVP scope

User Story 1 (Phases 1, 3) alone delivers the annotated, verifiable install
walkthrough, the highest-value outcome. User Story 3 (single-sourcing) and User
Story 2 (diagrams) complete the slice.
