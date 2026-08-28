# Implementation Plan: Deep Capture Compatibility Bootstrap

**Branch**: `codex/097-deep-capture-bootstrap` | **Date**: 2026-08-28 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/097-deep-capture-bootstrap/spec.md`

## Summary

Add an explicit `deep-capture --calibrate <reachability|tls> --launch-case <case>` path that lets an authorized operator collect local compatibility evidence for one unknown stored target. The command resolves and validates the target and launch state, emits a complete bounded plan, requires confirmation, then reuses the existing proxy, Capture, bundle, cleanup, fact-store, and controlled-target machinery. Reachability never touches trust. TLS requires a current same-case `proxy-routing=reached-client` observation. Ordinary Deep Capture keeps refusing insufficient evidence and consumes the same append-only facts.

S097 also corrects an existing fidelity defect: proxy traffic reaching a correlated client flow proves routing, but does not independently prove environment propagation. Real runs will no longer write `proxy-propagation=confirmed` from routing alone, and ordinary eligibility will use current final-client routing as its safety fact. The controlled target may still confirm propagation because it reports its own inherited proxy environment.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.82

**Primary Dependencies**: Existing `clap`, `serde_json`, `fragcap`, and `fragcap-targets` dependencies; external `mitmdump` remains the production proxy backend

**Storage**: Existing local SQLite `deep_capture_facts` table and Deep Capture session bundle

**Testing**: Rust unit tests, CLI integration tests, controlled loopback target tests, documentation and repository gates

**Target Platform**: Windows for real cold Steam launch and current-user CA trust; deterministic controlled verification is platform-neutral where the existing harness permits

**Project Type**: Rust workspace CLI application

**Performance Goals**: Every launch, observation, proxy shutdown, and cleanup phase has a displayed finite deadline; the command never waits indefinitely after confirmation

**Constraints**: No system proxy mutation, real trust mutation during automated tests, game account, private local evidence, new persistence path, or prohibited target instrumentation

**Scale/Scope**: One target, one declared launch case, and one declared calibration phase per invocation; one substantial slice without the library extraction, native backend spike, or direct-executable launch work

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **P-1 No covert target instrumentation**: Pass. Calibration is explicitly selected, plan-visible, confirmed, loopback-proxied, target-scoped, reversible, and audited. The design adds no injection, hooks, target memory reads, target key extraction, executable mutation, Winsock changes, interception drivers, pinning bypass, or system proxy fallback.
- **P-2 Core stays platform-neutral**: Pass. CLI orchestration and Windows trust remain in `fragcap-cli`; the optional narrow flow-registry observation API is platform-neutral and performs no I/O.
- **P-3 Capture and attribution stay separate**: Pass. Calibration reads the existing completed flow registry and emits proxy sidecars; neither acquisition nor attribution absorbs proxy behavior.
- **P-4 No silent loss**: Pass. Missing observations, partial fact writes, backend failure, interruption, and cleanup failure are explicit phase, omission, and resource outcomes.
- **P-5 Compatibility outranks richness**: Pass. `.fcapng` remains packet truth, while calibration data extends existing sidecars.
- **P-6 Glossary first**: Pass. The compatibility-calibration entry lands before public help and guidance use the term.
- **P-7 Wrappers stay thin**: Pass. No wrapper changes are planned.
- **P-8 House standards apply**: Pass. Existing format, lint, documentation, shell, and encoding gates remain unchanged.
- **P-9 The instrument does not lie**: Pass. Routing, propagation, trust, protocol, inspectability, phase outcome, and cleanup remain separate. A routing observation no longer fabricates propagation confirmation.
- **P-10 One path to a target**: Pass. Calibration uses the existing target resolver and append-only target-owned fact store.
- **P-11 The specification describes what shipped**: Pass. The master specification and public references change with the implementation.

Post-design re-check: passed. The contracts require plan-before-confirmation, phase-specific trust boundaries, observed-only fact writes, and independent finalization attempts. No constitution exception or complexity waiver is required.

## Project Structure

### Documentation (this feature)

```text
specs/097-deep-capture-bootstrap/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── calibration-cli.md
│   ├── calibration-evidence.md
│   └── calibration-events.md
├── checklists/
│   ├── calibration-safety.md
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-core/src/
└── flow.rs                         # narrow read-only completed-flow observations if required

crates/fragcap-cli/src/
├── cli.rs                          # calibration phase and launch-case arguments
├── commands/deep_capture.rs        # preflight, confirmation, phase orchestration, facts, bundle
├── emit.rs                         # plan and prompt output through the existing stderr owner
└── events.rs                       # structured calibration lifecycle records

crates/fragcap-cli/tests/
├── cli_args.rs                     # parser contract
├── cli_deep_capture.rs             # controlled end-to-end and refusal coverage
└── cli_reference.rs                # public CLI reference lock-step

docs/
├── fragcap-specification.md
├── fragcap-spec-outline.md
└── glossary/
    ├── capture-and-networking.md
    ├── command-line-and-diagnostics.md
    └── index.md

site/content/docs/
├── architecture.mdx
├── getting-started.mdx
└── reference/
    ├── cli.mdx
    ├── deep-capture-compatibility.mdx
    └── output-formats.mdx

README.md
changelog.d/S097-deep-capture-bootstrap.added.md
changelog.d/S097-deep-capture-bootstrap.decisions.md
```

**Structure Decision**: Keep S097 inside the existing CLI-owned Deep Capture module and use narrow private types and seams. This avoids preempting S098's library-first extraction while still making confirmation, classification, resource accounting, and fact mapping directly testable. No dependency or database migration is required.

## Implementation Sequence

1. Add failing parser, confirmation, classification, event, fact-mapping, and controlled end-to-end tests.
2. Add the backward-compatible calibration flags and pure preflight/plan model.
3. Emit the complete human and structured plan, require safe confirmation, and prove refusals cause zero mutation.
4. Split reachability and TLS execution around the existing proxy, capture, trust, bundle, and cleanup paths with finite deadlines.
5. Replace inferred fact writing with phase-aware pending observations and independent fact/bundle finalization.
6. Correct ordinary eligibility to require current same-case final-client routing and retain propagation only when independently observed.
7. Extend existing compatibility and cleanup sidecars plus structured events without adding another artifact authority.
8. Update the master specification, glossary, public help and guidance, and changelog fragments.
9. Run focused tests, the full repository gate, privacy and encoding checks, then commit locally.

## Complexity Tracking

No constitution violations require justification.
