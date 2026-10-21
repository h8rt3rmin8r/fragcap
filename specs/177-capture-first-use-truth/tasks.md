# Tasks: Truthful Capture First-Use Guidance

**Branch**: `codex/s177-capture-first-use-truth`

## Phase 1: Specification and analysis

- [x] T001 Create S177 `spec.md`, requirements and first-use checklists, `plan.md`, `research.md`, `data-model.md`, `contracts/first-use-results.md`, and `quickstart.md`.
- [x] T002 Run the Spec-Kit analyze gate against the constitution, spec, plan, tasks, and existing contracts; resolve findings before implementation.

## Phase 2: Current-session Doctor truth (US1)

- [x] T003 [US1] Add red elevated/unelevated current-token readiness, human/JSON/exit, and fix-recheck regressions in `crates/fragcap-cli/tests/cli_doctor.rs` and focused Doctor tests.
- [x] T004 [US1] Make `Platform/privilege` a blocking live-mode check with a concrete elevation action in `crates/fragcap-cli/src/doctor/checks.rs`; align Doctor wording and fix output without changing offline replay.

## Phase 3: Steam setup-only next action (US2)

- [x] T005 [US2] Add a red isolated-store setup-only then next-invocation workflow test, plus strict exact-LF refusal cases in `crates/fragcap-cli/tests/cli_calibrate.rs`.
- [x] T006 [US2] Emit a distinct verified setup-complete guidance/verdict with known target id, no-reachability reason, and quoted same-store rerun command in `crates/fragcap-cli/src/commands/calibrate.rs`.
- [x] T007 [US2] Confirm no workflow or compatibility fact is created by setup and no refusal path emits setup-complete.

## Phase 4: Mandatory Deep Capture help (US3)

- [x] T008 [US3] Add red parser, 80-column help, copyable example, and first-use terminology tests in `crates/fragcap-cli/tests/cli_help.rs`.
- [x] T009 [US3] Make `--launch` required and move it to Required inputs in `crates/fragcap-cli/src/cli.rs`; update direct-call refusal in `commands/deep_capture.rs`.
- [x] T010 [US3] Correct embedded PowerShell examples and concrete HAR, key-log, and mutual-TLS descriptions in `workflow_help.rs` and `cli.rs`, preserving exact-plan input.

## Phase 5: Documentation and verification

- [x] T011 Update `docs/fragcap-specification.md` and `docs/plans/README.md` in chronological order; add `changelog.d/S177-*.md` fragments.
- [x] T012 Re-run Spec-Kit analysis, focused tests, and `cargo xtask ci` on the final head; inspect diff, UTF-8, and mojibake before commit.
- [ ] T013 Commit, push the owner-authorized branch, publish an official PR closing #487/#494/#495, then handle CI and at most two review rounds before final merge handoff.
