<!-- spec-impact: 26.3 -->

**2026-08-26** Recorded S079 doctor probe timing evidence before optimizing
suspected slow checks. A terminal run of `cargo run -p fragcap-cli -- doctor
--timings` measured Deep Capture readiness as the dominant local probe at 623
ms, target stores at 2 ms, and platform, capture driver/interface, analyzer
integration, identity, process event tracing, and report rendering at 0 ms each.
The command exited 1 because the local dev binary lacked the live backend, not
because timing failed.
