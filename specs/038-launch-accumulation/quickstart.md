# Quickstart: validating launch-data accumulation (offline)

All validation is offline: no Steam, no network, no capture driver. The parser is
exercised against synthetic bytes; the orchestrator against a fixture Steam-root
tree and an in-memory store.

## Prerequisites

- The workspace builds: `cargo build --workspace`.
- The `targets` feature is what carries the store; the facade end-to-end test
  runs under `--features targets` (the CLI enables it unconditionally).

## Run the checks

```bash
cargo test -p fragcap-steam appinfo
cargo test -p fragcap-targets merge_launch
cargo test -p fragcap --features targets launch_accumulation
cargo xtask ci
cargo xtask msrv        # exits 2 if the 1.82 toolchain is absent; green when present
```

## Scenarios proven

1. **Parse a synthetic appinfo file** (`fragcap-steam`): a generated file with
   one app carrying a single Windows launch executable parses to one
   `AppInfoApp` with the right appid, change-number, and one entry. Covers the
   inline-key (v27/v28) and string-table (v29) header variants.
2. **Verbatim multi-entry** (`fragcap-steam`): an app with several os-filtered
   launch entries parses to that many entries in order, each with its oslist,
   osarch, and betakey preserved (P-9, FR-005).
3. **Section failure isolates and resyncs** (`fragcap-steam`): a file whose
   middle section is malformed yields the good apps plus one `AppInfoFailure`,
   proving the size-framing resync (FR-008).
4. **First-run write** (`fragcap` facade): a fixture root with two installed
   appmanifests and a synthetic appinfo file, against an empty in-memory store,
   ends with launch rows for both apps and a summary of `written = 2`,
   `considered = 2`, conserved.
5. **Second-run skip** (`fragcap` facade): re-running against the unchanged
   fixture reads no section into the store and reports `skipped = 2`,
   `written = 0`.
6. **Refresh on change-number bump** (`fragcap` facade): advancing one app's
   change-number in the fixture and re-running re-writes exactly that app
   (`written = 1`, `skipped = 1`) and leaves the other untouched.
7. **Tiers preserved** (`fragcap-targets`): a game pre-seeded with catalog and
   engine columns, after `merge_launch`, keeps its name, metrics, and engine, and
   gains only launch rows and the change-number.
8. **Migration is backward-safe** (`fragcap-targets`): a store created at
   schema v1 (DDL without the new column) opens under this build, migrates to v2
   with existing rows' `appinfo_change_number` NULL, and reads and writes
   correctly afterwards.
9. **Conservation under mixed outcomes** (`fragcap` facade): a fixture mixing
   writable, already-current, malformed, and appinfo-absent installed apps yields
   a summary whose four buckets sum to `considered` (FR-007, SC-004).

## Expected outcome

All listed tests pass; `cargo xtask ci` is green; `Cargo.lock` is unchanged
(`git diff --exit-code Cargo.lock`).
