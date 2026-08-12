# Contract: `doctor` readiness report

Covers #63, #69, #70.2. `doctor` is pure: `probe.rs` gathers an `Inputs`,
`checks.rs` classifies it into a `Report`, `mod.rs` renders and computes the
exit. This contract fixes the report's content; the classifier stays a pure
function so it is unit-tested off-Windows.

## Capability backend lines (#63)

Two new first-class checks in the "Capture driver" section, populated from
compile-time feature presence via `#[cfg(feature = ...)]` in `probe.rs`,
mirroring the existing `tracing_availability()` -> `etw_available` pattern:

| Check | Compiled in | Not compiled in |
| --- | --- | --- |
| live capture backend | `Ok` ("compiled in" / present) | `Fail` (blocking) with remediation: install/obtain a build with the `live` feature |
| socket-table backend | `Ok` | `Warn` (non-blocking): attribution degraded; ETW may still attribute |

- Absent `live` -> `Report::exit()` is FAILURE (exit 1) and the human verdict is
  "Not ready". This flows through the existing `exit()` logic (any `Fail`); no
  special-casing.
- Absent `socket-table` -> `Warn`, does not block.

## Interfaces reword when live absent (#63)

- Today an empty interface set always renders `Warn` "no interfaces were found".
- When the `live` backend is absent, the empty set is a *consequence* of the
  missing backend (enumeration belongs to the capture backend, which is not
  linked). The message MUST name the missing backend as the cause rather than
  implying an npcap/adapter fault. Implementation: the interfaces classifier
  consults `live_available`; when `None`, it emits the backend-pointing message
  (or is suppressed in favor of the live-backend `Fail` line - the contract
  requires only that the operator is not sent chasing adapters).

## Loopback severity (#69)

- `loopback` check: missing loopback support becomes `Warn` (was `Fail`) in
  standalone `doctor`, keeping the existing remediation text ("reinstall npcap
  with the Support loopback traffic option").
- Rationale: loopback capture is only needed with `--loopback`; its absence does
  not block ordinary game capture and must not force "not ready".
- The `run`/`extcap` path that carries `--loopback` still treats loopback
  absence as blocking on that path (out of scope for standalone `doctor` here,
  but the severity split is noted so a later slice does not re-escalate the
  doctor line).

## npcap version (#70.2)

- The probe currently hardcodes `NpcapInfo.version = "installed"`.
- It reads the real npcap version from the already-located `wpcap.dll`
  FileVersion resource (research R2: `GetFileVersionInfoSizeW` /
  `GetFileVersionInfoW` / `VerQueryValueW`, gated by the `Win32_Storage_FileSystem`
  windows-sys feature, no new crate), with no process handle and no elevation
  (P-1 safe). Registry read remains a documented fallback.
- On success the existing `format!("version {}", info.version)` renders the real
  version. On failure it falls back so the line does not claim a version it does
  not have (reword the fallback away from "version installed").

## JSON

- `doctor --json` keeps its existing per-check record shape (it is not the
  section 17.5 event stream). The new capability checks serialize through the
  same `render_json` path with no new machinery.
- Adding a terminal readiness/summary record to `doctor --json` is noted as an
  adjacent gap (see #65 "related") but is **not** required by this slice; if
  trivially available it may be added, otherwise deferred.

## Acceptance

- Unit tests in `checks.rs` (extend the existing suite): absent-live -> `Fail` +
  `Report::exit() == FAILURE`; absent-socket-table -> `Warn` + still ready;
  loopback-absent -> `Warn` + `report.ready()`; live-absent interfaces message
  names the backend.
- A featured Windows smoke run shows both backend lines present and a real npcap
  version.
