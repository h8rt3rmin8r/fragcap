# Implementation Plan: Installer npcap exit-dialog reconciliation

**Branch**: `060-installer-npcap` | **Date**: 2026-08-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/060-installer-npcap/spec.md`

## Summary

Make the MSI exit-dialog npcap prompt agree with the docs. Add a WiX `AppSearch`
property backed by a `FileSearch` for `wpcap.dll` under `[System64Folder]` (the same
WinPcap-API marker `doctor` reads), pre-check the exit-dialog download-page checkbox
only when that marker is absent (a `SetProperty` conditioned on it, replacing the
static `Value="1"`), and reword the label to state why npcap is wanted rather than
asserting an unconditional requirement. Reconcile the getting-started narrative and
the specification distribution note. Policy is unchanged: the checkbox still only
opens the vendor page, and only when left checked; fragcap downloads, bundles, and
installs nothing. WiX authoring plus docs; no Rust, no dependency, no `Cargo.lock`
change.

## Technical Context

**Language/Version**: WiX v3 XML (cargo-wix), plus MDX/Markdown docs. No Rust change.

**Primary Dependencies**: none added. Core WiX 3 `AppSearch`/`FileSearch`/`SetProperty`
only, no WiX extension (WixUIExtension and WixUtilExtension are already linked by
cargo-wix; re-passing an extension breaks `light`).

**Storage**: N/A.

**Testing**: `cargo xtask ci` (green; does not build the MSI), `cargo xtask spec`
(distribution note lockstep), `scripts/lint-docs.sh check`, docs site build, plus WiX
XML review against the WiX 3 schema. The install-time behavior is confirmed at
release-build time (candle/light run in the release job), stated openly.

**Target Platform**: Windows MSI; the docs site is static-exported.

**Project Type**: CLI (Rust workspace) with a co-located docs site and a WiX installer.

**Performance Goals**: N/A.

**Constraints**: No new WiX extension. UTF-8 no BOM, LF, no em-dashes or en-dashes.
Policy unchanged (P-1): detection and page-opening only, never download/bundle/install.
No `Cargo.lock` change.

**Scale/Scope**: one WiX property + one `SetProperty` + one label + one comment in
`main.wxs`; one paragraph in `getting-started.mdx`; one sentence in the spec; a
changelog fragment.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 (technique denylist / npcap never bundled)**: Preserved and strengthened. The
  installer still only opens the vendor download page under an explicit opt-in
  checkbox; it downloads, bundles, and installs nothing. The change makes the prompt
  more conservative (pre-checked only when the driver is absent). The constitution's
  npcap rule permits detecting npcap and reporting its absence with the official
  download location, which is exactly this. PASS.
- **P-6 (new term -> glossary same change)**: No new user-facing term (npcap, capture
  driver, and installer are existing vocabulary). PASS.
- **P-9 (honest reporting)**: The reworded label states why the driver is wanted and
  is pre-selected only when it is actually absent, so the prompt no longer asserts a
  requirement the machine has already met. PASS.
- **P-11 (docs describe what shipped)**: getting-started and the spec distribution note
  are reconciled with the conditional behavior; `cargo xtask spec` keeps the lockstep.
  PASS.
- **Pinned artifacts**: `wix/main.wxs` is not on the pinned list (workflows, toolchain,
  `release.toml`, `scripts/`, release docs), so a normal changelog fragment suffices,
  no dated decision fragment. PASS.
- **Architecture / deps**: No crate or dependency change. PASS.
- **Encoding / no dashes**: enforced across edited files. PASS (verified in tasks).

No violations. Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/060-installer-npcap/
├── plan.md              # This file
├── spec.md              # Feature spec
├── research.md          # Phase 0 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── exit-dialog-checkbox.md   # The detection + checkbox-default + label contract
└── checklists/
    ├── requirements.md
    └── installer-npcap.md
```

### Source Code (repository root)

```text
crates/fragcap-cli/wix/main.wxs      # + NPCAP marker AppSearch property; SetProperty
                                     #   gating WIXUI_EXITDIALOGOPTIONALCHECKBOX; reworded
                                     #   WIXUI_EXITDIALOGOPTIONALCHECKBOXTEXT; updated comment
site/content/docs/getting-started.mdx# reconcile the "1. Install fragcap" npcap paragraph
docs/fragcap-specification.md        # reconcile the distribution exit-dialog note (~3401)
changelog.d/S060-*.md                # changelog fragment (spec-impact: 20)
```

**Structure Decision**: A `data-model.md` is omitted; this slice adds no type and no
persistent state, only WiX authoring and prose. The single behavioral contract lives
in `contracts/exit-dialog-checkbox.md`.

## Complexity Tracking

No constitution violations; no entries.
