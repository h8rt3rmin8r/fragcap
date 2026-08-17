# Implementation Plan: The target entry model (handles, stable ids, selector resolution, cascade collapse)

**Branch**: `051-target-entry-model` | **Date**: 2026-08-16 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/051-target-entry-model/spec.md`

## Summary

A capture target stops being a loose JSON profile file and becomes a row in
`local.db`. This slice adds the target entry model (its fields, a `classification`
enum with `unknown` as a first-class value, and a `fidelity` column constrained
at the storage layer), the deterministic handle-normalization algorithm and its
uniqueness and fallback rules, the anchored/unanchored stable identifier, and the
selector resolution that refuses to guess. It makes resolution over the two
stores fidelity-ordered while preserving the four hint-database declines as
fidelity-aware query conditions. Retiring the profile-file surface (`--profile`,
the AppData profile directory, the `profile` command) is deferred to S054's
capture rework, since that surface is the only capture entry point; `schema
validate` is untouched (see spec Clarifications, deferrals session).

Operator and code-review decisions shape the approach: the identifier is the low
63 bits of BLAKE3 over the canonical anchor (add the `blake3` crate; the anchor
prefix is canonicalized and no locality bit is reserved), and the engine-layout
and platform-walker providers remain in the resolver this slice, becoming sources
in S052, so the literal reduction to three provider positions completes there.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82 (built/released on the pinned
toolchain; see `rust-toolchain.toml`).

**Primary Dependencies**: new to this slice, all landing only in `fragcap-targets`
behind the existing default-off `targets` feature: `blake3`
(`default-features = false`) for the anchor identifier; `unicode-normalization`
for NFKD; one Unicode general-category crate (candidate `unicode-properties`) for
the `So`/`Sk`/`Cf`/`Mn` category tests. Existing: `rusqlite` (bundled SQLite),
`serde_json`, `regex` (already in the graph; usable for the handle run-collapse,
though a plain char scan also suffices).

**Storage**: SQLite via `rusqlite`. A new `targets` table (and a
`target_id_aliases` table for superseded identifiers) added by an additive
migration bumping the shared schema from version 2 to version 3. The `targets`
data is conceptually `local.db` only; the shared store type carries the table on
both files (catalog leaves it empty), consistent with the S050 decision that
later slices add their own tables to `local.db`.

**Testing**: `cargo test --workspace --locked`; the Appendix A handle vectors are
unit tests in `fragcap-targets`; resolution and selector behavior are integration
tests. In this environment, SQLite-backed crates build under the GNU-host
toolchain (`cargo +1.96.0-x86_64-pc-windows-gnu test --workspace`); CI runs the
real MSVC build.

**Target Platform**: Windows (capture host). The entry model itself is pure
computation, but it lands in `fragcap-targets`, not `fragcap-core` (see Structure
Decision), so `fragcap-core` gains no dependency (P-2).

**Project Type**: Single Rust workspace (CLI + libraries).

**Performance Goals**: Not a hot path. Handle normalization runs at registration;
resolution runs once per capture start. No per-packet cost is introduced.

**Constraints**: Add no dependency to `fragcap-core`; every new crate resolves to
an allowlisted license (MIT / Apache-2.0 / BSD / ISC / Unicode-DFS-2016 /
Unicode-3.0 / Zlib); the exported identifier is a durable contract and must be
stable once shipped; the four hint declines must remain declines.

**Scale/Scope**: A user's `local.db` holds at most a few hundred targets;
`catalog.db` holds the shipped seed. Linear scans over the target table are
acceptable at this scale, but handle and identifier columns are `UNIQUE`/indexed.

## Constitution Check

*GATE: evaluated before Phase 0 and re-checked after Phase 1 design.*

- **P-1 Passive Observation Only (NON-NEGOTIABLE)**: PASS. The entry model reads
  and writes an embedded database and computes hashes over strings. No process
  handle, no memory read, no interception, no launch. `cargo xtask lint` still
  asserts the absence of `OpenProcess`/`ReadProcessMemory`/`WriteProcessMemory`.
- **P-2 Core Stays Platform-Neutral**: PASS. All new code and dependencies land
  in `fragcap-targets`, never `fragcap-core`. The `fragcap-core` dependency
  allowlist (`xtask/src/deps.rs`) is unchanged.
- **P-3 Capture And Attribution Stay Separate**: PASS. The entry model is
  attribution-side configuration; no capture crate gains a dependency on it, and
  the facade remains the only crate depending on both sides.
- **P-4 No Silent Loss**: PASS. A resolution that declines (including the four
  preserved hint declines) records why via the existing `ResolutionNotes`; an
  ambiguous selector lists its matches and exits non-zero rather than dropping
  one silently.
- **P-5 Compatibility Outranks Richness**: PASS. Output format (pcapng / JSON
  Lines) is untouched; this slice changes how a target is stored and selected,
  not what is written to a capture.
- **P-6 Glossary First**: ACTION. New terms (target entry, handle, anchor, stable
  identifier, fidelity ordering, superseded alias) get glossary entries in the
  same change (`docs/glossary/`). Tracked as a task.
- **P-7 Wrappers Stay Thin**: PASS. No wrapper parses output; selector resolution
  is a Rust capability, not shell glue.
- **P-8 House Standards Apply**: PASS. UTF-8 no BOM, LF, no em/en dashes;
  `cargo xtask ci` is the gate.
- **P-9 The Instrument Does Not Lie (NON-NEGOTIABLE)**: PASS, and central here.
  `classification` includes `unknown` so nothing is forced to guess; `fidelity`
  is a CHECK-constrained column, not a convention; the four declines are
  preserved as query conditions; an ambiguous name resolves nothing.
- **P-10 One Path To A Target**: PASS, and the point of the slice. Every entry,
  however produced, is one row in one table with one resolution path. The
  identifier derives only from the anchor so independent registrations merge.
  Full realization (walker/engine as sources) is S052; this slice builds the
  single store and form they will write into.
- **P-11 The Specification Describes What Shipped**: ACTION. The master spec
  sections this touches (5, 6, 15) are updated to describe the entry model and
  the fidelity-ordered store cascade as shipped, with the provider-reduction
  explicitly noted as completing in S052. `cargo xtask spec` binds the
  `Applies-To` field and the `spec-impact` fragment header; both are honored.

No violations require justification, so Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/051-target-entry-model/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── cli-surface.md          # selectors, retired/kept commands
│   ├── identifier-and-export.md # anchor canonicalization, 63-bit id, export/import merge
│   └── resolution-cascade.md   # fidelity ordering + the four preserved declines
├── checklists/
│   └── requirements.md  # spec quality checklist (from /speckit-specify)
└── tasks.md             # Phase 2 output (/speckit-tasks, not this command)
```

