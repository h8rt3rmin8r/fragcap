# Implementation Plan: Profile Schema, Parsing, and Validation

**Branch**: `feat/profile-schema-validation` | **Date**: 2026-08-09 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from
`/specs/008-profile-schema-validation/spec.md`

**Slice**: S05 (specification section 15)

## Summary

Fill in `fragcap-profile`, which has been a skeleton since S01. It gains the
section 15.2 schema, a parser that accumulates every diagnostic instead of
stopping at the first, the section 15.4 validation set plus three checks in the
same failure class, an exact decision procedure for the ambiguous image match,
and the section 15.3 resolution order. `fragcap-core` gains a duration grammar,
because three slices need the same one.

Four decisions shape everything else.

**Two runtime dependencies, and the choice of which was decided by evidence
rather than by reputation.** The obvious pick, `toml`, is not available at this
workspace's minimum toolchain: version 1.1 declares Rust 1.85, and pinning to
1.0 does not help because its own `toml_parser` dependency resolves to 1.1.3,
which declares 1.85 as well. `toml-span` declares 1.70, brings one transitive
crate rather than four, carries byte spans natively, and has no serde in its
graph. All of that was measured, not assumed; research R-1 records the commands.

**The parser cannot be a deserializer.** A derive-based deserializer returns the
first error and stops, which is exactly what FR-013 forbids. Field extraction is
therefore written by hand over the spanned document, and this is a consequence
of the requirement rather than a preference about style. It also removes the
question of whether to take serde at runtime: the answer is no, and nothing is
given up by it.

**A `Profile` cannot be constructed except by parsing.** The type's invariants
are the validation rules, so there is no path by which an unvalidated profile
reaches S12. Section 15.4's "validation runs implicitly before every capture"
then needs no discipline from any future caller.

**The ambiguity decision is exact.** Two `exe` patterns over `*`, `?`, and
literals either can match a common name or cannot, and a table walk over the
pair decides which. An approximation would have to choose between refusing legal
profiles and admitting the silent empty capture the check exists to prevent, and
neither is necessary.

## Technical Context

**Language/Version**: Rust, edition 2021. Toolchain 1.96.0; minimum 1.82,
verified for these dependencies by building under `rustup run 1.82`.

**Primary Dependencies**: Two added, both to `fragcap-profile` and neither to
`fragcap-core`.

| Crate | Version | License | Declared MSRV | Transitive |
| --- | --- | --- | --- | --- |
| `toml-span` | 0.7 | MIT OR Apache-2.0 | 1.70 | `smallvec` |
| `regex` | 1.13, default features off, `std` and `unicode` | MIT OR Apache-2.0 | 1.65 | `regex-automata`, `regex-syntax` |

Five crates enter the graph in total. Every license is inside the `deny.toml`
allowlist, and no version specification is a wildcard, which that file bans. The
workspace goes from one runtime dependency to three and keeps its single
dev-dependency (`serde_json`, test-only, S07).

**Storage**: Profile files on disk, read only. Nothing is written.

**Testing**: `cargo test --workspace --locked`. Unit tests in `fragcap-profile`
for the glob and ambiguity decisions and for each diagnostic code, unit tests in
`fragcap-core` for the duration grammar, and integration tests for parsing the
two section 15.2 examples and for the resolution order against directories the
test builds under `CARGO_TARGET_TMPDIR`.

**Target Platform**: Any target the standard library supports. Nothing here is
Windows-specific: a profile describes Windows processes but parsing a
description of one is arithmetic over text.

**Project Type**: Library crate within a Cargo workspace.

**Performance Goals**: None set. Validation runs once per capture against a file
of tens of lines. One cost is bounded rather than merely stated: the ambiguity
pass is quadratic in stage count with each decision allocating a table linear in
the product of two pattern lengths, and the file size limit does not bound that
product. An `exe` pattern is capped at 255 characters and a profile at 64
stages, which puts the worst case at about 1.3 times 10^8 cell visits and 64
kibibytes of peak table. See decision D-11.

