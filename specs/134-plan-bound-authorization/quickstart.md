# Quickstart: Validate Plan-Bound Deep Capture Authorization

## Prerequisites

- Rust 1.88 toolchain from `rust-toolchain.toml`.
- Repository dependencies available from the locked graph.
- No Npcap installation, elevation, game account, real target, external service, or trust-store mutation is required for controlled validation.

## 1. Inspect the command contract

```sh
cargo run -p fragcap-cli -- deep-capture --help
```

Expected: `--authorize-stdin` is present. Deep Capture `--trust-ca` and `--yes` are absent. Other commands retain their separately scoped confirmation flags.

## 2. Run focused authorization tests

```sh
cargo test -p fragcap-cli --test cli_deep_capture authorization
cargo test -p fragcap --features deep-capture native_proxy
```

Expected: exact interactive and structured approvals proceed through the controlled adapter; decline, EOF, mismatch, replay, and drift record zero effects; the runtime certificate thumbprint matches the plan.

## 3. Prove reachability has no trust action

```sh
cargo test -p fragcap-cli --test cli_deep_capture reachability
```

Expected: the controlled reachability plan says trust, HAR, and key log are absent, while its authorized launch and routing path remains testable.

## 4. Prove legacy flags cannot authorize

```sh
cargo test -p fragcap-cli --test cli_deep_capture legacy_authorization
```

Expected: each hidden legacy flag exits 2 with exact migration guidance and no adapter effect. Doctor, bundle cleanup, and target reconciliation confirmation tests remain unchanged.

## 5. Run repository gates

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
cargo xtask ci
```

Expected: every command succeeds. The dependency inventory adds direct edges only for packages already present in `Cargo.lock`, and all security, docs, shell, privacy, encoding, and specification checks pass.

## 6. Manual structured handshake

Start a controlled JSON command with `--authorize-stdin` under a small harness that keeps both pipes open. Read the `deep_capture.authorization_plan` event, write its exact `plan_id` plus one newline to standard input, and continue consuming events.

Expected: there is no prompt; the matching plan produces `deep_capture.authorization` with `authorized`; altering any identifier character produces a usage refusal before the listener or bundle exists.
