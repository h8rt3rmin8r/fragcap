# Spec-Kit Analysis: S199

**Date**: 2026-11-09
**State**: Initial implementation and local gate passed; first hosted audit and package runs identified repairs in progress.

## Coverage

FR-001 to FR-003 map to T005-006 and T013. FR-004 to FR-006 map to T007-010 and T015. FR-007 to FR-008 map to T011-012. FR-009 maps to T005, T012 and T013. FR-010 maps to T004, T014-017 and T019. FR-011 maps to T018. Each user story has a controlled test path; no private-title run is a completion condition.

## Governing conflict and resolution

Current constitution P-1 and licensing section categorically forbid the selected component. Issue #577 carries the owner's explicit approval and requires a versioned amendment. The amendment is a planned first deliverable and a separate governance commit before any component implementation. It must retain passive Capture, target memory and file boundaries, exact scope and cleanup, and truthful anti-cheat language. A major version bump is required because the absolute ban is materially redefined.

## Result

The feature has no unresolved scope question. Constitution 2.0.0 and the
master specification now agree on the bounded component permission, exact
exclusions and later route boundary. `cargo xtask spec`, `cargo xtask docs
check`, and `git diff --check` passed on the governance change. Implementation
may proceed after this gate is committed. The post-implementation pass must
check that static package readiness is not described as operational direct
routing and that the future driver-open boundary stays outside S199.

## Post-implementation pass

The S199 package contract pins one official binary asset, tag source archive,
four exact package entries, x64 machine and signed-driver identity. The Windows
workflow verifies inputs before staging, the MSI and ZIP share those four
entries, and certification compares the package and installed files across the
installer lifecycle. Deliberate missing, altered-architecture and altered-driver
mutations must be refused. The external component appears in the generated
SBOM, third-party notices and project notice, with source and replacement
instructions. Doctor hashes the exact files without opening the driver and
reports load compatibility as unverified. Missing files warn, so ordinary
Capture and offline tests remain independent of the component. No runtime
`WinDivertOpen` call, target traffic diversion, or private-title claim was
introduced. `cargo xtask ci` passed locally. Hosted certification and review
remain required before owner merge handoff.

## First hosted-run findings

The first PR review found a strict-mode Doctor verdict parser error; it was
fixed and the review thread resolved. Hosted published v0.10.14 replay then
passed. The hosted audit reported RUSTSEC-2026-0459 for the old bundled SQLite
version. The existing rusqlite and libsqlite3-sys packages are being updated
without a waiver, requiring Rust 1.89 and new reviewed graph identities.
Windows package certification completed the official release build, then
stopped because a transient `cargo-cyclonedx` download failure was not checked
at installation. The setup step now checks native exit codes and retries that
pinned download; exact-head reruns must establish the outcome.
