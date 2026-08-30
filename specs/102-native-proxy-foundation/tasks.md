# Tasks: Native Deep Capture Proxy Foundation

**Input**: Design documents in `/specs/102-native-proxy-foundation/`

**Tests**: Required. Runtime ownership, accounting, and product claims fail silently without mechanical coverage.

## Phase 1: Toolchain and crate setup

- [x] T001 Raise workspace and CI MSRV claims from 1.82 to 1.88 and update explanatory policy text.
- [x] T002 Add exact minimal-feature native proxy dependencies to the workspace and lockfile.
- [x] T003 Create publishable `crates/fragcap-proxy` metadata, README, and module skeleton.
- [x] T004 Extend the dependency-direction gate for `fragcap -> fragcap-proxy` and make the proxy crate a leaf sibling.

## Phase 2: Typed contract tests first

- [x] T005 [P] Add configuration tests for loopback-only endpoints and non-zero finite limits.
- [x] T006 [P] Add stable identity and no-inspection-capability tests.
- [x] T007 [P] Add lifecycle tests for start, observe, stop, cleanup, repeated calls, and bind failure.
- [x] T008 [P] Add bounded saturation and accounting-conservation tests.
- [x] T009 [P] Add cancellation, forced-timeout, panic/join-failure, and ten-cycle residue tests.

## Phase 3: Native runtime implementation

- [x] T010 Implement configuration, identity, lifecycle, observation, failure, and shutdown value types.
- [x] T011 Implement the owned Tokio runtime thread and explicit loopback listener startup.
- [x] T012 Implement finite connection permits, fixed buffers, non-detached tasks, and saturation accounting.
- [x] T013 Implement bounded stop/drain/force/join behavior and idempotent cached cleanup.
- [x] T014 Add internal failure injection limited to tests so panic and forced-timeout accounting is deterministic.

## Phase 4: Facade integration

- [x] T015 Add failing library integration tests that instantiate the native backend without the CLI.
- [x] T016 Expose `fragcap-proxy` through the facade's `deep-capture` feature.
- [x] T017 Implement the `ProxyBackend`/`ProxyLease` adapter with stable descriptor, empty application observations, and typed cleanup mapping.
- [x] T018 Prove controlled substitute backends and existing external CLI behavior remain unchanged.

## Phase 5: Architecture and public truth

- [x] T019 Add the complete #278 native ownership matrix, protocol/launch matrix, four milestone exits, and permanent refusals to the master specification.
- [x] T020 Record that S102 explicitly supersedes S100's external-backend end state while retaining its measurements.
- [x] T021 Update specification outline and plan index for the native completion architecture.
- [x] T022 Add prominent incomplete-status and #278 ownership language to README and contributor guidance.
- [x] T023 Update site architecture, index, getting-started, CLI, compatibility, output, and contributing pages while preserving v0.8 external behavior.
- [x] T024 Add owner links for every public limitation and provisional artifact contract.
- [x] T025 Add a dated decisions fragment and changelog fragment with specification impact.

## Phase 6: Verification

- [x] T026 Run formatting, focused crate tests, facade tests, and existing Deep Capture regressions.
- [x] T027 Run Rust 1.88 workspace build, Cargo deny, dependency direction, package verification, and Windows all-feature release build.
- [x] T028 Run full `cargo xtask ci` and documentation/site build in the foreground.
- [x] T029 Inspect the complete diff for generated files, dependency bloat, public overclaiming, UTF-8/LF integrity, and mojibake.
- [x] T030 Commit the verified slice locally and halt before push.

## Dependencies and execution order

- Phase 1 precedes compilation tests.
- Phase 2 must fail for the intended missing behavior before Phase 3 implementation.
- Phase 3 precedes facade integration.
- Documentation work can proceed after the architecture contract is stable, but must land with code.
- Verification is blocking and no push occurs in autopilot.

## Parallel opportunities

- T005 through T009 can be authored independently before implementation.
- T019 through T024 can be edited independently once the ownership matrix is fixed.
- Focused tests, documentation checks, and static audits can run concurrently only when their outputs remain readable; final `cargo xtask ci` runs alone in the foreground.
