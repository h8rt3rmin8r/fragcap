**2026-08-12** The profile format moved from TOML to JSON (issue #76), reversing
the S05 decision to parse profiles with `toml-span`. The reversal is deliberate:
S025 published a master JSON Schema and an all-errors-at-once validator, so the
runtime profile-load path now speaks the same JSON that the schema describes,
and `toml-span` is removed from `fragcap-profile`. Two consequences were
recorded rather than left implicit. First, the profile-load path reuses
`jsonschema::validate_value` for structural conformance so there is one
structural implementation bound to the published schema, and its
`SchemaDiagnostics` are mapped into the crate's existing `Diagnostics`; the
fragcap layer owns only the checks a schema cannot express (glob, regex, and
duration compilation, the stage-count limit, and the semantic graph checks), so
the two layers do not overlap. Second, diagnostic locations became JSON pointers
and byte-offset line/column positions were dropped, because serde_json exposes
no per-value span; the pointer names the exact value, and adding a
span-preserving parser would reintroduce a dependency to recover a line number
the pointer already localizes. `serde_json` was promoted to a runtime dependency
of `fragcap-steam` and `fragcap-cli` as well, so the scaffold and the ad-hoc tap
build their JSON through it rather than by hand; it was already in the graph, so
`Cargo.lock` gains no crate. MSRV stays 1.82.
