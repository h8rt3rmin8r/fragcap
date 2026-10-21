<!-- spec-impact: 17.2.1, 19.2 -->
An authorized cold Deep Capture session now requests graceful close of its exact launched applications, applies bounded force only to verified session-owned processes, and waits for observed exit before reporting route release. Doctor preserves failed historical cleanup and its reason even when a later resource journal is readable and no current blocker remains.
