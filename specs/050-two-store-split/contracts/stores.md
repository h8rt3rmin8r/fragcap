# Contracts: the two-store surface (S050)

## Files (AppData root, `%APPDATA%\fragcap\`)

- `catalog.db`: shipped, disposable, read-only from user actions.
- `local.db`: user-owned, durable, written by learned accumulation.

Invariant: the two are separate files sharing no storage; an operation on one
never reads or writes the other. Replacing `catalog.db` leaves `local.db`
byte-identical.

## Command line (`fragcap run` / capture path)

| Flag | Meaning | Default |
| --- | --- | --- |
| `--catalog-db <path>` | override the catalog store path | `%APPDATA%\fragcap\catalog.db` |
| `--local-db <path>` | override the local store path | `%APPDATA%\fragcap\local.db` |

- Env fallbacks `FRAGCAP_CATALOG_DB` / `FRAGCAP_LOCAL_DB`; flag wins.
- `--hint-db` and `FRAGCAP_HINT_DB` are removed with no alias.
- An explicitly named path that is absent is not created and is not an error
  (unchanged from the single-store behavior); a present-but-unopenable one is a
  loud error at the boundary.
- The defaulted paths are bootstrapped on first run; an explicitly named path is
  never created on the user's behalf.

## Resolution

- Learned launch data resolves a title the same as before the split; `local.db`
  is consulted before `catalog.db`.
- Both stores absent yields the same result as a build with no store (no hint
  provider); no error.

## doctor

- The data-directory report names both `catalog.db` and `local.db`, each with its
  path and whether it is present. No single `hint.db` line remains.

## Packaging

- The MSI installs the seed as `catalog.db` beside the binary.
- The release archive and the loose-download and checksum steps name `catalog.db`.

## Backward compatibility

- No migration from `hint.db`; an old one is ignored.
- No capture, attribution, or output-format change; only where learned data is
  stored and read moves.
