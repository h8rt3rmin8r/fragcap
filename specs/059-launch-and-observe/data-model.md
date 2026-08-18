# Phase 1 Data Model: Launch-and-observe promotion

No persistent schema change. `local.db` already carries `launch_entries` and
`fidelity` columns; `promote_target_launch` (S055) rewrites them. This slice adds
in-memory carriers only.

## New / changed types

### `CaptureStats.holder_tally` (fragcap-core, changed)

```text
holder_tally: BTreeMap<Arc<str>, u64>
```

- Per socket-holding process image, the count of attributed packets in this run.
- Ordered map so `CaptureStats` keeps `PartialEq`/`Eq` and the dominant-image
  tiebreak is total and deterministic.
- Incremented once per `AttributionState::Resolved` packet, keyed by
  `Attribution.process` (an `Arc<str>` clone / refcount bump).
- Folded per key in `CaptureStats::absorb` (several capture threads add their tallies).
- Additive: contributes to no drop total, no conservation term, no writer, no
  completion summary. Not `set` by `set_nth`/`sample` drop tests.

### `CaptureOutcome` (fragcap-cli orchestrator, new)

```text
CaptureOutcome {
    exit: Exit,
    observed_holder: Option<Arc<str>>,
}
```

- The value `orchestrator::capture` returns. `exit` is what it returned before.
- `observed_holder` is the dominant image from `report.stats.holder_tally`, or
  `None` when the run attributed nothing. extcap ignores it.

### `ResolvedTarget` (fragcap-cli target_resolve, new)

```text
ResolvedTarget {
    profile: Profile,
    promotion: Option<Promotion>,
}
```

- What `resolve_stored` returns. `profile` is the validated capture profile.
- `promotion` is `Some` only when an unresolved entry was resolved in observe mode.

### `Promotion` (fragcap-cli target_resolve, new)

```text
Promotion {
    target_id: i64,    // the resolved entry's durable row id (TargetEntry.id)
    local_db: PathBuf, // the local store the entry was resolved from
}
```

- Carries exactly what `capture.rs::run` needs to reopen the store and call
  `promote_target_launch` after the run.

## State transition: a stored target's fidelity/launch chain

```text
registered (no/unsure)          resolved in observe mode        after a run that
launch = { observed_exe/         profile = 2-stage observe       observed a holder
  executable, socket_holder:  ->                             ->  launch = [{ executable:
  "unresolved" }                 promotion = Some{id, db}         <observed image>,
fidelity = authored/observed                                     role: "client" }]
                                                                 fidelity = verified

                                                                 after a run that
                                                             ->  observed nothing:
                                                                 unchanged (P-9)
```

## Accessor added (fragcap-targets authoring)

```text
observed_executable(&TargetEntry) -> Option<&str>
```

- Reads the launch entries object: `observed_exe` (unsure case) then `executable`
  (no case). Returns `None` when neither is present (nothing to observe from).
- Sits beside `launch_is_unresolved` / `resolved_client_launch`, so all launch-shape
  knowledge stays in one module.
