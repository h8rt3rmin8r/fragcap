# Validation guide: S154

Run controlled tests and static/build checks only. Do not install or run the sensitive product, launch a game or mutate host trust. On Windows use the verified hidden redirected launcher.

## Focused checks

```text
cargo test -p fragcap-sink --test streaming_backpressure --test streaming_tcp --locked
cargo test -p xtask docs_coverage --locked
cargo test -p fragcap-cli --test cli_reference --locked
cargo test -p fragcap-cli --test cli_reference --features net --locked
cargo xtask docs check
cargo xtask ci
```

Expect twenty fresh controlled isolation scenarios, existing timeout/idle and real TCP coverage, closed inventory rejection tests, command parsing without dispatch and the ordinary gate.

## Production site

Use existing pinned site tooling for unit tests, static export and production accessibility tests. Hosted Linux/Windows checks must be green on the final PR head. Expected manual-soak and PR Pages skips remain distinct from executed checks.

## Handoff

Push and open the official PR automatically under owner authorization. Address every received review, request no more than one second round and hand off for human merge only after final-head green checks. Do not claim parent documentation or independent security completion.
