# Feature Specification: Installer npcap exit-dialog reconciliation

**Feature Branch**: `060-installer-npcap`

**Created**: 2026-08-18

**Status**: Draft

**Input**: User description: "Issue #133: the MSI exit-dialog npcap checkbox is pre-checked and reads 'required before capturing traffic', contradicting the docs and ignoring that npcap is usually already installed."

## Context

The Windows MSI ends with an exit-dialog checkbox that opens the npcap download
page. Today it is pre-checked unconditionally and labeled "Open the npcap download
page (required before capturing traffic)". To a user who followed the getting
started guide, which has them install npcap as a prerequisite through the Wireshark
installer, this is confusing on three counts: it never says why, it contradicts
having just satisfied the prerequisite, and it reads against the docs' stated
posture that fragcap never downloads, bundles, or installs npcap.

This slice makes the installer prompt tell the same story as the docs. It detects
whether the npcap capture driver fragcap actually uses is already present, pre-checks
the download-page option only when it is absent, and rewords the label to explain
why the driver is wanted rather than asserting an unconditional requirement. The
policy is unchanged: fragcap still never downloads, bundles, or installs npcap; the
checkbox still only opens the vendor's page, and only when the user leaves it
checked. This is a presentation fix, not a policy change.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A user who already has npcap (Priority: P1)

A user installed npcap as the prerequisite (through Wireshark), then runs the fragcap
installer. On completion, the npcap download-page option is present but not
pre-checked, and its label explains what npcap is for rather than asserting they
still need it. They click Finish and are not sent to a download page for something
they already have.

**Why this priority**: This is the reported confusion. The common case (npcap
already present) must not read as "you still need this".

**Acceptance Scenarios**:

1. **Given** npcap's WinPcap-API driver is present on the machine, **When** the
   installer reaches the exit dialog, **Then** the download-page checkbox is shown
   unchecked and its label does not assert an unconditional requirement.
2. **Given** that unchecked state, **When** the user clicks Finish without checking
   it, **Then** no npcap download page is opened.

---

### User Story 2 - A user who does not have npcap (Priority: P1)

A user without npcap runs the installer. On completion, the download-page option is
pre-checked and its label explains that fragcap needs the npcap capture driver.
Clicking Finish opens the vendor's download page, exactly as before.

**Why this priority**: The prompt must still guide a user who genuinely lacks the
driver, or the fix would trade one confusion for a silent gap.

**Acceptance Scenarios**:

1. **Given** npcap's WinPcap-API driver is absent, **When** the installer reaches the
   exit dialog, **Then** the download-page checkbox is pre-checked and its label
   states why the driver is wanted.
2. **Given** that pre-checked state, **When** the user clicks Finish, **Then** the
   vendor's npcap download page opens (fragcap downloads and installs nothing).

---

### Edge Cases

- npcap installed without the WinPcap API compatibility option (its own directory
  copy present but the System32 copy absent) is treated as absent, because that is
  the copy fragcap's live backend loads; the download page (to reinstall with the
  compatibility option) is still the right destination.
- A silent or unattended install shows no exit dialog, so the checkbox state is
  irrelevant there; the detection must not fail such an install.
- An administrator can still override the pre-check by checking or unchecking the box
  manually; detection only decides the default.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The installer MUST detect whether the npcap capture driver fragcap uses
  (the WinPcap-API-mode `wpcap.dll` in the native system directory) is present, using
  the same marker `doctor` uses, without any new WiX extension.
- **FR-002**: The exit-dialog download-page checkbox MUST be pre-checked only when
  that driver is absent. When it is present, the checkbox MUST default to unchecked.
- **FR-003**: The checkbox label MUST state why npcap is wanted (it is the capture
  driver) and MUST NOT assert an unconditional requirement that reads as "you still
  need this" to a user who already has it.
- **FR-004**: The checkbox MUST continue to only open the vendor's npcap download page
  when left checked, and the installer MUST still download, bundle, and install
  nothing (the policy is unchanged).
- **FR-005**: The getting-started documentation MUST describe the reconciled behavior,
  so the docs and the installer tell one coherent story.
- **FR-006**: The specification's distribution note describing the exit-dialog prompt
  MUST be reconciled with the conditional behavior, and `cargo xtask spec` MUST be
  green.
- **FR-007**: No new WiX extension MUST be introduced (re-passing extensions to the
  linker breaks the build), and no runtime dependency or `Cargo.lock` change MUST
  result.
- **FR-008**: `cargo xtask ci` MUST be green (it does not build the MSI, so the
  installer behavior is verified by WiX-schema review and confirmed at release-build
  time; this limitation MUST be stated, not hidden).

### Key Entities

- **npcap presence marker**: the `wpcap.dll` file in the native system directory, the
  WinPcap-API-mode copy fragcap's live backend loads and `doctor` probes. Its presence
  decides the checkbox default.
- **Exit-dialog download-page checkbox**: the optional exit-dialog control that opens
  the npcap vendor page; this slice changes its default checked state and its label,
  not its action.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On a machine with the npcap driver present, the exit-dialog checkbox is
  not pre-checked and its label does not read as an unconditional requirement.
- **SC-002**: On a machine without the driver, the checkbox is pre-checked and opens
  the vendor page on Finish, unchanged from today.
- **SC-003**: The installer downloads, bundles, and installs no npcap, unchanged.
- **SC-004**: The docs and the installer describe the same npcap prompt behavior.
- **SC-005**: `cargo xtask ci` is green with no new dependency and no `Cargo.lock`
  change.

## Assumptions

- The native system directory is addressed by the WiX `[System64Folder]` property, so
  the marker is found regardless of the MSI's own bitness; this matches `doctor`,
  which reads the real `System32\wpcap.dll`.
- A WiX `AppSearch`-backed property populated by a `FileSearch` is available in core
  WiX 3 (no extension), and a `SetProperty` conditioned on that property can set the
  checkbox default before the exit dialog renders.
- `crates/fragcap-cli/wix/main.wxs` is not on the pinned-artifacts list (that list is
  workflows, the toolchain file, `release.toml`, `scripts/`, and release docs), so
  this change needs a normal changelog fragment, not a dated decision fragment.

## Dependencies

- The npcap presence marker matches `crates/fragcap-cli/src/doctor/probe.rs`
  (`gather_windows`), so the installer and `doctor` agree on what "present" means.
- No code or runtime dependency changes; the change is WiX authoring plus
  documentation.
