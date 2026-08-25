# Tasks: Deep Capture session bundle

**Input**: Design documents from `/specs/073-deep-capture-session-bundle/`

**Prerequisites**: plan.md, research.md, data-model.md, analysis.md, quickstart.md, `contracts/manifest.md`, `contracts/application-jsonl.md`, `contracts/example-bundle.md`

## Phase 1: Setup

- [X] T001 Re-read issue #216, `docs/plans/deep-capture.md`, `docs/fragcap-specification.md` output-format and roadmap sections, and the prior compatibility-facts slice.

## Phase 2: Bundle Model

- [X] T002 Define the session manifest as the required bundle index.
- [X] T003 Define artifact roles, authorities, sensitivity levels, and omission rules.
- [X] T004 Define cleanup resource names and cleanup statuses for issue #218.

## Phase 3: Correlation Model

- [X] T005 Define required anchors for application records.
- [X] T006 Define how sidecars join to packet flows and process/role context.
- [X] T007 Define unavailable-attribution behavior without silent omission.

## Phase 4: Output Semantics

- [X] T008 Define `.fcapng` as packet truth rather than decrypted object storage.
- [X] T009 Define application JSONL as canonical application event stream.
- [X] T010 Define HAR as HTTP-observability output available to both Capture and Deep Capture where applicable.
- [X] T011 Define TLS key logs as sensitive proxy-owned analyzer aids.

## Phase 5: Documentation and Verification

- [X] T012 Add contracts and one complete example bundle layout.
- [X] T013 Update the master specification.
- [X] T014 Add a changelog decision fragment.
- [X] T015 Run `cargo fmt --check`.
- [X] T016 Run `git diff --check`.
- [X] T017 Run `cargo xtask lint`.
- [X] T018 Run `cargo xtask deps`.
- [X] T019 Run `cargo xtask spec`.
- [X] T020 Run `cargo xtask changelog --check`.
- [X] T021 Scan new/touched public artifacts for local paths, endpoints, account material, and real local title names from fact-finding.
- [X] T022 Run `cargo test --workspace --quiet`.
