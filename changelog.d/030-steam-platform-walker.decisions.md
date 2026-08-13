**2026-08-13** The Steam platform-walker refactor (issue #77, slice S030) landed,
completing #77, and six decisions were recorded rather than left implicit.

First, the walker provider lives in `fragcap-steam`, not in `fragcap-profile`
where the rest of the cascade lives. The provider needs Steam knowledge (library
enumeration, the scaffold classifier), and `xtask/src/deps.rs` allows
`fragcap-steam -> fragcap-profile` while forbidding the reverse. So the provider
implements `fragcap_profile::TargetProvider` from `fragcap-steam`, the no-op
`PlatformWalkerProvider` stub is removed from `fragcap-profile`, and the CLI (which
reaches both crates through the facade) assembles the resolver. `fragcap-profile`
gained no dependency on `fragcap-steam`; the dependency-direction check stays green.

Second, a walker answer gets its own `TargetOrigin::PlatformWalker(WalkerTarget)`
variant, distinct from the profile, engine-rule, and observed origins. It carries
the storefront (`steam`), the resolved client's image name and full path, and the
`MatchPredicates` the pipeline binds it by. Two small public constructors were
added to `fragcap-profile` so an external provider can build an answer without the
crate-internal setters: `MatchPredicates::with_exe` (build an exe-only identity)
and a public `Provenance::new`.

Third, the walker declines rather than guess a client. It resolves only when
exactly one plausible client executable remains after dropping non-game
executables and launcher stubs (the shared scaffold predicates); zero is a
no-match and several is an ambiguity, both declines. The scaffold picks the
largest non-launcher for a human-reviewed skeleton the author then corrects; for
automatic capture the walker must not present a guess as the answer (P-9), and the
library research found size-based client selection to be coincidental. Runtime
observation resolves the ambiguous and launcher-mediated cases from the live
socket-holding process, so declining and degrading is both honest and correct.

Fourth, the walker's provenance is `steam-library`, not `steam-appinfo`. The
walker resolves from the library manifests and by classifying install-directory
files; it does not read Steam's application info. Stamping `steam-appinfo` would
claim a source it did not consult, which P-9 forbids. This is a deliberate
deviation from the slice plan's original `steam-appinfo` wording, on honesty
grounds; `steam-appinfo` is reserved for a future slice that actually reads it.

Fifth, reading Steam application info (the `config.launch` launch array via
networked PICS or the local binary `appinfo.vdf` cache) is deferred. It requires
either a heavy networked Steam-client dependency (a large transitive graph and
network I/O in a passive local tool, P-1-adjacent) or a versioned binary-format
parser, and the engine rule plus the install-directory classifier already cover
the common cases while the hard titles degrade to runtime observation. The full
launch-array model and the launcher-mediated flag belong with the hint-database
(#78) revision. Consistent with the project's `boon`/`crossbeam` rejections, no
dependency is added.

Sixth, the walker provider is wired into the production resolver vec but, like the
S029 engine-rule provider, it cannot yet fire a capture in production: `run`
errors on a resolved target that carries no profile, and its module doc already
names driving a non-profile target as a later slice. A profile outranks the walker,
so the walker only matters for a no-profile capture, which needs that non-profile
capture path. Building it is a cross-cutting integration that S027 through S029 all
deferred and that this slice also defers; the enumeration-to-install-root helper
(`install_root_for`/`install_root_in`) is built and tested so the future path has it
ready. The walker, its composition with the engine rule, and its degradation to
runtime observation are proven end to end through the resolver in the facade's
`walker_cascade` integration test. MSRV stays 1.82.
