# Tasks: Managed Direct-Executable Launch

## Phase 1: Shared Launch Model

- [x] T001 Add failing public facade tests for direct launch path validation, explicit argv, environment overlay, and unchanged Steam representation.
- [x] T002 Implement the immutable managed-launch enum, direct launch value, typed preparation failures, and execution in `crates/fragcap/src/managed_launch.rs`.
- [x] T003 Export and document the public launch surface without changing `fragcap-core` or adding dependencies.

## Phase 2: Capture Integration

- [x] T004 Retain the selected `TargetEntry` from stored-target resolution through Capture preparation.
- [x] T005 Replace Capture's Steam-only effective launch field with the shared managed-launch value and prepare direct targets from the existing entry.
- [x] T006 Execute the prepared launch after watcher and packet pipeline arm, preserving launch-failure finalization and Steam behavior.
- [x] T007 Add Capture tests for direct preparation refusals and controlled child execution.

## Phase 3: Deep Capture Integration

- [x] T008 Admit `direct-exe-cold` and continue to refuse `direct-exe-warm` in library compatibility policy.
- [x] T009 Carry the retained prepared launch from ordinary Capture preparation into the Deep Capture launch adapter.
- [x] T010 Apply target-scoped proxy environment to the exact retained direct launch and prevent ordinary Capture from issuing it a second time.
- [x] T011 Add controlled public API and CLI tests for environment inheritance, proxy reachability, final socket ownership, launch failure, and cleanup truth.

## Phase 4: Documentation and Contract

- [x] T012 Update the master specification and outline for direct managed launch and replace the prior direct-launch refusal.
- [x] T013 Update glossary, security guidance, CLI help/reference, and public library documentation.
- [x] T014 Add a user-visible changelog fragment and verify documentation links and claims.

## Phase 5: Convergence and Gates

- [x] T015 Run the Spec Kit analysis pass and remediate every critical or high finding.
- [x] T016 Run targeted tests, formatting, Clippy, documentation, encoding, license, dependency, MSRV, and full repository gates.
- [x] T017 Mark completed tasks, inspect the final diff for mojibake and prohibited punctuation, and create the local S101 commit.
