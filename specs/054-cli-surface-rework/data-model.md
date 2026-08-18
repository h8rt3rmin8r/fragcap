# Phase 1 Data Model: CLI surface rework

This slice adds no persistent data. The "models" here are the shape of a `capture`
invocation and the command-namespace map. Persistent stores (`catalog.db`,
`local.db`) and their row types are unchanged from S050/S051/S053.

## Capture invocation

The resolved intent of one `capture` call. It is the union of what `run`, `tap`,
and `watch` each expressed partially, feeding the single `effective_config`
assembly and then the existing pipeline.

| Field | Source flag | Notes |
| --- | --- | --- |
| target input | `--target <selector>` XOR `--process <image>` | Exactly one required (clap `ArgGroup`, `required(true)`). Neither/both -> exit 2. |
| path anchor | `--path <substr>`, `--path-regex <re>` | Optional; valid with `--process` (and with a `--target` whose synthesized stage accepts one). Disambiguates same-named processes. |
| mode | `--mode file\|ring\|stream` | Orthogonal; `ring` requires `--out` + `--ring` (existing `reject_unsupported`). |
| ring window | `--ring <dur\|size>` | Ring mode only. |
| duration | `--duration <dur>` | Capture bound from arm. |
| acquisition wait | `--wait <dur>` | Give-up timeout for a target not yet running (was watch-only). |
| launch | `--launch` | Requires a launchable anchor on the resolved `--target`; usage error with `--process` or an anchorless target. |
| output | `--out <path>`, `--sink <spec>` (repeatable) | Existing sink specs. |
| stop bounds | `--max-packets`, `--max-bytes` | Not valid in ring mode. |
| scoping | `--roles`, `--direction`, `--interface`, `--loopback` | Existing. |
| payload | `--no-payload` | Existing. |
| offline substrate | hidden `--replay-source`, `--attr-script`, `--process-script`, `--local-addr`, `--fire-interrupt` | Hidden; drives tier-1 tests. |

**Resolution**:
- `--target` -> S051 selector resolution against `local.db` -> `TargetEntry` ->
  reduce `launch_entries` to one client image name (existing `windows_executables`)
  -> synthesize one-stage `Profile` (existing path). `--launch` reads the entry's
  `steam:<app_id>` anchor.
- `--process` -> synthesize one-stage `Profile` from the image name directly, with
  optional path anchors. `--launch` -> usage error (no anchor).

**Validation** (all pre-capture, exit 2):
- Exactly one target input present.
- `--launch` only with a launchable anchor.
- Existing mode/ring/volume rejections (`reject_unsupported`) unchanged.

## Command namespace map

Presentational grouping plus store binding. The four help groups and their members:

| Group | Commands | Store / purpose |
| --- | --- | --- |
| Capture | `capture`, `replay` (stub) | The capture engine. |
| Targets | `targets`, `technologies`, `steam` | User target authoring/inspection. |
| Environment | `doctor`, `extcap` | Readiness and analyzer integration. |
| Data | `catalog`, `schema` | Shipped catalog data; artifact validation. |

**`catalog` subcommands** (write/read `catalog.db`): `import`, `export`, `seed`,
`seed-engine`, `seed-signatures`, `update`. First five moved verbatim from
`targets`; `update` is new (net-gated fetch).

**`targets` subcommands** (write/read `local.db`): `add` (now with `--steam
<app_id>`), `list`, `show`, `discover`, `scan`. Catalog ops removed.

**`steam` subcommands**: installed-title enumeration and Steam metadata reads;
`profile` scaffolding removed (moved to `targets add --steam`).

**Removed**: top-level `run`, `tap`, `watch`; the `profile` command and all its
subcommands; the `--profile-dir` global.

## Bare invocation

| Invocation | Behaviour |
| --- | --- |
| `fragcap` (no subcommand) | Run the `targets` listing, then print a footer line pointing at `--help`. |
| `fragcap targets` (explicit) | The same listing, no footer. |
| `fragcap --help` | The four grouped headings, nothing hidden. |

The footer is a single boolean decided at the dispatch site (bare = append,
explicit = omit), so the two listings are identical except for the footer line.
