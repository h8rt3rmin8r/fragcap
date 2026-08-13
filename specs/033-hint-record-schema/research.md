# Phase 0 Research: Target-Hint-Record Schema Revision

## R1. JSON object shape vs the research's SQL tables

- **Decision**: Model the research's `launch_entries` table as a `launch` array of
  entry objects (the `appid`/`launch_index` foreign key and ordering become array
  membership and position), and the `games` engine columns (`engine`,
  `engine_source`, `engine_confidence`) as one `engine` object with `name`,
  `source`, `confidence`. `launcher_mediated` stays a scalar boolean.
- **Rationale**: The schema governs a per-title JSON document, not a relational
  store; a nested array and object are the natural JSON of a one-to-many launch
  list and a grouped engine attribution. The field names and semantics are
  preserved so #78 can map columns to JSON mechanically.
- **Alternatives considered**: Three flat `engine_*` top-level fields mirroring
  the columns (rejected: an `engine` object groups the attribution and its
  provenance/confidence together and reads honestly as one unit); a foreign-keyed
  separate launch document (rejected: the schema is one self-contained artifact
  per title).

## R2. Free-string filters vs enums on launch entries

- **Decision**: `os`, `osarch`, `launch_type`, `beta_branch` are optional free
  strings (`type: string`), not enums. Only the engine `source` and `confidence`
  are enums.
- **Rationale**: Steam's launch-filter vocabularies are external and evolve; an
  enum would reject a valid Steam value the moment Steam adds one, a correctness
  cost with no honesty benefit. The engine source and confidence vocabularies, by
  contrast, are fixed by this project's own research, so constraining them catches
  a real authoring or seeding error.
- **Alternatives considered**: Enumerating known Steam os/arch values (rejected:
  brittle against Steam's evolution and offering nothing, since the resolver does
  not branch on unknown values).

## R3. Engine confidence vs record fidelity (the P-9 reconciliation)

- **Decision**: Keep the record `fidelity` enum exactly as is
  (authored, verified, heuristic-unverified, observed) and add engine `confidence`
  (confirmed, high, medium, low, unknown) as a separate field on the `engine`
  object. The engine `source` (pcgamingwiki, exe_heuristic, depot_filename_rules)
  is separate from the record's provenance `source` (a free string). No field is
  required to match another.
- **Rationale**: The research's confidence vocabulary grades one heuristic field
  (the engine guess), while fidelity grades the whole record's trust. Remapping
  confidence onto fidelity, or making a low-confidence engine lower the record's
  fidelity, would let a single guessed field silently move the trust of the whole
  record, which P-9 forbids. Carrying both as independent fields is the honest
  model #83 explicitly allows ("or extend the target-hint-record subschema to
  carry both").
- **Alternatives considered**: A fifth fidelity tier for engine confidence
  (rejected by the memory and #83: engine_confidence is a within-tier gradation);
  reusing the provenance object for engine source (rejected: provenance is the
  record's origin, the engine source is one field's origin).

## R4. Gating the fields to the loose variants

- **Decision**: Add the three to top-level `properties` and to `$defs/record`,
  then an `allOf` conditional forbids them on `profile`, `package`, and `export`
  at the top level, mirroring the existing `records: false` gate. The hand-rolled
  validator's `allowed_top_keys` puts them on the `Hint` arm only, and
  `check_records` adds them to a record's allowed keys.
- **Rationale**: A hint is a single loose record (top level); an export is an
  envelope whose per-title metadata lives in each record. This places the fields
  exactly where a loose record lives and keeps the strict authored format clean,
  using the schema's own established gating idiom.
- **Alternatives considered**: Allowing them on all variants like `technologies`
  (rejected: `technologies` is install-observed metadata a profile could
  legitimately carry, whereas launch/engine are hint-DB seeding metadata that
  would blur the authored-vs-guessed line on a profile); a separate hint-only
  schema (rejected: defeats the shared-core design).

## R5. Diagnostic codes

- **Decision**: Add `InvalidEngineSource` and `InvalidEngineConfidence` to the
  schema diagnostic enum, mapped in `parse.rs` to `DiagnosticCode::WrongType`, as
  S031 did for `InvalidCategory`. A launch entry missing `executable` reuses
  `MissingField`; an empty `executable` reuses `EmptyString`; an unknown key in any
  new object reuses `UnknownKey`; a wrong type reuses `WrongType`.
- **Rationale**: The project uses specific codes for out-of-enum values
  (`InvalidMode`, `InvalidLifecycle`, `InvalidCategory`), so a bad engine source or
  confidence gets its own named code for a clear diagnostic and a testable
  assertion; the structural failures reuse the existing generic codes.
- **Alternatives considered**: One generic `InvalidEnum` code (rejected: the
  existing style is a specific code per enum field, and specific codes make the
  conformance assertions precise).
