# Tasks: S157 complete native documentation contract

**Input**: Design documents from `specs/157-native-documentation-contract/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/documentation-contract-v2.md`, and `quickstart.md`

**Tests**: Required. This slice changes a machine-enforced documentation contract, command corpus, artifact authority, and exported site behavior. Contract and mutation tests precede the registry and prose changes.

## Phase 1: Setup and baseline

- [x] T001 Confirm the branch, clean baseline, current release identity, active feature directory, issue #427 project fields, and existing S154 documentation authority without changing product behavior.
- [x] T002 Record the exact current documentation pages, section headings, fragcap command blocks, artifact examples, and existing executable owners needed by the thirteen topics.
- [x] T003 [P] Confirm the shared Cargo test-discovery constraints in `xtask/src/review_handoff.rs` and select only supported non-ignored test authorities.
- [x] T004 [P] Confirm the exported-site route, link, search, and accessibility assertions in `site/tests/production-accessibility.spec.mjs` that can prove the stable native product contract route.

## Phase 2: Foundation tests

- [x] T005 Add failing schema-version, exact-top-level-key, closed-topic-set, duplicate-identity, and canonical-order tests for registry version 2 in `xtask/src/docs_coverage.rs`.
- [x] T006 Add failing confined-page, historical-page refusal, required-section presence, duplicate-section, and section-order mutation tests in `xtask/src/docs_coverage.rs`.
- [x] T007 Add failing executable-authority, example-authority reference, kind-specific path, feature, and completion-boundary mutation tests in `xtask/src/docs_coverage.rs`.
- [x] T008 Add failing published-baseline and current-source-boundary truth tests against `docs/published-release.json` and the closed S154 through S157 source vocabulary in `xtask/src/docs_coverage.rs`.
- [x] T009 Extend `crates/fragcap-cli/tests/cli_reference.rs` with explicit corpus ownership assertions for README plus all current nonhistorical site MDX files, preserving no-dispatch default and network-capable parsing.
- [x] T010 Add failing production-site assertions for the stable native product contract title, current search queries, and required route links in `site/tests/production-accessibility.spec.mjs`.

## Phase 3: Registry version 2

- [x] T011 Implement the exact version-two registry parser and top-level closure checks in `xtask/src/docs_coverage.rs`.
- [x] T012 Implement fenced-code-aware exact heading discovery, current-page confinement, section presence, uniqueness, and order validation in `xtask/src/docs_coverage.rs`.
- [x] T013 Implement topic, executable authority, example authority, published/current boundary, and completion-boundary validation in `xtask/src/docs_coverage.rs` using the shared Cargo test discovery.
- [x] T014 Create `docs/audits/native-documentation-coverage.v2.json` with the thirteen canonical topics, exact required sections, supported executable authorities, command corpus, safe committed specimens, controlled contracts, and five-issue completion boundary.
- [x] T015 Remove `docs/audits/native-documentation-coverage.v1.json` from current authority after version-two validation owns every former topic.
- [x] T016 Run the focused xtask documentation-coverage tests and resolve every positive and mutation failure.

## Phase 4: Current product contract content

- [x] T017 Rewrite `site/content/docs/reference/native-documentation.mdx` as the stable Native product contract index, removing S154 candidate wording and explaining published versus current-source authority.
- [x] T018 Reconcile architecture and setup guidance in `site/content/docs/architecture.mdx` and `site/content/docs/getting-started.mdx` with the native backend, supported host boundary, and safe verification paths.
- [x] T019 Reconcile Capture and Deep Capture mode, routing/protocol, bypass, refusal, boundedness, and completion language in `site/content/docs/guides/capture-modes.mdx` and the selected current reference pages.
- [x] T020 Reconcile artifact, correlation, diagnostics, recovery, security/privacy, packaging/migration, and stable API guidance in their selected current pages without inventing evidence or changing product behavior.
- [x] T021 Update `site/content/docs/index.mdx`, `README.md`, and `CONTRIBUTING.md` so agents and operators enter through the stable contract and use only current commands.
- [x] T022 Audit every current fragcap command block and linked artifact example against the registered parser, reader, golden, or controlled-test authority; correct stale prose and examples at their owned pages.
- [x] T023 Run the CLI reference corpus in default and network-capable configurations and resolve every parse, retired-consent, coverage, or accidental-dispatch failure.

## Phase 5: Exported site and traceability

- [x] T024 Complete the production-site route, search, link, anchor, and accessibility assertions for the native product contract in `site/tests/production-accessibility.spec.mjs` and related existing site test data only where needed.
- [x] T025 Update the Deep Capture documentation contract and status boundary in `docs/fragcap-specification.md` section 22.7 without claiming #333, #413, #331, #334, or #278 complete.
- [x] T026 Update `docs/fragcap-spec-outline.md` and `docs/plans/README.md` with the S157 contract boundary and chronological dependency record.
- [x] T027 Add the S157 documentation and decisions changelog fragments with the user-visible documentation contract and durable authority decisions.
- [x] T028 Run documentation lint, site unit tests, production export, internal-link/anchor checks, search assertions, and accessibility tests through the repository's hidden non-interactive launcher.

## Phase 6: Product-authority verification

- [x] T029 Run the registered manifest specimen reader, packet golden, complete native bundle, and stable public API tests from `quickstart.md` without installed product execution.
- [x] T030 Run `cargo xtask docs check` and verify every current command, topic, section, example authority, and status boundary is covered.
- [x] T031 Run `cargo xtask ci` in the foreground and resolve every failure within S157 scope.
- [x] T032 Inspect the final diff for status truth, issue-boundary preservation, no runtime/dependency/schema/version/release change, UTF-8 without BOM, LF endings, trailing whitespace, mojibake, and generated-file drift.
- [x] T033 Complete all task checkboxes, write `specs/157-native-documentation-contract/verification.md` with exact commands and results, and retain issue #427 at In progress until publication creates the review boundary.
- [x] T034 Commit the verified S157 change locally with a conventional commit and halt before push for explicit publication authorization.

## Dependencies and execution order

- Phase 1 precedes all implementation.
- Phase 2 is test-first and must demonstrate meaningful failures against the version-one contract or stale current prose before Phase 3 and Phase 4 make those tests pass.
- T011 through T013 precede T014 and T015. T015 occurs only after the version-two registry covers every former topic.
- T017 through T022 depend on the selected page and example ownership captured by T002 through T004 and encoded by T014.
- T024 depends on the stable route content from T017 and entry-point links from T021.
- T025 through T027 follow the completed current contract so their wording records actual scope.
- Full CI, hygiene inspection, verification record, project transition, and local commit occur only after every focused gate passes.

## Parallel opportunities

- T003 and T004 are independent read-only baseline audits.
- After foundation tests exist, content groups T018 through T020 can be edited independently, but the final command and status audit T022 is serial.
- Product-authority tests in T029 are logically independent but must run in a bounded foreground sequence under the host rules.

## Implementation strategy

Deliver one coherent documentation authority boundary: first make version-two contract failures observable, then replace the current registry, then reconcile prose and executable examples, then prove the exported site and product-owned artifact contracts, and finally run full CI. Do not execute the installed product, mutate trust, launch a target, publish, or close external acceptance issues.
