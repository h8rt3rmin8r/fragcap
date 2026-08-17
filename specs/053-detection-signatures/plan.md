# Implementation Plan: Data-driven detection signatures

**Branch**: `053-detection-signatures` | **Date**: 2026-08-17 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from
`/specs/053-detection-signatures/spec.md`

## Summary

Move technology detection (engine, anti-cheat, DRM) from an embedded ruleset
compiled into `fragcap-profile` to a `signature` table in the shipped catalog
database, matched by one generic matcher. That matcher becomes the real
implementation of the S052 `DirectoryClassifier` seam, so detection runs
automatically in the scan phase of every discovery source, and it backs the
retained standalone `technologies` command. Locally detected engines are stamped
`verified` and outrank remote catalog attributions, which stay
`heuristic-unverified` (P-9). The existing vendored SteamDB ruleset, which is
depot-authored, path-only, and never validated against a real install, is
removed; the Appendix B install-layout set seeds the table.

The crate placement is forced by the dependency DAG and is the plan's central
decision: the pure matcher and the `Signature` value type live in
`fragcap-profile` (the one crate both `fragcap-steam` and `fragcap-targets`
already depend on), the signature store, seed, and classifier seam live in
`fragcap-targets`, and `fragcap-steam`'s scaffold receives an injected signature
set rather than reaching for a sibling crate. No new inter-crate edge is added.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82 (built and released on the pinned
toolchain; see `rust-toolchain.toml`).

**Primary Dependencies**: no new crate. The signature store reuses `rusqlite`
(bundled SQLite, already behind the `targets` feature). The seed document is
parsed with `serde_json` (already a runtime dependency of `fragcap-targets`). The
generic matcher is arithmetic over a directory listing and a byte slice: filename
and directory-shape matching reuse `regex` (already in `fragcap-profile` for
`path_regex`), and PE-version-string matching is a hand-rolled parse of the PE
version resource, consistent with the hand-rolled pcap, pcapng, and JSON Lines
parsers (no `goblin`/`object` dependency added).

**Storage**: SQLite via `rusqlite`. One new `signature` table added by an additive
migration bumping the shared schema from version 4 (S052) to version 5. It lives
conceptually in `catalog.db` (the shipped, refreshable catalog), the opposite side
of the S052 `volume_eligibility` table which is `local.db`; the shared store type
carries the table on both files and `local.db` leaves it empty, the same two-store
discipline S050 and S052 used.

