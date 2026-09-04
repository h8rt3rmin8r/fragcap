# Data Model: Native Deep Capture Performance Envelope

## Performance Registry

The immutable pre-measurement authority.

- `schema_version`: exactly `1`.
- `product_scope`: S128 native Deep Capture boundary.
- `seed`: fixed non-secret workload seed.
- `profiles`: short and soak definitions.
- `cases`: exactly fourteen protocol/retention combinations.
- `hard_limits`: campaign-wide memory, disk, task, cache, and shutdown ceilings.
- `comparison`: window count, boundary band, allowed retry count, and breach quorum.
- `evidence`: attributed executable and workflow references.

Validation rejects missing or unknown protocols, duplicate identities, absent retention pairs, zero workloads, unsafe ceilings, unbounded durations, stale sources, or an automatic budget-update path.

## Performance Case

Identity is `(protocol, retention)` and is rendered as `<protocol>-<on|off>`.

- `protocol`: `http1`, `http2`, `websocket`, `grpc`, `tcp`, `udp`, or `quic`.
- `retention`: Boolean payload-retention selection.
- `driver`: One of four harness driver families.
- `payload_bytes`: Useful bytes per operation.
- `operations_per_window`: Fixed work count for short runs.
- `concurrency`: Concurrent connection, stream, or datagram ownership.
- `throughput_ratio_min`: Minimum proxy/direct median ratio.
- `throughput_bytes_per_second_min`: Absolute safety floor.
- `added_latency_p95_ms_max`: Same-run proxy p95 minus direct p95 ceiling.
- `cpu_ms_per_mib_max`: Whole-worker normalized CPU ceiling.
- `artifact_bytes_per_record_max`: Structural output allowance.
- `test_reference`: Exact executable harness row.

## Campaign Profile

- `id`: `short` or `soak`.
- `minimum_duration_seconds`: Zero for fixed-work short rows; 7,200 for soak.
- `sample_interval_seconds`: Bounded sampling interval.
- `warmup_windows`: Unreported stabilization windows.
- `measured_windows`: Seven for short; duration-derived for soak.
- `cycles`: Repeated complete protocol, retention, connection, stream, datagram, certificate, artifact, and restart work for soak.
- `shutdown_deadline_ms`: Profile-specific terminal ceiling.

## Runtime Resource Accounting

- `failure_details`: Current bounded retained detail count.
- `failure_details_dropped_oldest`: Exact overflow total.
- `connection_tasks_current`, `connection_tasks_peak`, `connection_tasks_spawned`, `connection_tasks_completed`, `connection_tasks_aborted`: Accepted-connection owner task gauges. Nested work remains bounded and joined beneath this owner by protocol limits.
- `leaf_cache_entries`, `leaf_cache_peak_entries`: Cache-owned entry counts.
- `leaf_cache_bytes`, `leaf_cache_peak_bytes`, `leaf_cache_evictions`: Cache-owned byte and churn counts.
- `application_queue_capacity`, `application_queue_current`, `application_queue_peak`: Artifact writer pressure gauges.

All current counts are bounded by configuration or zero at terminal ownership. Totals saturate rather than wrap.

## Performance Sample

- Sequence and monotonic elapsed time.
- Useful operations and bytes.
- Direct and proxied latency observations.
- Worker CPU delta and logical CPU count.
- Resident, peak resident, and platform-private memory where available.
- Exact artifact logical bytes.
- Runtime resource accounting snapshot.
- Protocol loss and retention counters.
- Sample status and explicit reasons.

Samples are bounded by the profile. A sample does not authorize campaign success.

## Campaign Report

Newline-framed records:

1. `campaign.header`: schema, budget digest, source revision, product version, platform, architecture, logical CPUs, build profile, timer authority, profile, and start time.
2. `case.sample`: bounded window evidence.
3. `case.terminal`: one per required case in a short campaign and one per required case per completed soak cycle, with every budget result.
4. `campaign.sample`: periodic soak progress.
5. `campaign.terminal`: exact case identities, duration, memory span, report completeness, and overall disposition.

Only one valid `campaign-terminal` after every required `case-terminal` makes a report complete. Prefixes remain readable and explicitly incomplete.

## State Transitions

```text
declared -> warming -> measuring -> evaluating -> passed
                                            -> retrying -> passed
                                                        -> failed
                                            -> failed
any nonterminal state -> interrupted
```

An inconclusive boundary result may enter `retrying` once. A second inconclusive result is `failed`, never `passed`.

## Comparability Class

- Operating-system family.
- Architecture.
- Build profile.
- Registry and budget digest.
- Protocol case identity and workload settings.

Source revision, exact CPU model, runner name, and timestamps remain provenance but do not widen comparability. Mismatched comparability fields prohibit a regression conclusion.
