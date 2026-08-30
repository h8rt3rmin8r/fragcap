# Data Model: Smaller Native Proxy Fallback Spike

## ControlledScenario

Stable S099 identifier, protocol family, fixed request and response bytes with digests, finite deadline, and required proof-point keys. Every endpoint is loopback and every input is synthetic.

## ObservationResult

Backend, scenario, proof point, normalized status, optional protocol, length, digest, and sanitized detail. Status is one of complete, partial, empty, bounded, truncated, unsupported, failed, or not measured. Complete payload results require protocol, length, and digest.

## FallbackRun

Exact candidate version, sanitized environment, complete observations, ten lifecycle trials, and residue results. State moves from planned to running to completed or failed, then sanitized. Raw private data is deleted after sanitization.

## DependencyAudit

Exact features, normal and target package paths, licenses, sources, advisories, root-store paths, Rust 1.82 and pinned-toolchain parse/check/build results, build timings, size, and product graph comparison.

## ThreeWayComparison

One row per scenario and proof point with fallback, `hudsucker`, and `mitmdump` results. Pairwise parity is possible only between complete results whose protocol, length, and digest agree.

## BackendOutcome

Decision date, exactly one selected outcome, references to deciding evidence, no S100 shipping effect, and only bounded implementation obligations for the winner.
