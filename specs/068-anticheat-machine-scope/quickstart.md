# Quickstart: validating S068

## Build and unit/integration tests

```sh
cargo build -p fragcap-profile -p fragcap-steam -p fragcap-targets -p fragcap -p fragcap-cli
cargo test -p fragcap-profile signature
cargo test -p fragcap-steam anti_cheat
cargo test -p fragcap-targets machine_probe
cargo test -p fragcap detection
cargo test -p fragcap-cli targets
```

Expected: all green. Together these cover the new signature rows against the
`all_markers_tree` fixture, the launch-entry classifier's positive and MCC
negative cases, the probe trait via its fixture implementation, the
discovery-time merge, and the CLI's `Machine:` rendering.

## Manual validation (requires a Windows machine with an EAC-EOS title installed)

```sh
fragcap targets
```

Expected:

1. Any title shipping an `EasyAntiCheat/`, `EasyAntiCheat_EOS/` directory or
   an `EACLaunch.exe`/`AntiCheatInstaller.exe`/`start_protected_game.exe`
   executable reports Easy Anti-Cheat in its SENSITIVITIES column.
2. If the `EasyAntiCheat_EOS` service is registered on the machine (Task
   Manager > Services, or `sc query EasyAntiCheat_EOS`), a `Machine:` section
   appears after the table naming Easy Anti-Cheat, and no target row changes
   as a result.
3. A title shipping only `EOSSDK-Win64-Shipping.dll` (no EAC artifact) shows
   no anti-cheat product.

## What this does not validate

The Windows registry adapter (`WindowsMachineAntiCheatProbe`) is exercised
only by this manual step and by the tier-2-style facade test that calls the
real probe and asserts it does not panic (inconclusive on a machine with no
EAC service registered, matching the `windows_volumes.rs` precedent). CI
runs on hosts with no EAC installed, so the automated suite exercises the
probe's *seam* (via the fixture implementation), not the real registry read
finding anything.
