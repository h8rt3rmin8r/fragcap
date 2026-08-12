A target resolution cascade now decides what to capture for a game (issue #77,
the resolver core). A `TargetResolver` consults an ordered set of providers of
varying trust and returns the highest-precedence available answer, a `Target`,
stamped with a targeting fidelity tier (`authored` > `verified` >
`heuristic-unverified` > `observed`) and a provenance. Two providers carry data:
the profile provider wraps the section 15.3 lookup and stamps its answer with the
profile's own declared fidelity, and the runtime-observation provider matches a
live process by identity (an exe image name plus optional path anchors) and stamps
an `observed` answer, using only the process snapshot and opening no process
handle. Three providers (hint database, engine rule, platform walker) are
registered and decline until their own slices fill them, so adding their data is
additive. The precedence order is total and imposed rather than incidental,
proven by a permutation test. The in-memory `Profile` now retains and exposes the
`kind`, `fidelity`, `provenance`, and `notes` it declares, which it previously
discarded after validation. The `run` command resolves through the cascade and
captures byte-identically. This targeting fidelity is separate from the
attribution fidelity (`live`/`retained`/`none`) and neither is derived from the
other. No dependency is added and nothing is added to `fragcap-core`.
