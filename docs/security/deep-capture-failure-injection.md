# Native Deep Capture Failure Injection

S127 closes the native lifecycle, I/O, and cleanup failure-evidence boundary.
The canonical registry is
`docs/security/deep-capture-failures.v1.json`; `cargo xtask failure-matrix`
rejects incomplete rows, source inventory drift, and stale executable evidence.

## Matrix ownership

The registry owns seven journaled effects and eight executable, checked lifecycle
transitions. The gate extracts the coordinator's edge table and rejects any
registry disagreement, then deterministically expands every boundary into `before`
and `after` scenarios, yielding thirty stable cells. A before-effect failure
must prevent the effect. An after-effect failure treats acquisition as possible,
retains the synchronized obligation, and requires bounded cleanup or an exact
recovery decision. Matrix execution compares all seven declared outcome fields
with the production report and proves effect-side placement from the production
resource journal itself.

Every cell separately declares terminal, artifact, fact, event, cleanup,
journal, and recovery expectations. A successful cleanup cannot turn a failed
writer into a complete artifact, and failed event delivery cannot erase
retained evidence or suppress later cleanup.

## Controlled failure families

The matrix covers disk full, permission denial, broken pipe, task panic,
timeout, cancellation, trust denial, listener port theft, network reset, and
writer corruption. These labels describe the native failure presented at an
owned adapter or writer boundary. Tests use deterministic controlled adapters;
they do not exhaust a host disk, modify the real trust store, take an unrelated
port, launch a game, or route external traffic.

The generated coordinator test lives beside the existing controlled-adapter
harness in `deep_capture_session.rs`. Durable prefix tests use the production
resource journal and its shared Doctor recovery planner. This is deliberate:
there is no test-only runtime switch and no second lifecycle implementation.

## Reproduction

```text
cargo xtask failure-matrix
cargo test -p fragcap --test deep_capture_session --features deep-capture
cargo test -p fragcap --test deep_capture_journal --features deep-capture
cargo test -p xtask failure_matrix
```

The command follows the repository's `0` passed, `1` findings, and `2` unable
to run contract. It is part of `cargo xtask ci`.

## Scope boundary

S124 owns Doctor inventory and confirmed repair. S126 owns parser and artifact
reader fuzzing. S127 proves failure behavior and recovery authority, but adds no
runtime effect, dependency, packaging claim, performance claim, Windows-host
integration claim, or Deep Capture completion claim. Those remain with #326
through #334.
