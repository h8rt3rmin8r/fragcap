A master JSON Schema (Draft 2020-12) now governs every machine-readable
targeting and attribution artifact under one versioned vocabulary: a profile, a
hand-authored package, a heuristic hint, and a hint-database export share a
single schema discriminated by a top-level `kind`, each carrying a structured
`fidelity` tier (authored, verified, heuristic-unverified, observed) and, for the
loose forms, `provenance`. The schema is embedded in the binary as the single
source of truth, published under `docs/schema/`, and rendered on the
documentation site. A new `fragcap schema validate <file>` validates any JSON
file against it and reports every structural violation in one pass, distinguishing
a JSON syntax error from a schema violation; `fragcap schema print` emits the
embedded schema. Structural validation (types, required keys, enums, unknown-key
refusal, the discriminators) lives in the schema; the semantic invariants of
profile loading (acyclic ancestry, a single terminal stage, role reachability, no
ambiguous image match) remain the profile-load path's responsibility, and the
seam is documented (issue #75).
