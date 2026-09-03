# Compatibility Store Version 10 Contract

## Additive migration

Schema version 10 adds nullable columns to `deep_capture_facts`:

```text
routing_strategy TEXT NULL
address_family   TEXT NULL
protocol_family  TEXT NULL
```

Each column has a closed CHECK vocabulary. The migration performs no UPDATE, DELETE, backfill, or table rebuild. Every v9 row therefore retains the same id, target relation, fact key and value, provenance, versions, owner details, stale bit, and note.

## New-row validation

Observed S121 calibration rows require a routing strategy, address family, and explicit protocol applicability. Protocol-independent rows store `not-applicable`; protocol-specific rows store one concrete S120 family. Rust validation and SQLite constraints reject empty or out-of-set tokens.

## Current-case selection

An exact case comparison evaluates launch case, backend name, backend version, routing strategy, address family, fragcap version, and known target version. Protocol-specific prerequisites additionally compare protocol family. Missing required fields produce `legacy-incomplete`, stale signals produce `stale`, and unequal fields produce a named mismatch.

The latest applicable row for a prerequisite is the greatest durable row id after filtering. No query deletes, updates, coalesces, or aggregates conflict history.
