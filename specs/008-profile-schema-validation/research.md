# Research: Profile Schema, Parsing, and Validation

**Slice**: S05 | **Date**: 2026-08-09 | **Plan**: [plan.md](plan.md)

Phase 0 output. Each entry records a decision this slice had to make, the
alternatives, and the evidence. Where a claim could be measured it was measured,
and the command is included so the measurement can be repeated rather than
trusted.

## R-1. Which TOML parser

**Decision**: `toml-span` 0.7, default features.

**Why the question is not settled by reputation.** The default answer is `toml`,
and it fails a hard constraint this workspace has. Measured with `cargo info`:

```text
toml 1.1.4          rust-version: 1.85    MIT OR Apache-2.0
toml 1.0.7          rust-version: 1.76    MIT OR Apache-2.0
toml_parser 1.1.3   rust-version: 1.85    MIT OR Apache-2.0
toml-span 0.7.1     rust-version: 1.70    MIT OR Apache-2.0
basic-toml 0.1.10   rust-version: unknown MIT OR Apache-2.0
```

The workspace declares 1.82 and `cargo xtask msrv` builds against it, so 1.85 is
out. Pinning `toml = "~1.0"` looks like the fix and is not, because the
resolution underneath it is:

```sh
cargo tree -e normal   # with toml = { version = "~1.0", default-features = false, features = ["parse", "std"] }
```

```text
└── toml v1.0.7+spec-1.1.0
    ├── serde_spanned v1.1.1
    ├── toml_datetime v1.1.1+spec-1.1.0
    ├── toml_parser v1.1.3+spec-1.1.0
    │   └── winnow v1.0.4
    └── winnow v1.0.4
```

`toml_parser` resolves to 1.1.3, which declares 1.85. Holding the minimum would
mean adding a direct dependency on a crate this slice never calls, purely to
constrain it, and one `cargo update` would undo the arrangement without anything
failing loudly. That is a maintenance trap in a repository that treats the
minimum toolchain as a gate.

**What `toml-span` measures at.**

```sh
cargo tree -e normal   # with toml-span = "0.7"
```

```text
└── toml-span v0.7.1
    └── smallvec v1.15.2
```

Two crates against five, no serde in the graph, and 1.70 against 1.82 leaves
headroom rather than sitting one release below the ceiling.

**The property that decided it, beyond the numbers.** The deliverable of this
slice is diagnostics, and `toml-span` exists to support diagnostic-quality
reporting: every value carries a span. Verified against the section 15.2
example, including the array-of-tables and inline-table forms the schema uses:

```text
top keys: ["capture", "game", "schema", "stage"]
schema span: Span { start: 10, end: 11 }  value int: Some(1)
stage count: 2
  role="launcher" span=Span { start: 259, end: 267 } predicates=["exe", "path_contains"]
  role="client"   span=Span { start: 395, end: 401 } predicates=["exe"]
syntax err: expected a value, found an equals  span=Span { start: 4, end: 5 }
duplicate key refused: duplicate key: `a`
```

Three things came out of that run and are worth carrying forward. Stage order
inside the array is preserved, which FR-008 requires. Duplicate keys are refused
by the parser, which is the behavior the spec's edge case assumes rather than
something fragcap has to implement. And spans are byte offsets rather than line
and column, so the conversion is ours; plan decision D-4 gives it one home.

**Alternatives rejected.**

- `toml` 1.1: minimum toolchain 1.85. Not available.
- `toml` 1.0 pinned: effective minimum still 1.85 through `toml_parser`, and
  holding it requires pinning a crate we do not call.
- `toml_parser` directly: 1.1.3 declares 1.85, same wall. It is also a low-level
  event parser, so the document model would be ours to build.
- `basic-toml`: declares no minimum toolchain, which is worse than declaring a
  high one because nothing checks it, and it reports errors without spans, which
  removes the property the diagnostics are built on.

