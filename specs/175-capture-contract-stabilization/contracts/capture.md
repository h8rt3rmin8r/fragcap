# Contract: S175 Capture CLI and Artifacts

## Option semantics

| Input | Accepted result |
| --- | --- |
| `--duration 45s` | Stop no earlier than 45 seconds of arm-relative elapsed time unless another stop condition wins. Packet timestamps stay absolute. |
| `--roles target,launcher` | Exact names must be present in the resolved profile. Empty members and unknown case exit 2 before output. |
| `--direction both` | Existing bidirectional behavior. |
| `--direction in` or `out` | Admit only the selected known direction; count known opposite and unknown direction omissions separately. |
| `--no-payload` | Apply metadata-only retention to each sink unless an explicit sink payload modifier overrides it. |
| `--sink jsonl:Capture,, QA.jsonl,format=jsonl` | Write to `Capture, QA.jsonl` and apply the format modifier. |

## Packet outputs

In metadata-only pcapng, EPB `captured_len` equals the exact retained header prefix length and `original_len` equals the source-reported original length. If the header boundary is uncertain, `captured_len` is zero; no uncertain bytes are emitted. The packet annotation and interface identity stay present. Full-payload pcapng remains byte-identical for the same input.

JSON Lines metadata-only omits `data` while retaining packet metadata and existing trailer statistics. Ring dumps apply their encoder's payload mode after the ring window; pre-ring retained count and ring evictions remain separately reported. A stream consumer and each rotation segment use the same mode as the original sink specification.

## Human and machine reports

The scope line reports enforced role and direction settings. Completion JSON and human summary expose direction omissions as separate known-opposite and unknown counts, without changing the meaning of source `packets captured`, pre-ring `retained`, or ring `evicted`. Silent suppresses optional progress but never permits output that contradicts the requested filter.

## Compatibility

Existing sink specifications without doubled commas retain their meaning. `PcapngWriter::new` and payload-enabled output preserve existing bytes. No schema version changes are needed for unchanged packet annotation keys; new optional report counters require an additive event-contract update and regression snapshots.
