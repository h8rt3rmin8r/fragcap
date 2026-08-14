**2026-08-13** The hint-database provider (issue #78, slice S037) wired the
targets store into the resolution cascade, and the decisions behind it were
recorded.

First, the concrete provider lives in `fragcap-targets`, not `fragcap-profile`.
The resolver trait, the precedence enum, the request, and the target types stay in
`fragcap-profile`, but a provider that reads the store cannot live there:
`fragcap-profile` may not depend on `fragcap-targets` (the dependency direction
`cargo xtask deps` enforces), while `fragcap-targets` already depends on
`fragcap-profile` for the JSON-schema validator, so implementing the trait there
adds no edge. This is the exact precedent S030 set by putting the concrete
`SteamWalkerProvider` in `fragcap-steam` and removing the profile-crate stub; S037
removes the `HintProvider` no-answer stub from `fragcap-profile` the same way. The
resolver is assembled and the provider injected at the CLI, the only surface that
legitimately depends on both crates.

Second, the Steam application id reaches the provider through the resolution
request, as a new `steam_app_id` input with a `with_steam_app_id` builder mirroring
`with_install_root`. The request is already the channel by which per-provider
inputs arrive, and a provider whose input is absent declines, so a single `--steam`
request can offer the application id to the hint provider and the install root to
the engine rule and platform walker without the providers interfering. A new
`TargetOrigin::HintDatabase(HintTarget)` variant carries the answer; unlike the
engine-rule and walker origins it carries no on-disk path, because the store knows
the executable name but not the per-install location, and naming a path it did not
read would violate P-9. The engine fact is carried as a plain string, not the
`fragcap-targets` engine type, which `fragcap-profile` cannot name.

Third, the executable-selection rule is fixed and disciplined. A row's launch
entries are restricted to those applicable to Windows (operating-system filter
unset or naming Windows), then reduced to the set of distinct executable file names
compared case-insensitively. Exactly one name resolves; zero declines; two or more
is an ambiguity decline recording the application id and count, mirroring the engine
rule's and walker's ambiguity notes. Selecting among several by any coincidental
signal (order, size) is disallowed, the same no-guessing posture the earlier
providers take.

Fourth, a hint-database read that fails after the store has opened maps to a new
`ProviderError::Hint(String)` variant that aborts the cascade, rather than
declining silently (P-4). The variant carries a message so `fragcap-profile` names
no `fragcap-targets` type. A database that cannot be opened at all never reaches the
provider: the CLI opens the store once at resolver-assembly time, so a corrupt or
wrong-version database is surfaced at the boundary where the operator supplied the
path.

Fifth, the database is supplied for resolution only by an explicit path, through a
`--hint-db` option and a `FRAGCAP_HINT_DB` environment override, and this slice
introduces no automatic database-discovery convention. The explicit path matches
how the existing `targets` subcommands take a `--db`, and the environment override
lets the offline tests point at a scratch store with no developer-machine
dependency, exactly as `FRAGCAP_PROFILE_DIR` does. A default discovery location is a
packaging decision left to a later slice. The provenance label reuses the export's
existing `hint-db` constant so the database has one honest name across its read and
write surfaces.

No new dependency is taken and `Cargo.lock` is unchanged: `rusqlite` and
`serde_json` were already present, and `fragcap-profile` gains no dependency. MSRV
1.82 stays non-binding because the facade's `targets` feature is off in the
default and toolchain-check builds, so the store-reading code is not compiled for
the minimum toolchain, the same posture as `pcap` behind `live`.
