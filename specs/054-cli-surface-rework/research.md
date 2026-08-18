# Phase 0 Research: CLI surface rework

All decisions below are settled; no NEEDS CLARIFICATION remains. The three
Clarifications recorded in the spec (dropped ad-hoc capture inputs, `catalog update`
scope, `steam` residual) are the scope-level decisions; the ones here are the
mechanical decisions the implementation rests on.

## D1: How `capture --target` resolves a stored target into a capture

**Decision**: `--target` takes an S051 selector (handle, exact case-insensitive
name, 1-based row index, or `--id`) and resolves it against `local.db` using the
existing S051 selector resolution (the same code `targets show` uses, including its
exit-2 ambiguity behaviour). The resolved `TargetEntry`'s `launch_entries` are
reduced to a single client image name by the existing reduction (the
`windows_executables` logic in `fragcap-targets::hint_provider`), and that image
name is fed into the same one-stage `Profile` synthesis `tap`/`watch` already use.

**Rationale**: Reuses S051 resolution and the existing launch-entry reduction, so
`capture` gains no parallel resolution path (P-10). The internal `Profile` type
stays the capture-config representation; only the profile *file* surface is retired.

**Alternatives considered**: A new target-to-Profile bridge type (rejected: the
one-stage synthesis already exists and the reduction already exists; a new type is
duplication). Resolving against `catalog.db` (rejected: registered targets live in
`local.db`; the catalog is consulted during the hint cascade, not as the target
store).

## D2: How `capture --process` synthesizes the capture, with path anchors

**Decision**: `--process <image>` synthesizes the one-stage identity profile
directly from the image name, exactly as `tap` does today, and carries the optional
`--path` (case-insensitive substring) and `--path-regex` anchors that `watch`
already provides, folded onto the same synthesized stage. The hidden offline
substrate flags remain flattened onto `capture`.

**Rationale**: `--process` is the union of the old `tap` (name only) and `watch`
(name plus path anchor) target-identification paths, which were always the same
synthesized-profile mechanism differing only by the anchor. Collapsing them loses
no capability (FR-004).

**Alternatives considered**: Keeping `--exe` (watch's glob) as a third input
(rejected: the issue names exactly two inputs; a raw image name with optional path
anchors covers both old paths).

## D3: Where `--launch`'s platform anchor comes from once `--profile` is gone

**Decision**: With `--target`, `--launch` reads the Steam app id from the resolved
`TargetEntry`'s anchor (its `steam:<app_id>` provenance). With `--process`, there is
no anchor, so `--launch` is a usage error (exit 2) naming the reason. Non-Windows
builds refuse `--launch` as unsupported, as `build_launch` already does.

**Rationale**: The launch request needs a Steam app id; the stored target is now the
thing that carries it (it used to come from the profile). A raw process name carries
no platform identity, so launching it is genuinely unexpressible and must fail
loudly rather than silently (P-9). This is the FR-005 edge case.

**Alternatives considered**: A `--steam <app_id>` flag on `capture` to supply the
anchor ad hoc (rejected by the spec Clarifications: capturing a Steam title is
register-then-capture; the anchor lives on the target).

## D4: clap mechanics for grouped help and a no-subcommand default

**Decision**: Group the top-level subcommands with clap's `help_heading` (via
`#[command(help_heading = "...")]` on each variant, or the derive equivalent) into
Capture, Targets, Environment, and Data. Make the top-level subcommand optional
(`Option<Command>`) so bare `fragcap` parses; when `None`, dispatch runs the
`targets` listing and prints the footer. An explicit `targets` (with no subcommand,
or the list subcommand per the current grammar) runs the same listing without the
footer, distinguished by a flag threaded from the dispatch (the bare path sets
"append footer", the explicit path does not).

**Rationale**: `help_heading` is clap's supported grouping mechanism and is purely
presentational (nothing hidden, FR-014). An optional top-level subcommand is the
standard clap idiom for a default action, and the footer difference is a single
boolean decided at the dispatch site, keeping bare and explicit listings
byte-identical except for the footer line (SC-004).

**Alternatives considered**: A hidden `list` default command (rejected: an optional
subcommand is cleaner and keeps `fragcap` and `fragcap targets` sharing one listing
implementation). Printing the footer from inside the listing (rejected: the listing
must be reused by explicit `targets` without the footer, so the footer belongs at
the caller).

## D5: The `catalog` namespace and `catalog update` shape

**Decision**: Introduce a `Catalog` top-level command with subcommands `import`,
`export`, `seed`, `seed-engine`, `seed-signatures`, and `update`, moving the first
five verbatim from `TargetsCommand` (same args, same store handling, now bound to
`catalog.db` by name). `catalog update` is the fetch-the-published-catalog command;
its live network fetch reuses the existing net-gated seeder (`fragcap::targets`
catalog source, S035), compiled behind the `net` feature and not run in CI. When no
published catalog is reachable, it reports that honestly (P-9) rather than
fabricating a result.

**Rationale**: The five commands already write the catalog store; moving them under
`catalog` is a relocation, not a rewrite. `catalog update` establishes the command
home the issue names while reusing the seeder machinery, matching how S035 is
already gated and tested.

**Alternatives considered**: Implementing a brand-new published-catalog fetch and
remote artifact this slice (rejected by the spec Clarifications: out of scope;
establish the command and wire it to existing machinery).

## D6: `targets add --steam <app_id>` replacing `steam profile <app_id>`

**Decision**: Add a `--steam <app_id>` option to `targets add` that resolves the
installed Steam title (the same enumeration `steam profile` used) and registers it
as a target in `local.db` through the existing `targets add` path, carrying a
`steam:<app_id>` anchor. Remove the `steam profile` subcommand. `steam` retains
installed-title enumeration and Steam metadata reads.

**Rationale**: The result of scaffolding a Steam title is a stored target in the
user store, so it is a `targets` operation (P-10, and the namespace-follows-store
rule). Reuses the S051 `targets add` registration and the Steam enumeration.

**Alternatives considered**: Keeping `steam profile` as an alias (rejected: no
aliases, no userbase to shim for, per FR-001's spirit and issue section 2.2).

## D7: Removal set and stale-reference sweep

**Decision**: Remove `run`/`tap`/`watch` (and `RunArgs`/`TapArgs`/`WatchArgs`, the
`run.rs`/`tap.rs`/`watch.rs` command modules), and the whole profile-file surface
(`profile` command, `ProfileArgs`/`ProfileCommand`, `profile.rs`, the
`--profile-dir` global, the file-backed profile provider, and the `--profile`
capture selector). Sweep every shipped doc example and the master-spec section 17
for references to a removed or relocated command; a stale example is a defect this
slice must not leave (FR-017). `schema validate` stays.

**Rationale**: The profile-file surface is one coherent unit the S051 US5 deferral
assigned here; removing the selector before the `capture` replacement exists would
break capture, which is why S051 deferred it to now.

**Alternatives considered**: Leaving the file provider in place unused (rejected:
dead code contradicting "profiles are no longer files"; P-10 wants one target path).

## D8: Verification approach

**Decision**: Drive every capture assertion through the hidden offline substrate
(recorded source, scripted attributor, scripted process timeline) so the five
captures are tested with no capture driver, no elevation, no game. Assert the
removal negatives by parse failure, the namespace moves by store effect, and the
presentation by `--help` and bare-invocation output. Local gate under the GNU-host
toolchain; the real MSVC `cargo xtask ci` gate runs in CI.

**Rationale**: Matches the project's standing tier-1 testability (P-3 seam) and the
S050-S053 local-build note.
