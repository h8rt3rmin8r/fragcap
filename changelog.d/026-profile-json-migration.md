Game profiles are now authored and loaded as JSON rather than TOML, conforming
to the `profile` variant of the master schema (issue #76, building on #75). A
profile now declares `kind` and a `fidelity` tier alongside its game identity,
capture defaults, and stages. `Profile::parse` remains the only constructor, so
validation still runs before every capture; loading validates structurally
against the master schema (reusing the S025 validator) and semantically for the
checks a schema cannot express (acyclic descends_from, a single terminal stage,
at least one non-service stage, reachable capture roles, no ambiguous image
match, and glob/regex/duration compilation), reporting every problem across both
layers in one pass. Profile resolution now finds `<ref>.json`. The Steam
scaffold emits JSON carrying `fidelity: heuristic-unverified` and a `notes`
string with the warning that its stage classification is heuristic and must be
verified against a live capture, so that warning survives as structured data a
machine can act on rather than a stripped comment. The `toml-span` dependency is
removed. Capture output is unchanged: an equivalent profile drives byte-identical
pcapng and JSON Lines.
