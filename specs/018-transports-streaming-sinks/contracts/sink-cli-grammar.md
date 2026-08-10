# Contract: `--sink` scheme and options grammar

The command surface accepts one or more `--sink` values. Each is
`<destination>[,<key>=<value>]...`.

## Destinations (schemes)

| Scheme | Form | Transport | Availability |
| --- | --- | --- | --- |
| `file:` / `pcapng:` | `file:path` | Rotating file | all platforms |
| `jsonl:` | `jsonl:path` | Rotating file (JSON Lines) | all platforms |
| `pipe:` | `pipe:name` | Windows named pipe (`\\.\pipe\name`) | Windows only |
| `unix:` | `unix:path` | Unix domain socket | Unix only |
| `tcp://` | `tcp://host:port` | TCP listener | all platforms |

`--out`/`-o` remains a shorthand for a `file:` pcapng sink.

## Options (comma-separated, after the destination)

| Key | Applies to | Values | Default |
| --- | --- | --- | --- |
| `format` | any | `pcapng`, `jsonl` | inferred from file extension; required when no extension exists (pipe, tcp) |
| `payload` | any | `true`, `false` | `true` (mirrors `--no-payload`) |
| `rotate-size` | file | byte size (`100MB`, `1GB`) | none (no rotation) |
| `rotate-duration` | file | duration (`60s`, `5m`) | none (no rotation) |
| `queue` | streaming | packet count | `1024` |
| `timeout` | streaming | duration | `5s` |

Duration and size literals reuse the existing grammar (`fragcap-core::duration`
and the ring-window size parser already in the CLI).

## Resolution and refusal rules

- Format is inferred from a file extension (`.fcapng`/`.pcapng` -> pcapng,
  `.jsonl` -> JSON Lines). With no inferable extension and no `format=`, the
  sink is a configuration error naming the missing qualifier (FR-006, FR-014).
- A `pipe:` sink on a non-Windows target, or a `unix:` sink on a non-Unix
  target, is a configuration error naming the platform limitation (FR-014, D7).
- A file rotation option on a non-file destination, or a streaming option on a
  file destination, is a configuration error naming the mismatch.
- Two sinks binding the same pipe name or TCP address is a startup failure,
  reported, not silent (Edge Cases).
- `--mode stream` is accepted; a run whose only sinks are streaming transports
  is valid (FR-017). `--mode file` with a streaming sink, or `--mode stream`
  with only a file sink, is accepted (mode selects defaults, not exclusivity)
  unless a later slice constrains it.

## Examples

```text
--out capture.fcapng --sink jsonl:capture.jsonl
--sink file:session.fcapng,rotate-size=100MB
--sink file:session.fcapng,rotate-duration=60s
--sink pipe:fragcap,format=pcapng
--sink tcp://127.0.0.1:9999,format=jsonl,payload=false
--sink tcp://0.0.0.0:9999,queue=4096,timeout=10s
--mode stream --sink pipe:fragcap
```
