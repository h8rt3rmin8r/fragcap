### Decisions

**2026-08-10: transports and streaming sinks (slice S15), decisions worth
recording for promotion to specification section 29.**

- **A consumer's encoder is an ordinary `Sink` over its connection.** The
  streaming sink constructs one per connection through a `SinkFactory` that
  replays the header, so a mid-capture joiner and a fresh rotation segment each
  begin with their own valid header. Format stays orthogonal to transport with no
  new abstraction and the S06/S07 writers unchanged. The alternative, a single
  shared encoder fanning bytes to a broadcast writer, cannot give a mid-capture
  joiner a valid pcapng header and was rejected (P-5).
- **The streaming sink's `write` always returns success and never advances the
  capture-wide `sink_dropped`.** Per-consumer drops are the sink's own,
  separately reported accounting. This preserves the pipeline conservation
  identity (the sink received every packet) and keeps a slow downstream reader
  from retiring the sink, which folding per-consumer loss into `sink_dropped`
  would not (P-4).
- **A stalled consumer is unblocked by a stop flag polled through a short socket
  write timeout, not by `shutdown()`.** `TcpStream::shutdown` does not portably
  unblock a blocked send (notably on Windows), so a `PollingWriter` gives the
  socket a fixed short write timeout and rechecks a stop flag between attempts.
  The disconnect decision itself lives in the streaming sink (queue full past the
  timeout), so the disconnect reason is deterministic and finish is bounded
  regardless of the configured timeout. The stop-flag write aborts with
  `ConnectionAborted`, not `Interrupted`, because `write_all` retries the latter.
- **The Windows named pipe unblocks a stalled writer with `CancelIoEx`, not
  `DisconnectNamedPipe`.** `DisconnectNamedPipe` discards bytes already in the
  pipe buffer, truncating a consumer that kept up; `CancelIoEx` cancels only the
  in-flight write, and `CloseHandle` on drop then lets the client drain the
  remaining buffered bytes before end of stream.
- **No new third-party dependency.** The file, TCP, and Unix transports are the
  standard library; the named pipe reuses `windows-sys`, already pinned at 0.36
  for the attribution socket table, taken under `[target.'cfg(windows)']` so it
  adds no package to `Cargo.lock`. Additive `windows-sys` features
  (`Win32_System_Pipes`, `Win32_Storage_FileSystem`, `Win32_System_IO`,
  `Win32_Security`) were enabled; they change no resolved version.
- **The Unix domain socket transport is `cfg(unix)` and is not compiled or
  exercised by the gate on the Windows development machine or the Windows
  `platform` workflow.** It is present for parity and future platforms per
  specification 14.2; on the primary platform it is unexercised, recorded rather
  than hidden. The named-pipe tier-2 tests do run on the Windows dev machine and
  their result is reported.
