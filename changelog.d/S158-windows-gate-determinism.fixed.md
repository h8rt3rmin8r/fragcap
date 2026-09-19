<!-- spec-impact: 13.7, 17.2.1, 25.5, 27.3, 28.1 -->

Prevent Windows scheduling from exposing the bounded application queue before its writer is ready, and preserve completed terminal proxy observations without reapplying the earlier Capture deadline. A clean native stop report is now required alongside stopped observation predicates, so owner-thread failure and residue cannot authorize a complete drain. Genuine queue loss, incomplete drain, timeout, cancellation and cleanup failures remain exact and visible.
