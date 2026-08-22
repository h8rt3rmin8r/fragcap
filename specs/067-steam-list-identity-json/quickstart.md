# Quickstart: validating S067

Prerequisites: a Windows machine (or the workspace's normal dev setup);
Steam does not need to be installed for the unit-level checks below, since
the join logic is tested against a fixture-backed store and a synthetic
`InstalledTitle` list, not a real Steam installation.

## Build and unit/integration tests

```sh
cargo build -p fragcap-cli -p fragcap-targets
cargo test -p fragcap-targets listing_snapshot
cargo test -p fragcap-cli steam
```

Expected: all green. The `fragcap-targets` run exercises the new
`listing_snapshot_position` alongside the existing `listing_snapshot_nth`
round-trip test; the `fragcap-cli` run exercises the three identity states,
sort order, `--json` record shape, and the store-absent fallback.

## Manual validation (requires a Windows machine with Steam installed)

```sh
fragcap targets
fragcap steam list
fragcap steam list --json
```

Expected, in order:

1. `fragcap targets` runs discovery, registers any newly found Steam titles,
   and writes a listing snapshot.
2. `fragcap steam list` shows a header, and every title that step 1
   registered shows its handle and (for the ones in the snapshot) its row
   index; the identity states are visibly distinct by sight.
3. `fragcap steam list --json` shows the same identity facts as
   newline-delimited JSON, plus the install directory the human table never
   showed.
4. `fragcap capture <n>` (any `<n>` from step 1's listing) still resolves to
   the same target it would have resolved to before running `steam list`,
   confirm by re-running `fragcap targets` and checking the row at position
   `<n>` did not move.

## What this does not validate

No live capture, no npcap dependency, this slice touches no capture or
attribution code, only the `steam list` inspection command and one new
read-only store query.