**One measured divergence, found by the analyze gate.** `toml-span` is not a
whole-language TOML 1.0 implementation. Probed directly:

```text
basic string escapes (\t é \"):        ACCEPTED
literal string, Windows path, backslashes:  ACCEPTED, preserved verbatim
multi-line basic and multi-line literal:    ACCEPTED
dotted keys, quoted keys, inline tables:    ACCEPTED
arrays, nested arrays, arrays of tables:    ACCEPTED
integers (1_000, 0xff, 0o755, 0b1010, -17): ACCEPTED
floats (3.14, 5e+22, inf, nan), booleans:   ACCEPTED
offset datetime 1979-05-27T07:32:00Z:       refused, "invalid number"
local date 1979-05-27, local time 07:32:00: refused, "invalid number"
a = 01 (leading zero, illegal in TOML):     refused, correctly
unterminated string, table redefinition:    refused, correctly
```

Datetimes are legal TOML and this crate refuses them. That falsified FR-002 as
first written, which claimed a parser implementing the language rather than a
subset, and the requirement was corrected rather than the finding explained
away.

The divergence is confined to profiles that are invalid regardless. No key in
schema version 1 has a datetime type, so a datetime can appear only as a
wrong-typed value or under an unknown key, both of which are refusals. What
changes is the message: a syntax diagnostic instead of a `WrongType` diagnostic
located at the offending key. That is worse, and it is worth less than the two
things it buys, which are a minimum toolchain this workspace can actually meet
and a graph of two crates instead of five.

The line that mattered most in that probe is the second one. A profile author
writing `path_contains` for a Windows path will reach for a literal string to
avoid doubling every backslash, and that form parses with the backslashes
preserved. FR-002a states it as a requirement so it is tested rather than
assumed, and the datetime behavior is pinned by a test so the next reader finds
a recorded decision instead of a surprise.

**A note on serde.** No serde at runtime, and this is not merely avoided: a
derive-based deserializer returns the first error and stops, which is what
FR-013 forbids. The requirement rules out the ergonomic path, so the ergonomic
path's dependency was never on offer. S07's `serde_json` remains test-only and
this slice does not disturb that argument.

## R-2. Glob matching and the intersection decision

**Decision**: hand-rolled, one dynamic-programming walk serving both questions.

Section 15.4 requires deciding whether two `exe` patterns "can match the same
image name". That is glob intersection. Every glob crate answers glob matching,
which is the intersection of a pattern with a literal, so a crate would supply
half the requirement and the harder half would still be written by hand, leaving
two implementations of one syntax to drift apart.

**The syntax, stated completely** so that "well-formed" is decidable: `*`
matches any run of characters including none, `?` matches exactly one character,
every other character is a literal, and there is no escape sequence. No escape
is needed because Windows forbids `*` and `?` in a file name, so a pattern
containing one always means the wildcard. Every string of at most 255 characters
is therefore a well-formed pattern, so FR-020's check can fail in exactly two
ways: an empty pattern, refused because it matches only the empty image name,
and a pattern above the length limit, refused for the reason in the complexity
note below.

**The decision procedure.** Reachability over a table indexed by position in
each pattern. A cell is reachable when the corresponding prefixes can be
produced by a common string; `*` on either side may consume nothing or advance
the other side; `?` consumes exactly one character on the other side, which for
a `?` against a literal means that literal and for `?` against `?` means any
character; two literals must be equal under case folding. The intersection is
non-empty when the final cell is reachable. Matching one name is the same walk
with one side's literals fixed.

**Case handling and a fidelity note.** Section 10.3 makes `exe` comparison
case-insensitive, so the walk compares case-folded characters. The fold is
applied to copies at comparison time; the pattern stored on the parsed profile
is the author's text verbatim, which is what FR-009 requires. A reviewer
encountering a lowercase conversion in `glob.rs` should read it against this
distinction rather than as a P-9 violation.

