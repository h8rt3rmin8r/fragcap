# Quickstart: validate slice 043

This slice changes the Windows installer, which cannot be built here. Validation is
split into what runs in this environment and what the operator runs at the halt.

## In this environment

1. Repo gates:

   ```bash
   cargo xtask ci
   ```

   Expect green. The repo lint covers the WiX XML text (UTF-8, LF, no dashes).

2. WiX well-formedness (structure parses; not a build):

   ```bash
   python -c "import xml.dom.minidom,sys; xml.dom.minidom.parse('crates/fragcap-cli/wix/main.wxs'); print('main.wxs well-formed')"
   ```

3. Release consistency: confirm the release workflow still invokes `cargo wix` over
   `crates/fragcap-cli/wix/main.wxs` with no added extension flags.

## Manual, at the pre-push halt (operator, needs the WiX toolchain and Windows)

1. Build the MSI as the release job does (cargo-wix over `main.wxs`); it must build.
2. Per-user: run the MSI as a normal user with the register option selected, then
   `fragcap doctor`: the analyzer extcap row is `ok`, pointing at that user's
   `%APPDATA%\Wireshark\extcap`.
3. Machine-wide: on a box with Wireshark installed, run the MSI with the machine-wide
   option, then `fragcap doctor` as a second user: the row is `ok`.
4. Failure path: on a box without Wireshark, select machine-wide; the install still
   succeeds and nothing partial is left behind.
5. Skip path: leave the option unselected; nothing is registered, and `doctor` shows
   the optional warning with the `fragcap extcap install` guidance.

## Expected outcomes (maps to Success Criteria)

- SC-001 / SC-002: per-user and machine-wide installs make `doctor` report `ok`
  (manual).
- SC-003: a failed registration leaves the install successful (manual).
- SC-004: skipping registers nothing; installer and docs state the fallback and the
  per-user scope.
- SC-005: `cargo xtask ci` green; `main.wxs` well-formed; release workflow consistent.
- SC-006: no CLI surface changed.
