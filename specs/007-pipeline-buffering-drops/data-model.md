# Phase 1 Data Model: Pipeline, Buffering, and Drop Accounting

**Slice**: S08 | **Date**: 2026-08-08 | **Plan**: [plan.md](plan.md)

Everything below lives in `fragcap_core::pipeline`. Nothing in `stats.rs`,
`traits.rs`, `packet.rs`, or `error.rs` changes; the whole slice is additive,
which is the check that the S02 vocabulary was shaped correctly.

## Module layout

```text
fragcap_core::pipeline
├── mod.rs       Pipeline, PipelineConfig, ConfigError, StopHandle,
│                PipelineReport, EndReason, SinkFailure, PipelineError
└── buffer.rs    Item, Buffer, Producer, Consumer  (crate-private)
```

`buffer.rs` is private to the crate. Its contract is exercised by unit tests in
the same file rather than exposed, because a bounded queue with eviction is not
a seam the project promises to anyone.

## Buffer types (crate-private)

### `Item`

What crosses the buffer.

| Variant | Carries | Meaning |
| --- | --- | --- |
| `Packet` | `CapturedPacket` | One observation, parsed and attributed. |
| `End` | `CaptureStats` | The acquisition side finished. Sent exactly once, exempt from the capacity bound (research R-6). |

### `Buffer` shared state

| Field | Type | Rule |
| --- | --- | --- |
| `queue` | `VecDeque<Item>` | FIFO. Push at the back, pop from the front. |
| `capacity` | `usize` | Never zero; rejected at construction. |
| `evicted` | `u64` | Advanced once per eviction, under the same lock that performs it. Saturating. Lives here rather than producer-side so it survives an unwinding producer (research R-5). |
| `open` | `bool` | Set false when the `Producer` is dropped, for any reason including unwinding. |

Guarded by one `Mutex`, with one `Condvar` for the consumer's wait.

### `Producer`

| Operation | Behavior |
| --- | --- |
| `push(Item::Packet)` | If `queue.len() == capacity`, pop the front, advance `evicted`, then push. Notify. Never waits for the consumer. |
| `push(Item::End)` | Push regardless of length. Notify. |
| `drop` | Set `open = false`, notify. |

Not `Clone`. One producer, so `open = false` on drop is unambiguous.

### `Consumer`

| Operation | Behavior |
| --- | --- |
| `next()` | Return the front item if any. Otherwise wait on the condvar while the queue is empty and `open`. Return `None` when the queue is empty and `open` is false. |
| `evicted()` | Read the shared eviction count. |
| `drain_count()` | Number of items still queued. Used only in assertions. |

## Public types

### `PipelineConfig`

| Field | Type | Default | Rule |
| --- | --- | --- | --- |
| `capacity` | `usize` | 65,536 | Section 12.4. Zero is rejected at construction with `ConfigError::ZeroCapacity`. |
| `read_timeout` | `Duration` | 100 ms | Passed to `PacketSource::next_packet`. Bounds stop latency (research R-8). |
| `addrs` | `InterfaceAddrs` | empty | Handed to the header parser for section 12.6 direction determination. |

`Default` is implemented. An empty address set is legal and means every packet
gets `NoLocalEndpoint`, which is a configuration a test uses deliberately.

### `ConfigError`

`#[non_exhaustive]`, one variant.

| Variant | Meaning |
| --- | --- |
| `ZeroCapacity` | A buffer that drops everything can only be a mistake. |

### `StopHandle`

A `Clone` wrapper over `Arc<AtomicBool>`.

| Operation | Behavior |
| --- | --- |
| `stop()` | Request the end. Idempotent. |
| `is_stopped()` | Read the flag. |

Observed by the acquisition side between packets. Cooperative: a source already
inside `next_packet` finishes that call first, so stop latency is bounded by
`read_timeout`.

### `Pipeline`

Owns everything for one run.

| Field | Type | Note |
| --- | --- | --- |
| `source` | `Box<dyn PacketSource>` | Acquisition. Never named by the attributor. |
| `attributor` | `Box<dyn FlowAttributor>` | Owned by the acquisition side for this slice (plan D-7). |
| `sinks` | `Vec<Box<dyn Sink>>` | May be empty. Order is the report's index order. |
| `config` | `PipelineConfig` | |
| `stop` | `StopHandle` | Handed out before the run starts. |

| Operation | Signature shape | Note |
| --- | --- | --- |
| `new` | `(source, attributor, config) -> Result<Self, ConfigError>` | Validates capacity. |
| `add_sink` | `(&mut self, Box<dyn Sink>)` | Index is the order added. |
| `stop_handle` | `(&self) -> StopHandle` | Callable before `run` consumes the pipeline. |
| `run` | `(self) -> PipelineReport` | Blocks. `#[must_use]` through the report. |

