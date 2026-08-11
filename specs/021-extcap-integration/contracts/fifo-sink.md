# Contract: FIFO sink

The transport that streams a capture to the analyzer's FIFO or named pipe. It is
a direct single-writer pcapng sink, built through the existing `SinkFactory`.

## Sink grammar

- New scheme `fifo:<path>` parses to `SinkTransport::Fifo(PathBuf)`.
- The FIFO is pcapng only. An explicit `format=jsonl` on a `fifo:` sink is a
  configuration error (analyzers consume pcapng over extcap).
- Rotation options (`rotate-size`, `rotate-duration`) and streaming options
  (`queue`, `timeout`) do not apply to a FIFO and are refused with the existing
  option-mismatch messages.

## open_fifo(path)

`fragcap_sink::open_fifo(path: &Path) -> io::Result<Box<dyn Write + Send>>`.

| Path | Behavior |
| --- | --- |
| Windows, under `\\.\pipe\` | named-pipe client open for write, no create; short bounded retry on a busy pipe |
| any other path | open for write, create, truncate |

- Opens for writing only. Never reads. Never creates the Windows pipe (the
  analyzer owns it). Never transmits on a socket (P-1: `cargo xtask lint` still
  finds no transmit call).
- On failure returns the `io::Error`, which the sink builder surfaces as a run
  failure naming the path (before a started capture is reported).

## Build and behavior

- `build_fifo_sink` opens the path with `open_fifo`, builds a
  `SinkFactory::new(Format::Pcapng, interfaces)` encoder over the returned
  writer, and pushes it as the run's sink. No new format code.
- The written bytes are the unchanged pcapng writer's output: a Section Header
  Block, one Interface Description Block per declared interface, then Enhanced
  Packet Blocks with attribution comments. A single-interface FIFO stream is
  byte-comparable to a plain file capture of the same input (FR-005, the SC-002
  and SC-006 anchors).

## Conservation and disconnect (P-4)

- The sink's `write` returns `Ok` for a written packet, so the pipeline
  conservation invariant (received + buffer_dropped + refusals = captured) holds
  exactly as for a file sink.
- A slow reader backpressures the output loop; the pipeline's existing bounded
  drop-oldest buffer absorbs it and counts the drops (`buffer_dropped`). No new
  uncounted discard path.
- A reader that closes the FIFO breaks the write; the sink is retired. As the
  only sink, this ends the run cleanly (the intended stop when the analyst quits
  the analyzer). The retirement advances `sink_dropped` per the existing rule and
  is surfaced.

## Tier-1 test

Point `--fifo` at a regular temp file, drive the extcap capture through the
offline substrate (`--replay-source <fixture>`), read the file back with the
pcapng parser the writer tests use, and assert it reproduces the committed pcapng
golden for that fixture. The Windows named-pipe client connect is tier 2.
