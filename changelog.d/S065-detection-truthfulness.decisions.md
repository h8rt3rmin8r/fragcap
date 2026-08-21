<!-- spec-impact: 15.7, 17.1 -->
**2026-08-20** The two engine detectors stay separate, bound by a directed subset
invariant rather than folded into one list. fragcap has two mechanisms that read
an install directory about engines: the launch-resolution rules in
`crates/fragcap-profile/src/engine_rule.rs`, and the detection signature table.
Issue #168 preferred folding the first into the second, and that option is
rejected here with reasons, because the two answer different questions. The
launch rules do not merely recognize an engine, they apply a per-engine rule to
pick the socket-holding executable: an Unreal `*-Win64-Shipping.exe` beneath
`Binaries/Win64`, a Unity player named after the `*_Data` stem, a Godot binary
named after the `.pck` stem, a Ren'Py launcher in the root. The signature schema
carries category, kind, pattern, product, and confidence, and nothing that could
express any of those. Folding them would therefore require either extending the
signature schema with a per-engine client-selection rule, a larger change than
the slice that would carry it, or giving up the client selection the resolver
cascade depends on. There is a crate-direction obstacle too: `engine_rule` lives
in `fragcap-profile`, the seed lives in `fragcap-targets`, and `fragcap-targets`
depends on `fragcap-profile`, so making the rules a consumer of the table would
invert that edge or thread a loaded signature set through a cascade that takes no
such parameter.

What ships instead is stronger than the "check they agree" fallback the issue
offered, because the two sets should not agree. The signature table legitimately
names engines nobody has written a client-selection rule for, and requiring
equality would force either a fabricated rule or the removal of a true detection.
The invariant is directed: every engine the launch rules can select a client for
must have an engine-category signature naming the same product, and the reverse
is not required. It is enforced by a test that iterates the `Engine` enum itself
rather than a list maintained beside it, so adding a variant without adding a
signature fails, and it asserts no count and no width. The failure message names
the file to edit.

**2026-08-20** The Steamworks SDK signature rows are dropped rather than
recategorized. Issue #169 offered both: move them to a `platform-sdk` category
with product `Steamworks SDK`, or remove them. A fourth category would need a
fourth rendering bucket, in the same change whose subject is that the existing
columns conflate categories, and it would put a column of noise in front of the
operator on nearly every row, because the library ships with essentially every
Steam title. That a title links the Steamworks SDK is already implied by its
`steam:` anchor. Nothing is lost that was being used.

**2026-08-20** The detection coverage state is a typed column on the target
entry, and the local store schema advances from version 6 to version 7 for it.
The migration is a single additive nullable column with a CHECK set, applied
through the same ladder the five migrations before it use; an existing row keeps
every value and reads the column as NULL, which is exactly "no scan is recorded",
so nothing is backfilled. Backfilling would invent a scan that never ran.

The cheaper alternative, recording it inside the free-form `provenance` blob, was
rejected. It needs no migration and round-trips for free, but nothing validates
it: an out-of-set value there would read as "no scan recorded", which is a
coverage claim the tool cannot check, and #174 calls this distinction a P-4
concern rather than a cosmetic one. The target-entry export states in its own
documentation that its key set is a reviewed contract rather than a serde
accident, and this belongs in that contract. There is deliberately no
`not-scanned` variant in the value set: absence is `NULL`, because a variant
would let a row assert that no scan happened, where absence is simply the lack of
a claim.

**2026-08-20** The widened listing states its width budget over the columns the
tool controls, and overflows rather than truncating. With every bounded column at
its widest, the columns other than the target handle cost 53 of an 80 column
terminal, leaving 27 for a handle. The handle is operator data of unbounded
width, so a longer one overflows, with every value intact and nothing clipped.
Truncating to fit would be the same class of silent loss P-4 forbids for a
dropped packet, and wrapping would break the alignment the split exists to
provide.

This is not hypothetical on the operator's machine, and the first draft of the
slice said it was. The plan claimed a representative row fit in 74 columns, on
the strength of a 22 character handle taken from the sketch in #174. Rendering
the real listing showed the longest handle is 47 characters
(`warhammer_40_000_dawn_of_war_definitive_edition`), and rows running to 100.
The claim is corrected rather than quietly adjusted, because it came from reading
an issue instead of running the command, which is the failure mode the slice
exists to fix. Shortening the handles a target carries is issues #166 and #173.

**2026-08-20** `SignatureKind::is_implemented` is replaced by
`Signature::is_matchable`. Matchability now depends on the pattern as well as the
kind, since a `binary-marker` is matchable in its `section:` form and inert in
every other, so it is a property of the row. This removes a public function from a
pre-1.0 crate. Keeping the old one and adding a second gate beside it was
rejected: it would leave two places to forget, and the one that would be forgotten
is the one that decides whether a signature is silently ignored.

**2026-08-20** `fragcap_profile::pe::fixtures` is public rather than
`#[cfg(test)]`. Three crates now test the same matchers against generated PE
inputs, and a hand-rolled builder in each would be free to drift from the others,
which is precisely how a fixture stops testing what it claims to. It follows the
precedent already set by this workspace's other public fixture types
(`FixtureCatalog`, `FixtureClassifier`, `FixtureSource`, `FixtureEngineFeed`).