### `EndReason`

`#[non_exhaustive]`. Why the run stopped.

| Variant | Carries | Meaning |
| --- | --- | --- |
| `SourceClosed` | | The source reported `Closed`. The ordinary ending. |
| `Stopped` | | A stop was requested and observed. |
| `SourceFailed` | `SourceError` | A non-recoverable source error other than `Closed`. |
| `AllSinksRetired` | | Every attached sink returned a non-countable error, and the stop that caused is what ended acquisition. Only reachable with at least one sink. |

`AllSinksRetired` replaces `Stopped` and nothing else. If acquisition had
already ended on its own, by exhaustion or by a terminal source failure, before
the output side finished draining and retired the last sink, that earlier
ending is the reason and is reported. Burying a `DeviceLost` under a retirement
the output side found afterwards would hide the diagnostic an operator needs
most. The retirement is still reported, in `sink_failures`.

A recoverable source error produces none of these; the loop continues.

### `SinkFailure`

| Field | Type | Meaning |
| --- | --- | --- |
| `index` | `usize` | Position in the sink list as added. |
| `error` | `SinkError` | The error that retired the sink, or that it produced while being flushed or finished. |

Two distinct events produce one of these, and a caller must not assume which.
A `write` returning a non-countable error records one and retires the sink;
subsequent packets then advance `sink_dropped` rather than adding another
record. A `flush` or `finish` returning any error also records one, and there
is nothing left to retire or count, so the record is the only statement that
the output is probably incomplete.

One sink can therefore appear up to three times: once for retirement, once for
a failing flush, once for a failing finish. A `SinkError::Full` from `write`
produces no record at all; it is counted in `sink_dropped` and the sink stays
in service.

### `PipelineReport`

`#[must_use]`.

| Field | Type | Meaning |
| --- | --- | --- |
| `stats` | `CaptureStats` | The run's own final counters. The same value passed to every `Sink::finish`. |
| `ended` | `EndReason` | |
| `sink_failures` | `Vec<SinkFailure>` | Empty on a run where no sink retired. |

| Operation | Meaning |
| --- | --- |
| `is_clean()` | `ended` is `SourceClosed` and `sink_failures` is empty. Says nothing about drops; a clean ending can still have lost packets, and conflating the two would be the mistake `lost_anything` exists to prevent. |
| `into_result()` | `Ok(CaptureStats)` when clean, `Err(PipelineError)` otherwise. |

### `PipelineError`

Carries the whole report, so the accounting survives the failure path
(plan D-6). Implements `Display` and `std::error::Error`.

## Counter ownership

Which side of the buffer advances each counter, and on what event.

| Counter | Advanced by | Event |
| --- | --- | --- |
| `packets_captured` | acquisition | Each packet accepted from the source. |
| `packets_attributed` | acquisition | A flow key existed and `resolve` returned `Some`. |
| `packets_unattributed` | acquisition | A flow key existed and `resolve` returned `None`. |
| `parse.*` | acquisition | Copied from `HeaderParser::stats` at the end of the run. |
| `source.*` | acquisition | Copied from `PacketSource::stats` at the end of the run, unaltered. |
| `buffer_dropped` | buffer | Each eviction, under the buffer lock. |
| `sink_dropped` | sink thread | Each `Full` from a live sink, and each packet not offered to a retired sink. |
| `filter_gaps` | nobody | Stays zero. S13 owns it. |

The acquisition side's counters travel in `Item::End`. The sink thread adds
`Consumer::evicted()` and its own `sink_dropped` before calling `finish`, so
the value handed to every sink is the one the report carries.

## Invariants

These are asserted, not documented.

- **I-1, conservation.** For every sink index `i`:
  `received[i] + stats.buffer_dropped + refusals[i] == stats.packets_captured`.
- **I-2, attribution partition.**
  `packets_attributed + packets_unattributed <= packets_captured`, with
  equality only when every packet produced a flow key.
- **I-3, order.** The sequence a sink receives is a subsequence of the sequence
  the source produced, in the same relative order.
- **I-4, one terminal item.** Exactly one `Item::End` is ever pushed, and the
  consumer sees at most one.
- **I-5, no parse counter is loss.** `stats.fragcap_dropped()` is unaffected by
  any `parse` counter. Already asserted in `stats.rs`; restated here because the
  pipeline is the first thing to populate both.
- **I-6, backend relay.** `stats.source` equals what `PacketSource::stats`
  returned, field for field.
