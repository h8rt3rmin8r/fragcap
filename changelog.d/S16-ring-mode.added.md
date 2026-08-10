### Added

- **Ring mode: a rolling in-memory window dumped on trigger** (specification
  section 7.2, FR-8, roadmap slice S16). `fragcap run --mode ring --ring <window>
  --out <file>` retains the most recently captured packets, bounded by a duration
  or a byte size, discarding the oldest as new ones arrive, and writes the
  retained window to the output file when the capture ends. The worked headline
  is a rolling ten-minute window dumped on interrupt.
- **The dump fires on every stop condition, by one path.** Ring mode reuses the
  six session stop conditions already in place (operator interrupt, duration
  bound, terminal-stage exit, all-non-service-processes-exited, source
  exhaustion, unrecoverable sink error) through the sink's finish seam; it adds no
  stop condition of its own. The dumped file is a single, independently valid
  pcapng an unmodified analyzer opens, byte-comparable to a plain file capture
  when the window is larger than the whole input.
- **A size ring window is measured by captured length**, the same quantity the
  `--max-bytes` bound sums, so an operator reasons about one notion of capture
  size across `--ring 64mb` and `--max-bytes 64mb`. A window smaller than one
  packet still retains that one packet, so a capture that saw traffic never dumps
  an empty file.
- **The command surface wires `--mode ring` and `--ring`**, replacing the earlier
  stub refusals. Ring mode requires both `--out` and `--ring`; a volume stop bound
  (`--max-bytes`, `--max-packets`) is refused in ring mode because a rolling window
  does not stop on accumulated volume; and a `--ring` window given outside ring
  mode is refused rather than silently ignored. Each is a configuration error
  naming the cause, before capture starts. `--duration` remains valid in ring mode.
- **A ring eviction is the sink's own counted accounting, never a capture loss.**
  The ring sink accepts every packet the pipeline delivers, so the pipeline
  conservation identity (received + buffer_dropped + refusals = captured) is
  preserved; the evicted count is reported by the sink, the way a streaming sink's
  per-consumer drops are (constitution P-4, P-9).
