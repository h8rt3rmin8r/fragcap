# Data Model: S188

## Observed start

`pid`, `parent_pid`, `at`, `image`, and command-line availability are retained from the event source. `image` is never overwritten with a later query result. The process tree continues to identify a generation by creation event and attach its parent only through observed or explicitly weaker snapshot ancestry.

## Verified stage path

A candidate may carry one canonical executable path acquired from a query-only process object. It is eligible for matching only after PID, matching basename, same initiating account and session, and directional creation-time join (`0 <= ETW start time - OS creation time <= 1 second`) succeed. The stage matcher reads this path separately from the observed `image`; when absent, a basename-only event cannot satisfy an exact path predicate.

## Acquisition limitation

A managed candidate has either an eligible verified path or a reason such as `path-unavailable`, `creation-mismatch`, `image-mismatch`, `account-session-mismatch`, `path-mismatch`, or `ancestry-unverified`. Limitations are bounded and counted; they do not create stage transitions or process-control authority. The terminal acquisition stop reason remains a separate field.

## Capture outcome

The existing typed stop reason is retained through the CLI composition layer. Packet frame count and scope, proxy application observations, manifest finalization, and cleanup outcome retain independent authority. `acquisitiontimeout` can coexist with complete proxy exchanges and a valid zero-frame partial pcapng.
