# Native Parser Fuzz Gate Contract

## Stable Invocation

```text
cargo xtask fuzz
cargo test -p fragcap --test fuzz_seeds --features deep-capture
```

## Coverage-Guided Invocation

```text
cargo +nightly-2026-08-25 fuzz run TARGET -- -runs=256 -timeout=5 -max_len=65536
```

## Success

- Exit `0`.
- Print registry schema, target, surface, and seed counts.
- Every registered target and seed was validated or executed.

## Validation Failure

- Exit `1` with deterministic diagnostics for registry, target, corpus,
  tracking, content, version, bounds, stable dispatch, or CI matrix drift.
- A panic, sanitizer error, timeout, excessive allocation, or silent truncation
  is a failing finding, never a skipped or partial success.

## Unable to Run

- Exit `2` when required files or tools cannot be read or parsed enough to
  perform validation.

## Finding Promotion

1. Reproduce the exact artifact.
2. Minimize it with the pinned engine.
3. Remove any sensitive or real-world material.
4. Add a focused named regression test that fails before the fix.
5. Add the minimized synthetic input to the owning corpus.
6. Apply the fix and rerun stable replay plus the bounded campaign.
