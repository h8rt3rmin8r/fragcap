# Implementation Plan: Launcher TLS Refusal Diagnosis

**Branch**: `codex/s198-launcher-tls-refusals` | **Date**: 2026-11-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/198-launcher-tls-refusals/spec.md`.

## Summary

Close #574 by giving observable client-facing TLS transport endings and structured malformed TLS messages stable, truthful refusal codes while retaining existing boundary, alert, and transport evidence. A controlled launcher/updater journey proves explicit launch-chain proxy compatibility and checks exact process-owner correlation, failed-inspection truth, and the separate after-exchange HTTP timeout class. The private installed-title cause remains unclaimed.

## Technical Context

**Language/Version**: Rust workspace with MSRV 1.88 and pinned development toolchain 1.96.0; Markdown documentation

**Primary Dependencies**: Existing rustls 0.23.46, tokio-rustls, native proxy/facade crates, and controlled test dependencies only

**Storage**: Existing bounded connection diagnostic version 1 and application/lifecycle JSON Lines; no new store or migration

**Testing**: Targeted Rust classifier and real loopback proxy tests, exact process-correlation regressions, `cargo xtask ci`, hosted Windows native integration and package certification, exact-head CI

**Target Platform**: Windows 11 product with platform-neutral Rust diagnostics exercised on Linux and Windows CI

**Project Type**: Multi-crate Rust library and CLI

**Performance Goals**: No extra handshake read, network round trip, unbounded retained data, or per-packet work; diagnostic classification remains constant-time over the finite error source chain

**Constraints**: No target instrumentation, proxy bypass after failed inspection, trust relaxation, raw TLS-flight retention, new dependency, direct-socket route, or private-title success claim

**Scale/Scope**: One issue (#574), existing TLS classifier and diagnostic projection, controlled multi-connection fixture, docs and changelog

## Constitution Check

*GATE: Passed before research and rechecked after design.*

- **P-1**: The current explicit, authenticated, target-scoped local proxy remains the only active route. No target code or memory is touched, no new trust effect is introduced, and no pinning bypass is added.
- **P-2/P-3**: Proxy owns TLS classification, facade owns process correlation, CLI owns presentation. Existing dependency direction remains unchanged.
- **P-4/P-9**: Every accepted connection retains a terminal or a counted bounded-detail loss, and failure never becomes successful inspection or blind relay.
- **P-5**: Existing pcapng and application/lifecycle schemas retain their parser contracts; stable refusal codes are additive values in the established record.
- **P-6**: The master specification and operator-facing documentation explain the new categories and the generic-EOF limit.
- **P-7/P-8**: Rust formatting, encoding, LF, hidden Windows child-process rules, and shell/tooling gates remain binding.
- **P-10/P-11**: The change is an unreleased fragment against published v0.10.14; controlled source acceptance is distinct from later optional private-title field observation.
- **Workflow**: Spec-Kit analysis, controlled tests, local gate, exact-head CI, external review closeout, and owner final merge review are required. The owner explicitly authorized push and official PR creation for S198.

Post-design recheck: passed. Each new code is derived from structured rustls or I/O evidence, stored in an existing bounded diagnostic, and does not expand routing or trust authority.

## Project Structure

### Documentation

```text
specs/198-launcher-tls-refusals/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/refusal-evidence.md
├── checklists/requirements.md
└── tasks.md
```

### Source

```text
crates/fragcap-proxy/src/{upstream.rs,tls.rs,runtime.rs}
crates/fragcap-proxy/tests/{https_proxy.rs,http1_proxy.rs}
crates/fragcap/src/deep_capture/native.rs
crates/fragcap-cli/src/commands/calibrate/assessment.rs
docs/{fragcap-specification.md,plans/README.md,glossary/}
site/content/docs/
changelog.d/
```

**Structure Decision**: Extend the existing proxy classifier and controlled test seams. The facade correlation join and diagnostic schema remain the single authorities for ownership and presentation.

## Implementation Sequence

1. Write failing classifier regressions for bare EOF/reset, structured invalid TLS, wrapper-source detection, explicit alerts, protocol mismatch, and generic invalid-data restraint.
2. Add stable transport-ended and invalid-message refusal classes without changing the existing alert, client-auth, or protocol classification precedence.
3. Write controlled authenticated CONNECT tests for launcher/updater success and the EOF/malformed failure pair, checking accepted ID, stage, boundary, cause, alert, transport category, and no full inspection on failure.
4. Add or strengthen exact owner, unavailable-owner, final-client calibration, and HTTP timeout-phase regressions at the existing facade/CLI seams, without inventing proxy-local process identity.
5. Update master spec, slice ordering, operator-facing diagnostic language, and dated changelog decisions. Run requirements checklist and Spec-Kit analysis again after implementation.
6. Run local CI parity and relevant Windows-controlled checks, commit, push the authorized branch, open the official PR, then address exact-head CI and no more than one requested second review round before owner handoff.

## Complexity Tracking

No constitution violation or new architectural component requires justification.
