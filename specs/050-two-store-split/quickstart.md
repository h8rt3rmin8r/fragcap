# Quickstart: validating S050

Run under the GNU-host toolchain in this environment
(`cargo +1.96.0-x86_64-pc-windows-gnu ...`); CI runs the pinned msvc build. See
[contracts/stores.md](contracts/stores.md) and [data-model.md](data-model.md).

## 1. Fresh run yields both stores in AppData

Point AppData at a scratch dir and run a capture (offline replay is fine):

```bash
FRAGCAP_TESTMODE... (or set APPDATA to a temp dir), then run `fragcap run ...`
```

Expected: `catalog.db` and `local.db` both exist under `<appdata>\fragcap\`, with
no elevation. With a `catalog.db` template beside the binary, catalog is seeded
from it; otherwise it is empty. `local.db` is empty.

## 2. Learned data lands only in local.db

Run a capture that accumulates learned launch data (a machine with a Steam
appinfo cache). Hash `catalog.db` before and after.

Expected: `catalog.db` is byte-identical before and after; `local.db` changed.

## 3. A catalog refresh leaves local.db byte-identical

Hash `local.db`, replace `catalog.db` with a different file, hash `local.db`
again.

Expected: identical hashes. No operation on `catalog.db` touched `local.db`.

## 4. Resolution parity (no attribution regression)

A title resolvable only from learned launch data resolves to the same client
after the split as a single-store build did before.

Expected (unit/integration): the layered provider returns the learned client from
`local.db`; a seed-only title resolves from `catalog.db`.

## 5. CLI surface

```bash
cargo +1.96.0-x86_64-pc-windows-gnu run -p fragcap-cli -- run --help
```

Expected: `--catalog-db` and `--local-db` are present; `--hint-db` is gone.

## 6. doctor names both stores

```bash
cargo +1.96.0-x86_64-pc-windows-gnu run -p fragcap-cli -- doctor
```

Expected: the data-directory section lists `catalog.db` and `local.db`.

## 7. Whole gate set

```bash
cargo +1.96.0-x86_64-pc-windows-gnu fmt --all -- --check
cargo +1.96.0-x86_64-pc-windows-gnu clippy --workspace --all-targets
cargo +1.96.0-x86_64-pc-windows-gnu test --workspace
cargo +1.96.0-x86_64-pc-windows-gnu run -p xtask -- lint
cargo +1.96.0-x86_64-pc-windows-gnu run -p xtask -- spec
cargo +1.96.0-x86_64-pc-windows-gnu run -p xtask -- docs check
```

Expected: all pass, including the new `catalog.db`/`local.db` glossary entries.
