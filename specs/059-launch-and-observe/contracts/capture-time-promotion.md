# Contract: Capture-time promotion

## Resolution (shared seam)

`resolve_stored(target, inputs, emitter) -> Result<ResolvedTarget, CliError>`:

- For a resolved (`yes`) entry, a Steam-anchored entry, or a `--process` synthesis:
  `promotion == None` (unchanged behavior).
- For an unresolved, non-Steam entry with an observed executable: synthesize the
  observe-mode profile (see `observe-mode-profile.md`) and set
  `promotion == Some(Promotion { target_id: entry.id, local_db: <resolved local
  store path> })`.

Both `capture` and `extcap` call `resolve_stored`. `extcap` uses `.profile` and drops
`.promotion` (extcap never writes back).

## Orchestration

`orchestrator::capture(...) -> Result<CaptureOutcome, CliError>`:

- Returns `CaptureOutcome { exit, observed_holder }` on every path that has a
  pipeline report; `observed_holder = dominant_holder(&report.stats)`.
- On a no-target-acquired path (no report), `observed_holder = None`.

## Write-back (capture command only)

In `capture.rs::run`, after `orchestrator::capture` returns `CaptureOutcome`:

```text
if let Some(Promotion { target_id, local_db }) = resolved.promotion {
    if let Some(image) = outcome.observed_holder {
        let mut store = Store::open(&local_db)?;
        store.promote_target_launch(
            target_id,
            &authoring::resolved_client_launch(&image),
            FidelityTier::Verified,
        )?;
        // progress line / structured event naming the promotion
    }
    // observed_holder == None -> leave the target unchanged (P-9)
}
```

## Guarantees

- **Promote only on observation**: a run that observed a dominant image rewrites the
  launch chain to `[{ executable: <image>, role: "client" }]` and raises fidelity to
  `verified`.
- **Never fabricate**: a run that observed nothing writes nothing to the store.
- **Exit unchanged**: the run's exit code is what it was before this slice; promotion
  is a side effect on success, not a new failure mode. A promotion write error is
  surfaced (it does not silently pass) but does not rewrite a successful capture's
  meaning.
- **extcap does not promote**: the extcap path resolves the same profile and streams;
  the store is not modified.

## Tier boundary

The literal `steam://run` launch of an unresolved Steam-anchored target is Tier 2
(not exercised in CI). The resolve -> observe -> promote/leave decision is fully
offline-verifiable over the scripted-attributor substrate.
