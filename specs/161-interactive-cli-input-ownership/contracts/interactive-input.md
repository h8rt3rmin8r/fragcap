# Interactive Input Contract: S161

## Doctor action question

- Display each action already named by Doctor, then `perform this action? [y/N]`; flush before reading.
- A complete `y` or `yes` response, ignoring surrounding whitespace and letter case, confirms exactly that action.
- A complete empty or other response declines exactly that action and prints `skipped`.
- Prompt output failure, flush failure, EOF before a response, or input error stops the action phase, emits an error, and exits 1. It does not print `skipped` for that action or perform it.
- `--yes` retains its existing preconfirmation behavior; terminal-output and JSON gates do not change.

## Target socket-holder question

- Display and flush `Is the executable above the process that holds the sockets? [Y/n/unsure]` before input.
- Existing yes/no/unsure parser and repeat-on-invalid behavior remain.
- EOF maps to unsure, without a positive socket-holder assertion. I/O failure stops registration.

## Warm restart

- Display and flush the existing prompt before input.
- Complete `y` or `yes` is affirmative; other complete lines are negative.
- EOF and I/O failure stop with an error and never initiate restart.

## Deep Capture and calibration authorization

- One complete response per plan; exact identifier for structured mode.
- Existing size bound and rejection of extra input remain.
- Plan write, read, mismatch, expiry, and incomplete-line failures retain no-effect refusal.

## Print Screen

The reported elevated-window hotkey behavior is an observation, not an asserted fragcap contract change in this slice. The S161 implementation and report must not claim it fixed that behavior.
