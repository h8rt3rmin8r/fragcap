# Tasks: Deep Capture Compatibility Bootstrap

**Input**: Design documents from `/specs/097-deep-capture-bootstrap/`

**Tests**: Required. S097 changes a security-sensitive state machine, fact semantics, and auditable output contracts. Tests precede each implementation group.

## Phase 1: Contract Tests

- [x] T001 Re-read issue #251, `spec.md`, `research.md`, `data-model.md`, all S097 contracts, `AI_CONTEXT.md`, and the Deep Capture sections of the master specification.
- [x] T002 Add CLI parser and help tests in `crates/fragcap-cli/tests/cli_args.rs` and `crates/fragcap-cli/tests/cli_deep_capture.rs` for paired `--calibrate` and `--launch-case` arguments and both phase values.
- [x] T003 Add pre-side-effect refusal tests in `crates/fragcap-cli/tests/cli_deep_capture.rs` for invalid flag pairs, reachability trust/output flags, TLS without trust intent, unsupported launch cases, and absent same-case reachability.
- [x] T004 Add confirmation tests in `crates/fragcap-cli/src/commands/deep_capture.rs` and `crates/fragcap-cli/tests/cli_deep_capture.rs` proving the full plan precedes confirmation and decline or unavailable input causes zero mutation.
- [x] T005 Add structured event rendering tests in `crates/fragcap-cli/src/events.rs` for calibration plan and phase lifecycle records.
- [x] T006 Add pure outcome-classifier tests in `crates/fragcap-cli/src/commands/deep_capture.rs` covering reached client, launcher-only, escaped tree, proxy not reached, no relevant traffic, inconclusive, local CA accepted, pinned, unknown trust, metadata-only, unsupported protocol, interrupted, and failed.
- [x] T007 Add fact-mapping tests proving routing does not imply propagation, controlled self-report may confirm propagation, generic TLS failure does not imply pinning, and only directly observed facts are proposed.

## Phase 2: CLI And Side-Effect-Free Plan

- [x] T008 Add typed calibration phase and launch-case options to `crates/fragcap-cli/src/cli.rs` without changing ordinary Deep Capture syntax.
- [x] T009 Add private calibration plan, phase, outcome, fact-write-result, fact-context, and bundle-context types in `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T010 Implement pure flag validation and declared-versus-observed launch-case validation before backend, bundle, proxy, trust, launch, or fact mutation.
- [x] T011 Refactor backend resolution so plan construction can identify the selected backend without spawning its version probe before confirmation.
- [x] T012 Add emitter-owned plan writing and safe console and preconfirmed paths with separately tested answer parsing.
- [x] T013 Emit `deep_capture.calibration_plan`, require `--yes` in JSON or noninteractive mode, and keep the human or structured plan visible under preconfirmation.

## Phase 3: Reachability Calibration

- [x] T014 Add failing controlled reachability integration tests proving unknown facts are allowed, no trust manager or trust event exists, and the plan, bounded phases, bundle, facts, and cleanup are present.
- [x] T015 Add the narrow completed-flow observation view in `crates/fragcap-core/src/flow.rs` only if required to distinguish direct, escaped, and launcher-only traffic from proxy silence.
- [x] T016 Route confirmed reachability through existing proxy startup, prepared Capture launch, observation ingestion, packet correlation, and controlled target machinery without constructing trust state.
- [x] T017 Apply finite launch, observation, proxy-shutdown, and cleanup deadlines and emit visible plus structured progress for every phase.
- [x] T018 Implement evidence-based reachability classification, keeping silence inconclusive and requiring affirmative evidence for launcher-only, escaped-tree, and no-proxy outcomes.
- [x] T019 Map reachability observations to launch, routing, owner, protocol, inspectability, variable, and independently supported propagation facts with complete provenance.

## Phase 4: TLS Calibration

- [x] T020 Add failing controlled TLS tests for current same-case routing prerequisite, local CA acceptance, explicit pinning evidence, unknown trust, metadata-only, unsupported protocol, proxy-not-reached, interruption, and failure.
- [x] T021 Gate TLS before mutation on current same-target, same-launch `proxy-routing=reached-client` evidence and explicit trust intent.
- [x] T022 Route confirmed TLS calibration through the existing session-owned CA and current-user trust manager, preserving exact cleanup and refusing any pinning bypass.
- [x] T023 Implement TLS outcome and fact mapping so only explicit backend evidence records pinning and application semantics remain separate from trust behavior.

## Phase 5: Persistence, Audit, And Ordinary Reuse

- [x] T024 Add failing partial-finalization tests for bundle write failure, individual fact-write failure, proxy startup failure, interruption, shutdown timeout, and cleanup failure.
- [x] T025 Replace the all-or-nothing compatibility writer with phase-aware pending facts and independently recorded append results through the existing store.
- [x] T026 Correct stored owner executable and handoff fields from actual observations instead of controlled-only defaults.
- [x] T027 Extend `compatibility.json`, `cleanup.json`, and `manifest.json` generation with the calibration plan, phase outcome, omissions, fact-write results, and complete resource reconciliation.
- [x] T028 Route every post-confirm path through bounded cleanup and independent fact and bundle finalization, returning a combined error only after results are recorded.
- [x] T029 Correct ordinary Deep Capture eligibility to require current same-case final-client routing without fabricating or requiring independent propagation, and add regression tests for stale, conflicting, and other-case rows.
- [x] T030 Verify `targets show` renders every calibration evidence row through the existing non-aggregating projection; change presentation only if a required provenance field is currently absent.

## Phase 6: Documentation And Release Record

- [x] T031 Add the compatibility-calibration glossary entry and repair the stale lifecycle-event glossary count before using the term in public guidance.
- [x] T032 Update `docs/fragcap-specification.md` and `docs/fragcap-spec-outline.md` with the CLI, phase, evidence, bundle, event, security, testing, and diagnostic contracts plus the routing-versus-propagation correction.
- [x] T033 Update `README.md`, getting started, architecture, CLI reference, Deep Capture compatibility reference, and output-format reference with the staged unknown-target workflow.
- [x] T034 Add issue-linked added and fixed changelog fragments plus a dated decisions fragment recording the routing/propagation correction and scope boundaries.

## Phase 7: Verification And Local Commit

- [x] T035 Run formatting, focused CLI/core/targets tests, and documentation checks, fixing failures without weakening assertions.
- [x] T036 Run `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all --locked`, and `cargo xtask ci` in the foreground.
- [x] T037 Audit changed files for UTF-8 without BOM, LF endings, trailing whitespace, mojibake, disallowed dash characters, private local evidence, prohibited capabilities, and unintended dependencies.
- [x] T038 Review the final diff against every S097 requirement and checklist item, then create one local feature commit without pushing.
