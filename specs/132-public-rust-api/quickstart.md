# Quickstart: Stable Public Rust API

## Compile the external consumer contract

```text
cargo test -p fragcap --features deep-capture --test public_api
```

Expected result: the version 1 stable inventory, builders, non-exhaustive policy, concurrency guarantees, cancellation behavior, and CLI coverage contract pass.

## Run the production native controlled example

```text
cargo run -p fragcap --example native-deep-capture --features deep-capture
```

Expected result: the library starts the production native backend against bounded local controlled origins, completes HTTP and TLS observations without the CLI or host trust changes, and exits only after exact cleanup reports no owned residue.

## Check documentation examples

```text
cargo test -p fragcap --doc --features deep-capture
```

Expected result: all stable-surface documentation examples compile without importing implementation modules.

## Full repository gate

```text
cargo xtask ci
```

Expected result: formatting, all-target all-feature lint, workspace tests, repository conventions, dependency direction, licensing, API coverage, and the controlled example gate pass.

## Required mutation checks

- Remove or rename one stable export and require the external-consumer contract to fail.
- Route one CLI policy import around `deep_capture::api` and require the coverage contract to fail.
- Add one implementation-only backend type to the stable inventory and require the reviewed inventory assertion to fail.
- Remove one builder construction path and require the external consumer to fail.
- Make one evolvable stable enum exhaustive and require the policy assertion to fail.
- Remove a cancellation checkpoint and require the staged cancellation scenario to show a later effect, which must fail.
- Make cancellation skip cleanup or erase an observation and require terminal reconciliation to fail.
- Leave the native example listener or controlled origin alive and require residue reconciliation to fail.

## Text and repository hygiene

```text
git diff --check
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

Confirm UTF-8 without BOM, LF endings, no mojibake, no em-dashes or en-dashes, no Markdown hard wrapping, no dependency lock change, no staged `.specify/feature.json`, and S131 status corrected only from Draft to Complete.