**Constraints**: Every problem in one report. No declared value normalized. The
same regular expression engine validates and, in S12, evaluates. No platform
configuration location consulted. Diagnostics deterministically ordered.

**Scale/Scope**: One crate filled in, six modules; one new module in
`fragcap-core`. Seven glossary entries.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Applies how | Status |
| --- | --- | --- |
| P-1 Passive observation | Nothing here observes a process. A profile describes process topology; this slice reads the description. FR-055 states the prohibition so that a convenience check on whether an image name exists would be a visible violation. | Pass, not engaged |
| P-2 Core stays platform-neutral | Both dependencies land in `fragcap-profile`. The duration module added to `fragcap-core` is arithmetic over a string and adds nothing to that crate's allowlist, which `cargo xtask deps` asserts. Resolution takes its search path from the caller, so no platform configuration location is consulted here either. | Pass |
| P-3 Capture and attribution separate | Not engaged directly: this crate is neither. It gains no sibling edge, and `fragcap-profile -> fragcap-core` is already the expected graph. | Pass, not engaged |
| P-4 No silent loss | Engaged through configuration rather than packets. A profile that binds the wrong process loses every packet and counts nothing, which is the same defect class. The ambiguity check and the roles check exist for it. | Pass, and load-bearing |
| P-5 Compatibility outranks richness | Engaged as schema growth rather than output format. Strict key rejection is paired with the schema version that makes widening safe, and the duration grammar is deliberately narrow because widening keeps existing profiles valid. | Pass |
| P-6 Glossary first | Seven entries added: profile schema version, lifecycle class, terminal stage, match predicate, profile resolution order, ambiguous image match, duration literal. `Stage` and `Game profile` gain cross-references. | Pass |
| P-7 Wrappers stay thin | No wrapper touched. | Pass, not engaged |
| P-8 House standards | `CONVENTIONS.md` applies; `cargo xtask lint` enforces it. | Pass |
| P-9 The instrument does not lie | No declared value is normalized, case-folded, or trimmed, which matters because case-folding an `exe` pattern is the natural convenience. Parsing a duration literal into a span is reading configuration, not altering an observation, and the reasoning is recorded rather than assumed. | Pass |

Post-design re-check: unchanged. Two runtime dependencies are added, which is an
architectural event for this workspace but not a principle violation: P-2 binds
core and core is untouched, and the license policy admits both. The Complexity
Tracking table below is empty.

## Project Structure

### Documentation (this feature)

```text
specs/008-profile-schema-validation/
├── plan.md                   # This file
├── research.md               # Phase 0 output
├── data-model.md             # Phase 1 output
├── quickstart.md             # Phase 1 output
├── contracts/
│   └── profile-schema.md     # Phase 1 output
├── checklists/
│   ├── requirements.md
│   └── validation.md
├── spec.md
└── tasks.md                  # /speckit-tasks output
```

### Source Code (repository root)

```text
crates/
├── fragcap-core/
│   └── src/
│       ├── lib.rs                 # module declaration and re-export
│       └── duration.rs            # new: the duration grammar
└── fragcap-profile/
    ├── Cargo.toml                 # gains toml-span and regex
    ├── src/
    │   ├── lib.rs                 # module declarations, re-exports, crate docs
    │   ├── diagnostic.rs          # code enumeration, diagnostic, ordered set,
    │   │                          # byte offset to line and column
    │   ├── schema.rs              # Profile, Game, CaptureDefaults, Stage,
    │   │                          # Lifecycle, CaptureMode, MatchPredicates
    │   ├── parse.rs               # spanned document to draft, structural checks
    │   ├── validate.rs            # semantic checks over the draft
    │   ├── glob.rs                # pattern, matcher, intersection decision
    │   └── resolve.rs             # section 15.3 order over caller-supplied paths
    └── tests/
        ├── examples.rs            # the two section 15.2 profiles
        ├── diagnostics.rs         # every code, and multi-fault accumulation
        └── resolution.rs          # the four steps and both shadowing cases

docs/
└── glossary.md                    # seven new entries, two cross-references

changelog.d/
├── S05-profile-schema-validation.added.md
└── S05-profile-schema-validation.decisions.md
```

