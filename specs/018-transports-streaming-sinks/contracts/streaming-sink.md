# Contract: streaming sink, factory, and acceptor

These are internal `fragcap-sink` seams (not a public API surface for external
callers); the contract pins their behavior so tests and the CLI can rely on it.

## SinkFactory

```text
SinkFactory::build(conn: Box<dyn Write + Send>) -> Result<Box<dyn Sink>, WriteError>
```

- Constructs a fresh format encoder over `conn` and writes its header preamble
  before returning (pcapng: SHB + one IDB per declared interface; JSON Lines:
  no preamble).
- The returned `Box<dyn Sink>` is fed packets with `write` and finalized with
  `finish(stats)`, exactly as any sink.
- Guarantees: every built encoder is independently valid from its first byte and
  declares the identical interface set.

## Acceptor

```text
Acceptor::run(consumers: SharedConsumers, factory: SinkFactory)   // on its own thread
```

- Binds the transport (TCP listener, named-pipe first instance, Unix listener)
  at construction; a bind failure is returned before capture starts (surfaces as
  a configuration error).
- Loops accepting connections. For each, wraps the connection as a
  `Box<dyn Write + Send>`, has `factory.build` produce the consumer encoder,
  registers a `Consumer` (channel + writer thread) in `consumers`, and continues
  accepting.
- Stops when the streaming sink signals shutdown at `finish`.

Three implementations: `TcpAcceptor` (all platforms), `NamedPipeAcceptor`
(`cfg(windows)`), `UnixAcceptor` (`cfg(unix)`).

## StreamSink (implements core `Sink`)

- `write(packet)`:
  - For each live consumer, `try_send(Packet(packet.clone()))`.
    - `Ok`: increment `delivered`; clear `full_since`.
    - `Err(Full)`: increment `dropped`; set `full_since` if unset.
    - `Err(Disconnected)`: mark consumer removed (`client-closed`).
  - Any consumer with `full_since` older than `disconnect_timeout`: shut down its
    connection and remove it (`timeout`).
  - Returns `Ok(())` unconditionally (including with zero consumers).
- `flush()`: `Ok(())` (per-consumer threads flush their own encoders).
- `finish(stats)`: send `Finish(Arc::new(stats.clone()))` to every live
  consumer, join all consumer threads and the acceptor thread, close the
  transport, return `Ok(())`.

Guarantees:
- `write` never blocks and never returns `Err` (conservation invariant
  preserved; sink never retired for downstream slowness).
- A per-consumer drop never advances `CaptureStats.sink_dropped`.
- Each consumer's stream is valid from its connect point with no earlier packet.

## Consumer writer thread

```text
loop {
  match rx.recv() {
    Ok(Packet(p))      => encoder.write(&p)  // on Err: break, reason = write-error
    Ok(Finish(stats))  => { encoder.finish(stats); break }  // reason = capture-ended
    Err(Disconnected)  => break             // sink dropped the sender
  }
}
report ConsumerReport { id, delivered, dropped, disconnect_reason }
```

- Owns the only handle to the connection and the encoder; no lock guards a
  socket write.
- A write error (client vanished) ends the thread with reason `write-error`;
  the sink reaps it on the next `write`.

## Per-consumer accounting

- Each `Consumer` holds `delivered` and `dropped` (both monotonic).
- `delivered + dropped` equals packets offered while connected.
- Reports are surfaced through the CLI `--json` event stream and the end-of-run
  summary, separate from `CaptureStats`.
