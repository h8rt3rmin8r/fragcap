# Quickstart: Guided Calibration Case Proposals

## Prerequisites

- Work from `codex/s139-calibration-case-proposals`.
- `.specify/feature.json` points to `specs/139-calibration-case-proposals` and remains unstaged.
- Use synthetic target entries, process snapshots, and compatibility facts only. No game, capture driver, elevation, network listener, or trust mutation is required.

## Focused validation

Run the facade tests for proposal policy and stable API inventory:

```sh
cargo test -p fragcap deep_capture::proposal
cargo test -p fragcap deep_capture::api
```

Expected result: Steam, direct, publisher, warm, ambiguous, unavailable, exact-fact, conflict, ordering, and override cases pass offline.

## Repository validation

Run the repository-owned gate in the foreground:

```sh
cargo xtask ci
```

Expected result: format, lint, dependency, documentation, security, stable API, unit, integration, and platform-neutral checks pass.

## Manual review

Review the proposal module and confirm:

- It imports no platform process API and invokes no adapter.
- It has no store, filesystem, network, clock, trust, listener, launch, or artifact write path.
- Missing process authority cannot produce `Ready`.
- Current conflicting facts remain retest-worthy.
- Reachability is the only runnable step before exact final-client routing.
- Issue #392 closes on merge while parent #380 remains open.
