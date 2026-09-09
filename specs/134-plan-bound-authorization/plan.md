# Implementation Plan: Plan-Bound Deep Capture Authorization

**Branch**: `codex/134-plan-bound-authorization` | **Date**: 2026-09-09 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/134-plan-bound-authorization/spec.md`

## Summary

Close issue #382 by replacing Deep Capture's overlapping `--trust-ca` and `--yes` gates with one canonical authorization plan. Resolve the exact target and cold launch authority, select the bundle and bounded policy, prepare the session CA only in memory, emit the complete human or structured plan, and accept either one interactive affirmative answer or the exact plan identifier from dedicated standard input. Only after authorization may the existing library preflight reserve the listener and prepare Capture. Bind the library `PlanId` and native runtime to the same canonical plan and certificate, refuse all drift, retain hidden legacy flags only as one-release migration errors, and preserve unrelated confirmation models.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.88

**Primary Dependencies**: Existing `fragcap-cli`, `fragcap` facade, and `fragcap-proxy` crates; existing workspace `serde_json`, `blake3`, `subtle`, `ring`, `rcgen`, and `zeroize` packages

**Storage**: No new persistent store or schema; the authorization plan and prepared certificate authority remain process-local until authorization, and the existing session bundle begins only afterward

**Testing**: Pure canonical-plan and exact-input unit tests, CLI contract tests through an injected exact-plan authorizer, controlled Deep Capture integration tests, native runtime certificate-identity tests, existing cleanup and recovery suites, and full repository gates

**Target Platform**: Authorization planning and controlled tests are cross-platform; production current-user Root-store effects remain Windows-only behind the existing adapter

**Project Type**: Rust workspace library plus thin CLI consumer

**Performance Goals**: One bounded canonical serialization and digest per prepared plan; no filesystem scan, network request, target launch, listener bind, or bundle write before authorization

**Constraints**: No generic approval, no persistent consent, no private key serialization, no system proxy fallback, no real trust mutation in ordinary tests, no public API removal, no new lockfile package, and no scope from #379, #380, #332, or #331 through #334

**Scale/Scope**: One versioned authorization-plan contract, one exact-input authorizer seam, one prepared native CA handoff, one deterministic library plan id, two hidden legacy migration errors, structured events, focused documentation, and security regression coverage

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **P-1 No covert target instrumentation**: Pass. S134 strengthens explicit Deep Capture activation, binds consent to one target-scoped plan and certificate, delays listener and trust effects until authorization, and adds no prohibited technique.
- **P-2 Core stays platform-neutral**: Pass. No change enters `fragcap-core`; plan presentation stays in the CLI, and native certificate ownership stays in the existing facade and proxy leaves.
- **P-3 Capture and attribution stay separate**: Pass. No packet-source or attribution seam changes.
- **P-4 No silent loss**: Pass. Authorization refusal occurs before observation begins, while existing packet and application loss accounting remains unchanged.
- **P-5 Compatibility outranks richness**: Pass. Capture and bundle formats are unchanged; structured authorization adds versioned events without changing analyzer-readable artifacts.
- **P-6 Glossary first**: Pass. `plan-bound authorization`, `session plan`, `certificate authority`, and `trust store` already exist in the architecture of record. No new domain term is required.
- **P-7 Wrappers stay thin**: Pass. No shell wrapper changes.
- **P-8 House standards apply**: Pass. UTF-8 without BOM, LF, formatting, lint, tests, docs, dependency, privacy, and changelog gates remain blocking.
- **P-9 The instrument does not lie**: Pass. The plan reports the exact target, certificate, scope, paths, artifacts, and deadlines; reachability says trust is absent; drift is refused rather than normalized.
- **P-10 One path to a target**: Pass. Target resolution and compatibility remain in the existing store and facade path; planning retains that exact result without a new target representation in storage.
- **P-11 The specification describes what shipped**: Pass. Master specification, outline, site references, plan ordering, and changelog change with the implementation while the broader documentation issue stays open.

Post-design re-check: passed. The CLI plan is the presentation and input owner, while the existing facade `PreparedSession` remains the sole lifecycle owner. Its `PlanId` is supplied from the canonical authorization identifier, and the runtime consumes the same process-local CA whose thumbprint the plan displayed. No second execution coordinator, persistent consent object, or constitution exception is introduced.

## Project Structure

### Documentation (this feature)

```text
specs/134-plan-bound-authorization/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── deep-capture-authorization.md
├── checklists/
│   ├── requirements.md
│   └── authorization-security.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-proxy/src/
└── runtime.rs                         # accept one process-local prepared session CA

crates/fragcap/src/deep_capture/
├── model.rs                           # retain canonical PlanId contract
└── native.rs                          # prepare, summarize, and hand the exact CA to runtime

crates/fragcap/tests/
├── deep_capture_session.rs            # exact plan authorization remains single-use
└── native_proxy.rs                    # displayed and runtime CA identities agree

crates/fragcap-cli/src/
├── cli.rs                             # --authorize-stdin plus hidden legacy migration inputs
├── commands/deep_capture.rs           # canonical plan, rendering, authorizer, drift gate
├── events.rs                          # versioned plan and authorization outcome events
└── lib.rs                             # injectable input/authorizer path for tier-1 tests

crates/fragcap-cli/tests/
└── cli_deep_capture.rs                # human, JSON, legacy, refusal, drift, and automation contracts

docs/
├── fragcap-specification.md
├── fragcap-spec-outline.md
└── plans/README.md

site/content/docs/
├── architecture.mdx
├── getting-started.mdx
└── reference/
    ├── cli.mdx
    └── deep-capture-compatibility.mdx

changelog.d/
├── 382-plan-bound-authorization.added.md
└── s134-plan-bound-authorization.decisions.md
```

**Structure Decision**: Keep canonical plan construction, human and structured rendering, and standard-input authorization in `fragcap-cli`, which owns the command contract. Extend the existing native facade and proxy adapter only far enough to pre-generate and retain the exact CA named by that plan. Continue to execute through the S132 facade coordinator, whose `PlanId` receives the canonical digest.

## Implementation Sequence

1. Add failing canonical-plan, exact-input, help, legacy migration, no-effect refusal, reachability, drift, and native certificate-identity tests.
2. Add a process-local prepared native CA value and teach the runtime to consume it exactly once while preserving its current fallback for existing library consumers.
3. Add the versioned canonical authorization plan, deterministic digest, complete human and JSON rendering, and exact-input authorizer seam.
4. Move ordinary and calibration authorization ahead of facade preflight, bind the facade plan id to the digest, revalidate target launch authority, and retain one final plan decision after warm restart.
5. Hide and reject legacy Deep Capture flags with migration guidance while preserving Doctor, bundle, and target-reconciliation `--yes` behavior.
6. Synchronize specification, outline, roadmap, site references, changelog, and the controlled validation guide.
7. Run analyze, focused security and lifecycle tests, full CI parity, dependency stability, encoding and mojibake checks, and diff review.

## Complexity Tracking

No constitution violations require justification.
