# Quickstart: Validate Cold Platform-Client Ownership

## Prerequisites

- Windows 10 or later for the real adapter.
- A locally installed Steam client and a stored Steam target for manual tier-2 validation.
- No running Steam process for the cold path.
- No real credentials, account names, library inventories, or game captures in committed tests.

## Offline Contract Tests

Run the focused suites:

```powershell
cargo test -p fragcap managed_launch
cargo test -p fragcap session
cargo test -p fragcap-cli target_resolve
cargo test -p fragcap-cli deep_capture
```

Expected outcomes:

- a synthetic canonical platform plan preserves exact root and application dispatch;
- title dispatch remains pending until the platform role binds;
- repeated platform events dispatch exactly once;
- warm platform presence refuses before effects;
- platform exit, dispatch failure, escaped client, ambiguity, watcher loss, and timeout remain distinct;
- routing and propagation facts vary independently;
- ordinary Capture Steam protocol behavior remains unchanged.

## Controlled End-to-End Validation

Run the existing controlled Deep Capture integration suite:

```powershell
cargo test -p fragcap-cli --test cli_deep_capture --all-features
```

The controlled harness must use synthetic executable names, application identifiers, process timelines, and loopback traffic. It must not enumerate or serialize a real Steam account or library.

## Full Repository Gate

```powershell
cargo xtask ci
cargo xtask msrv
cargo xtask neutral
```

All commands must complete successfully. The dependency and lint gates must show no new runtime package, no prohibited target-process API, and no transmit path.

## Optional Real Steam Validation

On an authorized development machine with Steam fully stopped, run Deep Capture against a deliberately selected stored test title and inspect the emitted plan before confirmation. Confirm that the plan names the canonical local `steam.exe`, child-only routing, one application identifier, a finite launch deadline, and separate routing and propagation evidence. Do not commit the resulting bundle or local compatibility database.
