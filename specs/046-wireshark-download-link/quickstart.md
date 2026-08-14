# Quickstart / Verification Guide: Wireshark download link in doctor

## Build and test gate (mechanical, CI parity)

```bash
cargo xtask ci
```

Runs fmt, clippy (all targets, all features, deny warnings), the workspace tests,
and the lint/deps/license checks. The new unit tests in
`crates/fragcap-cli/src/doctor/checks.rs` and the unchanged `doctor-ready` golden
in `crates/fragcap-cli/tests/cli_doctor.rs` are part of the test run.

## Targeted checks

```bash
cargo test -p fragcap-cli
```

Expected: the doctor classifier tests pass, including the new assertions that the
integration not-registered detail and the npcap remediation contain the Wireshark
download URL, and the `doctor-ready` golden is byte-identical (no regeneration).

## Single-source check

```bash
grep -rn "wireshark.org" crates/fragcap-core crates/fragcap crates/fragcap-cli/src
```

Expected: exactly one `wireshark.org` literal, in
`crates/fragcap-core/src/interface.rs` (the constant). The doctor guidance strings
reference the constant, not a second literal.

## Manual (optional, on a machine with the tool built)

Run `fragcap doctor` on a machine where the extcap integration is not registered:
the `analyzer extcap` row is an optional `warn` and its guidance now names both
`fragcap extcap install` and the Wireshark download URL, noting the installer also
provides npcap. On a ready machine the output is unchanged.

## Done signal

`cargo xtask ci` green, the single-source grep shows one literal, and the ready
golden is unchanged.
