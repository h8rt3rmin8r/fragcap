# WiX and Release Gate Checklist: Slice 043

**Purpose**: Gate the installer change against the D-4 design, the release-adjacent
pinned-artifact rule, and the constitution before commit.
**Created**: 2026-08-14
**Feature**: [spec.md](../spec.md)

## D-4 design fidelity

- [x] The register control is optional and never forces registration.
- [x] Per-user registration is a deferred, user-IMPERSONATED custom action
      running `fragcap.exe extcap install` (not SYSTEM).
- [x] Machine-wide registration is gated on a Wireshark RegistrySearch and runs
      `fragcap.exe extcap install --dir <WiresharkDir>\extcap`.
- [x] No new `fragcap` CLI surface; the installer drives existing `extcap
      install` and `--dir` only.

## Robustness (mirror the Defender pattern)

- [x] Registration custom actions use `Return="ignore"` so a failure never fails
      the install.
- [x] A paired rollback unregisters on a later install failure.
- [x] Machine-wide degrades to a clean no-op when Wireshark is absent.

## Installer and docs text

- [x] The wizard text states per-user registration is for the current user only.
- [x] The wizard text and the docs name `fragcap extcap install` as the fallback.
- [x] The CLI/installer reference and getting-started document the option and
      both scopes; the slice 042 dependency model (extcap optional) is unchanged.

## Pinned artifact and consistency

- [x] `main.wxs` is well-formed XML (parses).
- [x] The release workflow and `release.toml` still reference the same
      `main.wxs` consistently.
- [x] A dated `changelog.d/<key>.decisions.md` fragment records the MSI extcap
      decision, plus a `.added.md` feature fragment.

## Constitution and verification

- [x] No em or en dashes anywhere added; UTF-8, LF, no BOM.
- [x] No Rust CLI surface or runtime behavior changed.
- [x] `cargo xtask ci` is green.
- [ ] MANUAL (operator, at halt): WiX build succeeds; per-user install registers
      into the user's Wireshark extcap dir; machine-wide install registers into
      the system dir; a failed registration leaves the install successful.