**Structure Decision**: Six modules rather than one, split along the lines the
tests want to attack separately. `glob.rs` carries the intersection decision,
which is the most intricate code in the slice and is testable against pattern
pairs with no profile involved. `diagnostic.rs` is separate because the ordering
and location invariants are properties of the set rather than of any check.
`parse.rs` and `validate.rs` are split on the structural and semantic boundary
that section 15.4 itself draws, and the draft type that passes between them is
what makes accumulation possible.

`resolve.rs` is separate because it is the only module that touches a
filesystem, which keeps the rest of the crate pure and makes the boundary
visible to a reviewer looking for it.

## Design decisions

**D-1. `toml-span` rather than `toml`.** Research R-1 has the measurements. The
short form: `toml` 1.1 declares Rust 1.85 against a workspace minimum of 1.82,
and pinning it to 1.0 does not fix that because `toml_parser` 1.1.3 resolves in
underneath and declares 1.85 too. Holding the minimum would mean pinning a
transitive crate this slice never names, which is a fragile arrangement that a
single `cargo update` undoes silently. `toml-span` declares 1.70, brings
`smallvec` and nothing else, has no serde in its graph, and returns byte spans
for every value, which is what the diagnostics need. It builds under `rustup run
1.82`, verified rather than assumed.

The cost is a smaller ecosystem presence than `toml`. That is a real
consideration and it is outweighed here: the crate exists specifically to
support diagnostic-quality error reporting, which is this slice's deliverable,
and the alternative is not available at our toolchain floor.

**D-2. Extraction is hand-written, and that is forced.** A derive-based
deserializer stops at the first error. FR-013 requires every problem in one
report, so the extraction walks the spanned document itself and pushes a
diagnostic per fault instead of returning one. This is the same reasoning that
makes the parse-then-validate split necessary: collecting every structural fault
requires a draft in which any field may be absent, so the draft exists and is
private.

**D-3. `regex` with the performance features off.** Default features pull
`aho-corasick` and `memchr` for literal-scanning optimizations that matter when
scanning large haystacks. Here a haystack is one process image path matched a
handful of times per session, so those two crates buy nothing measurable.
Turning default features off and enabling `std` and `unicode` gives three crates
with the full documented syntax, including `\p{...}`, which was verified by
compiling `\p{Greek}+` against the trimmed build.

`regex-lite` was the other candidate: one crate, no dependencies. It was
rejected because its Unicode support is reduced, and an image path can contain
non-ASCII characters through a user or localized directory name. A profile
author writing a pattern expects it to behave the way the documented engine
behaves, and quietly matching under different Unicode rules is the kind of
divergence that produces a wrong binding rather than an error.

**D-4. One conversion site from byte offset to line and column.** `toml-span`
reports spans as byte offsets, so line and column are computed by fragcap. The
conversion lives in `diagnostic.rs` and nowhere else, and diagnostics carry both
the offset and the derived position. This satisfies FR-048 honestly: the parser
supplies a location, and the form an author can act on is derived from it in one
place rather than at every call site.

**D-5. The glob matcher and the intersection decision share one walk.** Both are
a dynamic-programming table over two sequences. Matching a name is the
intersection of a pattern with a literal, so the matcher is a special case of
the decision rather than a second implementation. `*` consumes any run including
empty, `?` consumes exactly one character, everything else compares
case-insensitively, and there is no escape because no Windows image name can
contain `*` or `?`. Comparison is over Unicode scalar values with simple case
folding, which is what `char::eq_ignore_ascii_case` does not give and
`to_lowercase` does.

