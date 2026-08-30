# Native Proxy Research Harness

This nested Cargo workspace is the non-shipping S099 research artifact. It pins `hudsucker` 0.23.0, drives only synthetic loopback traffic, and compares normalized observations with an installed `mitmdump` baseline. It is not a fragcap workspace member, product dependency, default backend, installer input, or release artifact.

The harness does not modify system proxy settings or certificate stores. It generates private test certificate material in an operating-system temporary directory, configures only its owned clients, and removes the directory when the run ends. Committed output contains lengths, synthetic payload digests, versions, and result classifications, never private keys or raw traffic.

```powershell
cargo test --manifest-path spikes/native-proxy/Cargo.toml --locked
cargo run --manifest-path spikes/native-proxy/Cargo.toml --locked -- candidate
cargo run --manifest-path spikes/native-proxy/Cargo.toml --locked -- baseline
cargo run --manifest-path spikes/native-proxy/Cargo.toml --locked -- compare
```

`candidate` requires no external proxy. `baseline` and `compare` require `mitmdump` on `PATH`. Capability failures are recorded as evidence. A nonzero exit is reserved for failure to produce an authoritative evidence document.
