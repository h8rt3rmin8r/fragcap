# Data Model: Extcap analyzer integration

The entities this slice introduces or extends, with their invariants. No
persistent storage: the model is the command grammar, the transport, and the
doctor inputs.

## ExtcapArgs (fragcap-cli, `cli.rs`)

The clap grammar for the `extcap` command, replacing `StubArgs`.

| Field | Type | Meaning |
| --- | --- | --- |
| `extcap_interfaces` | `bool` | Print the interface list and exit. |
| `extcap_dlts` | `bool` | Print the link types for the selected interface. |
| `extcap_config` | `bool` | Print the configurable-option declaration. |
| `capture` | `bool` | Start a capture, streaming to `--fifo`. |
| `fifo` | `Option<PathBuf>` | The analyzer FIFO or named-pipe path to stream to. |
| `extcap_interface` | `Option<String>` | The selected extcap interface name. |
| `extcap_version` | `Option<String>` | The analyzer protocol version query; accepted, not acted on. |
| `profile` | `Option<String>` | Config option: the profile reference. |
| `roles` | `Option<Vec<String>>` | Config option: role scope (`parse_roles`). |
| `direction` | `Option<Direction>` | Config option: direction scope. |
| `loopback` | `bool` | Config option: include loopback. |
| `offline` | `OfflineArgs` | Flattened hidden substrate flags, for tier-1 tests. |

**Invariants**:

- Exactly one mode is acted on per invocation, chosen in a fixed precedence:
  a declaration flag, or `--capture`. An invocation with none is a usage error.
- `--capture` requires `--fifo`; a declaration invocation requiring a selected
  interface requires `--extcap-interface`. A missing required argument is a usage
  error (exit 2) naming it, before any capture starts.
- `--extcap-interface`, when given, MUST name a presented interface (`fragcap`);
  an unknown name is a usage error.
- The config option fields (`profile`, `roles`, `direction`, `loopback`) are the
  values the analyzer passes back at capture; they are meaningful only with
  `--capture` and are overlaid on the profile exactly as the `run` flags are.

## Extcap interface model (fragcap-cli, `commands/extcap`)

The fixed declaration the emitters render. Not a runtime struct so much as the
contract the pure emitters encode.

- **Interface**: one entry, `value=fragcap`, a human display string. Rendered by
  `--extcap-interfaces`.
- **DLT**: one entry for the `fragcap` interface, `number=1` (EN10MB), display
  `Ethernet`. Rendered by `--extcap-dlts`.
- **Config options**: four `arg` entries rendered by `--extcap-config`, with
  stable `number`s and `call` names matching the `run` flags:

  | number | call | type | notes |
  | --- | --- | --- | --- |
  | 0 | `--profile` | string | required; the profile reference |
  | 1 | `--roles` | string | comma-separated role names |
  | 2 | `--direction` | selector | values both (default), in, out |
  | 3 | `--loopback` | boolflag | include loopback |

**Invariant**: the emitted text conforms to the extcap control grammar (each line
is `word {key=value}{key=value}...`), so an analyzer parses it into a native
dialog. The `call` names are exactly the `run` flag names.

## SinkTransport::Fifo (fragcap-cli, `args.rs`)

A new variant of the existing `SinkTransport` enum, and a `fifo:` scheme in
`parse_sink`.

- `Fifo(PathBuf)` from `fifo:<path>`.
- Built through the existing `SinkSpec` and `build_sinks`, so the format and
  option validation is shared. The FIFO is pcapng only: with no `format=` it
  resolves to pcapng like the other extensionless transports require an explicit
  format, except the `fifo:` scheme fixes pcapng (analyzers consume pcapng).
- Rotation and streaming options do not apply to a FIFO (a single write-only
  stream); they are refused with the existing option-mismatch messages.

## open_fifo (fragcap-sink, `transport/fifo.rs`)

`open_fifo(path: &Path) -> io::Result<Box<dyn Write + Send>>`.

- **Windows, path under `\\.\pipe\`**: connect as a named-pipe client (open for
  write, no create; a short bounded retry when the pipe is momentarily busy).
- **Any other path**: open for writing, creating and truncating.

**Invariants**: opens for writing only (never reads, never transmits on a
socket); creates nothing on the Windows pipe path (the analyzer owns the pipe);
returns an error the caller surfaces as a run failure naming the path when the
open fails. The returned writer is fed to `SinkFactory::build`, unchanged.

## Inputs.extcap_dir (fragcap-cli, `doctor/mod.rs`)

The doctor classifier input gains one field.

| Field | Type | Meaning |
| --- | --- | --- |
| `extcap_installed` | `bool` | Existing. A fragcap binary is present in the extcap directory. |
| `extcap_dir` | `Option<PathBuf>` | New. The analyzer's extcap directory, when it can be determined. |

**Invariant**: `integration()` names `extcap_dir` in both the installed and
not-installed details. When `extcap_dir` is `None` (the platform location cannot
be determined), the report says so rather than naming a wrong path. The probe
sets both fields read-only.
