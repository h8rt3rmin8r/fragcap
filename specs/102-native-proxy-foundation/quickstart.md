# S102 Quickstart

```powershell
cargo build --package fragcap-proxy --locked
cargo test --package fragcap-proxy --locked
cargo test --package fragcap --features deep-capture --test native_proxy --locked
rustup run 1.88.0 cargo build --workspace --locked
cargo build --package fragcap-cli --all-features --release --locked
cargo xtask ci
```

The tests bind ephemeral loopback ports only. They do not launch a target, issue certificates, modify trust, configure a system proxy, or contact the Internet.

The native foundation is library-accessible and lifecycle-safe. The v0.8 CLI still selects external mitmdump for functional Deep Capture until #290 lands. S102 is not a native-proxy cutover and is not Deep Capture completion.
