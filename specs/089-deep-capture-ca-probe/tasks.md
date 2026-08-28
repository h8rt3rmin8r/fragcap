# Tasks: Deep Capture CA Trust-State Probe

**Input**: Design documents in `specs/089-deep-capture-ca-probe/`

## Phase 1: Evidence Model and Tests

- [x] T001 Add injected owned-identity and store-inventory test values in
  `crates/fragcap-cli/src/doctor/probe.rs`
- [x] T002 Add failing classification tests for absent, current-user, wrong-store,
  mismatch, unknown, ambiguity, and unrelated certificates
- [x] T003 Add failing report/action tests in
  `crates/fragcap-cli/src/doctor/checks.rs` and
  `crates/fragcap-cli/tests/cli_doctor.rs`

## Phase 2: Read-Only Production Probe

- [x] T004 Implement manifest thumbprint extraction, normalization, deduplication,
  and bundled-material validation in `crates/fragcap-cli/src/doctor/probe.rs`
- [x] T005 Implement read-only Windows current-user and local-machine Root
  enumeration using existing CryptoAPI bindings
- [x] T006 Connect gathered evidence to the pure classifier and replace the
  placeholder production result

## Phase 3: Safe Cleanup Reconciliation

- [x] T007 Ensure checks offer cleanup only for exact actionable resources
- [x] T008 Extend confirmation-gated cleanup to remove only an exact owned trust
  entry, preserving existing bounded file cleanup
- [x] T009 Test that malformed, unknown, and unrelated evidence never removes or
  offers removal of trust

## Phase 4: Documentation and Validation

- [x] T010 Update `docs/fragcap-specification.md` section 26.3 and
  `site/content/docs/reference/cli.mdx`
- [x] T011 Add the S089 changelog fragment and record Windows demonstration results
- [x] T012 Run formatting, targeted tests, full gates, encoding, and mojibake checks
- [x] T013 Commit the complete slice locally and halt before push

## Dependencies and Execution Order

T001-T003 establish red tests. T004-T006 implement the observable state. T007-T009
depend on exact state identity. T010-T013 follow green behavior. No task is delegated
because the slice is sequential and shares the same small set of files.
