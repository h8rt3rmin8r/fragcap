# Research: S204 Acceptance Baseline

## Existing implementation

S203 added selected-flow route events, a native route.jsonl stream, a derived bundle debug report, direct-route client-delivery proof, and corrected auto-scope help. Its task record leaves route and correlation edge regressions, full report/access fixtures, controlled non-loopback and package qualification, and terminal parity unchecked. These are concrete S204 work items, not a reason to rewrite the S203 architecture.

## Existing qualification authority

`integration/direct-route-matrix-v1.json` maps required driver rows to ignored disposable probes. `.github/workflows/windows-integration.yml` runs them serially on a disposable Windows runner, repeats the managed chain, builds a public-feature binary, and runs the hosted matrix. `.github/workflows/package-certification.yml` certifies MSI and ZIP packaging. The S204 gate should extend these owners for missing external and packaged rows, rather than add an unrelated test framework.

## Controlled peer

The owner approved the shared cloudbase host provided that existing workloads are not disrupted and all test effects are cleaned. Use a high, checked port and a bounded listener with no remote file. If cloudbase cannot serve IPv6, use another owned non-loopback peer; an unavailable family is a failed qualification row, not passing evidence. The tests must distinguish live selected-route behavior from driver-free fixtures and package structure checks.

## Issue boundaries

#593 and #594 are critical selected-route and calibration boundaries. #595 owns report and event completeness. #598 is active help and installed wording. #596 and #597 describe launcher-chain packet scope and TLS refusals and remain separate even if a direct-route test touches the same module. #580 remains deferred research.
