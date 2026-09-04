# Deep Capture performance envelope

S128 establishes the version 1 performance authority for the native proxy. The reviewed registry is `performance/native-proxy-budgets-v1.json`. A measurement process can read it but cannot rewrite it.

The required matrix contains HTTP/1.1, HTTP/2, WebSocket, gRPC, generic TCP, generic UDP, and scoped QUIC/HTTP/3, each with payload retention enabled and disabled. Every row runs through `NativeProxyBackend`. A successful campaign contains all fourteen terminal row identities. Missing, duplicated, skipped, or malformed rows make the campaign incomplete.

## Limits and degradation

The runtime publishes bounded failure-detail retention, accepted-connection task ownership, leaf-certificate cache occupancy, and application writer queue occupancy. Nested protocol work remains finite and joined beneath each accepted connection through the existing stream and attempt limits. Failure details and application observations discard the oldest detail only after their declared capacity is reached, and each such discard advances an exact counter. Forwarding remains independent from evidence retention.

The registry freezes timing and resource ceilings before measurements are collected. Timing uses seven windows and permits one retry only inside the five-percent guard band. Loss, capacity, memory, disk, and cleanup rules are hard invariants and have no timing retry.

| Protocol family | Minimum proxy/direct ratio | Maximum added p95 |
| --- | ---: | ---: |
| HTTP/1.1 | 2% | 200 ms |
| HTTP/2 | 2% | 200 ms |
| WebSocket | 2% | 400 ms |
| gRPC | 2% | 200 ms |
| Generic TCP | 2% | 400 ms |
| Generic UDP | 5% | 25 ms |
| QUIC and HTTP/3 | 1% | 750 ms |

Every row also has a 1 MiB/s useful-payload floor. The hard worker limits are 256 MiB peak memory, 32 MiB maximum same-case fresh-worker private-memory span, 1 second of CPU per useful MiB, 32 MiB per application artifact, eight accepted-connection tasks, 4,096 queued application events, 256 cached leaf certificates, 8 MiB of cached certificate material, and 5 seconds for shutdown. Current task and queue ownership must be zero at terminal, and spawned connection tasks must equal completed plus aborted tasks.

Payload-disabled rows must retain zero protocol payload bytes. Every sample reconciles observed payload bytes as retained, intentionally omitted, queue-dropped, or storage-dropped. Every QUIC worker also routes five admitted certificate identities through one four-entry production leaf cache, and the row fails unless its reported peak proves that churn reached the cache bound. Overload behavior is exercised deterministically in the product tests referenced by the registry: forwarding remains independent, finite detail stores retain a bounded suffix, and each discarded unit advances its named counter.

## Reproduction

Run the static authority check first:

```text
cargo xtask performance
```

Then run a release campaign:

```text
cargo run --release --locked --manifest-path performance/native-proxy/Cargo.toml -- --profile short --output target/performance/short.jsonl
cargo xtask performance --report target/performance/short.jsonl
```

The short profile is a pull-request regression signal. The soak profile is a genuine 7,200-second run and cannot be shortened while retaining a passing soak identity. Windows and Ubuntu run the short profile. Windows owns the scheduled and manually dispatched soak because the supported product is Windows.

Reports are JSON Lines with monotonic sequence values, a registry digest, exact row identities, and a terminal completeness decision. A retry is valid only when all seven samples of the preceding attempt carry the exact attempt and window identities, contain no hard failure, and produce the canonical timing guard band. They contain no credentials, capability material, private keys, payload contents, usernames, home paths, or machine identifiers. Raw timing values are comparable only within the recorded operating-system family, architecture, build profile, campaign profile, and registry digest. Two fresh Windows campaigns must agree on every outcome; raw medians use a 75-percent diagnostic tolerance because subsecond cross-process scheduling is noisy. Resource and conservation failures remain comparable across hosts.
