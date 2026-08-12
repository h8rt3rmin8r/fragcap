# Phase 0 Research: Master JSON Schema

## Decision 1: Publish a standard schema, validate internally by hand (do not embed a validator crate)

**Decision.** Author the master schema as a standard JSON Schema Draft 2020-12
document, embed it in the binary as the single source of truth, and publish it.
Perform fragcap's own internal validation by hand over a parsed
`serde_json::Value`, not through a JSON Schema validator crate. Bind the two with
a fixture-corpus conformance test so the published contract and the enforced
behavior cannot drift.

**Rationale.** The ecosystem value that motivated JSON plus JSON Schema (editors
validating against `$schema`, agents self-checking output, a validating
submission pipeline) is delivered by *publishing a standard schema document*. It
does not require fragcap's own binary to consume a heavyweight validator crate.
Fragcap already hand-rolls exactly this class of work (the `exe` glob matcher, the
pcap parser, the profile validator) specifically to keep the dependency graph
small, and the current profile validator already accumulates every diagnostic by
hand rather than short-circuiting. Hand-rolling structural checks over
`serde_json::Value` (type, required-key, enum, string-pattern, unknown-key
refusal, `kind` discrimination, all-errors-at-once with JSON-pointer locations)
is well within that established pattern and keeps the new footprint to a single,
already-vetted crate.

**Alternatives considered.**

- **`boon` v0.6.1 (a JSON Schema validator crate).** Pure Rust, Draft 2020-12,
  collects all errors, which fit the functional need. Rejected on dependency
  weight: adding it to `fragcap-profile` pulls **42 new transitive crates** into
  `Cargo.lock` (measured, not estimated), dominated by the ICU4X ecosystem
  (`icu_normalizer`, `icu_properties`, `icu_collections`, `icu_locale_core`,
  `zerovec`, `yoke`, `tinystr`, `zerocopy`, and more) reached through
  `url` -> `idna`, present to service `format: uri`/`idn-*` assertions and
  `$ref` resolution this schema does not need. This project rejected the
  mainstream `toml` crate over `toml-span` to avoid a single toolchain bump and a
  couple of transitive crates; 42 crates for structural validation is
  categorically out of character and hard to defend under P-8 and the AGENTS.md
  dependency inventory. The measurement command and result are preserved in the
  slice history.
- **`jsonschema` crate.** Comparable or heavier graph (also `url`, plus a regex
  engine and more). Same rejection on graph weight; no advantage over `boon` that
  would offset it.
- **`valico` / `jsonschema-valid`.** Draft-07 only and less actively maintained;
  rejected because the schema targets Draft 2020-12 and because unmaintained
  validation code is a poor foundation for a contract other subsystems depend on.
- **Hand-roll a general JSON Schema engine (interpret an arbitrary schema doc).**
  Rejected as more code and more drift surface than needed; fragcap only has to
  enforce *its own* schema, so direct hand-rolled validation plus a published
  descriptive schema is simpler and testable.

**Drift mitigation.** A conformance test runs a corpus of valid and deliberately
invalid fixtures (one per artifact variant, plus targeted violation cases)
through the hand-rolled validator and asserts the expected accept/reject outcome.
The published schema is the human-and-tooling-facing description; the corpus is
the machine-checked guarantee that the binary enforces what the schema advertises.
This mirrors the existing fixture-corpus drift check already in the gate.

## Decision 2: JSON Schema dialect is Draft 2020-12

**Decision.** The published schema declares Draft 2020-12
(`$schema: https://json-schema.org/draft/2020-12/schema`).

**Rationale.** It is the current standard, is what modern editors and agent
tooling assume by default, and expresses the `kind`-discriminated conditional
structure cleanly (`$defs` + `allOf`/`if`-`then` or `oneOf` on the discriminator).
Because fragcap does not consume the schema through a validator crate, the dialect
choice carries no internal dependency or MSRV cost; it is purely the external
contract's version.

## Decision 3: Artifact-form discrimination via an explicit `kind` field

**Decision.** Every document carries a required top-level `kind` (closed enum:
`profile`, `hint`, `package`, `export`) plus `schema` (integer version, currently
1). The validator selects the variant from `kind`.

**Rationale.** An explicit discriminator is unambiguous for both a validator and
an agent generating a file, and it cleanly separates `profile` and `package`
(which share a strict structural shape but differ in precedence/fidelity) from
`hint` and `export` (loose, partial, fidelity-and-provenance-bearing).
Overloading `fidelity` as the discriminator was rejected because it couples two
independent concerns and cannot express the profile/package structural identity.

## Decision 4: The single new runtime dependency is `serde_json` (promoted from dev)

**Decision.** Promote `serde_json` from a dev-only dependency to a runtime
dependency of `fragcap-profile`. Add no other runtime dependency. `serde` and
`regex` are already present.

**Dependency justification (AGENTS.md inventory).**

- **Crate**: `serde_json`. **Kind**: runtime (was dev-only since S07).
- **Why**: parse target files to a `Value` for hand-rolled structural validation,
  and serialize the embedded schema for `schema print`. It is already in the
  build graph as a dev dependency, so promotion adds no new crate to `Cargo.lock`.
- **MSRV**: `serde_json` and `serde` build at the workspace minimum of 1.82;
  confirmed by the existing dev usage already compiling under the gate. The MSRV
  risk that motivated verifying a validator crate does not arise, because no
  validator crate is taken.
- **License**: MIT OR Apache-2.0, already on the allowlist and already vetted by
  `cargo xtask license` and the `cargo deny` policy for its dev use.
- **Graph size**: zero new crates (promotion of an existing node).
- **Alternatives**: a bespoke JSON parser was rejected; JSON is not the
  arithmetic-over-bytes case that justified hand-rolling the pcap parser, and
  `serde_json` is the ecosystem standard already trusted in tests.

## Decision 5: MSRV gate

**Decision.** MSRV stays 1.82. Because the chosen approach adds no crate, the
1.82 build is unaffected beyond the `serde_json` promotion (already 1.82-clean).
`cargo xtask msrv` at 1.82 is run in verification; the 1.82 toolchain is
installed on the build machine, so the check executes rather than skipping.

**Note.** The `boon` MSRV question is moot under Decision 1. The evaluation that
produced the 42-crate measurement was performed and then reverted cleanly
(`cargo add boon --dry-run` and a full add/remove cycle; `Cargo.toml` and
`Cargo.lock` restored), so no trace remains in the tree.

## Open items carried to later slices (not this slice)

- The profile parser's move onto JSON and the rewiring of the semantic checks
  (acyclic ancestry, single terminal, role reachability, ambiguous image match)
  are #76.
- The resolver that reads `fidelity` and orders providers is #77.
- The hint database and its schema-conformant export producer are #78; this
  slice's `export` variant and the round-trip conformance requirement are the
  contract #78 builds against.
