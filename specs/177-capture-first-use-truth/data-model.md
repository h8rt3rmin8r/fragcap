# Data Model: First-Use Results

## Current-session readiness

- Existing Doctor check set retains machine prerequisite rows independently.
- `Platform/privilege` names the exact reporting token's elevation fact, status, and elevation action.
- Capture and Deep Capture `ready`, `blocking_checks`, human verdict, and exit derive from that same check set.
- Offline replay does not consume the live readiness verdict.

## Steam setup-only result

- Existing stored target has a verified authored client and stable target id.
- Existing `calibration.steam_client` remains the update authority.
- Terminal `calibration.verdict` adds a setup-complete status, no-reachability reason, durable target identity, and same-store next command.
- No workflow id, attempted case, compatibility fact, or Deep Capture session is inferred.

## First-use help

- Generated parser marks `--launch` as required.
- Embedded examples and flag descriptions use the same command grammar.
- The structured authorization line remains the exact plan identifier plus one LF.
