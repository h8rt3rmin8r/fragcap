# Implementation Plan: S186 Installed Deep Capture truth

**Branch**: `codex/s186-installed-deep-capture-truth` | **Date**: 2026-10-31 | **Spec**: [spec.md](spec.md)

## Summary

Close #534 and #535 in one v0.10.10 field-correction slice. Separate public installation CA health from privileged signer proof, and make HTTPS CONNECT request parsing use the actual tunnel scheme and effective port. Preserve exact authority, target scope, upstream verification, and honest partial-session evidence. Diagnose the independent header timeout at its observed read stage.

## Technical Context

**Language/Version**: Rust workspace, MSRV 1.88.
**Dependencies**: Existing Rust and Windows APIs only; no new runtime package anticipated.
**Storage**: Existing public CA identity, protected signer, LocalMachine/Root, and session artifacts. No storage migration.
**Testing**: Focused Rust tests, controlled TLS proxy lab, isolated Windows package and ordinary-account ACL check, `cargo xtask ci`, and exact-head hosted CI.
**Target Platform**: Windows product, platform-neutral proxy protocol tests.
**Project Type**: CLI, native proxy, and session facade.
**Performance Goals**: Preserve finite existing timeouts and bounded evidence paths.
**Constraints**: No signer ACL relaxation, unverified forwarding, silent fallback, private fixture publication, live-game run, or active-desktop focus.
**Scale/Scope**: Two open issue contracts and their direct diagnostics and evidence consumers.

## Constitution Check

P-1 permits the approved installation CA only with a protected signer and exact pre-effect signing proof. P-4 and P-9 require distinct observed causes and no full HTTPS claim from metadata, plain HTTP, or cleanup success. P-7 keeps wrappers thin and sensitive material confined. The plan changes no forbidden target technique and passes the pre-design check. The controlled test tier is the implementation gate; any later real-title result is a separate field claim.

## Project Structure

### Documentation

```text
specs/186-installed-deep-capture-truth/
  spec.md
  plan.md
  research.md
  data-model.md
  contracts/health-and-tunnel.md
  quickstart.md
  tasks.md
  checklists/
```

### Source Code

```text
crates/fragcap-cli/src/ca.rs
crates/fragcap-cli/src/lib.rs
crates/fragcap-cli/src/commands/doctor.rs
crates/fragcap-cli/tests/cli_ca_startup.rs
crates/fragcap-proxy/src/http1.rs
crates/fragcap-proxy/src/http2.rs
crates/fragcap-proxy/src/runtime.rs
crates/fragcap-proxy/tests/https_proxy.rs
crates/fragcap/tests/
docs/fragcap-specification.md
docs/plans/README.md
changelog.d/
```

**Structure Decision**: Keep CA public health in CLI, private proof at the existing Deep Capture preflight, and protocol context in the native proxy. Reuse artifact and classification owners instead of adding a parallel verdict path.

## Design and Execution Order

1. Reproduce the CA false warning with denied signer metadata and the tunnel default-port mismatch with a controlled HTTPS request. Record the header-timeout stage independently.
2. Add failing CA health and command-output tests, then implement public-resource-first evaluation and an explicit unverified signer status while retaining pre-effect sign/verify.
3. Add failing tunnel parser and verified round-trip tests. Pass HTTPS context through every tunneled HTTP/1.1 request, preserve raw bytes, normalize effective authority, and retain precise refusal for distinct unsafe requests. Check HTTP/2 parity.
4. Reconcile terminal cause, application, HAR, compatibility, packet, and cleanup evidence in controlled paths. Prove five mismatch plus one timeout causes remain distinct and cannot manufacture full HTTPS.
5. Update architecture of record and changelog fragment, complete tasks, run analysis again if scope changes, then run focused and full gates in the foreground.
6. Commit the exact S186 branch, push and open an official PR under the owner's explicit authorization, monitor exact-head checks and at most two external review rounds, and hand the reviewed green PR to the owner before merge.

## Post-Design Constitution Recheck

Pass. The public health verdict does not expose or weaken the signer. Tunnel context changes interpretation only where the protocol supplies an effective scheme and port; distinct-authority handling still requires target scope and verified upstream identity. Every observed failure remains attributable to its actual stage.
