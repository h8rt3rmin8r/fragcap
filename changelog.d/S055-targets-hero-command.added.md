<!-- spec-impact: 15.8, 17 -->

### The targets hero command and interactive authoring (slice S055)

`fragcap targets` (and a bare `fragcap`) becomes the hero command: it runs
discovery across its tiers, registers any newly found titles into `local.db`
idempotently, and presents the registered targets as a numbered table with a
CAPTURE column (`ready` or `needs a target`, derived from whether the entry names a
Windows client or carries a resolvable anchor) and a neutral KNOWN column (the
detected engine, anti-cheat, and DRM products, else "no online mode recorded", else
"no launch data known"). Rows are ordered by handle, the listing ends by naming the
next command (`fragcap capture <n>`), and an empty result prints the concrete
commands that populate the store rather than an empty table. Registration is
additive and idempotent: a repeat listing over an unchanged environment registers
nothing new and never modifies or removes an existing entry.

The listing now writes a durable snapshot of what it displayed to `local.db` (a new
`listing_snapshot` table, schema version 5 to 6), and a bare-integer selector
resolves against that snapshot rather than the live store order. So `fragcap capture
3` names the row the user just saw, even after an intervening add or remove shifts
the live order; a position past the snapshot, or before any listing has run, is an
out-of-range usage error (exit 2), distinct from a clean handle or name miss. This
changes the S054 behavior where `capture <n>` resolved over the live order.

`targets add` gains interactive authoring. Pointed at an executable, it runs
detection on the executable's directory and shows the engine, anti-cheat, and DRM
evidence inline, then asks whether that executable is the process that holds the
sockets: `[Y/n/unsure]`. The `unsure` branch is a first-class outcome and the reason
the prompt exists: it registers the entry with its launch chain unresolved and
records no socket holder the tool did not observe (P-9). `yes` records the executable
as the resolved client; `no` records it as a launcher with the holder unresolved.
When standard input is not a terminal, the same decision is supplied by
`--socket-holder yes|no|unsure` (which requires `--exe`), so every branch is
reachable without a terminal and a required-but-missing value is a usage error rather
than a blocking prompt.

The target lifecycle is rounded out: `targets scan <dir> --db <local.db>` registers
the titles it discovers (through the same idempotent registration the listing uses);
`targets remove <selector>` deletes exactly the resolved target and refuses an
ambiguous name (exit 2); and `targets export [selector]` / `targets import <file>`
move targets between stores as a dedicated JSON array of target-entry objects
(carrying each entry's identity), merging on the stable identifier so an export
round-trips through an import with identical identifiers and no duplicate rows. This
representation is deliberately not the published capture schema, whose export records
are catalog games and omit the entry identity that merge-on-id requires (operator
decision, 2026-08-18); the published schema is neither used for targets nor changed.

The capture-time promotion of an `unsure`-authored row to `verified` is partially
delivered and partially deferred, stated here rather than reported as complete. The
promotion mechanism ships and is tested: `Store::promote_target_launch` rewrites an
entry's launch chain and raises its fidelity, and the launch-chain resolution logic
is unit-tested end to end. The capture-time trigger is deferred: capturing an
unresolved target requires a capture-by-observation mode this architecture does not
have, since `capture --target` refuses a target that names no single Windows client.
Wiring that trigger is follow-up work in its own slice; no dead or fabricated
promotion path was added to stand in for it.
