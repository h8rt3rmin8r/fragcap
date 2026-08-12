# Tasks: Profile Format Migration from TOML to JSON

**Feature**: 026-profile-json-migration | **Branch**: `feat/profile-json-migration`
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

Test-driven. The load-parity tests (same diagnostics on equivalent invalid
inputs, all-errors-at-once) are the mandatory correctness tests and are never
weakened. No P-1 surface is touched.

## Phase 1: Setup

- [ ] T001 Remove `toml-span` from `crates/fragcap-profile/Cargo.toml` and update the AGENTS.md dependency inventory to record the removal (the format it parsed no longer exists; serde_json, already runtime from S025, replaces it), confirming `cargo tree` shows no `toml_span`/`toml-span` in the graph.

## Phase 2: Foundational (blocking prerequisites)

- [ ] T002 In `crates/fragcap-profile/src/diagnostic.rs`, repurpose the `Syntax` variant's doc from "not valid TOML" to "not valid JSON", and add a location-only constructor (for example `Diagnostic::located(code, pointer, message)`) that sets `offset`/`position` to `None`, for the JSON-pointer-located diagnostics the profile-load path now produces. Keep every existing code and the existing constructors.

## Phase 3: User Story 1 - Author and load a game profile as JSON (Priority: P1) [MVP]

**Goal**: `Profile::parse` reads JSON, validates via both layers, and reports
every problem in one pass; it remains the only constructor.

**Independent test**: load a valid JSON profile and confirm the same in-memory
Profile the TOML produced; feed a mixed-fault profile and see every diagnostic in
one pass.

### Tests (write/port first)

- [ ] T003 [P] [US1] Port `crates/fragcap-profile/tests/examples.rs` (the ESO launcher+client and Division 2 platform/anticheat/client profiles) from inline TOML to inline JSON with `kind: "profile"` and `fidelity: "verified"`; assert each loads and yields the expected game, capture defaults, and stages.
- [ ] T004 [P] [US1] Port `crates/fragcap-profile/tests/diagnostics.rs` fault cases from TOML to JSON; assert the same `DiagnosticCode` set on equivalent invalid inputs (structural and semantic), and that a profile with N mixed faults reports exactly N in one pass (SC-001, SC-003). Parity coverage MUST include the fragcap-only checks a schema cannot express: empty `capture.roles`, an undeclared capture role, and a `path_regex`/`exe`/`capture.duration` that fails to compile, so no such check is silently lost in the migration.
- [ ] T005 [P] [US1] Add a test that former TOML content (for example `schema = 1\n[game]\n...`) is refused as invalid JSON (`Syntax`), not half-parsed (SC-005), and that a mixed structural+semantic JSON profile reports both layers together.

### Implementation

- [ ] T006 [US1] In `crates/fragcap-profile/src/parse.rs`, replace `toml_span::parse` with `serde_json::from_str::<serde_json::Value>`; on a JSON syntax error emit one `Syntax` diagnostic at the document root and stop.
- [ ] T007 [US1] In `parse.rs`, run `jsonschema::validate_json(&value)` for the structural layer and map each `SchemaDiagnostic` into a `Diagnostic` (`SchemaCode` -> `DiagnosticCode`, JSON pointer -> `location`); preserve the unsupported-`schema`-version suppression (one diagnostic, stop).
- [ ] T008 [US1] In `parse.rs`, rewrite `draft()`/`read_game`/`read_capture`/`read_stage`/`read_predicates` to extract the lenient `Draft` from `serde_json::Value` (remove the toml_span byte-span fields; locate by JSON pointer), and compile the `exe` glob, `path_regex`, and `capture.duration` in this fragcap pass (the checks the schema cannot express), accumulating `InvalidGlob`/`InvalidRegex`/`InvalidDuration`.
- [ ] T009 [US1] In `crates/fragcap-profile/src/validate.rs`, change diagnostic locations to JSON pointers (rewrite `DraftStage::loc` to emit `/stage/{index}/...`) and use the location-only constructor; the semantic checks themselves are unchanged.
- [ ] T010 [US1] Confirm `Profile::parse` composes the JSON parse, the structural map, and the lenient fragcap pass into one accumulated `Diagnostics`, sorted deterministically, returning `Err` if non-empty and a validated `Profile` otherwise; it stays the only constructor. Verify US1 tests pass.

**Checkpoint**: profiles load from JSON with full, single-pass validation.

## Phase 4: User Story 2 - Resolve and validate profiles from the command line as JSON (Priority: P2)

**Goal**: resolution finds and validates `.json` profiles by path, directory,
user directory, and bundled game id.

