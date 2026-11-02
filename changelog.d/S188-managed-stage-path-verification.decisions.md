<!-- spec-impact: 10.3, 27.3 -->
# S188 managed stage path verification decisions

## 2026-11-02: Keep observed image and verified path separate

The process tree retains the raw ETW image and an optional verified canonical path on one PID generation. The profile matcher uses the verified path for path predicates and the observed image for executable-name predicates. A command line never supplies executable-path authority.

## 2026-11-02: Join a managed candidate to its creation event before matching

A query-only process observation is eligible for path matching only when PID, basename, account, session, exact declared path, and directional creation-time proximity agree. The observed process must have started no later than the ETW event and within one second of it. A later PID reuse, unavailable process, or other mismatch produces a bounded reason and no verified path. Existing managed shutdown receipts retain their separate five-second rule; stage binding does not confer shutdown authority.

## 2026-11-02: Preserve terminal and artifact truth separately

The live no-acquisition outcome carries its typed stop reason into the Deep Capture top-level error. Process-trace limitation rows retain stage identity failures independently from packet scope, proxy observations, manifest finalization, and cleanup results. No capture frame is inferred from a proxy exchange.
