# Phase 0 Research: Detection truthfulness and the column split

**Slice**: S065 | **Date**: 2026-08-20 |
**Spec**: [spec.md](./spec.md)

Every question below was answerable from the repository, the two issues'
measurements, and the published PE format. No new dependency is required and
none is proposed.

## R-1: How is a PE section table located and read?

**Decision**: Hand-parse it in `crates/fragcap-profile/src/pe.rs`, beside the
version-string reader that already parses the same file format, and read only a
bounded prefix of the file.

The layout, from the PE/COFF specification:

| Structure | Where | Field used |
| --- | --- | --- |
| DOS header | file offset `0x00` | `MZ` magic; `e_lfanew` (u32) at `0x3C` |
| PE signature | `e_lfanew` | `PE\0\0`, 4 bytes |
| COFF file header | `e_lfanew + 4` | `NumberOfSections` (u16) at `+2`, `SizeOfOptionalHeader` (u16) at `+16` |
| Optional header | `e_lfanew + 24` | length is `SizeOfOptionalHeader`; not read |
| Section table | `e_lfanew + 24 + SizeOfOptionalHeader` | `NumberOfSections` entries of 40 bytes; the name is the first 8 bytes, NUL-padded ASCII |

The optional header is never read, only stepped over, so the 32-bit and 64-bit
variants need no separate handling. That is the whole reason the section table
is cheaper to reach than the version resource: it needs no `.rsrc` walk.

**Rationale**: The alternative is a PE-parsing crate (`goblin`, `object`).
Rejected on the same grounds as every other hand-rolled parser in this
workspace: this is arithmetic over a byte slice with a published, stable layout,
which is the case where a dependency buys nothing. It is not the S09 case, where
the alternative was transcribing a C ABI with nothing checking the offsets;
here the offsets are checked by a fixture whose bytes this repository writes.

**Bounds**: the header and section table sit within the first few kilobytes of
every real image (`e_lfanew` is typically under `0x400`, and
`SizeOfOptionalHeader` is 240 for PE32+). A 64 KiB prefix read covers it with
enormous margin. A file whose section table would fall outside that prefix
reports no section names, which is the honest answer: absent, never guessed
(P-9). `NumberOfSections` is bounded both by the Windows loader's own limit of
96 and by the bytes actually available, so a corrupt count cannot drive an
unbounded read.

**Alternatives considered**: reading the whole file, as the version-string
matcher does with `fs::read`. Rejected: the version matcher only ever opens
files whose names look like images and is already the expensive path, while
this one runs against launch executables that can be hundreds of megabytes.

## R-2: What distinguishes a Steam-DRM-wrapped executable?

**Decision**: the presence of a PE section named `.bind`.

**Rationale**: measured directly by the operator across seven installed titles
(the table in the spec). Three wrapped titles carry it, four unwrapped ones do
not, and all seven ship `steam_api64.dll`. This is measurement, not inference
from documentation.

**Alternatives considered**: keeping the `steam_api*.dll` rows and adding
`.bind` alongside. Rejected: the SDK rows would still fire on nearly every row,
so the column would still be wrong; the point of the slice is that the label
records an observation that was not made.

## R-3: Drop the Steamworks SDK rows or recategorize them?

**Decision**: drop them.

**Rationale**: recorded in the spec's Clarifications. A `platform-sdk` category
would need a fourth rendering bucket, in the slice whose subject is that the
existing columns conflate categories, and a signal that fires on nearly every
Steam title carries no information. That a title links the Steamworks SDK is
already implied by its `steam:` anchor.

**Consequence for existing tests**: three tests currently assert on the Steam
DRM label produced by `steam_api64.dll`
(`crates/fragcap-cli/src/commands/technologies.rs`,
`crates/fragcap-targets/tests/signatures.rs`,
`crates/fragcap-profile/src/signature.rs`). They must be re-pointed at a
signature that still exists, not deleted, so the behavior they cover (a DRM
finding renders in the DRM category) stays covered.

## R-4: Do the new engine patterns work under the existing matcher semantics?

**Decision**: yes, with no change to `compile_pattern`.

