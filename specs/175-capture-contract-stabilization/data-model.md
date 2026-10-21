# Data Model: Capture Contract Stabilization

## Capture clock

`armed_elapsed` is zero at session arm. Live reads come from a monotonic instant; replay reads come from packet-timestamp differences against the first replay packet. It is never serialized as a packet timestamp. Duration and acquisition decisions consume only this value.

## Packet evidence

`packet.ts` remains the absolute source instant. Process events have their own absolute instants. `SessionGate` compares these for the half-open write interval `[admit_from, admit_until)`; pcapng, JSON Lines, and correlation retain the packet instant unchanged.

## Effective capture scope

The scope holds optional exact role names, `target` or `all` ownership scope, `both`, `in`, or `out` direction, and global plus per-sink payload retention. Role validation precedes side effects. Admission decisions happen once per packet before sinks and volume bounds.

## Admission accounting

The existing captured, admitted, watch-time, out-of-window, attributed scope, unresolved scope, buffer-loss, and sink-loss authorities remain. S175 adds separate known-opposite-direction and unknown-direction omission counts. Their sum with existing outcome partitions must equal observed packets at the corresponding boundary. Ring evictions remain a later sink-side window outcome and never change pre-ring admitted counts.

## Metadata-only pcapng packet

`orig_len` is the source-reported wire length. `captured_len` is the actual retained prefix length, zero when the header boundary cannot be proven. The EPB packet bytes contain only that prefix. The interface declaration supplies link type so a writer can evaluate the boundary without guessing. Full-payload mode remains byte-identical to existing output.

## Sink specification

`destination` is parsed before modifiers. `,,` decodes to a literal destination comma. A single comma begins an option. Each modifier stays one `key=value` pair. Parsing errors name dangling separators, invalid escapes, or actual malformed modifiers without creating a destination.