**Complexity, and a bound this entry originally got wrong.** The pass is
quadratic in stage count, and each decision allocates a table linear in the
product of the two pattern lengths. The first version of this entry concluded
that the one mebibyte file limit bounded both, and that conclusion is false: the
file limit bounds each factor and not their product. Two `exe` patterns of half
a megabyte each fit inside a one mebibyte profile and ask for a table of roughly
10^12 cells, which aborts the process rather than returning a diagnostic. Worse,
the total work is invariant under how the bytes are split: with `k` stages of
pattern length `L`, the pass costs about `(kL)^2 / 2`, so capping only one
factor moves the cost around without reducing it.

Pull request 11's review found this. Two limits now bound it, and both are
answers to the domain rather than round numbers:

- An `exe` pattern is capped at 255 characters. `exe` matches one Windows file
  name component and Windows caps that component at 255 characters, so a longer
  pattern is longer than anything it can be compared against.
- A profile is capped at 64 stages. The focal titles of section 5.4 declare two
  and three, so 64 is two orders of magnitude beyond any plausible launcher
  chain, and it bounds the pairwise pass at 2,016 decisions.

Together the worst case is about 1.3 times 10^8 cell visits and 64 kibibytes of
peak table, which is well under a second and a rounding error of memory. A
`debug_assert` in the walk states the invariant it depends on, so a future
caller that bypasses `ImagePattern::new` fails loudly in a test build rather
than quietly asking for a terabyte.

The general lesson is worth keeping: a quadratic pass over input an operator did
not write is not made safe by bounding the input, only by bounding the factors
the quadratic is taken over.

## R-3. Which regular expression engine, and where it lives

**Decision**: `regex` 1.13 with `default-features = false` and features `std`
and `unicode`, in `fragcap-profile`.

```sh
cargo tree -e normal   # regex = "1.13" (defaults)
```

```text
└── regex v1.13.1
    ├── aho-corasick v1.1.5
    │   └── memchr v2.8.3
    ├── memchr v2.8.3
    ├── regex-automata v0.4.18
    │   ├── aho-corasick v1.1.5 (*)
    │   ├── memchr v2.8.3
    │   └── regex-syntax v0.8.11
    └── regex-syntax v0.8.11
```

```sh
cargo tree -e normal   # default-features = false, features = ["std", "unicode"]
```

```text
└── regex v1.13.1
    ├── regex-automata v0.4.18
    │   └── regex-syntax v0.8.11
    └── regex-syntax v0.8.11
```

Two crates removed. `aho-corasick` and `memchr` accelerate literal scanning
across large haystacks; here a haystack is one image path matched a few times
per session, so the acceleration is unmeasurable and the crates are cost without
return. Correctness is unaffected: the dropped features are optimizations.

Behavior verified against the trimmed build:

```text
regex ok: true                  # (?i)elder\s+scrolls
unicode class ok: true          # \p{Greek}+
pathological refused: Compiled regex exceeds size limit of 10485760 bytes.
malformed refused: regex parse error
```

The third line is the answer to FR-046e. The engine already refuses a
pathological pattern through its own compiled-size limit, with a message an
author can act on. fragcap forms no second opinion about which patterns are too
large, because a second opinion is a thing to keep in step with the first.

**`regex-lite` rejected.** One crate and no dependencies, which is attractive in
this workspace. Rejected because its Unicode support is reduced and an image
path can contain non-ASCII characters through a user directory or a localized
install location. A profile author writing a pattern expects the documented
engine's behavior, and matching under quietly different Unicode rules produces a
wrong binding rather than an error, which is the failure mode this whole slice
is organized against.

**Placement.** Specification section 8.2 puts matching in `fragcap-profile`, so
the engine that validates a pattern here is the engine that evaluates it in S12.
That is a requirement (FR-019) rather than a coincidence of crate choice:
validating with one engine and matching with another would let a pattern pass
validation and fail during a capture.

## R-4. How every problem gets reported

