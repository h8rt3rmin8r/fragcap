# Tasks: Landing page and getting-started rewrite (S057)

**Feature dir**: `specs/057-landing-getting-started/`
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

Local build/test uses the GNU toolchain: `cargo +1.96.0-x86_64-pc-windows-gnu ...`.
CI runs the MSVC `cargo xtask ci`. No tests are requested beyond the existing
doctor unit tests (US4), which must be kept green and extended to guard the change.

## QA issue mapping (SC-006)

| Issue | Resolved by task |
| --- | --- |
| #130 (extcap command before install) | T014, T016 |
| #131 (no download link) | T015 |
| #132 (prereqs installed in step 2) | T014 |
| #133 (installer npcap exit-dialog narrative) | T014, T017 |
| #134 (run as Administrator; extcap home) | T016 |
| #135 (Get a profile unclear) | T018 |

## Phase 1: Setup

- [ ] T001 Confirm branch `057-landing-getting-started` and a clean tree; re-read the shipped surface in `crates/fragcap-cli/src/cli.rs` and the README quickstart so the docs mirror the real grammar (no file changes).

## Phase 2: Foundational (blocking prerequisite for the getting-started doctor sample)

**This phase is US4; it is P1 and MUST land before the getting-started rewrite (T016) because the doctor sample depends on the corrected binary.**

- [ ] T002 [US4] Remove the `profile dir` identity row from `identity()` in `crates/fragcap-cli/src/doctor/checks.rs` (leave `version`, `binary`, `catalog db`, `local db`).
- [ ] T003 [US4] Remove the `Profiles` section from `crates/fragcap-cli/src/doctor/checks.rs`: delete the `PROFILES` constant, the `profiles(inputs)` function, and its `checks.push(profiles(inputs))` in `assemble`/`report`.
- [ ] T004 [US4] Remove the `profile_dir`, `bundled_count`, and `user_count` fields from `Inputs` in `crates/fragcap-cli/src/doctor/mod.rs` (and their doc comments).
- [ ] T005 [US4] Remove the probe plumbing in `crates/fragcap-cli/src/doctor/probe.rs`: `count_profiles`, the `user_profile_dir()` count call, the `bundled().len()` assignment, and drop the three fields from both `gather`/`gather_windows` `Inputs` constructions and the `identity_fields` tuple destructuring if `profile_dir` becomes unused.
- [ ] T006 [US4] In `crates/fragcap-cli/src/paths.rs`, remove `user_profile_dir` and the profile-dir/`bundled()` plumbing only if the doctor removal leaves them unreferenced by non-doctor code; if `BundledSet`/`SearchPath` construction still needs them for capture, keep those and remove only the doctor-only path. Verify with a workspace grep before deleting.
- [ ] T007 [US4] Update the doctor unit tests in `crates/fragcap-cli/src/doctor/checks.rs`: fix the identity-row assertion to `version, binary, catalog db, local db`; drop the three removed fields from `ready_inputs()`/test `Inputs`; remove the `inputs.profile_dir = None` line; add an assertion that no section is named `Profiles` and no row is labeled `profile dir`.
- [ ] T008 [US4] Update any other test or fixture referencing the removed rows/fields (search `crates/fragcap-cli` for `profile dir`, `profile_dir`, `bundled_count`, `user_count`, `Profiles`), then run `cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-cli` to green.
- [ ] T009 [US4] Reconcile master specification section 26.3 (the doctor report rows) with the removed profile surface, then run `cargo +1.96.0-x86_64-pc-windows-gnu xtask spec` to refresh the Applies-To binding.

**Checkpoint (US4 done)**: `fragcap doctor` emits no `profile dir` row and no
`Profiles` section; exit status and all other rows unchanged; `-p fragcap-cli`
tests green.

## Phase 3: US1 - First-time reader reaches a completed capture (Priority: P1)

**Depends on Phase 2 (the doctor sample uses the corrected output).**

