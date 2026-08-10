### Added

- **The sink model gained its transports: file rotation, a Windows named pipe, a
  Unix domain socket, and TCP** (specification sections 14.1 to 14.4, roadmap
  slice S15). Format stays orthogonal to transport: a `SinkFactory` builds a
  fresh format encoder (pcapng or JSON Lines) over any connection, so any format
  writes to any transport.
- **A file sink rotates into numbered segments** by size or duration, closing
  each at a clean pcapng section boundary so every segment opens on its own in an
  unmodified analyzer. A capture with no rotation policy is a single segment,
  byte identical to before.
- **A streaming sink serves any number of live consumers** over the named pipe
  or TCP. Each consumer receives its own complete, independently valid stream,
  with its own header preamble replayed on connect, so a Wireshark client that
  opens `\\.\pipe\fragcap` mid-capture sees a valid capture from the connection
  point onward.
- **Per-consumer backpressure isolates a slow or dead reader.** Each consumer has
  an independent bounded queue; a full queue drops packets on that connection
  only, counted per consumer and surfaced, and never stalls the capture or any
  other sink. A consumer whose queue stays full past a timeout is disconnected
  and the disconnection reported. The file sink is unaffected by a stalled
  network consumer, which is what makes concurrent file-and-stream capture safe.
- **The command surface wires every `--sink` scheme to its transport** (`file:`,
  `pcapng:`, `jsonl:`, `pipe:`, `unix:`, `tcp://`), parses per-sink options
  (`format=`, `payload=`, `rotate-size=`, `rotate-duration=`, `queue=`,
  `timeout=`), and enables `--mode stream`. A streaming-only run with no capture
  file is valid. A sink whose format cannot be resolved, whose transport is
  unavailable on the current platform, or whose options mismatch its transport is
  a configuration error naming the cause, before capture starts.
