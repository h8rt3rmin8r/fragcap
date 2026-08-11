# Contract: extcap command grammar and declaration output

The `fragcap extcap` command surface and the exact shape of each declaration
invocation's output. The output grammar is the extcap control grammar an analyzer
parses.

## Invocations

| Invocation | Output stream | Exit |
| --- | --- | --- |
| `extcap --extcap-interfaces` | stdout: the interface list | 0 |
| `extcap --extcap-dlts --extcap-interface fragcap` | stdout: the DLT list | 0 |
| `extcap --extcap-config --extcap-interface fragcap` | stdout: the arg list | 0 |
| `extcap --capture --fifo <path>` | pcapng to `<path>`; diagnostics to stderr | 0 on clean stop |
| `extcap --extcap-version [value]` | accepted alongside the above | as above |

## `--extcap-interfaces`

Emits one interface line. The version line is emitted when the analyzer includes
its version query.

```text
extcap {version=<fragcap version>}{help=<url or short help>}
interface {value=fragcap}{display=fragcap: process-attributed capture}
```

- Exactly one `interface` line, `value=fragcap`.
- Exit 0.

## `--extcap-dlts`

Emits the link type for the selected interface.

```text
dlt {number=1}{name=EN10MB}{display=Ethernet}
```

- `number=1` (EN10MB / Ethernet), the top-level default; per-packet link types
  are carried by the stream's Interface Description Blocks.
- Requires `--extcap-interface fragcap`; a missing selector is a usage error
  (exit 2). An unknown interface is a usage error.
- Exit 0.

## `--extcap-config`

Emits the four configurable options as `arg` lines, plus the `value` lines for
the direction selector.

```text
arg {number=0}{call=--profile}{display=Profile}{type=string}{required=true}{tooltip=...}
arg {number=1}{call=--roles}{display=Roles}{type=string}{tooltip=comma-separated roles}
arg {number=2}{call=--direction}{display=Direction}{type=selector}
value {arg=2}{value=both}{display=Both}{default=true}
value {arg=2}{value=in}{display=Inbound}
value {arg=2}{value=out}{display=Outbound}
arg {number=3}{call=--loopback}{display=Include loopback}{type=boolflag}
```

- Exactly four `arg` lines, `number` 0 to 3, `call` names `--profile`, `--roles`,
  `--direction`, `--loopback`.
- The `--direction` selector declares `both` (default), `in`, `out`.
- Requires `--extcap-interface fragcap`.
- Exit 0.

## `--capture`

- Requires `--fifo <path>`; a missing `--fifo` is a usage error (exit 2) naming
  it, before any capture starts.
- Reads the config values back as `--profile <ref>`, `--roles <list>`,
  `--direction <both|in|out>`, `--loopback`, applying them through the same
  overlay `run` uses (FR-006).
- Streams pcapng to `<path>`: a Section Header Block, one Interface Description
  Block per declared capture interface, then the packets with attribution
  comments (FR-004, FR-005).
- On a clean stop (source exhaustion, a bound, an interrupt, or the analyzer
  closing the FIFO) exits 0; a profile that fails to resolve or validate exits 2
  before capture (a configuration error, not a started-but-empty capture).

## Error contract

| Condition | Result |
| --- | --- |
| No mode flag (no declaration, no `--capture`) | usage error, exit 2 |
| `--capture` without `--fifo` | usage error naming `--fifo`, exit 2 |
| declaration requiring a selector without `--extcap-interface` | usage error, exit 2 |
| `--extcap-interface <unknown>` | usage error naming the unknown interface, exit 2 |
| profile fails to resolve or validate | configuration error, exit 2, no capture started |

## Grammar-conformance check (tests)

A test parses each declaration invocation's stdout and asserts:

- every non-empty, non-`extcap` line matches `^\w+( \{[^=}]+=[^}]*\})+$` (a
  keyword followed by one or more `{key=value}` groups);
- the `interface`, `dlt`, and `arg`/`value` line counts and key contents match
  the tables above.

This is the SC-001 check, run with no analyzer installed.