**D-6. The duration grammar lives in `fragcap-core`.** Three consumers are
already visible: this slice, S14's `--duration` and `--wait`, and S16's ring
window. Core is the crate all three reach without a sibling edge, the grammar
adds no dependency there, and the alternative is two implementations of `30m`
that can disagree about a capture's length. Recorded for promotion to
specification section 29, since section 25.2 names duration parsing without
placing it.

**D-7. Resolution takes its search path as an argument.** The resolver receives
an ordered list of directories and a bundled set, and implements section 15.3's
order over them. It never asks where a user's configuration lives. This keeps a
`dirs`-style dependency out of the workspace, keeps the ordering testable
against directories a test creates, and leaves the platform question to S14,
which is the layer allowed to have an opinion. The slug check applies to steps
two through four only, because step one is an operator naming a file and the
later steps interpolate a name into a path.

**D-8. Diagnostics are ordered by byte offset, then by code.** The order has to
be stable because tests compare output and an operator compares two runs.
Offset-then-code is also the order an author reads their file in. `toml-span`
iterates its tables in key order rather than document order, so diagnostics are
collected and then sorted rather than emitted in traversal order; relying on
traversal would tie the output to a container choice inside a dependency.

**D-9. The size limit is checked before the read.** One mebibyte, from the
file's metadata, so an enormous file is refused without being loaded. Checking
after reading would make the limit a formality.

**D-10. Three checks beyond section 15.4, marked as such.** The roles check, the
terminal-lifecycle check, and the `descends_from` cycle check are additions.
Each is in the failure class section 15.4's two unusual checks were added for: a
run that succeeds and captures nothing. They are recorded in the changelog
decisions fragment as candidates for promotion into section 15.4 rather than
presented as readings of it, because a future reader comparing the code against
the specification should find the difference explained rather than have to
decide whether it is a defect.

**D-11. Two limits bound the ambiguity pass, reversing this slice's first
answer.** The slice originally stated the pass's complexity and declined to cap
it, on the reasoning that the one mebibyte file limit bounded both factors. Pull
request 11's review showed the reasoning was wrong rather than incomplete: the
file limit bounds each factor and not their product, so two half-megabyte `exe`
patterns ask the decision for about 10^12 table cells and abort the process
instead of producing a diagnostic. The total work is also invariant under how
the bytes are split between stage count and pattern length, so one cap alone
would have moved the cost rather than removed it.

`exe` patterns are capped at 255 characters, which is the Windows file name
component limit and therefore the length past which a pattern is longer than
anything it can match. Profiles are capped at 64 stages, which is two orders of
magnitude beyond the two and three the focal titles declare. Each limit is
refused with its own diagnostic naming it, and each has a test that accepts the
limit and refuses one past it.

The reversal is recorded in the changelog decisions fragment, and research R-2
keeps the original reasoning alongside why it failed, because a future reader
proposing to lift either limit should find the argument rather than reconstruct
it.

**D-12. The schema version gate runs before the top level key check.** A
correction found in the same review. Running the key check first meant a profile
declaring a later schema came back with both an `UnsupportedSchema` and an
`UnknownKey` diagnostic, where FR-012 promises one. A key this build does not
know is the most likely thing a later schema added, so reporting it beside the
version fault reports a consequence as though it were an independent problem.

**D-13. A wrongly typed entry in `capture.roles` does not suppress its
siblings.** Also from that review. The first implementation discarded the whole
list when any element failed to parse, which hid the undeclared-role fault the
surviving entries carried. Emptiness is judged on the declared count rather than
the surviving count, so a list with one bad element is not also reported as
empty.

## Complexity Tracking

No constitution violations. Table intentionally empty.

The one entry a reviewer might expect here is the pair of runtime dependencies.
It is not a violation: P-2 constrains `fragcap-core`, which acquires neither,
and the license policy admits both. It is recorded as decisions D-1 and D-3 and
in the changelog decisions fragment, which is where an architectural change of
this kind belongs.
