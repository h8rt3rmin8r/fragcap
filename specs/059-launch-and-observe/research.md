# Phase 0 Research: Launch-and-observe promotion

## Decisions

### D-1. Observe-mode profile is a two-stage launcher + descends-from client

**Decision**: Synthesize `{ role:"launcher", match:{ exe: observed_exe } }` plus
`{ role:"client", terminal:true, match:{ descends_from:"launcher" } }`.

**Why**: An empty-predicate stage is a hard validation error
(`MatchPredicates::is_empty`), so a bare wildcard is not an option. `descends_from`
is a non-empty predicate and carries no `exe`, so it passes validation and does not
trip `ambiguous_image_match` (which only fires when two stages both carry
intersecting `exe` predicates). The launcher stage binds the observed executable;
its terminal client stage binds any descendant, which is the `no` case (a child
process holds the sockets). The launcher stage is itself captured, so the `unsure`
case where the observed executable holds the sockets is also attributed and
promoted. Confirmed against `crates/fragcap-profile/src/validate.rs`
(`ambiguous_image_match`, empty-predicate check) and
`crates/fragcap/src/session.rs` (`ancestor_bound_to`: a strict ancestor bound to
`launcher` satisfies `descends_from`, and starts are applied in creation order so
the ancestor binds first).

**Alternatives rejected**: A single-stage `{ client: exe=observed_exe }` covers only
the case where the observed executable is itself the holder and cannot bind a child
holder, which is the `no` case the feature most needs. A wildcard stage fails
validation.

### D-2. Dominant holder is an ordered per-owner tally on CaptureStats

**Decision**: Add `holder_tally: BTreeMap<Arc<str>, u64>` to `CaptureStats`,
incremented at the `AttributionState::Resolved` arm in `pipeline/mod.rs` with
`packet.attribution.process`, and folded in `CaptureStats::absorb`. The dominant
image is arg-max by count with a lexical tiebreak over the ordered keys.

**Why**: The image already exists per packet on `Attribution.process` but is
aggregated nowhere. A `BTreeMap` gives a deterministic `Eq`/iteration order (so
`CaptureStats` keeps deriving `PartialEq`/`Eq` and the tiebreak is total), which the
socket-table permutation lesson (S10) says the join order must be. `Arc<str>` reuses
the same shared image string the attribution already holds, so the increment is a
refcount bump, not an allocation, on the hot path.

**Alternatives rejected**: A `HashMap` has no deterministic order, so a two-way tie
would promote a different image between runs. Reconstructing the dominant image from
the written file post-hoc would couple promotion to the sink format and would not
work for a streaming sink.

### D-3. Surface the holder via a CaptureOutcome, not through the summary

**Decision**: `orchestrator::capture` returns `CaptureOutcome { exit: Exit,
observed_holder: Option<Arc<str>> }`. Both drivers compute `observed_holder` from
`report.stats` via a `dominant_holder` helper. extcap ignores it.

**Why**: The completion summary and the file trailers are golden-pinned; a
nondeterministic per-image list in them would churn every golden. A dedicated return
value keeps the tally out of every pinned surface (FR-005) while giving
`capture.rs::run` exactly what it needs to decide promotion.

**Alternatives rejected**: Threading the tally into `CompletionSummary` would break
goldens. A side channel (a mutable out-param) is less clear than a return type.

### D-4. resolve_stored returns a promotion carrier so run can write back

**Decision**: `resolve_stored` returns `ResolvedTarget { profile: Profile,
promotion: Option<Promotion> }`, where `Promotion { target_id: i64, local_db:
PathBuf }` is `Some` only for an unresolved entry that was resolved in observe mode.
`capture.rs::run` reopens the local store at `local_db` and calls
`promote_target_launch(target_id, ...)` after the run when a holder was observed.
extcap takes `.profile` and drops `.promotion`.

**Why**: `resolve_stored` already resolves and opens the local store; carrying the
row id and the resolved store path lets `run` promote without re-deriving path
resolution. The `--process` path (`synthesize_named_profile`) is unchanged and
carries no promotion.

**Alternatives rejected**: Returning a bare `Option<i64>` forces `run` to re-run
store-path resolution, duplicating the `setup_stores` chain and risking divergence.

### D-5. No new direct-exe launcher (hard boundary)

**Decision**: Do not add a `ShellExecuteW`-on-observed-exe launcher. Live launch
stays on the existing Steam-anchored `config.launch` path (already Tier 2). An
operator starts the game by any means and observe-mode captures it.

**Why**: A direct-exe launcher is genuinely-new, CI-untestable launch surface with
no offline analog. The whole promotion path is offline-testable without it. A
direct-exe launcher, if ever wanted, is its own later slice.

## Unknowns resolved

- **Does adding a `BTreeMap` field keep `CaptureStats: Eq`?** Yes; `BTreeMap<Arc<str>,
  u64>` is `Eq` and `Default`. Confirmed no writer/summary blanket-serializes
  `CaptureStats` (both reference named fields), so goldens are unaffected.
- **Where is the observed executable stored?** In the launch entries object: the `no`
  case under `executable`, the `unsure` case under `observed_exe` (see
  `fragcap-targets` `authoring::launch_entries_for`). The new `observed_executable`
  accessor reads `observed_exe` then `executable`.
- **Is the child-holder binding testable offline?** Yes; the offline substrate drives
  a `--process-script` process tree and a `--attr-script` socket ownership map through
  the `RoleStampingAttributor`, so a launcher+child fixture exercises the full path.
