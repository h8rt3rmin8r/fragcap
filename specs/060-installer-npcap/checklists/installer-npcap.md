# Checklist: Installer npcap reconciliation requirements quality (S060)

**Purpose**: Unit-test the requirements for S060 before implementation.
**Created**: 2026-08-18
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [ ] CHK001 - Is the detection marker specified exactly (the WinPcap-API `wpcap.dll` in the native system dir, the same one `doctor` uses), and distinguished from the vendor-directory copy? [Completeness, Spec §FR-001, Edge Cases]
- [ ] CHK002 - Is the conditional pre-check fully specified for both states (absent -> pre-checked, present -> unchecked)? [Completeness, Spec §FR-002, US1, US2]
- [ ] CHK003 - Is the label requirement stated (says why, does not assert an unconditional requirement)? [Completeness, Spec §FR-003]
- [ ] CHK004 - Is the unchanged policy stated (only opens the vendor page, downloads/bundles/installs nothing)? [Completeness, Spec §FR-004, SC-003]

## Requirement Clarity

- [ ] CHK005 - Is "no new WiX extension" stated as a hard constraint (re-passing extensions breaks the linker)? [Clarity, Spec §FR-007]
- [ ] CHK006 - Is the silent-install case handled (no exit dialog, detection must not fail the install)? [Clarity, Spec Edge Cases]

## Requirement Consistency

- [ ] CHK007 - Does the installer marker match `doctor`'s marker so the two agree on "present"? [Consistency, Spec §FR-001, Dependencies]
- [ ] CHK008 - Are the docs, the spec distribution note, and the installer wording reconciled to one story? [Consistency, Spec §FR-005, FR-006, SC-004]

## Acceptance Criteria Quality

- [ ] CHK009 - Is each success criterion verifiable by WiX-schema review plus ci, given the MSI is not built in ci? [Measurability, Spec §SC-001..SC-005, FR-008]
- [ ] CHK010 - Is "no new dependency / no Cargo.lock change" a checkable constraint? [Measurability, Spec §FR-007, SC-005]

## Governance

- [ ] CHK011 - Is the CI-insufficiency disclosed (MSI built at release only; behavior confirmed at release-build time)? [Governance, Spec §FR-008, Notes]
- [ ] CHK012 - Is the pinned-artifact question settled (wix/main.wxs is not pinned, so a normal changelog fragment, no dated decision)? [Governance, Spec Assumptions]
- [ ] CHK013 - Is the P-1 posture preserved (detection and page-opening only; no download/bundle/install)? [Governance, Spec §FR-004]
