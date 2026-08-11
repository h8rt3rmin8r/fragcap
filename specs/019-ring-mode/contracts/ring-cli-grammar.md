# Contract: Ring-mode command-line grammar and refusals

The command-line surface for ring mode (specification section 17.2). The flags
already parse; this slice makes them do something and adds the refusals. The
grammar itself is unchanged from what `fragcap-cli` already accepts.

## Flags (already parsed, now honored)

```text
-m, --mode ring          Select ring mode (or profile [capture] mode = "ring")
    --ring <DUR|SIZE>     The rolling window: a duration (e.g. 10m) or a size (e.g. 64mb)
-o, --out <PATH>          The capture file the retained window is dumped to
-d, --duration <DUR>      Optional stop bound; ring is still dumped when it fires
```

`--ring` parses a duration first, then a size (existing `parse_ring`). `--out`
in ring mode is the pcapng dump target.

## Effective mode resolution

The effective mode is the command line over the profile's `[capture]` default
(specification FR-007): an explicit `--mode ring` wins; absent `--mode`, a
profile declaring `mode = "ring"` selects ring mode. Every refusal below is
evaluated against the **effective** mode, not only an explicit `--mode ring`, so
a profile-selected ring mode is validated identically.

## Refusals (configuration errors, exit code 2, before capture starts)

| Condition | Message names |
| --- | --- |
| Ring mode, no `--out` | the missing output file (ring needs a dump target) |
| Ring mode, no `--ring` | the missing ring window |
| Ring mode with `--max-bytes` or `--max-packets` | that a volume stop bound does not apply to a rolling window |
| `--ring` given without ring mode | that the ring window applies only in ring mode |

Each is a `CliError::usage` (exit 2). No capture is started. The messages name
the specific cause, matching the existing style of the transport and launch
refusals in `reject_unsupported`.

## Worked invocations

```bash
# The headline: a rolling ten-minute window, dumped on interrupt
fragcap run --profile eso --mode ring --ring 10m --out captures/eso-ring.fcapng

# A rolling 64 MB window, dumped when the terminal stage exits or on interrupt
fragcap run --profile eso --mode ring --ring 64mb --out captures/eso-ring.fcapng

# Ring plus a hard duration stop: dump the last 5 minutes' window after 30 minutes
fragcap run --profile eso --mode ring --ring 5m --duration 30m \
            --out captures/eso-ring.fcapng
```

## Rejected invocations

```bash
fragcap run --profile eso --mode ring --ring 10m
#   error: ring mode needs an output file; pass --out <PATH>

fragcap run --profile eso --mode ring --out captures/x.fcapng
#   error: ring mode needs a ring window; pass --ring <DURATION|SIZE>

fragcap run --profile eso --mode ring --ring 10m --out x.fcapng --max-packets 1000
#   error: --max-packets does not apply in ring mode; a rolling window does not
#          stop on accumulated volume

fragcap run --profile eso --ring 10m --out x.fcapng
#   error: --ring applies only in ring mode; add --mode ring
```

(Exact message wording is finalized in implementation; the named cause in each
row is the contract.)