| Pattern | Kind | Compiles to | Matches the observed tree |
| --- | --- | --- | --- |
| `data.win` | filename | `(?i)^data\.win$` against a basename | `Shale Hill Secrets/data.win` |
| `Steamworks_x64.dll` | filename | anchored basename | `Shale Hill Secrets/Steamworks_x64.dll` |
| `renpy/` | directory-shape | `(?i)(?:^|/)renpy/` against the relative path, directories carrying a trailing `/` | `renpy/` at the tree root |
| `librenpython.dll` | filename | anchored basename | `lib/py3-windows-x86_64/librenpython.dll`, depth 3, inside the depth-8 walk |
| `*.rpa` | filename | `(?i)^.*\.rpa$` | `game/archive.rpa` |

The glob-to-regex conversion escapes every non-wildcard character, so the `.`
in `data.win` is a literal and cannot match `dataXwin`. The directory-shape
prefix `(?:^|/)` means `renpy/` matches `renpy/` and `lib/renpy/` but not
`notrenpy/`, which is the boundary rule already tested.

**Confidence assignment**: `data.win`, `renpy/`, and `librenpython.dll` are
definitive (product-specific names or a product-specific package directory).
`Steamworks_x64.dll` and `*.rpa` are heuristic: the first is a GameMaker
extension convention rather than a runtime marker, and the second is a generic
archive extension. Deduplication keeps the strongest fidelity per product, so a
tree with both `renpy/` and `*.rpa` reports Ren'Py once, verified.

## R-5: How should the two engine detectors relate?

**Decision**: option (b) from #168, sharpened into a directed subset invariant,
enforced by a test. The signature set is the authority for the engine name set;
the launch-resolution rules declare which subset they can select a client
executable for, and a check fails if any engine they name has no signature.

**Why option (a) was rejected.** The two detectors answer different questions.
`crates/fragcap-profile/src/engine_rule.rs` does not merely recognize an
engine, it applies a per-engine rule to pick the socket-holding executable: an
Unreal `*-Win64-Shipping.exe` beneath `Binaries/Win64`, a Unity player named
after the `*_Data` stem, a Godot binary named after the `.pck` stem, a Ren'Py
launcher in the root. The signature schema carries `category`, `kind`,
`pattern`, `product`, `confidence`, and nothing that could express any of those
selection rules. Folding the detectors into one list would therefore mean
either extending the signature schema with a per-engine client-selection rule,
which is a larger schema change than this whole slice, or giving up the client
selection the resolver cascade depends on. Neither is a trade this slice should
make silently.

There is also a crate-direction obstacle worth recording: `engine_rule` lives in
`fragcap-profile`, the signature seed lives in `fragcap-targets`, and
`fragcap-targets` depends on `fragcap-profile`. Making `engine_rule` a consumer
of the table would either invert that edge or require threading a loaded
`SignatureSet` down through the resolver cascade, which today takes no such
parameter.

**Why the invariant is directed rather than an equality.** The signature set
legitimately names engines nobody has written a client-selection rule for
(Source, CryEngine, RE Engine, and now GameMaker). Requiring the two sets to be
equal would force either a fabricated selection rule or the removal of a true
detection. The failure that actually hurts is the other direction: an engine
the launch rules resolve a client for, that the listing cannot name. That is
what the check enforces.

**Where the check lives**: a test in `crates/fragcap-targets`, which owns the
seed document and already depends on `fragcap-profile` (so it can see the
`Engine` enum). It runs under `cargo test --workspace`, which
`cargo xtask ci` runs, so it is in the ordinary gate.

**How it stays structural**: `Engine` gains a `product_name()` returning the
exact product string the signature set uses, and an `ALL` array. The check
iterates `Engine::ALL`; adding a variant without a signature fails it. It
asserts no count and maintains no second list, which is the failure mode a
prior slice in this campaign shipped.

## R-6: Which executables should the section scan read?

**Decision**: files whose name ends in `.exe`, at walk depth 4 or less, in the
walk's existing sorted order, capped at 64 candidates. Candidates dropped by the
cap are counted; candidates outside the depth bound are not, because the depth
bound is part of the definition of a plausible launch target rather than a
truncation of it.

**Rationale for depth 4**: the deepest launch executable in the measured sample
is ARC Raiders' `Pioneer/Binaries/Win64/PioneerGame.exe`, which the walk sees at
depth 4. Unreal's convention puts every shipping binary at that depth. Depth 4
covers the observed layouts with no margin to spare, which is the right place
for a bound whose job is to exclude redistributable installers and crash
handlers buried deeper in a large tree.

