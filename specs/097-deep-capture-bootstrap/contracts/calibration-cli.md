# Contract: Compatibility Calibration CLI

## Command Shape

```powershell
fragcap deep-capture <TARGET> --launch --calibrate <reachability|tls> --launch-case <LAUNCH_CASE> [OPTIONS]
```

`--calibrate` and `--launch-case` are a pair. Both are absent for ordinary Deep Capture and both are present for calibration. Existing target selector forms remain mutually exclusive and unchanged.

## Supported Real Launch Case

S097 accepts `steam-protocol-cold`. Other known launch-case tokens parse and receive a precise pre-side-effect refusal. A declared cold case also refuses if Steam is observed running.

## Reachability Phase

- Permits unknown compatibility facts.
- Requires `--launch`.
- Refuses `--trust-ca`, `--har`, and `--key-log`.
- Never creates a trust manager or changes trust.
- Measures scoped proxy reachability and final ownership within finite displayed deadlines.

## TLS Phase

- Requires `--launch`.
- Requires current, non-stale `proxy-routing=reached-client` evidence for the same target and launch case.
- Requires `--trust-ca` or `--yes`.
- May produce HAR and proxy-owned key-log output when actually observable.
- Never bypasses certificate pinning.

## Plan And Confirmation

Every accepted request emits a full plan before any proxy, trust, launch, bundle, or fact mutation. `--yes` preconfirms the plan but does not suppress it. Without `--yes`, noninteractive input and JSON mode refuse before mutation. A negative or unreadable answer declines safely with no mutation.

## Ordinary Deep Capture

Without `--calibrate`, the current command contract remains intact. Ordinary Deep Capture requires current same-target, same-launch final-client routing evidence and continues to refuse unknown, stale, conflicting, or insufficient evidence.
