# Quickstart: Watch / Attach Mode

All three scenarios are exercised offline through the hidden `OfflineArgs`
substrate, with no capture driver, no elevation, and no game.

## 1. Capture a modded-Skyrim-shaped target (US1)

```bash
# The identity is exe plus a path anchor; the target launches from an arbitrary
# parent outside any storefront directory, no steam:// involved.
fragcap watch --exe SkyrimSE.exe --path "Mod Organizer 2" --out out.pcapng \
  --replay-source fixtures/... --process-script skyrim-modded.script
```

Test: a `ProcessScript` where `SkyrimSE.exe` under `...\Mod Organizer 2\...`
starts from an arbitrary parent; assert it acquires and captures, and that the
same script with a non-matching path anchor does not.

## 2. Attach to an already-running game (US2)

```rust
// A ProcessScript whose startup snapshot already contains the target, with no
// later start event:
ProcessScript::new().with_snapshot(vec![
    ProcessRecord::new(1234, 0, "C:\\Games\\ESO\\eso64.exe"),
]);
// watch --exe eso64.exe acquires at arm from the snapshot.
```

Test: assert acquisition happens at arm (before any streamed event), and that the
S027 ObservationProvider resolves the identity over the snapshot to an `observed`
target naming pid 1234.

## 3. Give up loudly (US3)

```bash
fragcap watch --exe never.exe --wait 5s ...   # no process ever matches
```

Test: assert the run ends `StopReason::AcquisitionTimeout`, the summary reports
the watch-time discard accounting, and the exit is 1; and that firing an interrupt
during the watch exits 0 instead.

## 4. Verify

```bash
cargo xtask ci
cargo xtask msrv     # 1.82
cargo xtask neutral
```
