# Implementation Plan: Stable Public Rust API

**Branch**: `codex/132-public-rust-api` | **Date**: 2026-09-05 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/132-public-rust-api/spec.md`

## Summary

Close issue #330 by defining `fragcap::deep_capture::api` as the curated version 1 compatibility surface, adding extensible configuration and adapter builders plus explicit cooperative cancellation, migrating the shipped CLI to consume the stable path, and proving coverage with an external-consumer contract and a runnable no-CLI production native controlled example. Preserve current runtime behavior and legacy re-exports, add no dependency, and keep issue #334 as the sole Deep Capture completion authority.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.88

**Primary Dependencies**: Existing workspace crates only, including the current `fragcap-proxy` native backend behind `deep-capture`

**Storage**: Existing target store and session bundle; no migration or new persistent format

**Testing**: Compile-time external-consumer contract, direct facade integration tests, cancellation fault cases, runnable Cargo example, CLI import coverage, doc tests, and full repository gates

**Target Platform**: Stable facade contract and controlled example compile cross-platform; production operating-system adapters retain their existing platform boundaries

**Project Type**: Rust workspace library facade plus thin CLI consumer

**Performance Goals**: Zero new unbounded waits, zero concurrent calls into one adapter set, and cancellation observed at the next coordinator boundary

**Constraints**: Preserve runtime, CLI, artifact, routing, and packaging behavior; no new dependency, backend protocol work, target effect, workflow, release action, or completion claim

**Scale/Scope**: One curated module, two public builders, one cancellation primitive, one exact inventory contract, one CLI migration, one controlled native example, synchronized specification and crate documentation, and one S131 status correction

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **P-1 No covert target instrumentation**: Pass. The example is bounded to loopback controlled peers, performs no real trust or target effect, and the API retains explicit authorization plus cleanup.
- **P-2 Core stays platform-neutral**: Pass. The change is contained in the `fragcap` facade, CLI imports, tests, examples, and documentation. `fragcap-core` gains nothing.
- **P-3 Capture and attribution stay separate**: Pass. Stable adapter contracts preserve ordinary Capture composition and expose no second packet path.
- **P-4 No silent loss**: Pass. Cancellation, observations, effect refusal, and cleanup stay explicit in terminal truth.
- **P-5 Compatibility outranks richness**: Pass. No output format changes. Curated Rust compatibility does not alter pcapng or sidecar readers.
- **P-6 Glossary first**: Pass. Existing vocabulary covers the design; any newly retained public term is added before use outside the slice.
- **P-7 Wrappers stay thin**: Pass. No wrapper changes.
- **P-8 House standards apply**: Pass. UTF-8, LF, punctuation, Markdown line, formatting, lint, tests, dependency, and license gates remain required.
- **P-9 The instrument does not lie**: Pass. Cancellation cannot erase retained observations or convert missing effects into success.
- **P-10 One path to a target**: Pass. The public builder feeds the existing resolver and store path. No alternative target form is introduced.
- **P-11 The specification describes what shipped**: Pass. The master specification, outline, roadmap, crate docs, contract, and changelog update with the code while preserving the incomplete-until-#334 statement.

Post-design re-check: passed. A curated facade module is the smallest boundary that satisfies issue #330 without freezing accidental exports. Thread confinement is documented rather than hidden or over-constrained. No constitution exception or complexity waiver is required.

## Project Structure

### Documentation (this feature)

```text
specs/132-public-rust-api/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── public-api-v1.md
├── checklists/
│   ├── requirements.md
│   └── api-stability.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap/src/deep_capture/
├── mod.rs                         # compatibility exports and stable api module
├── api.rs                         # curated version 1 inventory and contract docs
├── adapters.rs                    # adapter builder and ownership docs
├── model.rs                       # session builder, API version, and cancellation value
└── session.rs                     # cancellation checkpoints and terminal behavior

crates/fragcap/examples/
└── native-deep-capture.rs         # no-CLI production native controlled example

crates/fragcap/tests/
└── public_api.rs                  # external-consumer, inventory, cancellation, and example contract

crates/fragcap-cli/src/commands/
└── deep_capture.rs                # stable facade consumer

crates/fragcap/
└── README.md                      # canonical compatibility guidance and example

docs/
├── fragcap-specification.md
├── fragcap-spec-outline.md
└── plans/README.md

changelog.d/
├── s132-public-rust-api.feature.md
└── s132-public-rust-api.decisions.md
```

**Structure Decision**: Keep the stable contract at the existing `fragcap` facade. A named curated child module separates promised integration types from incidental top-level visibility without adding a crate, dependency edge, or alternate coordinator. Builders and cancellation extend existing values; the CLI remains a consumer rather than a second policy owner.

## Implementation Sequence

1. Add failing external-consumer and cancellation contract tests plus the example target.
2. Add the curated API module, version constant, exact reviewed inventory, and compatibility documentation.
3. Add extensible `SessionConfigBuilder` and `AdapterSetBuilder` construction paths with typed build errors.
4. Add the cloneable cooperative cancellation token and coordinator checkpoints without changing in-flight adapter budget behavior.
5. Migrate the CLI's Deep Capture policy imports to the stable facade path and add the zero-bypass coverage assertion.
6. Implement and execute the production native controlled example with exact cleanup.
7. Synchronize the master specification, outline, roadmap, crate README, changelog fragments, and S131 status.
8. Run analyze, focused gates, the example, doc tests, full `cargo xtask ci`, diff/text hygiene, and lockfile checks.

## Complexity Tracking

No constitution violations require justification.
