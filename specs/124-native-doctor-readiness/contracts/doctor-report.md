# Contract: Doctor Mode Verdicts and Native Findings

## Human output

Checks retain their stable section order and 80-column wrapping. The final
summary contains exactly:

```text
Capture: ready|not ready
Deep Capture: ready|not ready
```

Blocking check identifiers and existing remediation text remain adjacent to
their checks. A Deep Capture-only failure cannot change the Capture verdict.

## JSON Lines output

Existing check records remain valid one-line JSON objects. Two final verdict
records are appended in this stable order:

```json
{"type":"readiness","mode":"capture","ready":true,"blocking_checks":[]}
{"type":"readiness","mode":"deep_capture","ready":false,"blocking_checks":["Deep Capture/native resource session/resource"]}
```

Native inventory check names derive from stable session and resource identity,
not list position. Their detail exposes stable state and reason identifiers
without secret material. Human and JSON forms derive from the same `Report`
value.

## Exit status

The Doctor command succeeds only when both verdicts are ready. Warnings do not
block either verdict. This preserves the existing overall exit behavior while
making its two inputs explicit.