### Tests (port first)

- [ ] T011 [P] [US2] Port `crates/fragcap-profile/tests/resolution.rs` to `.json` filenames and JSON `profile_text`; assert the resolution order, the reported source, and that a reference resolving to nothing is a distinct expected failure.
- [ ] T012 [P] [US2] Port `crates/fragcap-cli/tests/cli_profile.rs` (valid/invalid literals) to JSON and update its data references.

### Implementation

- [ ] T013 [US2] In `crates/fragcap-profile/src/resolve.rs`, change `<ref>.toml` to `<ref>.json` (the `format!` at the resolution site) and update the module doc comments describing resolution steps 2 and 3.
- [ ] T014 [US2] Rename `crates/fragcap-cli/tests/data/game.toml` to `game.json` with JSON content (`kind: "profile"`, `fidelity: "verified"`) and update every reference to it in the CLI tests.

**Checkpoint**: operators reference and validate JSON profiles exactly as before.

## Phase 5: User Story 3 - Scaffold a JSON profile with a machine-readable heuristic warning (Priority: P3)

**Goal**: the Steam scaffold emits a schema-valid JSON profile stamped
heuristic-unverified with the verification warning as a `notes` string.

### Tests (port/add first)

- [ ] T015 [P] [US3] Update the scaffold tests in `crates/fragcap-steam/src/scaffold.rs` so they assert the rendered output is JSON that (a) validates against the master schema's profile variant, (b) carries `fidelity: "heuristic-unverified"` and a `notes` string containing the verification warning, and (c) loads via `Profile::parse`.

### Implementation

- [ ] T016 [US3] Rewrite `render()` in `crates/fragcap-steam/src/scaffold.rs` to emit JSON (`schema`, `kind: "profile"`, `fidelity: "heuristic-unverified"`, `notes` warning, `game`, `stage` array with `match` objects), removing the TOML header-comment path and `toml_escape`; ensure the output validates and re-parses (validity by construction, as the TOML renderer did).

**Checkpoint**: the on-ramp emits honest, loadable JSON.

## Phase 6: Polish & Cross-Cutting

- [ ] T017 [P] Migrate every remaining inline TOML profile literal to JSON: `crates/fragcap/tests/session.rs`, `crates/fragcap-cli/src/assemble.rs`, `crates/fragcap-cli/src/commands/tap.rs`, `crates/fragcap-cli/tests/cli_extcap.rs`, `crates/fragcap-profile/src/matching.rs` (test module), `crates/fragcap-profile/src/diagnostic.rs` (any test literal), and `crates/fragcap-steam/src/launch.rs` (test literal).
- [ ] T018 Reconcile `docs/fragcap-specification.md` section 15 (the schema example and prose from TOML to JSON, noting the kind/fidelity keys and the structural/semantic seam) and the Game profile entry in `docs/glossary/platform-and-distribution.md` (TOML -> JSON), regenerating the glossary index if needed.
- [ ] T019 Add `changelog.d/026-profile-json-migration.md` (feature) and `changelog.d/026-profile-json-migration.decisions.md` (dated: reverses the S05 toml-span choice; records the JSON-pointer/position tradeoff and the reuse of the S025 validator for structure).
- [ ] T020 Text-hygiene sweep (UTF-8 no BOM, LF, no em/en dashes across all changed files) and run the full gate in the foreground to green: `cargo xtask ci` and `cargo xtask msrv` at 1.82. The gate's corpus-pipeline and session tests are the downstream-parity assertion (FR-012, SC-006): an equivalent profile drives byte-identical capture output, proving the change was the input format and not the behavior; confirm they reproduce the committed goldens unchanged.

## Dependencies & Execution Order

- Setup (T001) -> Foundational (T002) -> US1 (T003-T010) -> US2 (T011-T014) -> US3 (T015-T016) -> Polish (T017-T020).
- US1 is the MVP and gates everything (the JSON load path). US2 and US3 build on it and are each independently testable.
- Within a story, `[P]` test tasks touch distinct files and may be ported in parallel; implementation tasks sharing `parse.rs` (T006-T008, T010) are sequential.

## Parallel Execution Examples

- US1 test ports T003/T004/T005 touch distinct test files and can be done together before the parse rewrite.
- Polish T017 migrations touch distinct files and are parallelizable.

## Implementation Strategy

MVP = Phase 1 + 2 + 3 (US1): JSON profiles load with full single-pass validation.
US2 makes resolution and the CLI JSON-native; US3 migrates the scaffold; Polish
sweeps the remaining literals, the docs, the changelog, and the gate.
