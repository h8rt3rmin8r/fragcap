# Implementation Plan: Native Proxy Backend Spike

**Branch**: `codex/099-native-proxy-spike` | **Date**: 2026-08-29 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/099-native-proxy-spike/spec.md`

## Summary

Build a self-contained, non-shipping Windows research harness around the issue-mandated `hudsucker` 0.23.0 feature set and compare it with the installed `mitmdump` baseline over one controlled local traffic matrix. Preserve complete request, response, protocol, WebSocket, lifecycle, certificate-cache, HAR-source, and proxy-owned key-log evidence. Audit the isolated dependency graph, licenses, Rust 1.82 behavior, build cost, and product-graph isolation. Record one of the four permitted backend decisions and one bounded follow-up without changing fragcap's shipping backend.

## Technical Context

**Language/Version**: Rust 2021; candidate declares Rust 1.75; measure with workspace MSRV 1.82 and pinned Rust 1.96

**Primary Dependencies**: Isolated `hudsucker = "=0.23.0"` with defaults off and `decoder`, `http2`, `native-tls-client`, and `rcgen-ca`; test-only local protocol support; external `mitmdump` 12.2.3 baseline

**Storage**: Ephemeral temporary directories for private CA material and raw run logs; sanitized JSON evidence and Markdown decision records for committed output

**Testing**: Isolated Cargo tests and harness runs, repeated lifecycle trials, baseline comparison, dependency and license audits, then full repository gates

**Target Platform**: Windows 11 x86_64-pc-windows-msvc on loopback only

**Project Type**: Disposable research binary and integration harness outside the released Cargo workspace

**Performance Goals**: Ten bounded startup and shutdown trials; clean and warm build timings; no unbounded connection drain or cache

**Constraints**: No shipping dependency or backend change; no system proxy mutation; no trust-store mutation; no uncontrolled remote traffic; no committed secrets or raw captures; no missing observation treated as parity

**Scale/Scope**: One native candidate version, one installed baseline version, four protocol families, one isolated harness, one evidence record, and one backend decision

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **P-1 No covert target instrumentation**: Pass. Both runs accept only explicit loopback proxy traffic from a controlled client. No target instrumentation, system proxy mutation, trust mutation, interception driver, pinning bypass, or target key extraction is used.
- **P-2 Core stays platform-neutral**: Pass. The spike is a separate nested workspace and changes no product crate or dependency edge.
- **P-3 Capture and attribution stay separate**: Pass. The spike tests only the S098 proxy adapter boundary and adds no packet acquisition or attribution code.
- **P-4 No silent loss**: Pass. Body and message outcomes distinguish complete, empty, bounded, truncated, failed, unsupported, and not measured.
- **P-5 Compatibility outranks richness**: Pass. The spike evaluates application observations and analyzer aids without changing `.fcapng` or bundle contracts.
- **P-6 Glossary first**: Pass. No new product term is planned; any term introduced by the decision record updates the glossary first.
- **P-7 Wrappers stay thin**: Pass. The experiment is a Rust harness, not a product wrapper.
- **P-8 House standards apply**: Pass. Committed source carries SPDX headers and repository formatting, encoding, documentation, license, and full CI gates still run.
- **P-9 The instrument does not lie**: Pass. Raw local evidence is summarized without inventing parity, suppressing failures, or normalizing away protocol differences.
- **P-10 One path to a target**: Pass. No target storage or resolution path changes.
- **P-11 The specification describes what shipped**: Pass. The shipping backend remains `mitmdump`; the master specification changes only to record the non-shipping decision and follow-up boundary.

Post-design re-check: passed. The nested workspace makes the graph boundary mechanical, the normalized evidence contract preserves negative results, and temporary private material never becomes a repository artifact. No constitution exception is required.

## Project Structure

### Documentation (this feature)

```text
specs/099-native-proxy-spike/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── evidence.md
│   └── harness.md
├── checklists/
│   ├── requirements.md
│   └── research-integrity.md
└── tasks.md
```

### Source Code (repository root)

```text
spikes/native-proxy/
├── Cargo.toml
├── Cargo.lock
├── audit/
│   ├── Cargo.toml
│   ├── Cargo.lock
│   └── src/lib.rs
├── README.md
├── deny.toml
├── src/
│   ├── main.rs
│   ├── candidate.rs
│   ├── baseline.rs
│   ├── scenario.rs
│   └── evidence.rs
└── tests/
    └── matrix.rs

docs/plans/
└── deep-capture-proxy-backends.md

docs/
├── fragcap-specification.md
└── fragcap-spec-outline.md

changelog.d/
└── S099-native-proxy-spike.decisions.md
```

**Structure Decision**: Put the executable research artifact in a nested Cargo workspace under `spikes/native-proxy`. This deliberately avoids a shipping crate example because examples participate in that package's dependency and release surface. A second minimal nested audit manifest contains only the candidate feature set, because the runnable harness necessarily adds client, server, serialization, and test packages that are not part of a product adoption delta. Both receive their own locks while the product workspace lock and manifests remain byte-identical. Raw evidence and private certificate material stay in ignored temporary directories; only sanitized aggregate results enter the slice and planning record.

## Implementation Sequence

1. Freeze the released workspace metadata and lock-file hashes, then create the nested spike workspace with exact versions and the repository license policy.
2. Add failing contract tests for normalized negative states, sanitized evidence, loopback-only addresses, bounded lifecycle, and product-graph isolation.
3. Implement controlled local HTTP/1.1, HTTPS, HTTP/2, and WebSocket scenarios with fixed payloads and no remote dependency.
4. Implement the `hudsucker` candidate adapter with full body buffering for measurement, explicit completeness metadata, bounded cache, a public-API key-log CA wrapper, and owned cancellation.
5. Implement the external `mitmdump` baseline adapter and normalized observation ingestion without changing system proxy or trust state.
6. Run the common matrix, repeat startup and shutdown ten times, and record failures or unsupported cases without inference.
7. Run dependency, target-conditional, license, Rust 1.82, pinned-toolchain, clean-build, warm-build, and size measurements against the nested workspace.
8. Update the evidence record, backend research plan, specification question, outline, and changelog with exactly one decision and one follow-up boundary.
9. Run isolated and full repository gates, audit private material and graph isolation, commit locally, and halt before push.

## Complexity Tracking

No constitution violations require justification.
