# Contract: the `capture` command

The single capture verb superseding `run`, `tap`, and `watch`.

## Grammar

```
fragcap capture (--target <SELECTOR> | --process <IMAGE>)
                [--path <SUBSTR>] [--path-regex <RE>]
                [--mode file|ring|stream] [--ring <DUR|SIZE>]
                [--duration <DUR>] [--wait <DUR>]
                [--launch]
                [--out <PATH>] [--sink <SPEC>]...
                [--max-packets <N>] [--max-bytes <SIZE>]
                [--roles <R,...>] [--direction <DIR>]
                [--interface <IF>]... [--loopback] [--no-payload]
```

`--target` and `--process` form a required, mutually exclusive group.

## Behavioural contract

| Given | Expect |
| --- | --- |
| `--target <selector>` resolving to one stored target | Capture that target's client process; `--launch` permitted (anchor from the target). |
| `--target <selector>` ambiguous over the listing | Exit 2, ambiguity message (S051 resolution). |
| `--target <selector>` matching nothing | Exit 2 / not-found (S051 resolution). |
| `--process <image>` | Capture the running process of that image name. |
| `--process <image> --path <substr>` / `--path-regex <re>` | Capture only a matching-path instance (disambiguation). |
| `--process <image> --wait <dur>` | Wait up to `<dur>` for the process to appear, then capture. |
| `--process <image> --mode ring --ring <win> --out <path>` | Ring-buffer the named process (previously inexpressible). |
| `--process <image> --launch` | Exit 2: a raw process name carries no launchable anchor. |
| neither `--target` nor `--process` | Exit 2: a target input is required. |
| both `--target` and `--process` | Exit 2: mutually exclusive. |
| `--mode ring` without `--out` or `--ring` | Exit 2 (existing `reject_unsupported`). |
| `--launch` on a non-Windows build | Exit 2: unsupported. |

## The five section-9.1 captures (each must be expressible and tested)

1. Profile-equivalent + ring: `capture --target <t> --mode ring --ring <win> --out <p>`
2. Named process + ring: `capture --process <img> --mode ring --ring <win> --out <p>`
3. Named process + wait-for-start: `capture --process <img> --wait <dur> --out <p>`
4. Named process + launch-under-capture: `capture --target <t> --launch ...` (launch
   needs an anchor, so the "named process launched under capture" is expressed via a
   registered target carrying the anchor; a bare `--process --launch` is the exit-2
   negative above).
5. Profile-equivalent + give-up timeout: `capture --target <t> --wait <dur> --out <p>`

## Non-goals

Capture internals (pipeline, attribution, ring, launch mechanics) are unchanged.
This command is the reshaped front door to them.
