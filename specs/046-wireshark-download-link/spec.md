# Feature Specification: surface a Wireshark download link in doctor

**Feature Branch**: `046-wireshark-download-link`

**Created**: 2026-08-14

**Status**: Draft

**Input**: GitHub issue #107. fragcap leans on Wireshark (the analyzer, the
extcap integration) but `doctor`'s integration guidance never points at where to
get it, unlike the npcap check which links its download. Add a single-sourced
Wireshark download link and surface it in the integration guidance, noting the
Wireshark installer also provides npcap.

## Clarifications

### Session 2026-08-14

Resolved from the issue, the approved roadmap plan, and the current source (no
operator escalation needed):

- Q: Where does the Wireshark URL live? -> A: A single constant
  `WIRESHARK_DOWNLOAD_URL` in `fragcap-core::interface`, beside the existing
  `DRIVER_DOWNLOAD_URL`, re-exported through the `fragcap` facade so the CLI
  reaches it the same way it reaches other core items. Value:
  `https://www.wireshark.org/download.html` (the download page named in the
  issue).
- Q: Which doctor surface gets the link? -> A: The `analyzer extcap`
  integration check's not-registered guidance, and the npcap-absent remediation
  (which already mentions Wireshark) is refactored to single-source its Wireshark
  URL from the same constant. The link is unconditional (doctor performs no
  Wireshark-presence detection), matching the npcap detect-and-link posture.
- Q: Does the onboarding/docs side change? -> A: No. The getting-started guide
  already links wireshark.org (S042/PR #113); #107's docs side is satisfied.
  This slice is CLI/core only.
- Q: Does doctor stay granular? -> A: Yes. The link is added; the per-option
  npcap precision and the optional-Warn severity of the integration check are
  unchanged. Only the not-registered guidance and the npcap remediation's
  Wireshark URL change.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A fresh user is pointed at Wireshark from doctor (Priority: P1)

A user whose analyzer extcap integration is not registered runs `fragcap doctor`
and is told not only to run `fragcap extcap install`, but also where to get
Wireshark (which their captures are meant to be read in, and whose installer also
provides the required npcap driver). Today the integration guidance names no
source for Wireshark at all.

**Why this priority**: It is the whole point of the slice (#107): fragcap depends
on Wireshark but never points at it.

**Independent Test**: Build `Inputs` with the extcap binary in neither scope; the
integration check detail contains the Wireshark download URL and notes the
installer also provides npcap.

**Acceptance Scenarios**:

1. **Given** the extcap binary in neither the per-user nor the system directory,
   **When** the integration check classifies, **Then** the detail names both
   `fragcap extcap install` and the Wireshark download URL, and stays an optional
   `Warn`.
2. **Given** npcap is absent, **When** the npcap check classifies, **Then** its
   remediation's Wireshark URL is the single-sourced `WIRESHARK_DOWNLOAD_URL`.
3. **Given** a ready machine (extcap registered, npcap present), **When** doctor
   runs, **Then** the output is unchanged from before this slice (the link
   appears only in the not-ready guidance).

### Edge Cases

- The extcap directory cannot be resolved (`None`): the guidance still names the
  Wireshark URL and `fragcap extcap install`, without a directory.
- The constant is used in exactly one place per URL; no second Wireshark URL
  literal remains in the doctor guidance.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A single constant `WIRESHARK_DOWNLOAD_URL` MUST exist in
  `fragcap-core::interface` (beside `DRIVER_DOWNLOAD_URL`) and be re-exported
  through the `fragcap` facade; it holds `https://www.wireshark.org/download.html`.
- **FR-002**: The `analyzer extcap` integration check's not-registered guidance
  MUST include the Wireshark download URL and note the installer also provides
  npcap, while keeping the `fragcap extcap install` guidance and the optional
  `Warn` severity.
- **FR-003**: The npcap-absent remediation MUST take its Wireshark URL from
  `WIRESHARK_DOWNLOAD_URL`, so no second Wireshark URL literal remains in the
  doctor guidance.
- **FR-004**: `fragcap-core` MUST stay platform-neutral (the constant is a plain
  `&str`, no dependency added); the default no-feature build and the Linux
  neutrality build MUST still compile.
- **FR-005**: doctor MUST stay granular and truthful: the npcap per-option
  precision is unchanged, and the integration check remains a non-blocking
  optional `Warn` when not registered.
- **FR-006**: The `doctor-ready` golden MUST be unchanged (the link appears only
  in not-ready guidance, which the ready golden does not exercise); unit tests
  MUST assert the Wireshark URL appears in the integration not-registered detail
  and in the npcap remediation.
- **FR-007**: All added or edited text MUST be UTF-8, LF, and free of em and en
  dashes; a changelog fragment MUST be added.

### Key Entities

- **`WIRESHARK_DOWNLOAD_URL`**: the single source of the Wireshark download URL,
  a `&str` constant in `fragcap-core::interface`, sibling to `DRIVER_DOWNLOAD_URL`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The Wireshark download URL appears in doctor's integration
  not-registered guidance (unit-tested).
- **SC-002**: The Wireshark URL is defined once (the constant) and referenced,
  not duplicated, in the doctor guidance (grep-verifiable: one `wireshark.org`
  literal, in `interface.rs`).
- **SC-003**: `cargo xtask ci` is green (fmt, clippy, tests, lint, deps,
  license), with the `doctor-ready` golden unchanged.
- **SC-004**: The default no-feature build and the Linux `fragcap-core`
  neutrality build compile.

## Assumptions

- doctor performs no Wireshark-presence detection today; an unconditional link is
  the agreed posture (the issue states even an unconditional line is an
  improvement), and runtime Wireshark detection is a possible later nicety, out
  of scope here.
- The getting-started onboarding already links Wireshark (S042); this slice does
  not edit `site/`.
- The npcap download URL literal (`https://npcap.com`) in the remediation is left
  unchanged; single-sourcing it to `DRIVER_DOWNLOAD_URL` is orthogonal to #107
  and would change unrelated output, so it is out of scope.