### Source Code (repository root)

```text
crates/fragcap-targets/
├── Cargo.toml                 # + blake3, unicode-normalization, unicode-properties (behind `targets`)
├── src/
│   ├── schema.rs              # SCHEMA_VERSION 2 -> 3; DDL gains `targets` + `target_id_aliases`; MIGRATE_2_TO_3
│   ├── store.rs               # migration arm for v2 -> v3; target CRUD, selector queries, alias handling
│   ├── entry.rs               # NEW: TargetEntry model, classification/classification_source/fidelity enums
│   ├── handle.rs              # NEW: normalization algorithm, fallback chain, collision auto-increment
│   ├── identifier.rs          # NEW: anchor canonicalization + 63-bit BLAKE3 + random-with-locality-bit
│   ├── selector.rs            # NEW: bare-int / handle / name / --id resolution, ambiguity -> exit 2
│   ├── hint_provider.rs       # declines become fidelity-aware query conditions; fidelity-ordered read
│   └── lib.rs                 # re-exports
└── tests/
    ├── handle_vectors.rs      # NEW: every Appendix A vector
    ├── identifier.rs          # NEW: same-anchor equality, different-anchor inequality, supersede/alias
    ├── selector.rs            # NEW: ambiguity, handle, name, --id, bare-int
    └── hint_cascade.rs        # extend: fidelity ordering + four declines preserved

crates/fragcap-profile/
├── src/                       # resolver keeps EngineRule + PlatformWalker (S052 removes them);
│                              # Profile file provider + file search retired; FidelityTier reused
└── src/jsonschema, schema.rs  # published master schema extended with the entry fields so an
                               # exported entry validates (handle/stable_id optional on input)

crates/fragcap-cli/
├── src/
│   ├── cli.rs                 # remove --profile selector + `profile validate`; add target selectors (--id, handle/name)
│   ├── paths.rs               # remove user_profile_dir / search_path / profile-dir env; keep catalog/local db paths
│   └── commands/
│       ├── profile.rs         # retired (profiles are no longer files); schema.rs keeps `schema validate`
│       └── targets.rs         # target selection + listing surfaces the entry model
└── tests/                     # update goldens/help for the retired + new surface

docs/
├── fragcap-specification.md   # sections 5, 6, 15 updated (P-11)
└── glossary/                  # new-term entries (P-6)
```

**Structure Decision**: The entry model is pure, platform-neutral computation, so
`fragcap-core` would satisfy P-2. It nonetheless lands in `fragcap-targets`
because the model is inseparable from the store that persists it and the resolver
read that consumes it, both of which already live in `fragcap-targets`; putting
the logic there keeps the new dependencies (`blake3`, the Unicode crates) behind
the existing default-off `targets` feature and out of `fragcap-core`'s allowlist,
exactly as `rusqlite` is. This mirrors the established placement rule from S10
(the socket-table read lives in the crate that owns the store).

## Complexity Tracking

No constitution violations require justification. (The dependency additions are
justified in `research.md` against the established `AGENTS.md` dependency-table
pattern, not as complexity deviations.)
