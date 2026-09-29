# Quickstart: S162 release verification

Do not install or execute released fragcap bytes, a real game, production Doctor, real trust mutation, or sensitive live capture on the local machine.

## Candidate

Preview `scripts/New-Release.ps1 patch -DryRun`, then execute the wrapper's documented version-only cargo-release command and owning golden and changelog steps directly on the existing `release/0.10.3` branch. The wrapper's non-dry-run preflight accepts only clean `main` and attempts to create this branch, so it cannot run after the S162 spec gate. Inspect the version, golden, changelog, and notes diff before running repository gates.

```powershell
cargo xtask ci
cargo xtask msrv
cargo xtask neutral
cargo xtask docs build
cargo xtask spec
cargo xtask notes 0.10.3
```

Use `CARGO_BUILD_JOBS=1` if the host cannot complete parallel Rust compilation. The candidate PR must pass hosted checks on its final head before operator merge.

## Publication

Verify `v0.10.3^{}` against the exact merged candidate, watch the four release jobs, and give the operator the release run and `crates-io` environment link if approval is requested. Download public files only for independent digest and certification validation; do not install or execute them.

## Registry

Query each of the ten crates.io 0.10.3 version records and require `yanked=false` with a nonempty checksum. Bounded propagation checks observe the registry; they do not retry publication.