**Rationale for the cap**: without one, an install tree carrying hundreds of
executables would read hundreds of file prefixes. 64 is generous against every
observed layout. The cap is a truncation of a defined set, so it is counted and
the count is exposed on the scan outcome (P-4), and a truncated scan is reported
as incomplete coverage rather than as a clean scan.

**Alternatives considered**: passing the resolved launch executable into
detection so only it is read. Rejected: detection runs during discovery, before
any launch resolution has happened, and the two call sites
(`SteamSource` and the known-roots classifier) have no launch entry to pass.

## R-7: How is the coverage state carried from a scan to the listing?

**Decision**: a new nullable `detection_scan` column on the `targets` table,
schema version 6 to 7, with an additive migration in the existing ladder; a
typed field on `TargetEntry`; an explicit export key; and a value plumbed from
every producing source through `CandidateTarget` and `ClassifierVerdict`.

The value set is `complete` and `incomplete`. `NULL` means no scan is recorded,
which is what a row registered by an earlier build, or by a source that ran no
detection, carries.

**Rationale**: the store already carries five additive migrations built exactly
this way, and the export module states in its own documentation that the JSON
key set is a reviewed contract rather than a serde accident. #174 calls the
three-state distinction a P-4 concern, and a P-4 signal buried in the free-form
`provenance` blob would be unvalidated: an out-of-set value there would read as
"no scan recorded", which is a scan claim the tool cannot check.

**Alternatives considered**: recording it inside `provenance`. Cheaper (no
migration, free round trip) and rejected for the reason above. Also considered:
distinguishing "scanned clean" from "never scanned" by storing `evidence` as an
empty array rather than `NULL`. Rejected: it distinguishes only two of the three
states, and it overloads the presence of a field with the meaning of a different
field.

**Mapping**: a scan is `complete` when it read everything it set out to read,
that is when the outcome has no unreadable paths and dropped no candidate to the
marker cap. Anything else is `incomplete`.

## R-8: What are the column contents and the width budget?

**Decision**: `ENGINE` carries the engine-category products; `SENSITIVITIES`
carries the anti-cheat and DRM products. When a column has no products it
carries a coverage marker instead: `-` for a complete scan that matched nothing,
`incomplete` for a scan whose coverage was reduced, and `not scanned` when no
scan is recorded.

Width, worst case, measured from the rendered output of a real run rather than
computed from the layout:

| Part | Width |
| --- | --- |
| leading indent | 2 |
| row number | 2 |
| gap | 2 |
| handle | operator data, unbounded |
| gap | 2 |
| readiness (`needs a target`) | 14 |
| gap | 2 |
| engine (`Unity, Unreal`, wider than the `not scanned` marker) | 13 |
| gap | 2 |
| sensitivity (`Easy Anti-Cheat`) | 15 |
| **everything but the handle** | **53** |

So a table whose handles are 27 characters or fewer fits 80 columns, and one
whose handles are longer does not.

**The operator's machine does not fit, and the first draft of this document said
it did.** The claim here was originally that the longest handle observed was
`trapped_with_ivy_piper` at 22 characters, taken from the sketch in #174.
Rendering the real listing showed the longest handle is
`warhammer_40_000_dawn_of_war_definitive_edition`, 47 characters, and rows
running to 100 columns. The correction is recorded rather than quietly applied,
because the original number came from reading an issue instead of running the
command, which is the failure mode this slice exists to fix.

That is the declared behavior rather than a defect: with no truncation and no
wrapping, a handle wider than the budget overflows visibly. Shortening the
handles a target carries is #166 and #173, which are slice S066.

The readiness column keeping its two existing labels is still what buys what
margin there is; carrying "ready, no online mode recorded" instead would have
cost 16 more and put the overhead at 69, leaving room for an 11 character
handle.

**Rationale for no truncation and no wrapping**: truncation is the silent loss
P-4 forbids. Wrapping breaks the column alignment the split exists to provide.
A value wider than the budget overflows visibly, which is a legible failure
rather than a lie.

**The coverage marker appears only when a column is empty.** #174's requirement
is specifically that the three states not all render as blank. A reduced-coverage
scan that did find an engine shows the engine; the specific path that could not
be read is already surfaced as a named discovery warning, which is where the
recoverable detail belongs (P-4 asks for the loss to be nameable, not for it to
be repeated in every cell).
