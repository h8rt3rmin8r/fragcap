# Contract: Detection Coverage Warnings

## Human Warning Body

When binary-marker detection skips executable candidates because the scan reached the candidate cap, the shared coverage warning body must include:

```text
binary marker detection for <root> read only the first <max_read> executable candidate(s); <skipped> more were not examined, so technology detection for this root may be incomplete
```

The command that prints the warning owns any surrounding prefix, indentation, color, or stream routing.

## Required Properties

- `<root>` is the scanned root for the `ScanOutcome`.
- `<max_read>` is the configured candidate-read cap.
- `<skipped>` is the exact number of candidate executables not examined.
- The warning is a single line.
- The warning does not promise a user-visible remedy or a cap override.

## Existing Shared Caller Contract

The following surfaces must consume the same shared warning body from the scan outcome:

- `fragcap technologies`
- `fragcap targets`
- `fragcap targets discover`
- target discovery sources that forward coverage warnings
