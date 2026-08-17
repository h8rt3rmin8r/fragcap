# Contract: signature table and seed

The catalog-side table that carries detection signatures as data, and the seed that
fills it from the bundled Appendix B document.

## Table

`signature(id, category, kind, pattern, product, confidence)` in the shared schema,
populated in `catalog.db`. See [data-model.md](../data-model.md) for field
semantics and CHECK constraints. Added by additive `MIGRATE_4_TO_5`; schema version
becomes 5.

## Seed

- `seed_signatures(store, source)` loads a signature document into the table,
  replacing the prior signature rows so a refresh is idempotent (re-seeding yields
  the same table).
- The default source is the bundled `fragcap-targets/assets/signatures.json`,
  parsed with `serde_json`. Offline and deterministic, like `targets seed`.
- CLI surface: `targets seed-signatures --db <catalog.db>`, alongside the existing
  `targets seed` and `targets seed-engine`.

## Guarantees

- After a fresh seed, every Appendix B product has at least one row (SC-001).
- A signature row of an implemented kind added to the table is honored on the next
  scan with no code change (FR-004, SC-002).
- Loading reports applied, inert, and skipped counts; their sum equals the rows
  loaded (data-model invariant). Inert (binary-marker) and skipped (malformed) rows
  are surfaced, never dropped (FR-013).
- A refresh of the catalog refreshes signatures through the same seed path (FR-005).

## Load

`Store::load_signatures() -> SignatureSet` reads the table and partitions rows into
applied / inert / skipped. It is the only path from the table to the matcher; the
matcher itself takes a `&[Signature]` and never touches the database (D1).
