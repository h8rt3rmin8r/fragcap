# Implementation Plan: Library-First Deep Capture Sessions

**Branch**: `codex/098-library-deep-capture` | **Date**: 2026-08-29 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/098-library-deep-capture/spec.md`

## Summary

Move Deep Capture policy and lifecycle ownership out of the 3,700-line CLI command into a public, split `fragcap::deep_capture` facade module. Introduce side-effect-free preflight, prepared-plan authorization, a checked session state machine, typed events and terminal reports, narrow effect adapters, at-most-once resource leases, and post-cleanup immutable bundle finalization. Lift ordinary Capture's reusable prepared execution seam into the facade so both commands use one acquisition and attribution path. Preserve the v0.7 command, event, fact, bundle, refusal, and exit contracts while reducing the CLI to mapping, prompting, presentation, and exit handling.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.82

**Primary Dependencies**: Existing workspace crates, `serde_json`, optional `windows-sys`, and the external production `mitmdump` process

**Storage**: Existing target-owned SQLite compatibility facts and Deep Capture filesystem bundle

**Testing**: Direct facade integration tests with controlled adapters, retained CLI integration tests, unit tests, and full repository gates

**Target Platform**: Public coordinator and controlled adapters are platform-neutral; production trust and process adapters target Windows

**Project Type**: Rust workspace library facade plus CLI application

**Performance Goals**: No unbounded lifecycle wait; no duplicated side effects; controlled tests complete without external processes or privileged access

**Constraints**: Preserve public behavior; no new packet path, system proxy mutation, real trust mutation in tests, native backend, direct-executable launch, or target instrumentation

**Scale/Scope**: One target and one launch case per session; one facade module, one lifted ordinary Capture seam, a thin CLI adapter, direct fault-injection coverage, and synchronized public documentation

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **P-1 No covert target instrumentation**: Pass. The library preserves explicit prepared-plan authorization, loopback scope, optional visible trust, bounded owned resources, and audited cleanup. No prohibited instrumentation or pinning bypass is added.
- **P-2 Core stays platform-neutral**: Pass. Orchestration lives in the facade. `fragcap-core` gains no platform, proxy, trust, target-store, or I/O concern.
- **P-3 Capture and attribution stay separate**: Pass. The existing ordinary Capture composition is lifted for reuse; Deep Capture adds no acquisition or attribution implementation.
- **P-4 No silent loss**: Pass. Observations, event gaps, fact failures, artifact omissions, deadline failures, and every cleanup result remain typed and visible.
- **P-5 Compatibility outranks richness**: Pass. `.fcapng` remains packet truth and sidecars remain explicit auxiliary evidence.
- **P-6 Glossary first**: Pass. Existing terms are retained; any new public lifecycle term will update the glossary before broader guidance.
- **P-7 Wrappers stay thin**: Pass. No wrapper changes are planned.
- **P-8 House standards apply**: Pass. The full formatting, lint, test, documentation, encoding, license, and MSRV gates remain required.
- **P-9 The instrument does not lie**: Pass. One post-cleanup immutable snapshot drives terminal authorities. Missing evidence stays inconclusive, and late storage or event failures cannot be reported as complete.
- **P-10 One path to a target**: Pass. Preflight uses the existing resolver once, and the selected target remains the sole append-only fact owner.
- **P-11 The specification describes what shipped**: Pass. Master specification, outline, public API documentation, site reference, and changelog change with implementation.

Post-design re-check: passed. The split facade module preserves the dependency graph, and the feature design keeps live capture independent so controlled consumers remain offline-capable. No constitution exception or complexity waiver is required.

## Project Structure

### Documentation (this feature)

```text
specs/098-library-deep-capture/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── adapters.md
│   ├── events-and-results.md
│   └── public-session-api.md
├── checklists/
│   ├── lifecycle-safety.md
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap/src/
├── lib.rs
├── capture.rs                         # reusable ordinary Capture preparation/execution seam
└── deep_capture/
    ├── mod.rs                         # documented public surface
    ├── model.rs                       # config, plan, events, reports, reason codes
    ├── adapters.rs                    # narrow public traits and effect inputs/results
    ├── session.rs                     # checked coordinator and resource ownership
    ├── policy.rs                      # observation classification and fact selection
    ├── bundle.rs                      # immutable snapshot rendering and persistence
    ├── proxy.rs                       # production mitmdump adapter
    ├── trust.rs                       # production current-user trust adapter
    └── store.rs                       # existing target-store fact adapter

crates/fragcap/tests/
└── deep_capture_session.rs            # controlled public-API transition and fault matrix

crates/fragcap-cli/src/
├── commands/capture.rs                # consumes facade ordinary Capture seam
├── commands/deep_capture.rs           # argument mapping, confirmation, presentation, exit mapping
├── emit.rs                            # human presentation
└── events.rs                          # pure library-event JSON rendering

crates/fragcap-cli/tests/
└── cli_deep_capture.rs                # retained outer compatibility contract

docs/
├── fragcap-specification.md
├── fragcap-spec-outline.md
└── glossary/

site/content/docs/
├── architecture.mdx
└── reference/
    ├── cli.mdx
    ├── deep-capture-compatibility.mdx
    └── output-formats.mdx

changelog.d/
├── S098-library-deep-capture.added.md
└── S098-library-deep-capture.decisions.md
```

**Structure Decision**: Use the existing `fragcap` facade as the assembly and public product boundary. Split policy, models, adapters, production effects, and bundle rendering by responsibility. Lift ordinary Capture reuse into that same facade. Keep only command-interface responsibilities in `fragcap-cli`. No new workspace crate or dependency edge is required.

## Implementation Sequence

1. Add facade feature metadata and failing public contract tests for preflight, state transitions, authorization, failures, cleanup, events, facts, and artifacts.
2. Lift the ordinary Capture prepared execution seam out of CLI-private ownership and keep Capture command behavior unchanged.
3. Add typed Deep Capture models, adapter traits, prepared preflight, plan binding, and coordinator state machine.
4. Move evidence classification and fact selection into pure facade policy with existing v0.7 semantics.
5. Move production proxy, trust, fact-store, and bundle implementations into the facade behind narrow traits.
6. Implement independent fact attempts, at-most-once cleanup, immutable terminal snapshot creation, and manifest-last artifact finalization.
7. Replace CLI orchestration with mapping, plan presentation and confirmation, event rendering, and exit mapping.
8. Run direct fault injection and CLI parity tests, then update master specification, API docs, site guidance, and changelog.
9. Run focused and full gates, audit the diff, commit locally, and halt before push.

## Complexity Tracking

No constitution violations require justification.
