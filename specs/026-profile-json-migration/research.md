# Phase 0 Research: Profile Format Migration

## Decision 1: Two-layer validation (reuse the S025 validator for structure)

**Decision.** `Profile::parse` parses JSON to a `serde_json::Value`, then runs two
non-overlapping validation layers that both accumulate into one `Diagnostics`:

1. **Structural** via `jsonschema::validate_json(&value)` (from S025). Owns
   types, required keys, enum ranges, unknown-key refusal, and the `schema` and
   `kind` discriminators. Its `SchemaDiagnostics` are mapped into this crate's
   `Diagnostics` (`SchemaCode` -> `DiagnosticCode`, JSON pointer -> `location`).
2. **fragcap-specific** in a lenient pass that extracts an all-optional `Draft`
   from the `Value` and runs only what a schema cannot express: compiling the
   `exe` glob, the `path_regex`, and the `capture.duration` literal, and the
   semantic graph checks already in `validate.rs` (acyclic `descends_from`, at
   most one terminal stage, at least one non-service stage, declared roles
   reachable, no ambiguous image match).

**Rationale.** There must be exactly one structural implementation, and it must be
the one bound to the published schema, or the profile-load path and the published
contract can drift. Reusing `jsonschema::validate_json` guarantees that a profile
fragcap loads is structurally identical to what the published schema accepts. The
fragcap pass is confined to the checks a JSON Schema provably cannot express, so
the two layers do not overlap and nothing is double-reported. Both accumulate, so
a profile with a mix of structural and semantic faults reports all of them in one
pass (SC-001, section 15.4).

**Alternatives considered.**

- **Rewrite `parse.rs`'s structural checks over serde_json and keep them (do not
  call the S025 validator).** Rejected: it creates a third thing to keep in sync
  (the published schema, the S025 validator, and a second structural
  implementation), which is exactly the drift the S025 slice's conformance test
  exists to prevent.
- **Deserialize straight into typed structs with serde derive and rely on serde
  errors.** Rejected: serde stops at the first error, which violates
  all-errors-at-once, the property section 15.4 makes non-negotiable.

## Decision 2: Diagnostic locations are JSON pointers; byte positions are dropped

**Decision.** Diagnostics on the profile-load path locate faults by JSON pointer
(RFC 6901, for example `/stage/1/match/exe`). The `Diagnostic` byte-offset and
line/column `position` fields are left `None` on this path.

**Rationale.** serde_json exposes a byte span only for the top-level parse error,
not for values inside the document, so a per-value line and column is not
available without a different parser. A JSON pointer names the exact value
unambiguously; the loss is the line and column, not the identity of the fault.
The `Diagnostic` type already carries an optional position, so this is a values
change, not a type change.

**Alternatives considered.**

- **Add a span-preserving JSON parser (`serde_spanned`, `jiter`, a spanned
  `Value`).** Rejected: it reintroduces a dependency to recover a line and column
  that the JSON pointer already localizes, against a crate whose whole point in
  this slice is to remove a dependency.
- **Compute offsets by re-scanning the text for the pointer.** Rejected: fragile
  and O(n) per diagnostic for a cosmetic gain.

## Decision 3: `DiagnosticCode` reuse; `Syntax` repurposed

**Decision.** Keep the existing `DiagnosticCode` enum. `Syntax` now means "not
valid JSON" (its doc comment is updated). The schema's new required keys map onto
existing codes: a missing `kind` or `fidelity` is `MissingField`; an out-of-range
`fidelity` maps to an existing code or a single added variant if none fits. No
code is removed and no check is lost.

**Rationale.** The public `DiagnosticCode` surface is what tests and the CLI key
on; preserving it keeps the migration a format change rather than an API break.

## Decision 4: `toml-span` removal is clean

**Finding.** `toml_span` is imported only in `fragcap-profile` (grep over the
workspace confirms no other crate references it). Removing it from
`fragcap-profile/Cargo.toml` drops it from the graph entirely. The dependency
inventory in AGENTS.md is updated to record the removal and the reason (the format
it parsed no longer exists).

## Decision 5: Capture output is unaffected (SC-006 verification)

**Finding.** The pcapng writer embeds a static `PROFILE_COMMENT`
(`"fragcap:profile=0.1.0"`), not the profile's raw text, and no writer embeds
profile source. Therefore changing the profile format from TOML to JSON does not
change any capture output, and the committed capture goldens do not move. SC-006
holds without touching goldens; the parity is verified by the corpus pipeline
tests continuing to reproduce the goldens.

## Decision 6: Required top-level keys on a migrated profile

**Decision.** Every JSON profile carries `schema`, `kind: "profile"`, and a
`fidelity`. Hand-authored examples and bundled profiles use `verified`; the Steam
scaffold uses `heuristic-unverified`. `provenance` is optional on a profile (it is
required only on the loose hint and export forms).

**Rationale.** The master schema requires `kind` and `fidelity` on every artifact;
a profile is the `profile` variant. `verified` is the honest tier for a
hand-authored, development-validated profile, and `heuristic-unverified` is the
honest tier for a machine guess (P-9).
