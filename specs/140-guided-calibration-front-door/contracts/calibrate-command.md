# CLI Contract: `fragcap calibrate`

## Synopsis

```text
fragcap calibrate <SELECTOR> [OPTIONS]
fragcap calibrate --target <SELECTOR> [OPTIONS]
fragcap calibrate --id <STABLE_ID> [OPTIONS]
```

Exactly one target input is required. A positional integer retains the existing one-based listing-row meaning. A Steam application identifier is not an implicit registration request.

## Options

The surface accepts existing store overrides, reachability capture bounds, bundle destination, payload choice, explicit warm restart, and same-process structured authorization. It does not expose launch-case, phase, protocol, routing, family, trust, HAR, key-log, client-identity, or bypass selection in S140.

## Decision Event

Every resolved invocation emits `calibration.guidance` with stable target identity, topology, action, status, observed and selected launch cases, reason, images, limitations, process-control truth, and next command. Nullable values remain present as JSON null. Human mode renders the same decision facts.

## Actions

### `run-reachability`

The first S139 step delegates in-process to the existing equivalent:

```text
fragcap deep-capture --id <STABLE_ID> --launch --calibrate reachability --calibration-protocol routing --launch-case <EXACT_COLD_CASE>
```

Selected bounds and authorization are preserved. On success, the next command is `fragcap calibrate --id <STABLE_ID> --local-db <EFFECTIVE_PATH>` so new evidence is reassessed against the same store.

### `ready`

No session starts. The next command is `fragcap deep-capture --id <STABLE_ID> --local-db <EFFECTIVE_PATH> --launch`.

### `operator-action`

Without `--restart-warm`, no session starts. The next command is `fragcap calibrate --id <STABLE_ID> --local-db <EFFECTIVE_PATH> --restart-warm`. With it, the existing S113 confirmation, bounded wait, fresh resolution, authority comparison, and cold verification precede the low-level plan.

Every effective path is emitted as one PowerShell-quoted argument. A generated command must parse back to the same path and resolve the durable identifier from that store.

### `refused`

No execution command is invented. Every typed limitation is emitted before a usage or operational error returns.

## Compatibility

- Existing command, event, store, and public Rust API contracts remain unchanged.
- Existing global JSON, quiet, and silent behavior applies.
- JSON execution still requires `--authorize-stdin`.
- Parent #380 remains open.
