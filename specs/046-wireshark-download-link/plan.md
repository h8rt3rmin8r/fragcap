# Implementation Plan: surface a Wireshark download link in doctor

**Branch**: `046-wireshark-download-link` | **Date**: 2026-08-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/046-wireshark-download-link/spec.md`

## Summary

Add a single `WIRESHARK_DOWNLOAD_URL` constant to `fragcap-core::interface`
(beside `DRIVER_DOWNLOAD_URL`), re-export it through the `fragcap` facade, and use
it in `doctor`: surface the Wireshark download link in the `analyzer extcap`
not-registered guidance, and single-source the Wireshark URL in the npcap-absent
remediation. CLI/core only; the getting-started docs already link Wireshark
(S042). The `doctor-ready` golden is unchanged because the link appears only in
not-ready guidance.

## Technical Context

**Language/Version**: Rust (workspace MSRV 1.82). Changes in `fragcap-core`,
`fragcap` (facade), `fragcap-cli`.

**Primary Dependencies**: none added. A `pub const &str` and its re-export.

**Testing**: `cargo test -p fragcap-cli` (the doctor classifier unit tests in
`checks.rs` and the golden-driven `cli_doctor.rs`), then full `cargo xtask ci`.

**Target Platform**: The constant and classifier are platform-neutral; the value
is Windows-relevant but the string is not platform code.

**Project Type**: Rust workspace (core library + facade + CLI).

**Constraints**: `fragcap-core` stays platform-neutral (P-2); doctor stays
truthful and granular (P-9); the integration check stays a non-blocking optional
`Warn`; UTF-8, LF, no em/en dashes.

**Scale/Scope**: one core constant, one facade re-export line, one CLI function
(`NPCAP_SOURCE` const becomes `npcap_source()`), one integration arm edited, two
or three unit tests, one changelog fragment.

## Constitution Check

- **P-1 Passive Observation**: No capture behavior changes. PASS.
- **P-2 Core Platform-Neutral**: The core change is a `&str` constant, no
  dependency, no platform surface. PASS.
- **P-5 Compatibility**: Not engaged. N/A.
- **P-6 Glossary First**: No new term (Wireshark, extcap, npcap already defined).
  PASS.
- **P-8 House Standards**: UTF-8, LF, no dashes. PASS.
- **P-9 The Instrument Does Not Lie**: doctor gains a truthful, actionable link;
  granularity and the optional-Warn severity are preserved. PASS.

No violations. Complexity Tracking not required.

## Project Structure

### Documentation (this feature)

```text
specs/046-wireshark-download-link/
├── spec.md
├── plan.md
├── research.md
├── quickstart.md
├── checklists/requirements.md
└── tasks.md
```

data-model.md and contracts/ omitted: no data entity, no new machine interface
(the constant is an internal string; doctor's output contract is covered by the
existing goldens/tests).

### Source (paths touched)

```text
crates/fragcap-core/src/interface.rs          # + WIRESHARK_DOWNLOAD_URL const beside DRIVER_DOWNLOAD_URL
crates/fragcap/src/lib.rs                      # re-export WIRESHARK_DOWNLOAD_URL through fragcap::core
crates/fragcap-cli/src/doctor/checks.rs        # NPCAP_SOURCE const -> npcap_source() fn using the const; integration() not-registered arm adds the link; + unit tests
changelog.d/046-wireshark-download-link.added.md
```

**Structure Decision**: Reach the constant from the CLI the established way, via
the `fragcap` facade re-export (the CLI depends on `fragcap`, not directly on
`fragcap-core`). `NPCAP_SOURCE` becomes a function because a `const` cannot format
another constant into itself; `Check::fail`/`warn` already take
`impl Into<String>`, so the call sites accept the produced `String` unchanged in
shape.

## Design decisions (see research.md)

- The Wireshark URL is single-sourced in the constant and used in both the
  integration not-registered arm and the npcap remediation. The npcap URL literal
  (`https://npcap.com`) is left unchanged (single-sourcing it to
  `DRIVER_DOWNLOAD_URL` is orthogonal to #107 and would change unrelated output).
- The link is unconditional (no Wireshark detection), matching the npcap posture.
- No golden regeneration: the ready golden has npcap ok and extcap installed, so
  neither changed string appears in it. Unit tests assert the new link text.

## Complexity Tracking

No constitution violations; no entries.
