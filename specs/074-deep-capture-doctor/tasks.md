# Tasks: Deep Capture doctor readiness and cleanup

**Input**: Design documents from `/specs/074-deep-capture-doctor/`

## Phase 1: Setup

- [X] T001 Re-read `AI_CONTEXT.md`, issue #218, `docs/plans/deep-capture.md`, and the current doctor implementation.
- [X] T002 Create branch `codex/218-deep-capture-doctor`.
- [X] T003 Update `.specify/feature.json` to the active slice.

## Phase 2: Doctor Model

- [X] T004 Add Deep Capture fact types for proxy backend, CA trust, session storage, and residue.
- [X] T005 Extend the pure doctor classifier with Deep Capture rows.
- [X] T006 Ensure stale residue findings carry a structured cleanup action.

## Phase 3: Probe And Cleanup

- [X] T007 Add read-only session storage resolution and bounded residue scan.
- [X] T008 Detect external `mitmdump` availability and version when present.
- [X] T009 Detect analyzer key-log environment readiness.
- [X] T010 Add confirmation-gated cleanup of known Deep Capture session files under session storage.

## Phase 4: Tests And Docs

- [X] T011 Add unit tests for ready Deep Capture rows.
- [X] T012 Add unit tests for stale residue warnings and cleanup actions.
- [X] T013 Update doctor human and JSON goldens.
- [X] T014 Update the master specification.
- [X] T015 Add changelog fragment.

## Phase 5: Verification

- [X] T016 Run `cargo fmt --check`.
- [X] T017 Run `git diff --check`.
- [X] T018 Run `cargo test -p fragcap-cli --quiet`.
- [X] T019 Run `cargo test --workspace --quiet`.
- [X] T020 Run `cargo xtask lint`.
- [X] T021 Run `cargo xtask deps`.
- [X] T022 Run `cargo xtask spec`.
- [X] T023 Run `cargo xtask changelog --check`.
- [X] T024 Scan added lines for local PII, actual title names, endpoints, and account material.