**Testing**: `cargo test --workspace --locked`. Detection is exercised entirely
from fixture directories: filename and directory-shape matching from temp trees of
empty files (as the existing detector's tests already do), and PE-version-string
matching from a minimal PE image built by a test helper (fixtures are generated,
not hand-made). The classifier seam is tested through the S052 `FixtureSource`
plumbing. SQLite-backed crates build under the GNU host toolchain here
(`cargo +1.96.0-x86_64-pc-windows-gnu test --workspace`); CI runs the real MSVC
build.

**Target Platform**: Windows (capture host). The matcher and `Signature` type in
`fragcap-profile` are portable byte-and-listing computation (no Windows API: the
PE parse reads the file's own bytes, it does not call the OS). `fragcap-core`
gains nothing (P-2); `fragcap-targets` stays free of the Windows API.

**Project Type**: Single Rust workspace (CLI + libraries).

**Performance Goals**: The classifier runs at discovery/authoring time, never
during capture, so no per-packet cost. The stop-on-hit descent contract (S052
FR-015) bounds the known-roots walk: a directory that matches a signature emits one
candidate and is not descended into. PE-version-string matching opens a file only
for a signature of that kind against a candidate binary, not for every file in a
tree.

**Constraints**: Add no dependency to `fragcap-core` and no new inter-crate edge
(the deps gate fails closed on any unexpected edge). `fragcap-profile` gains no
database dependency (it matches over a caller-provided signature set).
`fragcap-steam` gains no dependency on `fragcap-targets` (its scaffold receives an
injected signature set). Every considered directory lands in exactly one S052
discovery-account outcome (P-4); a loaded signature of an inert kind and a
malformed row are surfaced, never silently dropped. No output path frames a
detected anti-cheat or DRM as a reason not to capture (FR-011).

**Scale/Scope**: The Appendix B seed is 16 products across a few dozen signature
rows. A signature scan reads one directory listing per directory and, for a
PE-version-string signature, one binary's version resource. Linear over the walk
S052 already bounds.

## Constitution Check

*GATE: evaluated before Phase 0 and re-checked after Phase 1 design.*

- **P-1 Passive Observation Only (NON-NEGOTIABLE)**: PASS. Detection reads
  directory listings and, for a PE-version-string signature, the version resource
  in a binary's on-disk PE header. It opens no process handle, reads no process
  memory, intercepts nothing, hooks nothing, and launches nothing. Reading a file
  the operator already has on disk is not reaching into a process; the module's
  prior "reads no file content" note is relaxed to "reads no process memory and no
  file content beyond an on-disk PE version resource," which P-1 permits.
  `cargo xtask lint` still asserts the absence of
  `OpenProcess`/`ReadProcessMemory`/`WriteProcessMemory`.
- **P-2 Core Stays Platform-Neutral**: PASS. Nothing lands in `fragcap-core`. The
  matcher and `Signature` type land in `fragcap-profile` (which keeps its existing
  `fragcap-core`-only edge and adds no platform dependency); the store, seed, and
  classifier land in `fragcap-targets`.
- **P-3 Capture And Attribution Stay Separate**: PASS. Detection is neither capture
  nor attribution; it touches neither `fragcap-capture` nor `fragcap-attr`.
  `cargo xtask deps` sees no new inter-crate edge: the matcher lives in the shared
  `fragcap-profile`, and `fragcap-steam`'s scaffold takes an injected signature set
  rather than a new edge to `fragcap-targets`.
- **P-4 No Silent Loss**: PASS by design. A signature row of an inert kind
  (binary-marker this slice) is loaded, counted, and surfaced as not-yet-matchable;
  a malformed or empty pattern is rejected at load with a surfaced diagnostic and
  does not disable the rest of the table; an unreadable subtree is surfaced. The
  S052 discovery account stays conserved across every source test.
- **P-5 Compatibility Outranks Richness**: PASS. Detection annotates candidates and
  a profile scaffold; it changes no output format an unmodified analyzer reads.
- **P-6 Glossary First**: ACTION (a task). New terms enter the glossary in the same
  change: detection signature, signature table, signature kind (filename /
  directory-shape / PE-version-string / binary-marker), signature category (engine
  / anti-cheat / drm), the generic signature matcher, and neutral evidence.
- **P-7 Wrappers Stay Thin**: PASS. The `technologies` CLI stays a thin call into
  the matcher; no wrapper parses output.
- **P-8 House Standards Apply**: ACTION. UTF-8 without BOM, LF, no em or en dashes,
  including in the removed-and-replaced module and the seed document.
- **P-9 The Instrument Does Not Lie (NON-NEGOTIABLE)**: PASS, and it is the point.
  A locally detected engine is `verified`; a remote catalog engine attribution is
  `heuristic-unverified`; where both exist, the local value is presented. A
  detected product is neutral evidence, never a gate (FR-011). A directory with no
  signature match is surfaced as no-match, never guessed.
- **P-10 One Path To A Target**: PASS. The matcher fills the single S052 classifier
  seam, so every source classifies through one path.
- **P-11 The Specification Describes What Shipped**: ACTION (a task). The master
  specification sections that describe detection (3.6, 8, Appendix B) are reconciled
  with what ships (data-driven table, three implemented kinds, the removed embedded
  ruleset), and the changelog fragment carries the `<!-- spec-impact: N -->` header
  the S049 gate requires.

No violations require Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/053-detection-signatures/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── signature-table.md
│   ├── signature-matcher.md
│   └── directory-classifier.md
├── checklists/
│   └── requirements.md  # from /speckit-specify
└── tasks.md             # /speckit-tasks output (not this command)
```

### Source Code (repository root)

```text
crates/fragcap-profile/
├── src/
│   ├── technologies.rs      # REWRITTEN: embedded ruleset -> data-driven
│   │                        #   matcher over a caller-provided &[Signature];
│   │                        #   Signature, SignatureKind, SignatureCategory,
│   │                        #   PE-version-string reader; fidelity from
│   │                        #   confidence. CompiledRuleset/RULES_INI removed.
│   ├── sha256.rs            # REMOVE if orphaned once the ruleset lock test goes
│   └── lib.rs               # re-exports updated (drop RULES_INI/RULES_LOCK/
│                            #   CompiledRuleset; export Signature + matcher)
└── assets/steamdb/          # REMOVED: rules.ini, rules.lock.json,
                             #   THIRD_PARTY_NOTICES.md

crates/fragcap-targets/
├── src/
│   ├── schema.rs            # SCHEMA_VERSION 4 -> 5; signature DDL; MIGRATE_4_TO_5
│   ├── store.rs             # load_signatures(); apply MIGRATE_4_TO_5
│   ├── signatures.rs        # NEW: seed_signatures() from the bundled asset;
│   │                        #   Store <-> Signature row mapping
│   ├── classifier.rs        # SignatureClassifier: real DirectoryClassifier over
│   │                        #   loaded signatures, delegating to the matcher
│   └── assets/signatures.json  # NEW: the Appendix B seed document
└── tests/                   # signature seed, classifier stop-on-hit, neutral out

crates/fragcap-steam/
└── src/scaffold.rs          # scaffold takes an injected &[Signature] (or matcher)
                             #   instead of CompiledRuleset::embedded()

crates/fragcap/               # facade: load signatures from catalog.db and hand
└── src/                      #   them to the technologies command and the scaffold

crates/fragcap-cli/
└── src/commands/technologies.rs  # repointed at the table-backed matcher; a
                                  #   catalog.db path argument added
```

**Structure Decision**: Single Rust workspace. The pure matcher and `Signature`
type live in `fragcap-profile` because it is the only crate both `fragcap-steam`
and `fragcap-targets` depend on, so both consumers reach detection with no new
edge. The signature store, the Appendix B seed, and the `DirectoryClassifier`
implementation live in `fragcap-targets`, which owns the catalog database and the
S052 seam. The facade composes them (load signatures, hand them to the matcher and
the scaffold). This is the minimal-edge design; the alternatives (matcher in
`fragcap-targets` with a new `fragcap-steam -> fragcap-targets` edge, or the
matcher duplicated) are rejected in research.md.

## Complexity Tracking

No constitution violations; no entries required.
