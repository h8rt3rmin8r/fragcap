# Implementation Plan: Profile Format Migration from TOML to JSON

**Branch**: `feat/profile-json-migration` | **Date**: 2026-08-12 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/026-profile-json-migration/spec.md`

## Summary

Move the runtime profile-load path from TOML onto the master JSON Schema shipped
in S025. `Profile::parse` still takes text and returns either a validated
`Profile` or the complete `Diagnostics` set, but the text is now JSON. Structural
conformance is delegated to `jsonschema::validate_json` (one structural
implementation, bound to the published schema); a lenient fragcap pass extracts a
`Draft` from the parsed `serde_json::Value` and runs only what a schema cannot
express (glob, regex, and duration compilation, and the semantic graph checks in
`validate.rs`). Both layers accumulate into one `Diagnostics`, reported in a
single pass. Diagnostic locations become JSON pointers; byte-offset positions are
dropped because serde_json exposes no per-value spans. `toml-span` is removed.

The Steam scaffold emits JSON carrying `kind: "profile"`, `fidelity:
"heuristic-unverified"`, and a `notes` string holding the verification warning
that is a TOML comment today. Examples, fixtures, the `.toml` test data file, the
resolution extension, master spec section 15, and the glossary follow.

## Technical Context

**Language/Version**: Rust, MSRV 1.82 (toolchain pinned 1.96)

**Primary Dependencies**: `serde_json` (already runtime in fragcap-profile from
S025), `regex` (already runtime). **Removed**: `toml-span`. No new dependency.

**Storage**: JSON profile files on disk.

**Testing**: `cargo test`, `cargo xtask ci`, `cargo xtask msrv` at 1.82.

**Target Platform**: platform-neutral crate (fragcap-profile).

**Project Type**: Rust workspace (library crates + CLI).

**Performance Goals**: profile load is interactive; not a hot path.

**Constraints**: MSRV 1.82 green; all-errors-at-once across both layers; UTF-8 no
BOM, LF, no em/en dashes; capture output byte-identical before and after (SC-006).

**Scale/Scope**: one crate's parser rewritten; ~12 files carrying inline TOML
profile literals reauthored to JSON; one `.toml` data file; scaffold renderer;
resolution extension; spec and glossary.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive Observation**: Not engaged; reading and validating files. PASS.
- **P-2 Core Stays Platform-Neutral**: fragcap-profile stays neutral; the change
  removes `toml-span` and adds no dependency (`serde_json` already runtime). PASS.
- **P-3 Capture And Attribution Stay Separate**: Not engaged. PASS.
- **P-4 No Silent Loss**: Directly served. All-errors-at-once is preserved across
  both validation layers; unknown keys stay refused; every check the TOML path
  ran runs on the JSON path (parity is a success criterion). PASS.
- **P-5 Compatibility Outranks Richness**: The profile is input, not output; the
  pcapng and JSON Lines writers are untouched, and capture output stays
  byte-identical (SC-006). PASS, with one verification item (see research: confirm
  no writer embeds the profile's raw text).
- **P-6 Glossary First**: The Game profile glossary entry is reconciled to JSON in
  this change. PLANNED.
- **P-7 Wrappers Stay Thin**: The `profile` CLI keeps its shape; only the format
  it loads changes. PASS.
- **P-8 House Standards Apply**: `toml-span` removed from the dependency
  inventory; UTF-8/LF/no dashes; `cargo xtask ci` the gate. PLANNED.
- **P-9 The Instrument Does Not Lie**: Central. The scaffold's heuristic warning
  becomes structured `fidelity` + `notes` a machine can act on, rather than a
  comment a parser strips. PASS.

No gate violations. The one watch item (P-5) is a verification, not a risk: if a
writer embeds profile text, SC-006 forces re-authoring the affected golden, which
is in scope.

## Project Structure

### Documentation (this feature)

```text
specs/026-profile-json-migration/
├── plan.md, research.md, data-model.md, quickstart.md
├── contracts/
│   └── profile-load.md      # Profile::parse contract and the two-layer report
└── checklists/requirements.md
```

### Source Code (repository root)

```text
crates/fragcap-profile/
├── Cargo.toml                    # - toml-span
├── src/
│   ├── parse.rs                  # REWRITE: serde_json::Value tree walk; structural via jsonschema::validate_json (mapped); lenient Draft extraction; glob/regex/duration compile
│   ├── validate.rs               # semantic checks retained; locations become JSON pointers, spans dropped
│   ├── diagnostic.rs             # Syntax repurposed to "not valid JSON"; a located(pointer) constructor; positions optional (unused on this path)
│   ├── resolve.rs                # <ref>.json resolution
│   └── (schema.rs unchanged: the typed Profile/Game/Stage/GameId/Lifecycle/etc.)
└── tests/
    ├── examples.rs, diagnostics.rs, resolution.rs   # TOML literals -> JSON
    └── (schema_*.rs unchanged: they are S025's)

crates/fragcap-steam/src/scaffold.rs   # render() emits JSON with kind/fidelity/notes; drop the TOML header comment path
crates/fragcap-cli/
├── tests/data/game.toml -> game.json
├── src/{assemble.rs, commands/tap.rs}  # inline TOML profile literals -> JSON
└── tests/{cli_profile.rs, cli_extcap.rs}  # inline literals + .toml refs -> JSON
crates/fragcap/tests/session.rs         # inline TOML literal -> JSON
crates/fragcap-steam/src/launch.rs      # inline literal -> JSON (test)

docs/fragcap-specification.md           # section 15 reconciled to JSON
docs/glossary/platform-and-distribution.md  # Game profile entry -> JSON
changelog.d/026-profile-json-migration.md + .decisions.md
AGENTS.md                               # dependency inventory: remove toml-span
```

**Structure decision**: The parser is rewritten in place in `fragcap-profile`;
the typed `schema.rs` model is unchanged, so downstream consumers (matching,
session, capture) see the same `Profile`. The two-layer report reuses the S025
validator for structure, keeping one structural implementation bound to the
published schema.

## Complexity Tracking

No constitution gate is violated. The one design decision with lasting
consequence (dropping byte-offset positions in favor of JSON pointers, because
serde_json exposes no per-value spans) is recorded in research.md and surfaced at
the pre-push halt; it is a precision tradeoff, not a lost check.
