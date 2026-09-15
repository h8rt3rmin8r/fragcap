# Quickstart Validation: S150

## Preconditions

Use the declared Rust toolchain and existing site dependencies. Run local console tools through the verified hidden, redirected, non-interactive launcher on Windows. No game, installed product, live capture, or real trust mutation is needed.

## Controlled checks

```bash
cargo test -p fragcap-cli --test cli_reference --locked
cargo test -p fragcap-cli --test cli_reference --locked --features net
cargo xtask review-handoff
cargo test -p fragcap --lib --features deep-capture --locked published_examples_match_the_versioned_reader
cargo xtask notes 0.10.0
cargo xtask ci
cargo +1.88.0 build --workspace --locked
cargo xtask docs build
```

Run `pnpm test:unit` and `pnpm test:accessibility` inside `site/` after the production export. Expected outcomes are parsed current examples, valid artifact specimens and review readiness, one-screen release notes, clean version-bound repository checks, and accessible searchable navigation. Invalid command, missing evidence, or premature review approval fixtures must fail their negative checks.

## Handoff only

The release guide supplies exact post-merge operator tag and publication steps. The independent reviewer records actual published identities and installed-build results under #333; the committed not-started template is not a completed audit.

## Authorized runtime correction checks

Run the application-writer unit tests with `cargo test -p fragcap --lib --features deep-capture --locked deep_capture::application::tests`. Run the existing release short harness and canonical report check documented in `docs/security/deep-capture-performance.md`; use two fresh Windows campaigns against fixed code and inspect all case outcomes plus comparability. Do not trigger the separate two-hour soak or execute installed sensitive software. Preserve original failed evidence rather than overwriting it with passing measurements.