**Decision**: parse into a private draft, then validate the draft, accumulating
into a sorted set.

The `?` operator is the wrong shape for the whole slice. Structural extraction
pushes a diagnostic and continues with the field absent, so a profile missing
three fields reports three. Semantic checks then run over a draft where any
field may be missing, and each check either fires or is skipped for want of an
input, never aborts.

Two places stop accumulation, and both are deliberate.

A TOML syntax fault yields one diagnostic and nothing else, because a document
that did not parse has no tables to check and recovering into a guess would
report faults against a file the author did not write.

An unsupported schema version yields one diagnostic and suppresses the semantic
set, because every other fault in the file is likely a consequence of reading a
later schema with this one's rules. Reporting forty unknown-key faults when the
real answer is "this profile is newer than your fragcap" is worse than useless;
it is misleading.

**Ordering.** Diagnostics are collected then sorted by byte offset and then by
code. `toml-span` iterates tables in key order rather than document order, so
traversal order is neither the author's reading order nor stable against a
container change inside a dependency. Sorting is one line and removes both
problems.

## R-5. Resolution, and the traversal question

**Decision**: caller-supplied search path; slug check on steps two through four
only.

Section 15.3's four steps are an order over locations, and the locations are
platform knowledge. Taking them as an argument keeps `fragcap-profile` free of a
`dirs`-style dependency, keeps the order testable against directories a test
builds, and puts the platform question in S14 where a command line already has
to know it.

**The traversal case is real and is closed twice.** A profile reference arrives
from a command line argument, and steps two through four join it to a directory.
A reference of `../../../windows/system32/drivers/etc/hosts` would otherwise
reach outside every search directory. The slug rule (lowercase ASCII
alphanumerics, hyphen, underscore) refuses it before any join happens, which is
why the requirement is written as "before any path is joined" rather than "the
open fails": a check that relies on the open failing is a check that depends on
what happens to be at the target.

Step one is exempt on purpose. An operator who types a path has named a file,
and refusing an absolute path there would break the ordinary case section 15.3
puts first. The distinction is between naming a file and interpolating a name
into a search path, and only the second is a traversal surface.

`game.id` gets the same charset rule at validation, because resolution step four
matches on it and a bundled profile is a file that gets named. Two checks in two
places rather than one shared assumption, since resolution can be reached with a
reference that never passed through validation.

**Present but unreadable is an error, not a skip.** A missing search directory
is the ordinary state of a fresh install and is skipped. A candidate file that
exists and cannot be read has already won its step, and falling through would
silently select a profile the operator did not choose. Distinguishing the two is
the difference between tolerating an absence and hiding a failure.

## R-6. The duration grammar

**Decision**: `fragcap-core`, one integer and one required unit from `ms`, `s`,
`m`, `h`.

Section 25.2 lists duration parsing as a tier 0 concern without saying which
crate owns it, and three consumers are visible: `capture.duration` here,
`--duration` and `--wait` in S14, and the ring window in S16. Core is the only
crate all three reach without a sibling edge, which section 8.3 forbids, and the
grammar adds no dependency there so the allowlist is untouched.

Keeping it in `fragcap-profile` was the alternative. It fails on S16: a ring
window would either depend on a sibling or carry a second grammar, and two
implementations of `30m` that disagree produce a capture of the wrong length,
which is a defect an operator cannot see in the output.

**What is refused and why.** A bare integer, because its unit would be a guess
and the guess is about how much of a session an operator loses. Zero, for the
reason S08 rejected a zero-capacity buffer: the only possible meaning is a
mistake. Fractions, signs, internal whitespace, and unknown units, because each
would otherwise be interpreted rather than refused. Overflow, refused rather
than saturated, because a saturating parse turns a typo into a capture that runs
for a hundred years.

Compound forms such as `1h30m` are refused and may be added later. Widening an
accepted syntax keeps every profile written today valid; narrowing one does not.
The conservative direction is the reversible one, which is the only argument
needed.
