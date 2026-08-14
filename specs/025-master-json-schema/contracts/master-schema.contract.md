# Contract: the master schema document

The published, embedded artifact. A standard JSON Schema Draft 2020-12 document.
This contract fixes its shape and the rules a conformant validator (fragcap's
hand-rolled one, or any external validator) must enforce. The authoritative
artifact ships at `crates/fragcap-profile/assets/target-schema.v1.json`.

## Identity

- `$schema`: `https://json-schema.org/draft/2020-12/schema`
- `$id`: a stable fragcap URI for version 1, for example
  `https://fragcap.com/schema/target/v1.json`. The host is fixed at authoring
  time and, once published, changed only by a deliberate recorded decision. It
  was corrected once before 1.0, from `fragcap.dev` to `fragcap.com`, because
  `fragcap.dev` was never a domain the project owned; see
  `changelog.d/047-schema-id-host.decisions.md`.
- Top-level `type`: `object`.
- `unevaluatedProperties: false` (or `additionalProperties: false` at each
  object) so unknown keys are refused everywhere.

## Structure

- Top-level requires `schema`, `kind`, `fidelity`.
- `$defs` holds the shared core (`game`, `capture`, `stage`, `match`,
  `provenance`, `fidelity` enum) and the four variant subschemas.
- Discrimination: an `allOf` with an `if` on `kind` selecting each variant's
  `then`, or a `oneOf` over the four variant subschemas keyed on `const kind`.
  The `profile` and `package` variants reference the same strict core subschema.
  The `hint` and `export` variants reference the loose subschema and additionally
  require `provenance`.

## Rules the schema encodes (structural)

- `schema` is an integer and, for version 1, `const 1`.
- `kind` is one of `profile`, `hint`, `package`, `export`.
- `fidelity` is one of `authored`, `verified`, `heuristic-unverified`,
  `observed`.
- `game.id` matches `^[a-z0-9_-]+$`; `game.name` is a non-empty string.
- `stage` is a non-empty array in strict variants; each stage requires `role`,
  `lifecycle` (`transient`|`session`|`service`), and `match`.
- `match` requires at least one of `exe`, `path_contains`, `path_regex`,
  `cmdline_contains`, `descends_from` (`minProperties: 1`).
- `hint` and `export` require `provenance` with a non-empty `source`.
- No additional/unknown properties anywhere.

## Rules the schema deliberately does NOT encode (semantic, out of scope here)

Acyclic `descends_from`, at most one `terminal` stage, at least one non-service
stage, no ambiguous image match, and regex/glob/duration compilation. These are
not expressible in JSON Schema and remain the profile-load path's responsibility
(#76). A validator passing this schema is asserting structural conformance only,
and the tooling says so.

## Text hygiene

Every `description` string in the schema is UTF-8, contains no em-dashes or
en-dashes, and the file is LF-terminated without a BOM.

## Conformance obligation

fragcap's hand-rolled validator MUST accept exactly the documents this schema
accepts and reject exactly those it rejects, proven by a shared fixture corpus
run through the validator in tests. The schema is the contract; the validator is
a tested implementation of it.
