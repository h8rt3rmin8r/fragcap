# Contract: Doctor Live Probe

## Scope

This contract governs the capture driver and interfaces probe used by `fragcap doctor`. It does not define a public CLI output format and does not add a stable machine-readable field.

## Inputs

- `live backend linked`: whether the binary includes live capture support.
- `wpcap loadable`: whether `wpcap.dll` can be loaded.
- `enumerate interfaces`: live backend operation returning either an interface inventory or an enumeration error.

## Outputs

- `interfaces`: records to feed the existing doctor interface report.
- `loopback_supported`: three-valued loopback verdict.
- `interface_error`: existing doctor input for enumeration failure.

## Rules

1. If the live backend is absent, the probe does not enumerate and returns `loopback_supported = None`.
2. If `wpcap.dll` is not loadable, the probe does not enumerate and returns `loopback_supported = None`.
3. If enumeration fails, the probe returns the error, no interfaces from that failed call, and `loopback_supported = None`.
4. If enumeration succeeds, the same inventory supplies both `interfaces` and `loopback_supported`.
5. A successful inventory yields `Some(true)` when any record has explicit loopback evidence or the existing npcap loopback description marker.
6. A successful inventory yields `Some(false)` only when no record matches the loopback predicate.
7. The successful path performs exactly one live device-list enumeration through the doctor probe seam.
