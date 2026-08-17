# Phase 0 Research: The target entry model

All Technical Context unknowns are resolved below. Each decision follows the
`AGENTS.md` dependency-table discipline: state what is chosen, why, and what was
rejected, and record any verification the sandbox cannot perform (no MSVC linker
here) as an implementation task rather than an assumption.

## D-1. Anchor identifier hash: BLAKE3 (operator-directed)

**Decision**: Add the `blake3` crate with `default-features = false`, and compute
the anchored identifier as the low 63 bits of BLAKE3 over the canonical anchor
string. This is a settled operator decision (spec Clarifications, session 2).

**Rationale**: The handoff plan and issue #138 name BLAKE3 explicitly, and the
identifier is a durable exported contract (it lands in JSON exports and drives
import merge), so honoring the literal specification avoids a silent divergence
from any tool that computes the same anchor id. BLAKE3 is more complex to
hand-roll correctly than SHA-256 (chunk state, chaining values, domain-separation
flags), and a durable identity hash is exactly the case where a well-tested
reference implementation is worth a dependency, matching the reasoning that added
`pcap` and `rusqlite` rather than transcribing a spec by hand.

**Alternatives considered**:
- *Reuse the hand-rolled SHA-256 truncated to 63 bits* (the repo pattern that
  motivated `fragcap-profile/src/sha256.rs`): functionally identical determinism
  and collision resistance, zero new dependencies. Rejected by the operator
  because it deviates from the handoff's explicit id contract for no functional
  gain and forecloses BLAKE3-based interop.
- *Hand-roll BLAKE3*: no new dependency, but a from-scratch BLAKE3 is materially
  more error-prone than the SHA-256 already vendored, and a wrong identity hash
  is a P-9 defect that ships silently. Rejected.

**Dependency shape and verification (task)**:
- License: `blake3` is `CC0-1.0 OR Apache-2.0`; the OR resolves to the
  allowlisted `Apache-2.0`, so `cargo deny` accepts it. Transitive
  `constant_time_eq` must be checked: recent versions are
  `CC0-1.0 OR Apache-2.0 OR MIT` (accepted), but if the resolved version is
  CC0-only it needs a `[[licenses.exceptions]]` entry in `deny.toml` (a pinned
  artifact; the change is a dated `changelog.d/*.decisions.md` fragment, not an
  edit to `CHANGELOG.md`). `arrayref` is `BSD-2-Clause` (accepted), `arrayvec`
  and `cfg-if` are `MIT OR Apache-2.0` (accepted).
- `default-features = false` drops the `std`-gated rayon parallelism and any
  SIMD-forcing features that would enlarge the graph; portable Rust is correct
  for hashing sub-32-byte anchor strings.
- MSRV: `blake3` lands only in `fragcap-targets` behind the default-off `targets`
  feature, and `cargo xtask msrv` builds default features only, so BLAKE3 is
  never compiled under 1.82 (the same posture as `pcap` behind `live` and
  `http_req` behind `net`). It only needs to build under the pinned toolchain,
  which it does. Pin as a caret range and let `cargo xtask msrv` stand as the
  gate; verify the exact `Cargo.lock` delta and licenses on a linker-capable
  machine as a task.

## D-2. Handle normalization: `unicode-normalization` + a general-category crate

**Decision**: Implement the exact ordered algorithm (FR-004) over two Unicode
primitives: `unicode-normalization` for NFKD, and a general-category lookup
(candidate `unicode-properties`, `default-features` trimmed to the general-
category data) to test `So`/`Sk`/`Cf` (stripped before NFKD) and `Mn` (stripped
after NFKD). The run-collapse of characters outside `[a-z0-9]` to a single `_`,
the apostrophe/quote deletion, the lowercasing, the trim, and the 64-character
truncation-then-trim are plain Rust over `char`s; the already-present `regex`
crate is available for the run-collapse but a single linear scan is simpler and
is preferred.

