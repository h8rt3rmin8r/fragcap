# Quickstart: Inspect Deep Capture Compatibility

## Read The Protocol Boundary

Open the Deep Capture compatibility reference in the documentation site. Check
the row for the traffic family you need before starting a session. The table
distinguishes packet visibility from application semantics and names current
blockers such as certificate pinning, QUIC, and custom protocols.

## Inspect One Local Target

```powershell
fragcap targets show sample-target
```

If facts exist, the compatibility section lists every stored observation with
its launch case, source, and freshness. If none exist, it reports unknown. The
command does not launch the target, start a proxy, or refresh evidence.

## Interpret Evidence

- `observed-run`: fragcap recorded the fact during an authorized run.
- `user-confirmed`: an operator explicitly confirmed the fact.
- `imported-catalog`: the fact came from an external catalog source.
- `stale-observation`: retained historical context, always shown as stale.
- `current`: the row is not marked stale.
- `stale`: the row is explicitly stale or has a stale-observation source.
- `unknown`: no local fact exists. It is not an inferred failure.

## Refresh Evidence

Viewing the matrix is read-only. Refresh requires a separate explicit,
authorized measurement path such as a compatible Deep Capture run. New
launch-specific observations are added as evidence; the display does not erase
or hide older rows.

## Keep Public Artifacts Scrubbed

Use placeholders such as `sample-target`, `sample.exe`, and loopback examples in
committed tests and documentation. Do not publish local title names, accounts,
tokens, filesystem paths, private endpoints, or host identifiers.
