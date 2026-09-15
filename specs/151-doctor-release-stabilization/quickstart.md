# S151 Controlled Validation

## Preconditions

Use the repository Rust toolchain and S151 source checkout. Do not install/execute packages, launch real games, mutate real trust, invoke production Doctor for #372 reproduction, or start real ETW as a new test. Tests inject probe work and use parser-only command trees.

## Focused Validation

Run `cargo test -p fragcap-cli --lib doctor --locked` for controlled delay, state, failure, suppression and existing report checks. Run `cargo test -p fragcap-cli --test cli_reference --locked` and its `--features net` variant for visible options/examples without dispatch. Run `cargo test -p xtask --locked` for pure publication negatives and `cargo xtask spec` for actual current applicability.

Delayed tests report waiting before injected release, preserve original positive/negative/indeterminate values and join work. Fixed-instant tests cover threshold/cadence and parent resumption. Broken output and worker/coordinator panic tests cover lifetime without cancellation claims.

## Whole-Slice Gates

Run `cargo xtask ci`, `cargo xtask msrv`, docs checks and production site build/unit/accessibility tests using verified hidden Windows tooling. Preserve real failures. Automatically push/open S151 PR under explicit authorization, address all comments with at most one manual second round, and give green current-head CI to the human for merge. Do not publish another release.

## External Follow-Up

After separately authorized publication containing S151, the operator can record private elevated first-run/repeat timings and attach only scrubbed phase/duration summaries to #372. Independent #333/#413 and final #334 remain external. Published v0.10.0 does not contain S151 diagnostics.
