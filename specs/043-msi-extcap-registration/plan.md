# Implementation Plan: MSI extcap registration, both scopes

**Branch**: `043-msi-extcap-registration` | **Date**: 2026-08-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/043-msi-extcap-registration/spec.md`

## Summary

Add an optional Wireshark extcap registration to the Windows installer, offering a
per-user default and a machine-wide option, by driving the already-shipped
`fragcap extcap install` command from WiX custom actions that mirror the existing
Defender-exclusion pattern. A custom wizard dialog carries the opt-in checkboxes
and the per-user-scope note. No `fragcap` CLI surface changes; the installer only
invokes `extcap install` (and `--dir` for machine-wide). The MSI cannot be built
or install-tested here, so the slice verifies WiX and docs consistency and
`cargo xtask ci`, and enumerates the build and install tests as manual
verification at the halt (decision D-4).

## Technical Context

**Language/Version**: WiX v3 XML (`crates/fragcap-cli/wix/main.wxs`), built by
cargo-wix in the release workflow. Documentation is Markdown/MDX. No Rust change.

**Primary Dependencies**: cargo-wix over WiX v3; WixUIExtension (WixUI_InstallDir)
and WixUtilExtension (WixQuietExec) are already linked by cargo-wix. No new
extension is introduced (the release workflow warns that passing extension flags
causes duplicate-table collisions).

**Storage**: N/A.

**Testing**: `cargo xtask ci` (unchanged Rust, plus repo lint over the WiX XML
text) and an XML well-formedness parse of `main.wxs`. The MSI build and the
per-user and machine-wide install tests are manual (no WiX toolchain here).

**Target Platform**: The Windows Installer package for a release.

**Project Type**: Windows installer (WiX) within the Rust workspace, plus docs.

**Performance Goals**: N/A.

**Constraints**: `main.wxs` is release-adjacent and pinned, so a dated changelog
decision is required (P-8 house standards, and the pinned-artifact rule). A
registration failure must never fail the install (mirror the Defender pattern).
No new CLI surface. No em or en dashes; UTF-8, LF.

**Scale/Scope**: One WiX file extended (a custom dialog, a registry search, and
paired custom actions per scope), one reference doc and getting-started updated,
two changelog fragments.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive Observation**: The registration is an installer and
  operating-system configuration action (copying the extcap binary into a
  Wireshark directory by running fragcap's own command), exactly the class the
  Defender-exclusion action already occupies: it opens no process handle and
  touches no target process, its memory, or the network stack. Outside the
  technique denylist. PASS.
- **P-2 / P-3 / P-4**: No crate or pipeline change. N/A.
- **P-5 Compatibility**: The extcap integration remains standard Wireshark
  extcap; unchanged. PASS.
- **P-6 Glossary First**: No new term (extcap, dependency model, and the tiers
  already have entries from earlier slices). If a new term appears, it gets an
  entry same-change. PASS.
- **P-7 Wrappers Stay Thin**: The installer drives the existing command rather
  than reimplementing registration; no parsing of command output. PASS.
- **P-8 House Standards**: WiX XML text and docs are UTF-8, LF, dash-free. The
  release-adjacent `main.wxs` change carries a dated changelog decision. PASS.
- **P-9 The Instrument Does Not Lie**: `doctor` is unchanged; the extcap row
  stays an optional warning, and the docs keep the slice 042 dependency model.
  The installer never claims to have registered when the action was skipped or
  failed (it returns ignore, and doctor reports the true state). PASS.

No violations. Complexity Tracking not required.

## Project Structure

### Documentation (this feature)

```text
specs/043-msi-extcap-registration/
├── plan.md, spec.md, research.md, data-model.md, quickstart.md
└── checklists/{requirements.md, wix.md}
```

### Source (repository paths touched)

```text
crates/fragcap-cli/wix/main.wxs                 # custom dialog + RegistrySearch + paired custom actions per scope
site/content/docs/reference/cli.mdx             # document the installer's optional extcap registration and both scopes
site/content/docs/getting-started.mdx           # note the installer option alongside `fragcap extcap install`
changelog.d/043-msi-extcap-registration.added.md
changelog.d/043-msi-extcap-registration.decisions.md   # dated: main.wxs is pinned/release-adjacent
```

**Structure Decision**: Extend `main.wxs` only; no Rust. Add:

1. A custom dialog `ExtcapDlg` inserted into the WixUI_InstallDir flow between
   `InstallDirDlg` and `VerifyReadyDlg`, carrying a per-user checkbox (property
   `REGISTEREXTCAP_USER`, opt-in) and a machine-wide checkbox (property
   `REGISTEREXTCAP_MACHINE`, for administrators), the "registers for the current
   user only" note, and the `fragcap extcap install` fallback text. The two
   properties are also public, so an administrator can set them for a silent
   install (`msiexec /i ... REGISTEREXTCAP_MACHINE=1`).
2. A `RegistrySearch` resolving Wireshark's install directory (property
   `WIRESHARK_DIR`) from its Windows registry entry; the machine-wide action is
   conditioned on it.
3. Two pairs of custom actions mirroring the Defender pattern: an immediate
   action sets the deferred action's CustomActionData to the command line, and a
   deferred `WixQuietExec` action runs it with `Return="ignore"`. Per-user is
   `Impersonate="yes"` (so `fragcap.exe extcap install` targets the installing
   user's `%APPDATA%\Wireshark\extcap`); machine-wide is `Impersonate="no"` (so
   the elevated installer can write `[WIRESHARK_DIR]extcap`) and runs `extcap
   install --dir "[WIRESHARK_DIR]extcap"`. Each has a rollback that runs `extcap
   uninstall`, and an uninstall action removes the registration.

## Complexity Tracking

No constitution violations; no entries.
