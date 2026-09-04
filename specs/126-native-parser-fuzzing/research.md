# Research: Native Deep Capture Parser Fuzzing

## R-1: Coverage engine and tool isolation

**Decision**: Use cargo-fuzz 0.13.2 with libFuzzer through exact
`libfuzzer-sys` 0.4.13 in an excluded `fuzz/` workspace and independent lockfile.

**Rationale**: The Rust Fuzz Book recommends cargo-fuzz and documents bounded CI
smoke runs. The isolated workspace prevents sanitizer-only dependencies and
nightly requirements from changing the product graph or MSRV.

**Alternatives rejected**: Property tests alone are deterministic but not
coverage guided. Adding the fuzz crate to the product workspace would make
`--all-targets --all-features` build sanitizer harnesses and expand the shipped
dependency review boundary.

## R-2: Reproducible toolchain

**Decision**: Pin `nightly-2026-08-25`, cargo-fuzz 0.13.2, and
libfuzzer-sys 0.4.13, and make the validator compare documentation, manifest,
and workflow values.

**Rationale**: An unqualified nightly changes daily. Exact pins make a local
reproduction match CI and make tool upgrades explicit review events.

The isolated lockfile retains `tinyvec` 1.12.0 because 1.13.0 fails to compile
with that exact nightly. The CI lock check makes this compatibility choice
reproducible and prevents a silent resolver upgrade.

## R-3: Surface identity

**Decision**: Maintain one versioned JSON registry with stable surface ids,
owners, target ids, corpus paths, properties, maximum input sizes, and named
third-party dependency exclusions.

**Rationale**: Target filenames alone cannot prove every owned boundary is
covered or prevent two targets from accidentally claiming the same surface.
The registry is both review authority and executable validator input.

## R-4: Exercise seams

**Decision**: Add `#[doc(hidden)]` bounded exercise functions beside the actual
proxy and facade parsers. Stable tests and libFuzzer call the same functions.

**Rationale**: Reimplementing parsers in a fuzz crate can produce excellent
coverage of code the product never runs. Owner-local seams can access private
parsers and state machines without making their internal types public.

## R-5: Fragmentation and stateful input

**Decision**: Reserve leading control bytes for deterministic split schedules,
limit selection, direction, cancellation, and terminal behavior, while the
remaining bytes remain attacker-controlled payload.

**Rationale**: Raw single-buffer fuzzing misses incremental transitions. A
small control grammar reaches boundary states while mutations still explore
arbitrary data.

## R-6: Stable replay

**Decision**: Replay every seed through ordinary Rust tests and validate the
entire inventory with `cargo xtask fuzz` before nightly CI.

**Rationale**: Contributors should not need nightly, a C++ compiler, or
cargo-fuzz to reproduce committed cases. Fast stable replay also prevents a
broken harness from being hidden by a skipped fuzz job.

## R-7: Corpus safety

**Decision**: Use hand-constructed minimal protocol fragments and invented
JSON identities only. Validate size, tracking, non-emptiness, duplicate hashes,
private-key markers, authorization headers, token-like fields, and non-loopback
literal endpoints.

**Rationale**: Captured traffic and real artifacts can contain credentials,
addresses, and personal data. The public corpus does not need them to exercise
the parsers.

## R-8: Scope boundary

**Decision**: Exercise fragcap-owned HTTP/1 framing and metadata, proxy auth,
SOCKS, streaming protocols, identity parsing, QUIC classification/evidence, and
artifact semantics. Record rustls, h2, h3, Quinn, httparse, and serde_json wire
or syntax decoding as dependency-owned.

**Rationale**: Claiming direct coverage of third-party internals would make the
instrument lie. Their inputs still pass through the product during integration
and conformance tests, but their own parser campaigns belong upstream.
