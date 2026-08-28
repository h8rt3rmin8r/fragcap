# Contract: Doctor Deep Capture CA Check

## Input Contract

The production probe reads only:

- `manifest.json` files within the bounded Deep Capture session-root scan
- bundled CA certificate files named by the established session layout, if present
- current-user Root and local-machine Root certificate inventories

It performs no filesystem or trust-store writes and starts no proxy.

## Output Contract

The classifier emits exactly one `DeepCaptureCa` state. Observable thumbprints use
40 uppercase hexadecimal digits in human and JSON detail strings.

| Evidence | State | Status | Cleanup |
| --- | --- | --- | --- |
| No owned identity observed | Absent | OK | None |
| Owned identity in current-user Root | CurrentUser | OK | None |
| Owned identity in local-machine Root | WrongStore | WARN | Exact resource only |
| Manifest and bundled material differ | Mismatched | WARN | Exact observed resource only |
| Incomplete, malformed, or ambiguous | Unknown | WARN | None |

Every state is non-blocking for passive Capture readiness.
