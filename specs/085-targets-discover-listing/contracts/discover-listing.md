# Contract: Human Discovery Listing

`fragcap targets discover` and the discovery portion of `fragcap targets scan` render existing discovery results as human output.

## Store Block

The command prints a labelled block before candidate rows:

```text
Discovery stores:
  catalog: C:\Users\name\AppData\Local\fragcap\catalog.db
  local:   C:\Users\name\AppData\Local\fragcap\local.db
```

For `targets scan`, only stores resolved by that command are printed before the shared discovery listing when the caller has them. The shared discovery printer itself does not invent paths.

## Candidate Table

A non-empty discovery result prints a table:

```text
  SOURCE  IDENTITY   FIDELITY               NAME
  steam   steam:620  heuristic-unverified   Portal 2
    engine: Source (verified)
```

Rules:

- The visible columns are source, identity, fidelity, and name.
- There is no classification column in the human table.
- Rows contain spaces, not tab characters.
- Every column except the final name column is padded to the widest rendered content or heading.
- No value is truncated or wrapped.
- Evidence lines are indented under the owning row and preserve category, product, and fidelity.

## Empty Result

An empty result prints:

```text
No candidates discovered.
```

The discovery account block still follows.

## Account Block

The account block prints required totals and grouped zero outcomes:

```text
Discovery account:
  considered: 32
  produced: 30
  not a game: 2
  zero: parse failed, declined, container descended, container descent truncated, volume skipped, access error
```

Rules:

- `considered` and `produced` always print.
- Non-zero outcomes print as individual labelled lines.
- Zero outcomes are grouped in at most one `zero:` line when any zero outcomes exist.
- `container descended` and `container descent truncated` remain separate labels.

## Diagnostics

Warnings remain diagnostics owned by the emitter. Human warnings stay on standard error, JSON warnings stay structured, and stdout remains the command-result stream.
