# Native Deep Capture Fuzzing

S126 maintains coverage-guided fuzzing for every fragcap-owned native protocol
parser, state machine, and Deep Capture artifact reader shipped through S125.
The canonical map is `fuzz/fuzz-targets.json`; `cargo xtask fuzz` rejects drift
between that registry, target sources, stable replay, corpus, and CI.

## Ownership boundary

fragcap directly exercises its HTTP/1 destination and framing decisions, proxy
authentication, SOCKS5 negotiation and datagrams, WebSocket, SSE, gRPC,
destination and certificate identities, QUIC classification and HTTP/3 evidence,
plus application, lifecycle, resource-journal, process-trace, and manifest
semantics. Rustls, h2, h3, Quinn, httparse, and serde_json own their internal
wire or syntax decoders. Their code is not counted as a fragcap-owned fuzz
surface.

Every entry point rejects input above 65,536 bytes before parser work. Stateful
targets derive chunk widths, direction, limits, and terminal behavior from the
input while retaining no more than their registry cap. They use no listener,
network, trust store, real capability, target process, or external session.

## Stable replay

Run the complete committed corpus without nightly tooling:

```text
cargo xtask fuzz
cargo test -p fragcap --test fuzz_seeds --features deep-capture
```

The replay sorts target and seed paths and runs every seed twice. This is the
fast permanent regression gate and must remain green even when coverage-guided
CI is unavailable.

## Exact coverage-guided setup

```text
rustup toolchain install nightly-2026-08-25
cargo +1.96.0 install cargo-fuzz --version 0.13.2 --locked
cargo +nightly-2026-08-25 fuzz build
```

The fuzz harness is a separate workspace with an independent lockfile. Do not
add it to the product workspace or move `libfuzzer-sys` into the product lock.

Run the same bounded smoke profile as CI:

```text
cargo +nightly-2026-08-25 fuzz run TARGET -- -runs=256 -timeout=5 -max_len=65536
```

Use a longer local campaign by replacing `-runs=256` with a finite time budget:

```text
cargo +nightly-2026-08-25 fuzz run TARGET -- -max_total_time=3600 -timeout=5 -max_len=65536
```

## Findings

Treat every file under `fuzz/artifacts/` as untrusted and potentially sensitive.
Reproduce and minimize it with the exact pinned target:

```text
cargo +nightly-2026-08-25 fuzz run TARGET fuzz/artifacts/TARGET/ARTIFACT
cargo +nightly-2026-08-25 fuzz tmin TARGET fuzz/artifacts/TARGET/ARTIFACT
```

Before committing anything, inspect the bytes and replace any real address,
identifier, credential, token, key, or traffic with a synthetic equivalent.
Add a focused named Rust regression test that fails before the fix, then place
the minimized synthetic input under `fuzz/corpus/TARGET/`. Rerun stable replay,
the bounded target, and the full repository gate.

Coverage inspection uses the same corpus:

```text
rustup component add --toolchain nightly-2026-08-25 llvm-tools-preview
cargo +nightly-2026-08-25 fuzz coverage TARGET
```

Generated `fuzz/artifacts/` and `fuzz/coverage/` directories are ignored. Delete
them after promotion when they are no longer needed. The committed corpus is
the durable evidence, not a local engine work directory.
