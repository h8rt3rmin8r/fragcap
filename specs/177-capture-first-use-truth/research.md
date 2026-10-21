# Research: Capture First-Use Truth

## Existing boundaries

- `assemble::elevation_refusal` refuses an unelevated live capture before opening the driver. Doctor's `privilege` row currently warns, and `Report::exit`, Capture readiness, and Deep Capture readiness ignore warnings. This is the direct contradiction in #487.
- `prepare_steam_client` verifies a conditional target update, emits `calibration.steam_client` with `authored-client-persisted`, and returns without creating the workflow. The outer terminal verdict falls back to blocked with no target id or next command. This is #494.
- `DeepCaptureArgs::launch` is a non-required boolean in generated help, while `run_with_outcome` refuses its absence. The long-help recipient example crosses a line at ordinary width. This is #495.

## Decisions and alternatives

| Decision | Chosen approach | Rejected alternative and reason |
| --- | --- | --- |
| Unelevated Doctor | Failure-class current-token privilege row with the existing action | A second readiness axis duplicates the command's sole live precondition and can disagree with exit status |
| Steam completion | Distinct terminal setup state after verified write, with no reachability claim | Treating setup as calibrated or leaving the generic blocked fallback misstates what occurred |
| Public launch grammar | Required `--launch` flag in parser and usage | Runtime-only validation keeps generated help misleading |
| PowerShell authorization example | Exact identifier plus `[char]10` through redirected stdin | `.WriteLine()` emits CRLF on Windows and fails the intentional exact-line boundary |
| Output-recipient example | Short, complete single line | An implicit wrapped line is not a valid standalone PowerShell command |

## Verification boundary

Controlled tests can prove the CLI behavior and both privilege classes with injected Windows facts. They cannot claim that an arbitrary installed title or current unelevated desktop can complete Deep Capture. Hosted CI and reviews are checked separately from local tests.
