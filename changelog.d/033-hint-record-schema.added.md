The master target schema's loose hint-record subschema (issue #75) was revised so
the targets hint database (#78) can emit conformant JSON, per the Steam catalog
research #83 absorbed (slice S033). Three optional structures now appear on a
`hint` at the top level and inside each `export` record: a `launch` array of a
title's Steam launch configurations (each entry an optional os/arch/launch-type/
beta-branch filter, a required non-empty `executable`, and optional arguments and
description), a `launcher_mediated` boolean marking titles Steam starts through a
publisher launcher, and an `engine` object carrying an optional engine name, a
`source` (`pcgamingwiki` | `exe_heuristic` | `depot_filename_rules`), and a
`confidence` (`confirmed` | `high` | `medium` | `low` | `unknown`).

The launch array is carried whole and is never reduced at seeding time to a single
game binary: for a launcher-mediated title the invoked entry is a publisher
launcher, not the socket holder, and deciding which entry (or descendant) holds
the sockets is the resolution cascade's runtime job (#77), not a seeding-time
transformation. The engine `confidence` is a within-field grading of one heuristic
guess, deliberately not a rung on the record `fidelity` ladder, so a low-confidence
engine guess never silently moves the record's overall trust (P-9); the engine
`source` is likewise distinct from the record's provenance source. A failed engine
lookup leaves the object absent rather than present with a fabricated value.

The change is additive within schema version 1, applied byte-identically to the
embedded and published schema copies, so every pre-existing artifact still
validates and no version bump is made. The three fields are refused on the strict
`profile` and `package` variants and on the export envelope's own top level, so the
authored capture format stays free of hint-seeding metadata. The hand-rolled
variant validator shape-checks the new structures wherever they are permitted, two
new diagnostic codes name an out-of-enum engine source or confidence, and the
conformance corpus gained fixtures for a full valid hint, an export record carrying
the fields, an out-of-enum source, an out-of-enum confidence, a launch entry with
no executable, and a strict profile that carries a hint-only field (rejected). No
runtime code consumes these fields yet; that is #78. No dependency is added and the
minimum supported toolchain stays green.
