# Quickstart / Validation Guide: Non-Profile Capture Path

Validates the slice offline, with no game, capture driver, or Steam install.
Implementation lives in `tasks.md`.

## Prerequisites

- The workspace builds (`cargo build --workspace`).

## 1. The input group is mutually exclusive and one is required

```sh
cargo test -p fragcap-cli run_args
```

Expected: supplying none of `--profile`/`--install-dir`/`--steam`, or more than
one, is a clap usage error (exit 2); exactly one parses.

## 2. `Target::identity` returns the resolved non-profile identity

```sh
cargo test -p fragcap-profile identity
```

Expected: a target with an engine-rule/walker/observed origin returns
`Some(identity)`; a profile-origin target returns `None`.

## 3. Non-profile capture from an install directory (the core)

The offline test builds a fixture install directory with a recognized engine
layout (an Unreal twin-exe tree) and a process script that starts the resolved
client, then runs `run --install-dir <fixture>` through the offline harness.

```sh
cargo test -p fragcap-cli nonprofile
```

Expected: the resolved shipping client is captured through a synthesized
`heuristic-unverified` one-stage identity, reproducing the attribution an
equivalent authored identity produces.

## 4. Honest fidelity and honest decline

Expected (tests):
- the synthesized profile carries `heuristic-unverified`, never `authored`;
- `run --install-dir <dir>` over an unrecognized, an ambiguous, and an unreadable
  directory each exits 1 with a message naming the reason, capturing nothing.

## 5. `--steam` delegates to the install-directory path

```sh
cargo test -p fragcap-cli steam
```

Expected: a fake Steam library fixture whose app id maps to a recognized-engine
install directory drives the same non-profile capture path; a not-installed app
id exits 1 with a surfaced message.

## 6. The profile path is byte-identical

```sh
cargo test -p fragcap-cli --test corpus_pipeline
cargo test --workspace --locked
```

Expected: the `run --profile` capture output matches the existing goldens (the
profile path is untouched).

## 7. Full gate

```sh
cargo xtask ci
cargo xtask msrv
```

Expected: fmt, clippy, tests, lint, deps, license, docs all pass; MSRV 1.82
green; no new dependency.
