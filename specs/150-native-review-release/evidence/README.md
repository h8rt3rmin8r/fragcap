# S150 hosted Windows performance evidence

These are exact observed reports, not illustrative fixtures, recreated measurements, or independent audit approval. Both came from [run 34965333411](https://github.com/h8rt3rmin8r/fragcap/actions/runs/34965333411), PR source 4eb147161966cac45e07b6ed59984755b6c8f5ea and synthetic hosted merge revision 52ff39e07aeaebbf80b349051c09bfe615f9d7ed. Both used v0.10.0, Windows x86_64/release, four logical CPUs, and registry digest `fnv1a64:559c42bc9ce8f33e`.

| Report | Artifact ID | SHA-256 | Actual outcome |
| --- | --- | --- | --- |
| [Attempt 1](windows-performance-attempt1.jsonl) | 10395675933 | `17be4093086396283709e11106f81a99562dec441e03ad94828d6570d9ad47a4` | Complete fourteen-case campaign failed: one hard queue-loss window in `quic-off`. |
| [Attempt 2](windows-performance-attempt2.jsonl) | 10396111150 | `516afd746ca406aa4a5fa412b2596df8b15a26626b8784755fe0a18193a901ac` | Unchanged diagnostic rerun completed and passed all fourteen cases. |

The checked-in bytes match the downloaded report digests exactly. Attempt 1 is still failed. Attempt 2 is a separate observed campaign, not a valid timing retry or acceptance of the preceding hard breach. The official report validator rejects attempt 1 and accepts attempt 2 individually. Their hard-outcome disagreement remains [#413](https://github.com/h8rt3rmin8r/fragcap/issues/413); root cause, bounded remediation, and final release-verification disposition are not established. Do not discard a report, pool windows, rewrite budgets, or declare the finding cleared because current CI became green.
