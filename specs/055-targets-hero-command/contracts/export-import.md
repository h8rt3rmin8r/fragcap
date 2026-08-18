# Contract: target-entry export / import

**Feature**: S055 | **Date**: 2026-08-18

Operator decision (2026-08-18): a dedicated target-entry array, not the published
capture schema (`target-schema.v1.json`).

## Document

A JSON array; each element is one target entry. Required per element: `stable_id`,
`handle`, `name`, `classification`, `classification_source`, `fidelity`. Optional
(emitted only when present): `anchor`, `launch_entries`, `install_root`,
`evidence`, `provenance`.

```json
[
  {
    "stable_id": 306130,
    "handle": "the_elder_scrolls_online",
    "name": "The Elder Scrolls Online",
    "classification": "game",
    "classification_source": "user",
    "fidelity": "authored",
    "anchor": "steam:306130",
    "launch_entries": [ { "executable": "eso64.exe", "role": "client" } ]
  }
]
```

Field value domains mirror the `targets` table CHECKs (classification,
classification_source, fidelity enums; handle not purely numeric). The mapping is
explicit (hand-written, like export.rs), so the JSON key set is a reviewed
contract rather than a serde-derive accident.

## `export [SELECTOR]`

- No selector: every registered entry, ordered by handle (stable output).
- With a selector: a one-element array (or empty array on a clean miss, exit 0).
- Ambiguous name: lists matches, refuses, exit 2 (does not export an arbitrary
  one).
- Emitted to stdout, pretty-printed, trailing newline (matches export.rs habit).

## `import <FILE>`

- Parses the array; validates each element structurally (required fields present,
  enum values legal). A nonconforming file is rejected with diagnostics and
  applies **nothing** (all-or-nothing; no partial import) (FR-019).
- Merge on `stable_id`: an element whose `stable_id` already exists updates that
  row in place; a new `stable_id` inserts (reusing `insert_target`, with handle
  disambiguation on collision).
- Reports counts (inserted, updated).

## Round-trip guarantee (FR-020, SC-005)

Export a store, import the document into a fresh store: the id set is identical and
there are no duplicate rows. Importing the same document twice into the same store
is idempotent on identity (updates in place, never duplicates). This is the
primary acceptance test for the surface.
