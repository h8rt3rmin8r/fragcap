# Implementation Plan: correct the schema $id host to fragcap.com

**Branch**: `047-schema-id-host` | **Date**: 2026-08-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/047-schema-id-host/spec.md`

## Summary

Replace the schema `$id` host `fragcap.dev` with `fragcap.com` in the four
locations that carry it, keeping the two schema JSON copies byte-identical,
updating the CLI test that asserts the exact string, annotating the identity
contract, and recording the deliberate pre-1.0 identity correction as a dated
changelog decision.

## Technical Context

**Language/Version**: JSON (schema), Rust (a test assertion), Markdown (contract,
changelog). Workspace MSRV 1.82.

**Primary Dependencies**: none. A string change plus a test-string update.

**Testing**: `cargo test -p fragcap-profile` (the drift tests
`the_embedded_schema_matches_the_published_repository_copy` and
`print_output_equals_the_embedded_asset`), `cargo test -p fragcap-cli --test
cli_schema` (asserts the printed `$id`), then full `cargo xtask ci`.

**Target Platform**: N/A (identifier string in committed files).

**Project Type**: Rust workspace plus published schema doc.

**Constraints**: the two schema copies stay byte-identical (enforced by a drift
test); UTF-8, LF, no em/en dashes; record the decision because the S025 contract
declared the host immutable.

**Scale/Scope**: four one-line edits, one changelog decision fragment.

## Constitution Check

- **P-1 / P-2 / P-3**: No capture, core, or attribution behavior changes. PASS.
- **P-5 Compatibility Outranks Richness**: The schema stays valid and unchanged
  except for its identifier string; unmodified analyzers are unaffected. PASS.
- **P-6 Glossary First**: No new term. PASS.
- **P-8 House Standards**: UTF-8, LF, no dashes. PASS.
- **P-9 The Instrument Does Not Lie**: The schema now identifies itself with a
  domain the project actually owns instead of a nonexistent one. PASS.

No violations. The change touches no pinned artifact (`docs/schema/**` is not on
the pinned list); the recorded decision exists because the identity contract
declared the host immutable, which is a governance note, not a pinned-file gate.

## Project Structure

### Documentation (this feature)

```text
specs/047-schema-id-host/
├── spec.md
├── plan.md
├── research.md
├── quickstart.md
├── checklists/requirements.md
└── tasks.md
```

data-model.md and contracts/ omitted: no data entity and no new interface; the
schema's own contract already exists in specs/025 and is edited, not recreated.

### Source (paths touched)

```text
docs/schema/target-schema.v1.json                              # $id host -> fragcap.com
crates/fragcap-profile/assets/target-schema.v1.json            # identical edit (byte-identity)
crates/fragcap-cli/tests/cli_schema.rs                         # asserted $id string -> fragcap.com
specs/025-master-json-schema/contracts/master-schema.contract.md  # example host + dated note
changelog.d/047-schema-id-host.decisions.md                    # recorded identity correction
```

**Structure Decision**: Edit the two JSON files identically in one commit so the
drift test stays green; update the assertion in the same commit so the gate does
not go red between edits. The contract note is annotated (not deleted) so the
governance record shows the host was changed once, deliberately, before 1.0.

## Complexity Tracking

No constitution violations; no entries.
