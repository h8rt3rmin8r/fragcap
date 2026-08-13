**2026-08-13** The non-profile capture path (slice S032) activated the target
resolution cascade for capture, and five decisions were recorded rather than left
implicit.

First, the three target inputs are one required, mutually-exclusive clap group
(`--profile`, `--install-dir`, `--steam`) rather than a separate subcommand. The
operation is a capture, and `run` already owns the effective-config overlay, the
sinks, the orchestrator, and the offline test harness; a clap group expresses the
exactly-one rule declaratively with the standard usage-error exit (2), so no
hand-rolled validation is written and existing `run --profile` invocations are
unchanged. `RunArgs.profile` changed from a required `String` to an
`Option<String>` in the group.

Second, the non-profile branch reuses `assemble::effective_config(args, &profile)`
with the synthesized profile rather than a new overlay. A synthesized profile
declares no capture defaults, so every option comes from the command line, which
is exactly the intended behavior; this keeps the full `run` option surface (mode,
roles, direction, interfaces, sinks, bounds) on the non-profile path for free.
`watch` needed its own `effective_config_for_watch` only because `WatchArgs` is a
smaller argument set; `RunArgs` is the full set, so there was nothing to
specialize.

Third, the synthesized identity is stamped `heuristic-unverified`, never
`authored`. `watch` stamps its synthesized profile `authored` because an operator
typed the identity; here the identity was resolved by an install-layout heuristic
or runtime observation, so `authored` would be a lie (P-9). The schema permits
`heuristic-unverified` on a profile and refuses only `observed`, so the
synthesized profile parses. The game identity is a generic placeholder, plus the
Steam app id as a fact for `--steam`; fabricating a title name would be the kind of
tidy-looking lie the principle forbids.

Fourth, a small pure accessor `Target::identity(&self) -> Option<&MatchPredicates>`
was added in `fragcap-profile` rather than matching the origin enum inline in the
command. The three non-profile origins already expose an identity, but
`TargetOrigin` had no uniform accessor; a single pure accessor keeps the "which
origins carry an identity" rule in the crate that owns the type and is
unit-testable there, mirroring the small accessors S029 and S030 added. It adds no
dependency.

Fifth, a declined non-profile resolution renders the resolver's decline notes
explicitly. The generic `From<ResolutionError>` reduces an unresolved outcome to a
profile-not-found class, and the error's `Display` names the unreadable cases but
not the ambiguity ones, so the ambiguity notes (an engine layout recognized with
several candidate clients, or several plausible clients in an install directory)
are rendered by the command so the surfaced failure names the reason (FR-007). The
`--profile` branch keeps the existing mapping unchanged, preserving its behavior.

One interaction is noted rather than newly handled: `--launch` reads the profile's
Steam app id, which a `--install-dir` synthesized profile does not carry, so a
`--launch` with `--install-dir` fails through the existing launch-build error
rather than a new check; a `--steam` synthesized profile does carry the app id.
No dependency is added and the minimum supported toolchain stays green.
