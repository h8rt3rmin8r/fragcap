# Tasks: Direct Route Qualification

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [operator contract](contracts/operator-route.md)

## Phase 1: Candidate reliability (#587)

- [x] T001 Add failing candidate tests for proof at expiry, proven nonselected owner, missing proof, reuse, saturation, cancellation, and fixed disposition accounting (FR-001 to FR-005).
- [x] T002 Add controlled receipt, socket-event, process-proof, decision and hold-time measurements without unrelated payload retention (FR-001).
- [x] T003 Evaluate exact proof before deadline expiry and promptly pass through a proven nonselected owner at receipt or during reconsideration (FR-002, FR-003).
- [ ] T004 Qualify the smallest candidate wait beginning at 100 ms with event history and dependent deadlines scaled together; retain finite queue and independent watchdog (FR-004, FR-005).
- [ ] T005 Re-run managed TCP/UDP chain and active parallel same-destination sentinels, including saturation, cancellation, UDP reply ownership and zero observed loss (FR-005, FR-010).

## Phase 2: Public route and fact authority (#583)

- [x] T006 Add failing tests for exact route-kind/version fact applicability and legacy environment-only refusal (FR-007, FR-008).
- [x] T007 Add the additive compatibility case/fact authority and migration or existing-schema equivalent; require final-client receipt, proxy and packet evidence for direct positive facts (FR-007, FR-008).
- [ ] T008 Add failing public CLI tests for one composite plan, guided next action, mixed route precedence, authorization drift, and ordinary admission (FR-006 to FR-009).
- [x] T009 Wire guided calibration and ordinary Deep Capture to the native direct adapter under one immutable reviewed plan, with deterministic self-exclusion and exact route identity (FR-006 to FR-009).
- [ ] T010 Reconcile human and JSON output, Doctor, artifact summaries, cancellation, exact owner release, cleanup and recovery (FR-009).
- [ ] T011 Run the controlled public CLI journey and exact MSI/ZIP certification against signed component bytes and active unrelated sentinels (FR-010).

## Phase 3: Operator journey (#584)

- [x] T012 Verify actual command and installer privilege paths, including extcap registration and live execution, before writing a mode table (FR-011).
- [x] T013 Reconcile README, getting started, capture modes, compatibility, CLI help/reference, Doctor/troubleshooting, packaging/migration and affected screenshots (FR-011).
- [ ] T014 Validate parser examples, links, package contents and documentation checks (FR-011).

## Phase 4: Full qualification and publication

- [x] T015 Update master specification, plan index, glossary, S200 FR-003, security inventory and dated changelog decisions for actual proved behavior (FR-004, FR-012).
- [x] T016 Run non-destructive spec-kit analyze; fix every critical and high finding without weakening the gate (FR-012).
- [ ] T017 Run complete foreground local gate and exact-head hosted CI/Windows/package matrices with zero skipped required rows (SC-001 to SC-005).
- [ ] T018 Commit the complete slice, push the authorized branch, and open the official PR with issue-level closure references (FR-012).
- [ ] T019 Respond to every CI and external Codex/security review finding, run at most one explicit second review round, and hand off the green reviewed PR for owner final review and merge (FR-012).
