# Implementation Plan: Cold Platform-Client Ownership

**Branch**: `codex/112-platform-client-ownership` | **Date**: 2026-09-01 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/112-platform-client-ownership/spec.md`

## Summary

S112 closes issue #308 by replacing the unsupported Deep Capture Steam protocol boundary with an exact cold platform plan. The existing Steam integration resolves the installed platform root and application identifier. A reusable platform adapter prepares the canonical `steam.exe`, a root start action, and a separate application dispatch action. Ordinary Capture is already armed before launch. It starts the exact platform root with the S109 child-only route, binds the observed platform role, dispatches the application only after that binding, and then uses the S111 exact ancestry and terminal ownership rules for the selected client.

Warm, uncertain, escaped, ambiguous, missing, and prematurely exited paths remain named failures. Existing routing and propagation facts stay separate: client reachability describes what reached the proxy, while propagation is confirmed only by a final-client proxy connection reconciled under the owned platform ancestry. Capture's existing protocol-handler behavior remains unchanged; the owned adapter is selected only by Deep Capture preparation.

## Technical Context

**Language/Version**: Rust 1.96.0 pinned, Rust 1.88 minimum supported version

**Primary Dependencies**: Existing standard library process API, `fragcap-steam` registry and installation discovery, `fragcap-profile` stage predicates, `fragcap-core` process tree, S108 flow correlation, S109 routing, and S111 exact stage ownership

**Storage**: Existing target store compatibility facts and session bundle artifacts; no schema migration

**Testing**: Rust unit tests, controlled offline process timelines, CLI integration tests, existing controlled native proxy harness, and full repository CI parity

**Target Platform**: Windows 10 and later for the real Steam adapter, with platform-neutral plan and reconciliation tests

**Project Type**: Rust workspace library and CLI

**Performance Goals**: Linear reconciliation over the bounded declared process chain; zero polling and zero packet-path work added; title dispatch occurs once

**Constraints**: No target process handle, no shell, no injection or hooks, no executable modification, no global proxy, no silent event loss, exact canonical platform identity, observe-before-dispatch, immutable pre-effect preparation, finite deadline, and no new dependency

**Scale/Scope**: One selected target, one exact cold platform root, one application dispatch, bounded intermediates and helpers, one terminal client, and one session

## Constitution Check

### Pre-design gate

- **P-1 No Covert Target Instrumentation**: PASS. The design uses exact ordinary child creation, ETW process events, socket attribution, and child-only proxy environment. It adds no process handle, memory right, shell, hook, driver, image mutation, target key extraction, or global proxy mutation.
- **P-2 Core Stays Platform-Neutral**: PASS. No platform dependency enters `fragcap-core`. Platform plan values and adapters live in the facade and existing Steam crate.
- **P-3 Capture And Attribution Stay Separate**: PASS. Process ownership gates dispatch and socket attribution remains an independent existing input.
- **P-4 No Silent Loss**: PASS. Watcher and evidence losses remain named and counted; escaped and unlocalized lifecycle events cannot disappear into success.
- **P-5 Compatibility Outranks Richness**: PASS. Existing pcapng, JSON Lines, HAR, and bundle readers remain compatible.
- **P-6 Glossary First**: PASS. The existing platform client, managed launch, routing, and propagation vocabulary is reused. Any new durable term will enter the glossary before use.
- **P-7 Wrappers Stay Thin**: PASS. No wrapper changes are planned.
- **P-8 House Standards Apply**: PASS. Source and documentation remain subject to repository format, lint, encoding, and prose gates.
- **P-9 The Instrument Does Not Lie**: PASS. Cold ownership, routing, and propagation require independent positive evidence. Warm, escaped, missing, lost, and ambiguous outcomes remain distinct.
- **P-10 One Path To A Target**: PASS. The selected stored target and existing resolver remain authoritative. The platform adapter adds no target storage, precedence, or platform-only selector.
- **P-11 The Specification Describes What Shipped**: PASS. The master specification, outline, plan index, and agent guide advance in this slice without claiming warm restart or completion.

### Post-design gate

All checks remain PASS. Research rejects protocol-handler launch as the ownership boundary, delayed platform discovery after proxy startup, global proxy fallback, process-path queries through target handles, and a Steam-specific Deep Capture coordinator. The selected design extends the existing managed-launch value, Capture preparation, and process session in dependency order.

## Project Structure

### Documentation (this feature)

```text
specs/112-platform-client-ownership/
├── checklists/
│   ├── requirements.md
│   └── security.md
├── contracts/
│   └── platform-launch-api.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-steam/src/
├── launch.rs
└── lib.rs

crates/fragcap/src/
├── managed_launch.rs
└── session.rs

crates/fragcap-cli/src/
├── assemble.rs
├── orchestrator.rs
└── commands/
    ├── capture.rs
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
├── S112-platform-client-ownership.added.md
└── S112-platform-client-ownership.decisions.md
```

**Structure Decision**: Add a reusable value-producing platform adapter at the existing facade managed-launch seam. Deep Capture requests owned platform preparation explicitly, while ordinary Capture continues to build its existing Steam protocol request. The ordinary Capture orchestrator already owns the armed watcher, process event loop, session state, and managed launch timing, so it is the sole authority that may transition from observed platform ownership to title dispatch.

## Complexity Tracking

No constitution violation or exceptional complexity waiver is required.