- [ ] T010 [US1] Draft the new getting-started section order and the two Mermaid/diagram assets (reuse the dependency-model diagram from `site/content/docs/architecture.mdx`) per [data-model.md](data-model.md); confirm every command used exists in the CLI surface contract.
- [ ] T011 [US1] Capture the corrected `fragcap doctor` sample text (from T002-T008 output) to embed in the guide, so the sample matches the binary (FR-019).
- [ ] T012 [US1] Rewrite the "Before you begin" section of `site/content/docs/getting-started.mdx`: dependency-model diagram + prose (npcap required, Wireshark recommended, extcap optional); the three capture expectations; the extcap integration named descriptively with NO command (fixes #130 half); the conditional prerequisite-install walkthrough (the npcap/Wireshark screenshots moved here, framed "if you do not already have them") (fixes #132).
- [ ] T013 [US1] Rewrite "1. Install fragcap" in `getting-started.mdx`: add the GitHub releases download affordance and name the `.msi`, `.zip`, and `.sha256` assets; keep the SmartScreen/checksum guidance and the MSI optional-extcap note (fixes #131).
- [ ] T014 [US1] Reconcile the npcap narrative across the guide so the prerequisite, the installer exit-dialog npcap prompt, and the S056 `doctor --fix` npcap action tell one coherent story (detection-only; user-confirmed vendor-installer fetch permitted; nothing bundled/hosted) (fixes #133 docs half).
- [ ] T015 [US1] Rewrite "2. Verify the install" in `getting-started.mdx`: tell the reader to open the terminal as Administrator (right-click, Run as administrator) and explain elevation turns `privilege`/capture green; embed the corrected doctor sample; make this the single home of the optional `fragcap extcap install` guidance and the `analyzer extcap` optional-row note (fixes #134, #130 other half).
- [ ] T016 [US1] Rewrite "3. Find a target" (replacing "4. Get a profile") in `getting-started.mdx`: `fragcap targets` as the automatic-discovery happy path; define a Steam App ID inline (the number in a Steam store/library URL) and show the non-Steam `targets add`/`scan` route; state that the numbered row is what `fragcap capture <n>` honors; remove all profile-file/`steam profile`/`--profile` references (fixes #135).
- [ ] T017 [US1] Rewrite "4. Capture" and "5. Open the result in Wireshark" in `getting-started.mdx`: `fragcap capture <n>` (or `--target <handle>`) producing `capture.fcapng`; keep the Wireshark endpoint prose; ensure the guide ends with a capture file on disk (FR-006, SC-001).
- [ ] T018 [US1] Update the frontmatter `description` of `getting-started.mdx` if it still implies profile authoring; verify the whole file for retired tokens per the inventory contract.

**Checkpoint (US1 done)**: reading the guide literally reaches a `.fcapng`; no
retired token remains in the file; the doctor sample matches the binary.

## Phase 4: US2 - Landing page strong opening (Priority: P1)

- [ ] T019 [P] [US2] Rewrite `site/app/(home)/page.tsx`: the settled opening sentence ("Your capture recorded 40,000 packets. It cannot tell you which one your game sent.") and the two explanatory paragraphs from issue #144 verbatim (FR-001); keep the masthead and prerequisite callout.
- [ ] T020 [US2] Replace the retired `fragcap run --profile eso` code block in `page.tsx` with the `fragcap targets` hero listing (numbered `# / TARGET / CAPTURE / KNOWN` columns + the `fragcap capture 1` hint) in the monospace face (FR-002).
- [ ] T021 [US2] Add the dependency-model diagram to `page.tsx` (reuse the Architecture Mermaid content, rendered appropriately for the landing page) and ensure exactly one primary call to action (Get started); reroute the capability bullet that linked to `writing-a-profile` to a discovery/getting-started link that still resolves (FR-003, FR-015).
- [ ] T022 [P] [US2] Fix the specimen line in `site/app/(home)/brand/page.tsx`: replace `fragcap run --profile eso --out capture.fcapng` with a current invocation (e.g. `fragcap capture --target eso --out capture.fcapng` or `fragcap targets`) (FR-005).
- [ ] T023 [US2] Confirm `page.tsx` carries none of the section-23.1 prohibitions (testimonials, feature grids, badges, pricing, sponsorship) and holds section-23.3 voice (FR-004).

**Checkpoint (US2 done)**: landing and brand pages carry the settled copy, a real
`fragcap targets` example, the diagram, one CTA, and no retired token.

## Phase 5: US3 - Reference set matches the shipped surface (Priority: P2)

- [ ] T024 [US3] Rewrite `site/content/docs/reference/cli.mdx` to the surface in [contracts/cli-reference-surface.md](contracts/cli-reference-surface.md): the nine commands grouped Capture / Targets / Environment / Data, global options, real flags; remove the `run`, `tap`, `profile`, and `steam` (`steam profile`) sections and the `--profile` selector; keep `steam list`.
- [ ] T025 [US3] Fix `site/content/docs/guides/capture-modes.mdx`: replace the three `fragcap run --profile <game-id>` examples with `fragcap capture --target <selector>` and reword the "profile's `[capture] mode`" reference to the current capture-config framing.
- [ ] T026 [US3] Delete `site/content/docs/guides/writing-a-profile.mdx`.
- [ ] T027 [US3] Delete `site/content/docs/reference/profile-schema.mdx`.
- [ ] T028 [US3] Update `site/content/docs/meta.json`: remove the `guides/writing-a-profile` and `reference/profile-schema` nav entries; reflow the Guides/Reference groupings so no empty section remains.
- [ ] T029 [US3] Update `site/content/docs/index.mdx`: drop the `writing-a-profile` and `profile-schema` links; reframe the Guides bullet around discovery/capture and point Reference at `cli`, `target-schema`, and `output-formats`.
- [ ] T030 [US3] Update the `writing-a-profile` inbound link in `site/content/docs/architecture.mdx` (the "Naming an indirectly launched client" section) to reference the surviving conceptual anchor rather than the deleted page.

**Checkpoint (US3 done)**: the reference set documents only shipped commands; no
nav entry or link points at a deleted page.

## Phase 6: Polish and cross-cutting

- [ ] T031 Run the retired-token grep over `site/` per [quickstart.md](quickstart.md) step 2-3; confirm zero retired usages and zero dangling links (SC-002).
- [ ] T032 P-6 glossary check: confirm the rewrites introduce no undefined term; if any new term was added, add its glossary entry in this change; run `cargo +1.96.0-x86_64-pc-windows-gnu xtask docs check`.
- [ ] T033 If `pnpm` is available, run `cargo xtask docs build` (or `pnpm --dir site build`) and confirm no broken internal links; otherwise note it is verified in CI.
- [ ] T034 Encoding sweep: confirm every edited/new file is UTF-8 without BOM, LF, and contains no em-dashes or en-dashes (including the doctor source comments).
- [ ] T035 Add the changelog fragment `changelog.d/S057-landing-getting-started.added.md` (with the `spec-impact: 23.1, 26.3` marker), noting the docs convergence, the doctor profile-row removal, the QA issue closures (#130-#135, #133 docs half), and the IGDB deferral rationale.
- [ ] T036 Run the full gate: `cargo +1.96.0-x86_64-pc-windows-gnu xtask ci` locally (MSVC gate runs in CI); confirm green (FR-020, SC-005).

## Dependencies

- Phase 2 (US4) blocks T011/T015 in Phase 3 (the doctor sample).
- Phase 3 (US1), Phase 4 (US2), Phase 5 (US3) are otherwise independent and could
  proceed in parallel, but share the retired-token goal verified in T031.
- Phase 6 depends on all prior phases.

## Parallel opportunities

- T019 and T022 touch different files ([P]).
- Within Phase 5, T026/T027 (deletions) and T024/T025 (rewrites) touch different
  files and can interleave, but T028-T030 (link/nav updates) must follow the
  deletions.

## MVP scope

US4 + US1 (the doctor fix and the getting-started rewrite) is the MVP: it removes
the wall and delivers a dead-end-free first-run path ending at a capture file. US2
(landing) and US3 (reference convergence) complete the acceptance criteria and land
in the same slice because the retired-token criterion is site-wide.
