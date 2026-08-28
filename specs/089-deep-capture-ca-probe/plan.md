# Implementation Plan: Deep Capture CA Trust-State Probe

**Branch**: `codex/089-deep-capture-ca-probe` | **Date**: 2026-08-28 |
**Spec**: [spec.md](spec.md)

## Summary

Replace doctor's placeholder CA result with a read-only Windows probe. It reads
exact owned identities from Deep Capture manifests, validates optional bundled CA
material, enumerates current-user and local-machine Root stores through CryptoAPI,
and feeds a pure classifier that is tested using injected inventories. Cleanup is
offered only for an exact observed manifest-backed certificate.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.82

**Primary Dependencies**: standard library, `serde_json`, existing `windows-sys`
0.36 CryptoAPI bindings

**Storage**: existing Deep Capture session bundles and Windows certificate stores

**Testing**: Rust unit and CLI integration tests, `cargo xtask ci`

**Target Platform**: Windows production probe; pure classification tests on every
workspace platform

**Project Type**: Rust workspace CLI

**Performance Goals**: bounded scan of at most 200 session entries and two Root
stores during doctor

**Constraints**: ordinary doctor is read-only; no subject matching; no new
dependency; Capture readiness remains non-blocking

**Scale/Scope**: one probe module, existing classifier/action integration,
specification and public CLI documentation

## Constitution Check

- **P-1**: Pass. The probe reads fragcap-owned manifest evidence and public
  certificate stores; it opens no process and mutates no trust.
- **P-2/P-3**: Pass. Platform enumeration remains behind a narrow seam in the CLI;
  pure evidence classification is platform-independent.
- **P-4**: Not applicable to the packet data path.
- **P-5**: Pass. No fixture or capture change.
- **P-6/P-9**: Pass. Partial reads are unknown, actual thumbprints are surfaced,
  and tests prove unrelated certificates are ignored.
- **P-7**: Pass. No dependency or lockfile change.
- **P-10**: Pass. TDD and full gates are required.
- **P-11**: Pass. Section 26.3 and public CLI documentation change with behavior.

The check remains green after design. No constitutional exception is required.

## Project Structure

### Documentation

```text
specs/089-deep-capture-ca-probe/
├── checklists/requirements.md
├── contracts/trust-probe.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code

```text
crates/fragcap-cli/src/doctor/
├── checks.rs
├── fix.rs
├── mod.rs
└── probe.rs

crates/fragcap-cli/tests/cli_doctor.rs
docs/fragcap-specification.md
site/content/docs/reference/cli.mdx
```

**Structure Decision**: Keep the feature in the existing doctor boundary. The
probe gathers values, `DeepCaptureCa` carries classification, checks render it,
and the existing action layer enforces confirmation.

## Complexity Tracking

No violations.
