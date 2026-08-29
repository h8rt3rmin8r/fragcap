# Contract: Calibration Status Events

Calibration reuses existing `deep_capture.proxy_started`, `deep_capture.trust`, `deep_capture.launch`, `deep_capture.application`, `deep_capture.bundle`, `deep_capture.cleanup`, and `deep_capture.complete` events.

## Calibration Plan

`deep_capture.calibration_plan` is emitted before confirmation and contains:

- target handle;
- phase;
- declared and observed launch case;
- backend and loopback proxy mode;
- bundle destination;
- launch, observation, shutdown, and cleanup deadlines;
- planned fact families;
- trust action;
- cleanup obligations;
- explicit non-actions, including no system proxy mutation and no publication.

## Calibration Phase

`deep_capture.calibration_phase` marks phase transitions and terminal measurement outcome. Fields are `session_id` when assigned, `phase`, `stage`, `status`, and `reason`. Stable stages include `confirmed`, `proxy`, `launch`, `observe`, `facts`, `finalize`, and `complete`.

## Calibration Complete

The existing terminal Deep Capture event retains bundle and cleanup state. Calibration adds phase and outcome fields or emits a paired terminal calibration-phase record so consumers never infer phase outcome from session state.

## Output Rules

- Every JSON line retains the existing `ts` and `event` fields.
- Human mode summarizes the same lifecycle without requiring parsers to scrape prose.
- JSON mode never prompts. It requires `--yes` and still emits the plan.
- Events never contain certificate material, key-log contents, payloads, tokens, account data, or undeclared local paths.