**Rationale**: NFKD and Unicode general categories are large generated tables;
hand-rolling them is the absurd-transcription case the project rejects
dependencies to avoid (the same argument as `pcap`'s struct layouts). Both crates
are the standard, widely used, semi-official Rust Unicode crates. Every Appendix
A vector was traced by hand against this exact primitive set and passes:
- `Tom Clancy's(TM) ...` -> strip `So` (TM), delete apostrophe ->
  `tom_clancys_the_division_2`.
- `Pokemon` with acute -> NFKD splits the accent to a combining mark, strip `Mn`
  -> `pokemon`.
- `Final Fantasy` + Roman-numeral four (U+2163, category `Nl`, not stripped) ->
  NFKD compatibility-decomposes to `IV` -> lowercase -> `final_fantasy_iv`.
- Vulgar half (U+00BD, category `No`, deliberately not in the strip set) -> NFKD
  -> `1`, fraction slash, `2`; the fraction slash is outside `[a-z0-9]` ->
  `1_2_life`. (Stripping `No` would wrongly delete the digits; the strip set is
  exactly `So`/`Sk`/`Cf`/`Mn` for this reason.)
- Degree sign (U+00B0, `So`) stripped -> `rock_band_360`; `S.T.A.L.K.E.R.` ->
  `s_t_a_l_k_e_r`.

**Alternatives considered**:
- *A slug crate (`slug`, `deunicode`)*: encodes its own normalization rules that
  do not match the specified algorithm (e.g. transliteration, different fraction
  and roman-numeral handling), so the Appendix A vectors would not all pass.
  Rejected: the algorithm is specified exactly and must be implemented, not
  approximated.
- *ICU4X*: the 42-crate stack the project already rejected for schema validation
  (S025) and HTTP (S035). Rejected on graph size.
- *Strip combining marks by canonical combining class (ccc != 0) instead of
  category `Mn`*: close but not identical (a few `Mn` characters have ccc 0). The
  spec says category `Mn`; implement `Mn` to match it exactly, pinned by the
  `Pokemon` vector and an explicit ccc-0-Mn test.

**Verification (task)**: confirm the chosen category crate exposes `So`/`Sk`/`Cf`/
`Mn`, its license is MIT/Apache/Unicode (all allowlisted), its MSRV is <= the
pinned toolchain, and the `Cargo.lock` delta (both crates plus
`unicode-normalization`'s `tinyvec`/`tinyvec_macros`) on a linker-capable
machine. Like BLAKE3, both sit behind the default-off `targets` feature, so they
are outside the 1.82 MSRV gate's default-feature build.

## D-3. Store schema: additive migration to version 3

**Decision**: Bump `SCHEMA_VERSION` 2 -> 3 and add, in one transactional
`MIGRATE_2_TO_3`, a `targets` table carrying the FR-001 fields and a
`target_id_aliases` table holding superseded identifiers (FR-013). The migration
arm in `Store::open` mirrors the existing v1 -> v2 arm: apply the DDL and stamp
`user_version` in one transaction so a partial failure rolls back. Fresh stores
get the full v3 DDL.

**Rationale**: The v1 -> v2 migration (S050 era) already established the additive,
transactional pattern; reusing it keeps a v2 store (existing installs) upgrading
in place with existing rows intact. The `targets` table lives in the shared store
type so both files carry it; `catalog.db` simply never populates it, consistent
with the S050 decision that later slices add their own tables to `local.db`.

**Constraints enforced at the storage layer (P-9)**:
- `fidelity TEXT CHECK (fidelity IN ('authored','verified','heuristic-unverified','observed'))`.
- `classification TEXT CHECK (classification IN ('game','launcher','tool','mod','emulator','unknown'))`.
- `classification_source TEXT CHECK (... IN ('catalog','engine-signature','platform','user','unset'))`.
- `handle TEXT UNIQUE` and a CHECK that the handle is not purely numeric
  (`CHECK (handle GLOB '*[^0-9]*')`, i.e. it contains at least one non-digit),
  so a purely numeric handle is unstorable even if a caller bypasses the Rust
  path.
- `id INTEGER PRIMARY KEY`; the 63-bit stable identifier is a separate
  `stable_id INTEGER UNIQUE` column (63 bits fits SQLite's signed 64-bit integer
  with the sign bit clear).

**Alternatives considered**: a separate schema/store type for `local.db`.
Rejected: S050 deliberately kept one shared store type; diverging now would
reintroduce the two-shapes problem P-10 forbids.

## D-4. Fidelity-ordered resolution and the preserved declines

**Decision**: Make the store read (`hint_provider.rs`) fidelity-aware: a
`local.db` target row carries its own `fidelity`; a `catalog.db` row answers
`heuristic-unverified`; among competing rows the highest fidelity wins
(`authored` > `verified` > `heuristic-unverified` > `observed`). The four existing
declines (sparse catalog-only row, engine-only row with no launch executable,
launcher-mediated row, row with more than one distinct Windows executable) are
re-expressed as fidelity-aware query conditions on the read, preserving their
behavior. The engine-layout and platform-walker providers stay at their
precedence slots this slice (operator decision, session 2); the Profile *file*
provider and file search are retired because profiles-as-files are retired.

**Rationale**: The declines are load-bearing P-9 behavior (`hint_provider.rs`
documents each); the permutation and mediation tests already guard them and must
keep passing. Keeping engine/walker avoids a transitional window where installed-
but-unregistered titles stop resolving before S052 reintroduces them as sources.
Retiring only the Profile *file* provider (not the whole crate) is safe because
selector resolution (FR-015) replaces `--profile` reference resolution.

**Alternatives considered**: removing engine/walker now (the literal three-
position collapse). Rejected by the operator for the transitional-gap reason;
deferred to S052 where those providers become `TargetSource`s.

## D-5. Selector resolution and the retirement surface

**Decision**: A selector resolves as: a bare integer is an ephemeral row index
over the current listing; a token matches an exact handle, then a case-
insensitive exact name; `--id <n>` selects by `stable_id`. A name matching more
than one row prints the matches with handles and identifiers and exits 2
(configuration error, matching the existing `profile validate` exit convention),
resolving nothing. Retire `--profile <path>`, the AppData profile directory
(`paths.rs` `user_profile_dir`/`search_path` and the profile-dir env override),
and the `profile validate` subcommand; keep `schema validate` under the separate
`schema` command (already independent).

**Rationale**: Exit 2 for ambiguity reuses the established
configuration-error code and keeps the instrument from guessing (P-9). The
retirement is issue scope item 6; `schema validate` is untouched because it
already lives in its own command reading a JSON document against the published
schema.

**Open sub-question resolved**: the `profile` command also had `list`/`show`
subcommands over profile files. With profiles-as-files retired, their subject is
gone; listing and showing targets is the job of the `targets` command surface
(hero command is S055, basic listing exists today). This slice retires the
`profile` command; any target listing it exposed moves to `targets`. Flag for the
analyze phase to confirm no test depends on `profile list`/`show` output beyond
what `targets` covers.

## D-6. Randomness source for the unanchored identifier

**Decision**: Generate the unanchored 63-bit value (with the locality bit set)
from OS entropy via `getrandom`, taken as a direct dependency of
`fragcap-targets` behind the `targets` feature.

**Rationale**: The unanchored id must be unique and uncoordinated across
independent registrations; OS entropy is the direct way to get that without a
counter or clock that could collide. `getrandom` is already present in
`Cargo.lock` (pulled transitively today), so adding it as a direct dependency
adds no new crate to the lock graph, only an explicit edge. It is MIT OR
Apache-2.0 (allowlisted).

**Alternatives considered**:
- *`fastrand`* (also already in the graph): a small non-cryptographic PRNG.
  Usable, but it needs seeding from entropy anyway to avoid cross-process
  correlation, so it does not remove the entropy dependency; `getrandom` is the
  simpler primitive for a one-shot 63-bit draw.
- *Derive from a timestamp + counter*: not "random" as specified, and
  `Date.now()`-style clock reads are exactly the kind of hidden nondeterminism
  that makes collisions possible under fast repeated registration. Rejected.
- *Hash a unique seed with the already-present BLAKE3*: still needs a random or
  unique seed, so it reduces to the same entropy question. Rejected as
  indirection.

**Note**: the anchored path uses BLAKE3 and no entropy (it must be
deterministic); only the unanchored path draws from `getrandom`.

## Summary of new dependencies (for the AGENTS.md table)

| Crate | Kind | Why | Feature gate |
| --- | --- | --- | --- |
| `blake3` | runtime, optional | 63-bit anchor identifier (durable id contract) | `targets` |
| `unicode-normalization` | runtime, optional | NFKD step of handle normalization | `targets` |
| `unicode-properties` (candidate) | runtime, optional | `So`/`Sk`/`Cf`/`Mn` category tests | `targets` |
| `getrandom` | runtime, optional | entropy for the unanchored 63-bit id | `targets` |

The first three add to `Cargo.lock`; `getrandom` is already in the graph, so it
adds only a direct edge. All sit behind the default-off `targets` feature, so a
default library build and the 1.82 MSRV gate compile none of them, and
`fragcap-core` gains nothing (P-2). Licenses resolve to the allowlist; the exact
`Cargo.lock` delta, license resolution, and MSRV-under-`targets` are verified on a
linker-capable machine as tasks before the gate is claimed green.
