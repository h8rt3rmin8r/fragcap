# Implementation Plan: Smaller Native Proxy Fallback Spike

**Branch**: `codex/100-http-mitm-proxy-spike` | **Date**: 2026-08-30 | **Spec**: [spec.md](spec.md)

## Summary

Build a self-contained, non-shipping Windows research harness around `http-mitm-proxy` 0.18.0 and run the S099 controlled traffic and audit contract. Preserve explicit protocol, body, WebSocket, lifecycle, certificate-cache, HAR-source, and proxy-owned key-log evidence. Compare it with committed `hudsucker` and `mitmdump` evidence, then select exactly one backend outcome.

## Technical Context

**Language/Version**: Rust 2024 candidate crate; Rust 1.82 and pinned Rust 1.96 measurements

**Primary Dependencies**: Exact `http-mitm-proxy = "=0.18.0"`, defaults off, `native-tls-client`; isolated S099 harness support crates

**Storage**: Temporary private run directories; sanitized JSON and Markdown evidence

**Testing**: Isolated Cargo tests and runs, ten lifecycle trials, dependency and license audits, full repository gates

**Target Platform**: Windows 11 x86_64-pc-windows-msvc, loopback only

**Project Type**: Disposable nested Cargo workspace outside the released graph

**Performance Goals**: Bounded startup, cancellation, cleanup, cache, clean build, and warm build measurements

**Constraints**: No product dependency, proxy default, trust-store, system-proxy, validation, or release change

**Scale/Scope**: One exact fallback, four protocol families, one audit, one comparison, one outcome

## Constitution Check

*GATE: Passed before research and after design.*

- **P-1**: Explicit loopback clients only; no target instrumentation, redirector, silent trust, pinning bypass, or key extraction.
- **P-2/P-3**: Nested workspace changes no core, capture, attribution, or product dependency edge.
- **P-4/P-9**: Every proof point is seeded; negative states stay visible; parity requires complete protocol, length, and digest agreement.
- **P-5/P-10**: No shipped output, target, or resolution contract changes.
- **P-6/P-7**: No new product term or shipping wrapper.
- **P-8**: SPDX, formatting, encoding, policy, and full CI gates apply.
- **P-11**: Documentation records research only; `mitmdump` remains shipped.

No exception is required.

## Project Structure

```text
specs/100-http-mitm-proxy-spike/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/{evidence,harness}.md
├── checklists/{requirements,research-integrity}.md
└── tasks.md

spikes/http-mitm-proxy/
├── Cargo.toml
├── Cargo.lock
├── audit/{Cargo.toml,Cargo.lock,src/lib.rs}
├── README.md
├── deny.toml
├── src/{main,candidate,scenario,evidence}.rs
└── tests/matrix.rs
```

**Structure Decision**: Preserve S099 as historical evidence and create a second isolated workspace. Reuse its scenario identities and evidence meanings without coupling the disposable crates. Keep a minimal candidate-only audit manifest separate from harness support dependencies. Commit only sanitized summaries.

## Implementation Sequence

1. Freeze root graph hashes and create isolated candidate and audit workspaces.
2. Adapt S099 evidence and safety tests before candidate implementation.
3. Implement public request, response, CONNECT, HTTP/2, upgrade, cache, and listener paths.
4. Run the matrix and ten lifecycle trials, retaining all negative results.
5. Audit dependency paths, licenses, advisories, toolchains, build cost, and root isolation.
6. Join results to S099 evidence and select one backend outcome.
7. Update research, master specification, outline, and changelog.
8. Run all gates, commit locally, and halt before push.

## Complexity Tracking

No constitution violations require justification.
