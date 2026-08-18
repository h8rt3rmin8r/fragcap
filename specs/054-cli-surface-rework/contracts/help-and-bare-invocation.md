# Contract: grouped help and bare invocation

## Grouped `--help`

`fragcap --help` MUST list every top-level command under one of four headings,
with nothing hidden:

```
Capture:      capture, replay
Targets:      targets, technologies, steam
Environment:  doctor, extcap
Data:         catalog, schema
```

- The grouping is presentational only; it changes no capability.
- Every command present in the grammar appears under exactly one heading.

## Bare invocation

| Invocation | Output |
| --- | --- |
| `fragcap` (no args) | The `targets` listing, followed by a footer line pointing at `--help`. |
| `fragcap targets` (explicit) | The same listing, WITHOUT the footer. |
| `fragcap` with an empty `local.db` | A coherent empty listing plus the footer (not an error). |

The two listings MUST differ only by the footer line (SC-004). The footer decision
is made at the dispatch site, not inside the listing renderer, so the renderer is
shared.

## Exit codes

- Bare `fragcap` and explicit `fragcap targets` exit 0 on a successful listing.
- Usage errors (bad flags) exit 2, per the existing exit contract.
