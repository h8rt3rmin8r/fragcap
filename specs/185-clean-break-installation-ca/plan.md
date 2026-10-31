# Implementation Plan: S185 Clean-break installation CA

**Branch**: `codex/s185-installation-ca-continuity` | **Date**: 2026-10-31 | **Spec**: [spec.md](spec.md)

## Summary

Close #528 and #529 together by replacing ephemeral per-session roots with one installation-owned machine CA. The first S185 install removes identified old Fragcap state; later installs reuse the new CA. Every command reports read-only health, Deep Capture proves signing and trust before effects, and session teardown cannot revoke trust under a live client.

## Technical Context

**Language/Version**: Rust workspace, Windows production adapters, MSRV 1.88, WiX v3 MSI.
**Dependencies**: Existing `rcgen`, `ring`, `zeroize`, `windows-sys`, DPAPI, CryptoAPI, ACL helpers. Avoid new runtime packages.
**Storage**: ProgramData/fragcap/pki exact metadata and encrypted key; LocalMachine/Root public certificate. No old-state migration or preservation.
**Verification**: Spec-Kit analysis, focused CA/store/ACL/cutover/session tests, Windows controlled integration, `cargo xtask ci`, package certification, exact-head PR CI and bounded reviews.

## Constitution Check

P-1 version 1.7.0 explicitly permits a disclosed persistent installation CA and requires Deep Capture pre-effect proof. No target process memory access or hidden routing is introduced. P-4 and P-10 require truthful resource/continuity reports; #528 route/proxy outcomes stay separate. P-7 protects key and bundle material. The clean cutover is explicitly owner-directed. Pre-design check passes.

## Structure

- `crates/fragcap-proxy/src/certificate.rs`, `windows/acl.rs`, `windows/trust.rs`: reloadable signer, protected storage, exact machine root operations.
- `crates/fragcap-cli/src/ca.rs`, `cli.rs`, `lib.rs`, `commands/deep_capture.rs`, `doctor/`: administrative commands, startup status, pre-effect gate, trust lifecycle, Doctor.
- `crates/fragcap/src/deep_capture/native.rs`: prepared authority from installed signer.
- `crates/fragcap-cli/wix/main.wxs`, package workflow and tests: cutover/provision actions, disclosure, rollback, portable path.
- `docs/fragcap-specification.md`, help, threat model, `docs/plans/README.md`, `changelog.d/`, and S185 Spec-Kit artifacts.

## Design

1. Implement a stable versioned installation identity and protected signer. Exact DER and hashes are validated on every load; privileged sign/verify proves pairing. Keep public read-only status separate from privileged proof.
2. Build one transaction backend for MSI and `ca install`, with exact inventory/reuse/rollback. The first S185 install purges identified pre-S185 canonical state and verifies absence; repeated S185 maintenance retains the new identity.
3. Route CLI startup through read-only health, including clap help/version and bare invocation. Emit one diagnostic warning and stable JSON event without touching stdout.
4. Load and prove the installed authority before Deep Capture effects, bind fingerprint to plan/manifest, and remove production session root add/remove. Preserve controlled fixture injection.
5. Expose explicit administrative status/repair/remove with exact preview and readback. Doctor reports installation health and no longer treats absent session roots as readiness.
6. Qualify two sessions and failure cases, including a live controlled route owner. Check package contents, exact MSI action ordering, and rollback behavior.

## Implementation Sequence

1. Complete specify, clarify, checklist, plan, tasks, and read-only analysis for literal S185 identity.
2. Write failing focused tests for identity reuse, health states, pre-effect refusal, two-session root continuity, and exact cleanup.
3. Implement signer/store, CLI/Doctor, runtime/teardown, MSI/portable setup, and cutover.
4. Reconcile docs and release notes, run full gates, commit, push, and publish an official PR.
5. Monitor exact-head CI and all reviews, request at most one second round, address every actionable comment, and hand the green reviewed PR to the owner before merge.

## Post-Design Constitution Recheck

Pass. Persistent machine trust is disclosed once at installation; each Deep Capture session is still explicitly scoped and authorized. The signer never enters a bundle or child process. The old state is discarded as directed, and exact certificate ownership limits avoid deleting unrelated roots.
