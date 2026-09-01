# Implementation Plan: Managed Publisher-Launcher Chains

**Branch**: `codex/111-publisher-launcher-chains` | **Date**: 2026-09-01 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/111-publisher-launcher-chains/spec.md`

## Summary

S111 closes issue #307 by extending the existing shared managed-launch value from one direct executable to one exact, ordered publisher chain. Preparation continues to happen in the Capture path before Deep Capture starts proxy, trust, routing, or launch effects. The stored target remains the sole identity source. A prepared publisher value retains the exact root executable, arguments, ordered declared stages, and a validated Capture profile. The existing process session performs creation-time ancestry matching and terminal lifecycle ownership once the root is launched with the S109 child-only route.

Only a proven fully cold chain is supported. A warm launcher, game-start-clean warm launcher, ambiguous declaration, escaped descendant, or incomplete observation remains a named refusal or inconclusive result. No new dependency, storage shape, process handle, shell, or global routing mechanism is introduced.

## Technical Context

**Language/Version**: Rust 1.96.0 pinned, Rust 1.88 minimum supported version

**Primary Dependencies**: Existing standard library process API, `fragcap-profile` stage matching, `fragcap-core` process tree, `fragcap-targets` stored launch entries, and S109 Deep Capture routing and lifecycle contracts

**Storage**: Existing local target-store `launch_entries` JSON and existing session bundle artifacts; no schema migration

**Testing**: Rust unit tests, controlled offline process timelines, Windows child-process launch probe, CLI integration tests, and the full repository CI parity suite

**Target Platform**: Windows 10 and later for real managed launch, with platform-neutral preparation and reconciliation tests where filesystem semantics permit

**Project Type**: Rust workspace library and CLI

**Performance Goals**: Linear preparation and reconciliation over a bounded publisher chain; no polling or packet-path work added

**Constraints**: No target process handle, no shell, no injection or hooks, no executable modification, no hidden global proxy, no silent event loss, exact stored path identity, relative-path containment beneath the install root, immutable pre-effect preparation, and no new runtime dependency

**Scale/Scope**: One selected target, one root publisher launcher, zero or more ordered intermediate stages, one terminal client, and bounded competing observations per session

## Constitution Check

### Pre-design gate

- **P-1 No Covert Target Instrumentation**: PASS. The design launches one exact stored executable through the standard process API, observes process creation externally, and uses child-only environment routing. It adds no process handle, memory right, shell, hook, driver, image mutation, or target key extraction.
- **P-2 Core Stays Platform-Neutral**: PASS. No platform dependency enters `fragcap-core`. Publisher preparation remains in the facade and target parsing remains value-only.
- **P-3 Capture And Attribution Stay Separate**: PASS. Process-stage binding and socket attribution remain separate existing seams.
- **P-4 No Silent Loss**: PASS. Existing watcher loss remains surfaced, and any bounded chain reconciliation overflow receives its own exact count.
- **P-5 Compatibility Outranks Richness**: PASS. Existing pcapng and application artifacts remain readable by their current consumers.
- **P-6 Glossary First**: PASS. Existing terms are reused. Any new durable term introduced during implementation will receive a glossary entry before use.
- **P-7 Wrappers Stay Thin**: PASS. No wrapper changes are planned.
- **P-8 House Standards Apply**: PASS. All artifacts and source changes remain under repository lint, formatting, encoding, and prose gates.
- **P-9 The Instrument Does Not Lie**: PASS. Warm, escaped, ambiguous, and missing outcomes remain distinct and no client identity is inferred from silence or image name alone.
- **P-10 One Path To A Target**: PASS. Existing target entries and resolution supply the chain. No publisher-specific store or precedence path is added.
- **P-11 The Specification Describes What Shipped**: PASS. The master specification and outline will be updated in the same slice, while Deep Capture remains incomplete.

### Post-design gate

All checks remain PASS. Research rejects a separate publisher orchestrator, a second target schema, and late CLI reconstruction. The selected design extends existing values and seams in dependency order.

## Project Structure

### Documentation (this feature)

```text
specs/111-publisher-launcher-chains/
├── checklists/
│   ├── requirements.md
│   └── security.md
├── contracts/
│   └── publisher-chain-api.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-targets/src/
├── hint_provider.rs
└── model.rs

crates/fragcap/src/
├── managed_launch.rs
└── deep_capture/
    └── policy.rs

crates/fragcap-cli/src/
├── assemble.rs
└── commands/
    ├── deep_capture.rs
    └── target_resolve.rs

crates/fragcap-cli/tests/
├── cli_capture.rs
└── cli_deep_capture.rs

docs/
├── fragcap-spec-outline.md
├── fragcap-specification.md
└── plans/README.md

changelog.d/
├── S111-publisher-launcher-chains.added.md
└── S111-publisher-launcher-chains.decisions.md
```

**Structure Decision**: Extend the existing target parsing, facade managed-launch value, shared Capture preparation, and Deep Capture policy. The ordinary `CaptureSession` already owns ordered process matching, creation-time ancestry, intermediate lifecycle, terminal exit, and role publication, so duplicating that state machine inside Deep Capture would create two authorities and is rejected.

## Complexity Tracking

No constitution violation or exceptional complexity waiver is required.
