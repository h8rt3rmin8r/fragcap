# Implementation Plan: Native Deep Capture Parser Fuzzing

**Branch**: `codex/126-native-parser-fuzzing` | **Date**: 2026-09-03 | **Spec**: `specs/126-native-parser-fuzzing/spec.md`

## Summary

Publish an exhaustive registry of fragcap-owned native parser and artifact
surfaces, expose bounded byte-oriented exercise seams, replay minimized
synthetic seeds under the stable test gate, and run isolated libFuzzer targets
under exact nightly and cargo-fuzz pins in bounded CI.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88, product toolchain 1.96.0, isolated nightly-2026-08-25 fuzz toolchain

**Primary Dependencies**: Existing product graph; isolated exact `libfuzzer-sys` 0.4.13 and cargo-fuzz 0.13.2 for fuzz execution

**Storage**: Versioned JSON surface registry, binary/text seed corpus, separate fuzz lockfile

**Testing**: Stable Rust seed replay and registry rejection tests, cargo-fuzz/libFuzzer bounded smoke, existing full xtask CI

**Target Platform**: Portable stable replay; x86-64 Linux AddressSanitizer coverage campaigns

**Project Type**: Rust workspace plus an excluded fuzz harness workspace

**Performance Goals**: Each stable seed replay completes in milliseconds; CI runs a fixed small iteration budget per target with a five-second per-input timeout

**Constraints**: Synthetic inputs only, maximum 64 KiB input, bounded parser retention, no network or trust effect, no product lockfile delta, deterministic validation

**Scale/Scope**: Six coverage-guided targets covering twenty owned parser and state-machine surfaces through S125

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1**: PASS. Exercise seams consume synthetic bytes only and cannot access a
  target process, trust store, route, listener, or live network.
- **P-2/P-3**: PASS. Proxy-owned protocol seams remain in `fragcap-proxy`,
  artifact seams remain in the facade, and the external fuzz harness depends in
  the existing direction.
- **P-4/P-9**: PASS. Every input cap, incomplete record, target, corpus, and
  campaign result is explicit. Registry drift and skipped coverage fail closed.
- **P-5**: PASS. Packet acquisition and pcapng compatibility are unchanged.
- **P-6/P-8**: PASS. Surface, seed, campaign, finding, and dependency-boundary
  vocabulary are recorded alongside the normative testing strategy.
- **P-10/P-11**: PASS. Stable replay is offline and deterministic, while the
  spec and changelog describe only S126 and preserve the #334 completion gate.

Post-design check: PASS. The fuzz dependency graph is isolated, production
behavior is unchanged, and all byte entry points enforce fixed bounds before
parsing or state transitions.

## Architecture and Phases

1. Define a closed registry mapping twenty owned surfaces to six fuzz targets,
   their corpora, properties, limits, and dependency boundaries.
2. Add pure bounded exercise seams beside the owning proxy and artifact code,
   reusing actual parsers and observers rather than parallel implementations.
3. Add stable seed replay and a repository validator that cross-checks registry,
   source targets, corpus, CI, tracking, content safety, and exact tool pins.
4. Add the excluded cargo-fuzz workspace, one binary per target, dictionaries,
   minimized synthetic seeds, and its independent lockfile.
5. Add bounded Linux CI smoke and campaign/reproduction/minimization guidance.
6. Update architectural records, run focused campaigns, then run full CI.

## Project Structure

```text
specs/126-native-parser-fuzzing/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
├── checklists/
└── tasks.md
docs/security/
└── deep-capture-fuzzing.md
fuzz/
├── Cargo.toml
├── Cargo.lock
├── fuzz-targets.json
├── dictionaries/
├── corpus/
└── fuzz_targets/
crates/fragcap-proxy/src/fuzz_support.rs
crates/fragcap/src/deep_capture/fuzz_support.rs
crates/fragcap/tests/fuzz_seeds.rs
xtask/src/fuzz.rs
.github/workflows/fuzz.yml
docs/fragcap-specification.md
docs/fragcap-spec-outline.md
docs/plans/README.md
AGENTS.md
changelog.d/
```

**Structure Decision**: Keep exercise seams with their authority, keep stable
validation in the repository task runner and facade tests, and isolate nightly
and libFuzzer dependencies in `fuzz/` so ordinary consumers never resolve or
compile them.

## Complexity Tracking

No constitution violation requires an exception.
