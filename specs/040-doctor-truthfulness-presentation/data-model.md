# Data Model: doctor truthfulness and presentation

The doctor command is a pure classifier over an `Inputs` value. This slice
extends `Inputs` with identity fields and a three-valued loopback signal, adds a
leading section of checks, and leaves the `Check`/`Report` shapes otherwise
intact.

## Inputs (extended)

Existing fields are unchanged. New and changed fields:

| Field | Type | Source | Notes |
| --- | --- | --- | --- |
| `fragcap_version` | `String` | `env!("CARGO_PKG_VERSION")` in `gather`; fixed literal in test fixtures | Carried on Inputs so goldens do not churn (R-2, D-4) |
| `binary_path` | `Option<PathBuf>` | `std::env::current_exe()` | `None` -> reported as undetermined |
| `profile_dir` | `Option<PathBuf>` | `paths::user_profile_dir()` | Reported regardless of existence (FR-004) |
| `hint_db_path` | `Option<PathBuf>` | `paths::default_hint_db_path()` | Reported regardless of existence |
| `interfaces` | `Vec<IfaceInfo>` | `fragcap::enumerate()` under `cfg(all(feature=live, windows))`, else empty | Was hardcoded empty (bug #102) |

The loopback signal stays in its natural home on `NpcapInfo` but changes type and
source. Implementation refinement (recorded here as a deviation from the initial
"add `Inputs.loopback`" sketch): rather than a separate `Inputs` field, the
existing `NpcapInfo.loopback_adapter: bool` becomes
`NpcapInfo.loopback_supported: Option<bool>`, sourced from
`fragcap::detect_driver().loopback_supported` under
`cfg(all(feature=live, windows))` (else `None`). This keeps the "npcap absent ->
loopback skip" behavior intact and minimizes churn. `NpcapInfo` keeps `version`
and `winpcap_api_mode`.

## IfaceInfo (unchanged shape, now populated)

| Field | Type | Mapped from `InterfaceRecord` |
| --- | --- | --- |
| `name` | `String` | `record.name` |
| `addr` | `Option<String>` | `record.addresses.first().map(to_string)` |
| `up` | `bool` | `record.is_up` |
| `is_virtual` | `bool` | `fragcap::core::virtual_verdict(&record).is_virtual()` |

## Check and Report (unchanged shape)

`Check { section, name, detail, status, remediation }` and
`Report { checks }` are unchanged. New behavior:

- A leading `Identity` section is pushed first in `checks::run`: one `Check::ok`
  row each for version, binary path, profile dir, and hint-db path. An
  unresolvable path renders as an `ok` row whose detail says "undetermined"
  (informational, never blocking; Clarifications).
- The `loopback` classifier maps `Option<bool>`:
  `Some(true)` -> `ok`; `Some(false)` -> `warn`; `None` -> `warn` with the
  "could not be determined" detail.
- The `interfaces` classifier is unchanged in shape but now receives a populated
  vector on a live+windows build; the "no interfaces were found" warning remains
  only for the genuinely empty case, and the live-absent message is retained.

## Loopback state (three-valued)

| Value | Meaning | Doctor rendering | Blocking |
| --- | --- | --- | --- |
| `Some(true)` | loopback adapter present | ok "loopback capture supported" | no |
| `Some(false)` | determined absent | warn (only needed with loopback capture) | no |
| `None` | not determined | warn "loopback support could not be determined" | no |
