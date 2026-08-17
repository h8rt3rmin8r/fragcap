<!-- spec-impact: 15 -->

A capture target can now be a stored entry rather than a profile file. `fragcap
targets add <name>` registers a target in `local.db`, deriving a unique,
human-readable handle from the name (Unicode-normalized, never purely numeric,
collisions suffixed `_2`), and assigning a durable 63-bit identifier: the low 63
bits of BLAKE3 over a canonical platform anchor such as `steam:620` when one is
given, so two independent registrations of a title merge on identity, or a random
63-bit value otherwise. `fragcap targets list` and `fragcap targets show`
select a target by handle, case-insensitive name, row index, or `--id`; a name
that matches more than one target lists the matches and exits 2 rather than
guessing.

Resolution over the two stores is now fidelity-ordered: a locally authored or
verified target entry outranks the shipped catalog's heuristic hint for the same
title, with the order `authored > verified > heuristic-unverified > observed`. The
four declines that keep the resolver from naming a launcher as the game or
guessing among clients (a sparse row, an engine-only row, a launcher-mediated row,
and a row naming more than one distinct client) are preserved for entries as well.

This slice ships the entry model, handles, the identifier scheme, the selector,
and the fidelity-ordered store read. The remaining v0.5.0 target work is staged
across later slices: the engine and platform-walker providers become target
sources in S052; the JSON export and import of entry documents and their
unification with the master schema land in S055 and S057; and the profile-file
surface is retired by S054's command rework. Until then the `profile` command and
the `--profile` capture selector are unchanged.
