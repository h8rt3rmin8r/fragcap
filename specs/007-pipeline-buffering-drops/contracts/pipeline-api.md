# Contract: The pipeline public interface

**Slice**: S08 | **Date**: 2026-08-08 | **Plan**: [../plan.md](../plan.md)

fragcap's library surface is its contract; the command line tool is one
consumer of it. This document fixes what `fragcap_core::pipeline` promises,
what it does not, and which parts are expected to change in a named later
slice. Type shapes are in [../data-model.md](../data-model.md); this is the
behavioral contract.

## Construction

A pipeline is built from a source, an attributor, and a configuration, then
given zero or more sinks. Construction is the only place a caller can be told
its configuration is invalid, and the only invalid configuration is a zero
capacity.

- Construction MUST NOT start a thread, open a file, or read a packet.
- `stop_handle` MUST be callable before `run`, and the handle it returns MUST
  remain valid for the whole run.
- Adding a sink after `run` is not expressible, because `run` consumes the
  pipeline.

## Running

`run` consumes the pipeline and blocks until the run ends. It always returns a
report; there is no path on which the caller learns the outcome without also
being handed the counters.

The acquisition loop, per packet:

1. Check the stop flag. If set, end with `Stopped`.
2. Call `next_packet` with the configured timeout.
   - `Ok(Some(raw))`: continue below.
   - `Ok(None)`: a timeout that the source chose to report as success. Continue
     the loop. Nothing is counted.
   - `Err(e)` where `e.is_recoverable()`: continue the loop. Nothing is
     counted.
   - `Err(SourceError::Closed)`: end with `SourceClosed`.
   - `Err(e)`: end with `SourceFailed(e)`.
3. Wrap in a `CapturedPacket`, advance `packets_captured`.
4. Parse. The parser's own counters accumulate inside it.
5. If a flow key was produced, call `resolve(key, packet.ts)` with the packet's
   own timestamp, and advance exactly one of `packets_attributed` or
   `packets_unattributed`. If no flow key was produced, advance neither.
6. Push into the buffer. This never waits for the sink thread.

On ending, the acquisition side copies `PacketSource::stats` and
`HeaderParser::stats` into its counters and pushes exactly one terminal item.

The output loop, per item:

1. `Item::Packet`: offer to every sink in index order.
   - `Ok(())`: that sink's received count advances.
   - `Err(e)` where `e.is_countable()`: `sink_dropped` advances by one. The
     sink stays in service.
   - `Err(e)` otherwise: the sink is retired, a `SinkFailure` is recorded with
     its index, and `sink_dropped` advances by one for this packet.
   - A retired sink is not offered the packet and `sink_dropped` advances by
     one.
2. `Item::End`: take the acquisition counters and leave the loop.
3. `None` from the consumer, meaning the producer went away without a terminal
   item: leave the loop with whatever the output side counted. Only reachable
   when the acquisition side panicked.

When every sink has retired, the output side requests the stop so the
acquisition side winds down, and continues to drain and count until the
terminal item arrives. The end reason is `AllSinksRetired`.

## Finishing

Whatever the ending:

- Every sink MUST be flushed and then finished, exactly once each, retired or
  not.
- Every sink MUST receive the same `CaptureStats` value, and that value MUST be
  the one the report carries.
- Finishing MUST happen after the buffer is drained, so the last buffered
  packet is written before the trailing statistics that describe it.

An error returned by `flush` or `finish` is recorded as a `SinkFailure` for
that sink. It cannot retire the sink for future writes, because there are none.

## Panic behavior

- If a sink panics, the pipeline does not catch it. The panic unwinds the
  output thread and reaches the caller when `run` joins it.
- If the acquisition side panics, the producer is dropped during unwinding,
  which closes the buffer. The output side drains, flushes, and finishes. The
  guard holding the output thread's join handle joins it during the same
  unwind, so this completes before the panic escapes `run`.
- A panic is never converted into an `EndReason`.
- The acquisition side's counters are lost with its stack. The buffer's
  eviction count is not, because it lives in the shared state.

## Guarantees

- **Order.** Packets reach a sink in the order the source produced them.
  Eviction removes packets; it never reorders the survivors.
- **Fidelity.** No field of a `CapturedPacket` is read, rewritten, normalized,
  or repaired between the source and the sink, except the parser writing the
  fields the parser owns (`flow`, `direction`) and the attributor's answer
  being placed in `attribution`.
- **Retention.** A packet with no flow key is written and marked. It is never
  dropped for being unparseable.
- **Conservation.** For every sink, the number it received plus
  `buffer_dropped` plus its refusals equals `packets_captured`.
- **Relay.** `stats.source` is what the backend reported, field for field.

## Non-guarantees

Stated so that a caller does not build on an accident.

- **No timing guarantee.** No throughput or latency figure is promised or
  measured. See spec SC-012.
- **No stop-latency guarantee below the read timeout.** A stop takes effect
  after the in-flight `next_packet` returns.
- **No guarantee about which packets are evicted** beyond "the oldest
  buffered". A caller must not infer that a particular packet survived.
- **No interface identity.** `CapturedPacket` carries no interface identifier
  until S09, so a multi-interface capture is not expressible and both writers
  still refuse a second interface.
- **No logging.** The report and the counters are the whole reporting surface.
- **No background form.** A caller wanting `run` off the current thread spawns
  a thread and keeps the stop handle.

## Expected to change

| What | When | Why |
| --- | --- | --- |
| `PacketSource` gains `Send` | S09 | Section 12.1 puts one capture thread on each interface, which the current arrangement cannot express. |
| The attributor moves behind a published snapshot | S10 or S13 | Section 8.6's control thread needs a publisher and a snapshot type that do not exist yet. |
| `filter_gaps` gains a producer | S13 | Nothing narrows a filter yet. |
| The session anchor is recorded at capture start | S09 | Needs a capture driver clock reading. |
| Retirement policy revisited | S15 | A streaming sink whose failure is routine may want different behavior; the counter it would use already exists. |

## Stability

`Pipeline`, `PipelineConfig`, `StopHandle`, `PipelineReport`, `EndReason`, and
`SinkFailure` are public and expected to survive to 1.0.0 with additions rather
than changes. `EndReason`, `ConfigError`, and `SinkFailure` are
`#[non_exhaustive]` for that reason. The buffer is crate-private and carries no
promise at all.
