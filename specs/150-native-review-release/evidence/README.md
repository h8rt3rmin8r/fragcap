# S150 hosted Windows performance evidence

These are exact observed reports, not illustrative fixtures, recreated measurements, or independent audit approval. Both came from [run 34965333411](https://github.com/h8rt3rmin8r/fragcap/actions/runs/34965333411), PR source 4eb147161966cac45e07b6ed59984755b6c8f5ea and synthetic hosted merge revision 52ff39e07aeaebbf80b349051c09bfe615f9d7ed. Both used v0.10.0, Windows x86_64/release, four logical CPUs, and registry digest `fnv1a64:559c42bc9ce8f33e`.

| Report | Artifact ID | SHA-256 | Actual outcome |
| --- | --- | --- | --- |
| [Attempt 1](windows-performance-attempt1.jsonl) | 10395675933 | `17be4093086396283709e11106f81a99562dec441e03ad94828d6570d9ad47a4` | Complete fourteen-case campaign failed: one hard queue-loss window in `quic-off`. |
| [Attempt 2](windows-performance-attempt2.jsonl) | 10396111150 | `516afd746ca406aa4a5fa412b2596df8b15a26626b8784755fe0a18193a901ac` | Unchanged diagnostic rerun completed and passed all fourteen cases. |

The checked-in bytes match the downloaded report digests exactly. Attempt 1 is still failed. Attempt 2 is a separate observed campaign, not a valid timing retry or acceptance of the preceding hard breach. The official report validator rejects attempt 1 and accepts attempt 2 individually. Their hard-outcome disagreement opened [#413](https://github.com/h8rt3rmin8r/fragcap/issues/413); the authorized correction and separate fixed-code verification below establish its bounded engineering disposition, not the precise historical platform stall. Do not discard a report, pool windows, rewrite budgets, or declare a finding cleared merely because current CI became green.

## Authorized remediation measurements (2026-09-15)

The following records extend the chronological investigation and do not supersede either original report. The operator explicitly authorized the runtime correction. Local reports ran controlled synthetic loopback harnesses, not installed-product or real-host trust sessions; their `source_dirty` fields truthfully record in-progress artifact/code changes. Baseline and local fixed measurements both passed and do not by themselves satisfy two fresh hosted fixed-source verification.

| Report | Identity | SHA-256 | Actual outcome |
| --- | --- | --- | --- |
| [Repeated hosted failure](windows-performance-repeat-failed.jsonl) | Run 34967816667, source synthetic merge `6148765542737eed2f99cfb8fd88c65b5bdb9cd8` | `9c861b124091228b8359e941bbd2c61f720b4d3df128ce6fbe787e1fbce2649b` | Complete campaign failed with two `quic-off` hard-loss windows. |
| [Local baseline](windows-performance-local-baseline.jsonl) | Original runtime, in-progress S150 scope records | `56dc782ae47c177c6c323111f20f91f6c5a620208457ab0938240b5bbf9a33c7` | Complete fourteen-case campaign passed, no hard failures. |
| [Local correction](windows-performance-local-fixed.jsonl) | Finite 64 KiB byte buffer and moved JSON maps | `6bb1a6ae02a5a70eb06916b772c5eb160b6a431998dec2f5d60132b5d04005eb` | Complete fourteen-case campaign passed, no hard failures. |

Deterministic regression separately demonstrated 373 physical writes before correction versus 64 afterward for 4,096 exact metadata records, with unchanged serialization and pending limit. That proves reduced write amplification, not the precise historical host stall.

## Fresh fixed-code hosted verification

Both campaigns ran normally on separate PR pushes, without a job rerun, workload change, budget change, or pooled result. The second push changed only `release-notes/v0.10.0.md`. Both actual hosted source trees contain application writer blob `6bf04919ac5ce39099d54989f5cf52c0607ed4d3`, workload blob `f2621c02e68a8281ab2ecb90e1fb3c1ef75ac080`, and registry blob `ec384c38838a3a0486889fe5703a497b945115e8`. Headers record clean v0.10.0 Windows x86_64/release, four logical CPUs, short profile, and unchanged comparability class `windows:x86_64:release:fnv1a64:559c42bc9ce8f33e`.

| Exact report | Run / artifact | Hosted synthetic merge revision | SHA-256 |
| --- | --- | --- | --- |
| [Fixed campaign 1](windows-performance-fixed-34974704086.jsonl) | [34974704086](https://github.com/h8rt3rmin8r/fragcap/actions/runs/34974704086) / 10398918693 | `df8d07f8f4c4b904eaca0a0dd74b8bc26f32e514` | `a1ad920ee535941b6bca8b3497fb2a980a998e773ee885cfbbe7431f4d599837` |
| [Fixed campaign 2](windows-performance-fixed-34974881912.jsonl) | [34974881912](https://github.com/h8rt3rmin8r/fragcap/actions/runs/34974881912) / 10399283106 | `6f8a0a53cd3eacf208c374e243290404564b0732` | `eba179f818c2094d2775724ecdd9113a974f5aec97092b6abb3b1d8f8150f2f8` |

Both copied digests match the downloaded bytes. Each canonical `cargo xtask performance --report <report>` passed independently: fourteen complete passing case terminals, seven windows per case, one attempt per case, zero hard failures and zero timing-breaching windows. Every one of the combined 196 samples reports successful exchange, clean shutdown, and zero application-event loss, queue-byte loss, storage-byte loss, and failure-detail loss. Maximum observed queue peaks were 215 and 348, below the unchanged 4,096-event limit. Raw sample resource, forwarding, and conservation facts passed the canonical validator.

Pairwise comparison uses the existing S128 diagnostic formula `100 * abs(a - b) / max(a, b)` for each case's median throughput and added p95. All fourteen outcomes agree. Maximum throughput difference was 42.068% (`grpc-off`); maximum latency difference was 28.395% (`http2-off`). Both are below the unchanged 75% diagnostic tolerance. These finite measurements satisfy the scoped fixed-code engineering gate; they are not universal losslessness, independent #333 installed-build retest, or operator publication approval. All earlier failed reports remain failed and retained.
