**2026-08-13** The target-hint-record schema revision (issue #75 follow-up per
#83, slice S033) landed, and five decisions were recorded rather than left
implicit.

First, the three new fields live where a single loose record lives. In the JSON
Schema they are top-level properties (so a `hint`, which is one loose record,
carries them) and members of the `record` definition (so each `export` record
carries them), and a new `allOf` conditional forbids them on `profile`, `package`,
and the `export` envelope top level, mirroring exactly how `records: false` gates
the records array off the non-export kinds. The strict authored format stays clean:
a profile that carries a launch array, a launcher flag, or an engine object is
refused as an unknown key. This keeps the authored-versus-guessed line the fidelity
model protects.

Second, engine confidence is not a fidelity tier. The research's confidence
vocabulary (confirmed, high, medium, low, unknown) grades one heuristic field, the
engine guess, while the record fidelity (authored, verified, heuristic-unverified,
observed) grades the whole record's trust. Remapping confidence onto fidelity, or
letting a low engine confidence lower the record's fidelity, would let one guessed
field silently move the record's overall trust, which P-9 forbids. Both are carried
as independent fields, which is the reconciliation #83 explicitly permits. The
engine `source` is likewise separate from the record's provenance `source`: same
field name across two objects, two different vocabularies, no cross-constraint.

Third, the launch data is modeled as an array and is never flattened. Steam's
`config.launch` is a list, and for launcher-mediated titles the invoked entry is a
publisher launcher rather than the socket holder, so the schema preserves the whole
array with its filters intact and imposes no reduction. Reducing the array to the
one binary that holds sockets is the resolution cascade's runtime job (#77), not a
seeding-time transformation; encoding a reduction in the schema would bake a guess
into the data.

Fourth, the launch-entry filter fields (`os`, `osarch`, `launch_type`,
`beta_branch`) are free strings, not enums; only the engine `source` and
`confidence`, whose vocabularies this project's own research fixes, are enums.
Steam's launch-filter vocabularies are external and evolve, so an enum would reject
a valid Steam value the moment Steam adds one, a correctness cost with no honesty
benefit. The engine `name` is likewise a free string.

Fifth, the change is additive within schema version 1 with no bump, applied
byte-identically to the embedded and published schema copies (the drift check
enforces it), and the hand-rolled variant validator was extended rather than
replaced, gaining `check_launch`, `check_launch_entry`, and `check_engine` helpers
and two new diagnostic codes (`InvalidEngineSource`, `InvalidEngineConfidence`)
mapped in the profile-load path as `InvalidCategory` was. This is the same
additive-extension discipline slice S031 used for the `technologies` structure. No
runtime code consumes the new fields; that is #78. No dependency is added and the
minimum supported toolchain stays green.
