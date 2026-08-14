# Tasks: surface a Wireshark download link in doctor

**Feature**: 046-wireshark-download-link | **Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

CLI/core only. No `site/` change. No golden regeneration (the ready golden does
not exercise the changed strings). Verify with `cargo xtask ci`.

## Phase 1: Foundational (the constant + its reach)

- [ ] T001 Add `pub const WIRESHARK_DOWNLOAD_URL: &str = "https://www.wireshark.org/download.html";` to `crates/fragcap-core/src/interface.rs`, beside `DRIVER_DOWNLOAD_URL`, with a doc comment mirroring it (fragcap never fetches it; this exists so a diagnostic can say where to go, and the installer also provides npcap).
- [ ] T002 Re-export it through the facade: add `WIRESHARK_DOWNLOAD_URL` (and `DRIVER_DOWNLOAD_URL` if convenient for symmetry, but not required) to the `pub use fragcap_core::interface::{ ... }` block in `crates/fragcap/src/lib.rs` so it is reachable as `fragcap::core::WIRESHARK_DOWNLOAD_URL`.

## Phase 2: US1 - the link in doctor guidance

- [ ] T003 [US1] In `crates/fragcap-cli/src/doctor/checks.rs`, import `fragcap::core::WIRESHARK_DOWNLOAD_URL`, and convert the `NPCAP_SOURCE` const to `fn npcap_source() -> String` that formats using `WIRESHARK_DOWNLOAD_URL` for the Wireshark URL and keeps the existing `https://npcap.com` npcap literal; update the single call site (`Check::fail(DRIVER, "npcap", "npcap is not installed", npcap_source())`).
- [ ] T004 [US1] In the same file, extend the `integration()` not-registered arm (`(false, false)`, both the `Some(dir)` and `None` variants) so the guidance names the Wireshark download URL and notes the installer also provides npcap, while keeping the `fragcap extcap install` guidance and the optional `Warn` severity.

## Phase 3: Tests

- [ ] T005 [US1] Add unit tests in `checks.rs`: (a) the `integration()` detail for the not-registered case (both scopes false) contains `WIRESHARK_DOWNLOAD_URL`; (b) `npcap_source()` (or the npcap check's remediation) contains `WIRESHARK_DOWNLOAD_URL`. Keep the existing not-registered optional-Warn assertions.

## Phase 4: Polish & verification

- [ ] T006 Add `changelog.d/046-wireshark-download-link.added.md` describing the Wireshark download link in doctor, dash-free, UTF-8/LF.
- [ ] T007 Run `cargo xtask ci` in the foreground to green (fmt, clippy, tests, lint, deps, license). Confirm the `doctor-ready` golden is unchanged.
- [ ] T008 Run the single-source grep (`grep -rn "wireshark.org" crates/fragcap-core crates/fragcap crates/fragcap-cli/src`): expect exactly one literal, in `interface.rs`.
- [ ] T009 Confirm `git diff --stat` touches only `fragcap-core`, `fragcap` (facade), `fragcap-cli`, the changelog, and `specs/046-...` (no `site/`, no fixture, no golden).

## Dependencies

- T001 before T002 before T003/T004 (the constant must exist and be re-exported).
- T005 after T003/T004. T006-T009 after the code is final.

## MVP

T001-T004 is the whole feature (the constant, its reach, and the two guidance
edits); T005 locks it with tests and T007 keeps the gate green.
