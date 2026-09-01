# Implementation Plan: TLS Evidence and Sensitive Artifact Lifecycle

**Branch**: `codex/107-tls-sensitive-artifacts` | **Date**: 2026-09-01 | **Spec**: `specs/107-tls-sensitive-artifacts/spec.md`

## Summary

Complete the native TLS evidence boundary by attaching an explicit, proxy-owned, live-flushed key logger only to client-facing TLS, accepting only operator-supplied upstream client identities, and preserving evidence-backed TLS refusal categories. Move sensitive bundle preparation ahead of proxy startup, add exact journaled cleanup and immutable share-copy services, and remove the platform workflow path filter because that workflow tests the whole workspace.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88

**Primary Dependencies**: Existing Tokio 1.53.1, rustls 0.23.43, tokio-rustls, ring, zeroize, rustls-pki-types, serde_json, and windows-sys 0.36

**Storage**: Session bundle files, append-and-sync sensitive action JSON Lines journal, atomic temporary-file and temporary-directory publication

**Testing**: Cargo unit and integration tests, controlled TLS 1.2/TLS 1.3 and mutual-TLS loopback tests, Windows ACL tests, fault injection, CLI tests, and repository xtask gates

**Target Platform**: Windows product runtime; portable filesystem and TLS contract tests run without elevation, trust mutation, a capture driver, an account, or Internet access

**Project Type**: Rust workspace containing libraries, a facade, a CLI, and GitHub Actions workflows

**Performance Goals**: Key-log writes serialize one bounded record and flush immediately; sensitive journal and manifests remain bounded; sharing streams files rather than retaining bundles in memory

**Constraints**: Explicit authorization, no target key access, no pinning bypass, no upstream key logging, fail-closed access control, exact path containment, idempotent cleanup, immutable source evidence, no new registry packages

**Scale/Scope**: Three Deep Capture issues (#300, #304, #322), one workflow correction, existing proxy/facade/CLI boundaries, and a narrow sensitive-action recovery journal that leaves #320 open

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1 Safety boundaries**: PASS. Key logging and client identities require the exact confirmed Deep Capture plan. No target-process access or pinning bypass is added.
- **P-2 Least privilege**: PASS. Access control is current-user scoped; no machine trust or elevation is required.
- **P-3 Layered architecture**: PASS. TLS secrets and handshake classification stay in `fragcap-proxy`; policy, artifact lifecycle, and assembly stay in `fragcap`; CLI owns presentation and confirmation.
- **P-4 Loss visibility**: PASS. Key-log, permission, journal, cleanup, copy, and refusal outcomes remain distinct and auditable.
- **P-5 Determinism**: PASS. Stable refusal tokens, normalized relative paths, paired journal intent/result records, and exhaustive manifests define total output order.
- **P-6 Testability**: PASS. Injected I/O and controlled loopback TLS peers cover the feature without external systems.
- **P-7 Bounded resources**: PASS. Record sizes, journal entries, paths, copy buffers, waits, and recovery work are finite.
- **P-8 Dependency discipline**: PASS. Existing exact dependencies expose every required API; no package or lockfile addition is planned.
- **P-9 Honest capability claims**: PASS. Ambiguous trust refusal remains unknown, upstream secrets are excluded, and the narrow journal does not claim general session recovery.

Post-design check: PASS. The design moves artifact protection earlier in the lifecycle because the existing CLI-only finalization occurs after plaintext application evidence is opened. This is an explicit architectural correction required by the before-exposure guarantee.

## Architecture and Phases

1. Introduce stable TLS key-log, client-identity, and refusal contracts in the proxy.
2. Attach one shared key logger only to client-facing server configurations and prove analyzer-compatible TLS 1.2/TLS 1.3 output under concurrency.
3. Parse explicit PEM/DER client identities, configure upstream mutual TLS, and preserve typed rustls failure evidence.
4. Prepare protected bundle ownership before proxy startup, then add bounded journaled retain/delete/recovery and atomic share-copy services.
5. Expose explicit client identity and bundle cleanup/export surfaces through the CLI, preserving confirmation and redaction.
6. Remove incomplete platform workflow filters and pin the full-workspace trigger contract in xtask.
7. Update specifications, schema-facing documentation, changelog records, and the security checklist.
8. Run convergence and the complete repository gate, then commit locally and halt before push.

## Project Structure

```text
specs/107-tls-sensitive-artifacts/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
├── checklists/
└── tasks.md

crates/fragcap-proxy/src/
├── key_log.rs
├── tls.rs
├── upstream.rs
├── application.rs
├── model.rs
└── runtime.rs

crates/fragcap/src/deep_capture/
├── artifacts.rs
├── adapters.rs
├── model.rs
├── native.rs
└── session.rs

crates/fragcap-cli/src/
├── cli.rs
├── commands/bundle.rs
├── commands/deep_capture.rs
└── doctor/

xtask/src/
.github/workflows/platform.yml
docs/
```

**Structure Decision**: Extend the existing proxy, facade, and CLI ownership boundaries. Add no crate and no reverse dependency.

## Complexity Tracking

No constitution violation requires an exception.
