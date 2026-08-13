`run` can now capture a target the resolution cascade resolves without an
authored profile (slice S032). Slices S027 through S030 built a cascade that
resolves a socket-holding client from install layout (an engine rule, a Steam
platform walker) or runtime observation, but `run` refused any resolved target
that had no backing profile, so those providers resolved targets nothing could
then capture. This slice closes that gap.

`run` takes exactly one of three mutually-exclusive target inputs (a clap group
enforces it, so supplying none or more than one is a usage error before any
resolution): `--profile <ref>` (unchanged, byte-identical), `--install-dir <path>`
(resolve the cascade over a given install directory), and `--steam <app_id>`
(resolve the app id to its install directory through the local Steam library
lookup, then take the same path). For a resolved target that has no profile, `run`
reads the resolved target's match predicates (its image name plus any path
anchors), synthesizes a one-stage profile from them, and captures it through the
same launch-agnostic engine `watch` uses.

The synthesized identity is built through the same validating profile
construction an authored profile takes, and is stamped `heuristic-unverified`,
never `authored`: it was resolved by an install-layout heuristic or runtime
observation, not typed by an operator (P-9). Its game identity is a generic
placeholder, plus the Steam app id carried as a fact on `game.app_id` when the
input was `--steam`. The capture reaches the target the same passive way as every
capture: the session arms, folds a query-only startup snapshot to attach to an
already-running target, and attributes from outside the process; no process handle
is opened and no process memory is read (P-1). An install location the cascade
cannot resolve to a single client, an unreadable install tree, or a Steam app id
that is not installed each produce a surfaced command failure (exit 1) that names
the reason and captures nothing (P-4), distinguishable from a game that ran but
sent no traffic; a command-line misuse is a usage error (exit 2). The `--profile`
path is unchanged and its output byte-identical to the existing goldens. A small
pure accessor, `Target::identity`, returns the resolved identity for a non-profile
target. No dependency is added, nothing is added to `fragcap-core`, and the whole
slice lives in `fragcap-cli` over contracts the other crates already expose.
