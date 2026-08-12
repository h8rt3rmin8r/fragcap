# Data Model: CLI readiness, help, and output-contract polish

This slice adds no persistent data. The "entities" are the in-memory value
types that carry readiness facts and structured events. Only the deltas from the
current types are described; existing fields are unchanged unless noted.

## Doctor readiness inputs (`crates/fragcap-cli/src/doctor/mod.rs`, `Inputs`)

The pure-function `doctor` design injects an `Inputs` value and classifies it.
Two capability-presence fields are added, mirroring the existing
`etw_available: Option<bool>` field and its `None = not compiled in` convention.

| Field | Type | Meaning |
| --- | --- | --- |
| `etw_available` | `Option<bool>` | Existing. `None` = tracing feature not built in. |
| `live_available` | `Option<bool>` (new) | `None` = the `live` capture backend is not compiled into this binary. `Some(true)` = compiled in. (`Some(false)` is reserved; today presence is compile-time, so the probe yields `Some(true)`/`None`.) |
| `socket_table_available` | `Option<bool>` (new) | `None` = the `socket-table` attribution backend is not compiled in. |
| `npcap` | `Option<NpcapInfo>` | Existing. `version` field now carries the real detected version (see below). |

### `NpcapInfo.version`

- Was hardcoded to `"installed"` in the Windows probe.
- Becomes the detected npcap version string when readable; falls back to
  `"installed"` when it cannot be determined. The `Check` detail renders
  `format!("version {}", info.version)` unchanged, so a real version flows
  through with no rendering change; the fallback reads "version installed" today
  and is reworded so the fallback does not claim a version it lacks.

## Readiness check (`Check` in `doctor/mod.rs`)

No structural change to `Check` (section/name/status/detail/remediation). This
slice adds new `Check` instances and adjusts one severity:

| Check (section) | Prior status | New status | Issue |
| --- | --- | --- | --- |
| live backend (Capture driver) | absent entirely | `Skip`/`Fail`: `Fail` (blocking) when the `live` backend is not compiled in | #63 |
| socket-table backend (Capture driver) | absent entirely | `Skip`/`Warn`: `Warn` when the `socket-table` backend is not compiled in | #63 |
| loopback adapter (Capture driver) | `Fail` when missing | `Warn` when missing (standalone doctor) | #69 |
| adapters (Interfaces) | `Warn` "no interfaces were found" | when live backend absent, reworded to name the missing backend as the cause | #63 |

`Report::exit()` is unchanged in logic (FAILURE if any `Fail`, else SUCCESS): the
new `live` check reaching `Fail` naturally drives the "not ready" verdict, and
the downgraded loopback `Warn` naturally stops forcing it. No special-casing.

## Structured event (`Event` in `crates/fragcap-cli/src/events.rs`)

The `--json` stream is the section 17.5 NDJSON form (`{"ts","event",...}`). New
event variants extend the existing enum; the timestamp/envelope and escaping are
the existing `render()` machinery.

| Event | Fields | Purpose | Issue |
| --- | --- | --- | --- |
| `diagnostic` (new) | `code`, `path`, `line`, `col`, `message` | One per profile-validation diagnostic, replacing the collapsed single-string `error`. | #65 |
| `summary` (new) | at minimum a `diagnostics` count and an ok/failed indicator | Terminal record after validation so a consumer distinguishes "clean" from "no output". | #65 |
| profile-list counts (new) | e.g. `event:"profiles"` with `bundled`, and per-directory / `user_total` counts | `profile list --json` structured output. | #65 |

Exact field names are finalized in `contracts/profile-json.md`. The human-mode
formatter already computes these fields; the change preserves them as structured
data instead of pre-rendering to a string.

## Exit contract (`CliError`/`Exit` in `crates/fragcap-cli/src/exit.rs`)

No new type. Per research R1 the `From<ResolveError>` mapping reclassifies
`ResolveError::InvalidReference` from `Usage` (2) to `Failure` (1), so any
reference that resolves to no profile (absent slug via `NotFound`, or
unresolvable path-shaped via `InvalidReference`) is exit 1 uniformly, while
`Load { LoadError::Invalid }` (an invalid profile *file*) stays `Usage` (2). See
`contracts/exit-codes.md`. The 0/1/2 classes themselves are unchanged (master
spec section 17.4).

## Elevation state (`crates/fragcap-cli/src/assemble.rs`, Windows-only)

Not a stored entity: a boolean precondition read at assembly time from the
current process token via the existing `is_elevated()` (`doctor/probe.rs`). When
false and the command opens the live capture path, assembly returns a
`CliError::failure(...)` (exit 1) with the elevation message before the driver is
touched. No handle to any other process is created (P-1).
