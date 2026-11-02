# Stage Acquisition Contract: S188

## Identity and ancestry

For a managed Steam root, preserve S163's launch-receipt gate. For each declared client or publisher stage, evaluate the existing `exe`, exact `path_regex`, command-line predicate if declared, and `descends_from` against one process-tree generation. A basename-only start can satisfy the exact path predicate only through the separately verified path joined to that same start. A full-path event retains existing matching when no later query is available. A failed query, wrong path, stale PID, or missing ancestor never substitutes a name-only match.

## Observable results

On success, one stage transition names the declared role and process generation, and the ordinary scoped packet gate opens at acquisition. On failure, process evidence retains the original event and a bounded limitation code while stage transitions remain absent. A finite timeout retains `acquisitiontimeout` in the capture summary, top-level human error, structured error, and process trace; it does not manufacture packets or positive calibration.

## Acceptance matrix

| Case | Stage result | Packet/result truth |
| --- | --- | --- |
| Basename-only exact Steam client under owned root | Client binds before deadline | Controlled target-owned frames appear in analyzer-readable scoped pcapng |
| Basename-only ordered publisher and client | Each declared stage binds in order | Final client opens scoped capture |
| Wrong path or basename conflict | Affected stage does not bind | Named identity limitation |
| OS creation after ETW event or too old for event | Affected stage does not bind | Named creation mismatch |
| Escaped or ambiguous parent instance | Affected stage does not bind | Named ancestry limitation or existing escaped-client stop |
| Path query unavailable | Affected stage does not bind | Named unavailable limitation and later exact stop reason |
