# Performance Gate Contract

## Commands

```text
cargo xtask performance
cargo run --release --locked --manifest-path performance/native-proxy/Cargo.toml -- --profile short --output <path>
cargo run --release --locked --manifest-path performance/native-proxy/Cargo.toml -- --profile soak --output <path>
```

`cargo xtask performance` validates the canonical registry, isolated manifest and lock, runtime accounting inventory, executable evidence references, workflow triggers, report fixtures, and the absence of an automatic budget rewrite path.

The harness command returns:

- `0` only for a complete passing campaign;
- `1` for a complete campaign with one or more failed or twice-inconclusive cases;
- `2` for invalid input, unavailable required measurement authority, malformed registry, non-comparable input, incomplete execution, or report-write failure.

## Registry Authority

`performance/native-proxy-budgets-v1.json` is reviewed input. A measurement run never edits it. Every report carries a deterministic digest over the registry bytes and fails validation when that digest differs.

The required matrix is the Cartesian product of:

```text
protocol = http1 | http2 | websocket | grpc | tcp | udp | quic
retention = off | on
```

No skipped, ignored, absent, duplicated, or synthetic-success row satisfies the matrix.

## Timing Evaluation

Each row performs warmup followed by seven paired direct and proxied windows. It records all observations, median throughput ratio, and added p95 latency.

- An absolute floor, ratio, or latency budget fails when its aggregate threshold breaches and at least five windows breach.
- A result inside the configured five-percent guard band reruns the row exactly once.
- A second guard-band result or any retry failure is terminal failure.
- Results are never pooled across row identities.

## Resource and Conservation Evaluation

Memory, disk, queue, task, cache, retention, loss, and shutdown rules are hard invariants evaluated on every applicable sample. There is no guard band or retry that converts their breach into success.

The report names the exact conservation equation used by each protocol row. Every observed unit has one terminal disposition. Forwarded units use complete payloads independently from evidence retention.

## Report Integrity

Every JSON Lines record includes `schema_version`, `kind`, and monotonic `sequence`. The terminal record includes the expected and observed row identity sets and the registry digest. Duplicate sequence values, malformed lines, unknown versions, missing terminals, or mismatched sets make the report incomplete.

Reports exclude credentials, capability material, private keys, ephemeral payload contents, usernames, absolute home paths, and machine-unique identifiers.

## Automation

Pull requests and pushes to `main` run the short profile in release mode on Windows and Ubuntu. Reports upload even on failure.

The soak job runs on Windows only through explicit `workflow_dispatch`. Its default is 7,200 seconds, samples at least every 60 seconds, and has a bounded workflow timeout above its declared shutdown allowance. An explicit shorter run remains an incomplete raw campaign. A sanitized summary may separately record project-owner acceptance after at least 1,875 zero-failure case terminals and one continuous hour; it never fabricates a raw terminal.
