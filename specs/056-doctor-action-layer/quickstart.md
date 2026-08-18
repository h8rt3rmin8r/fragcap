# Quickstart: doctor gains an action layer (--fix)

Validation scenarios that prove the slice works end to end. Tier 1 scenarios run on
any machine with no capture driver, no elevation, and no network. Tier 2 scenarios
need Windows and are demonstrated out of CI, stated not hidden.

## Prerequisites

- The workspace builds: `cargo build -p fragcap-cli`.
- Local dev without MSVC: `cargo +1.96.0-x86_64-pc-windows-gnu {build,test,clippy}
  -p fragcap-cli`. CI runs the MSVC `cargo xtask ci`.

## Tier 1: read-only doctor is unchanged (SC-001)

```
cargo test -p fragcap-cli --test cli_doctor
cargo test -p fragcap-cli doctor::
```

Expected: every existing classifier test passes unmodified. The `doctor` and
`doctor --json` goldens for a ready machine are byte-identical to before the slice.

## Tier 1: refusal rules (SC-004)

- `doctor --fix --json` exits 2 with a usage error and performs no action.
- `doctor --fix` with stdout redirected to a file or pipe exits 2 and performs no
  action (holds with `--yes`).
- `doctor --yes` (no `--fix`) exits 2.

Driven in `cli_doctor.rs` with a non-terminal stdout and asserted on exit code and
"no action taken".

## Tier 1: action selection is a subset of the report (SC-003)

With an injected `Report` and `Capabilities`:

- `offered_actions` returns exactly the actions carried by checks in the report, in
  report order.
- An `ActionKind` whose check is absent from the report never appears.
- When `net` is false, a net-required action appears in its degraded form with a
  label naming the fallback; when `net` is true, the primary form appears.

## Tier 1: the confirm loop with a scripted double (SC-002, SC-006)

With a `ScriptedConfirm`:

- Confirming an action records `Performed` (or `Degraded` for a degraded action) and
  the driver advances.
- Declining an action records `Skipped` and changes nothing.
- After the loop, the classifier is re-run and the updated verdict is printed.
- A failed action records `Failed` and is never reported as performed (P-9).

## Tier 1: the two new checks over hand-built Inputs (FR-019)

- `catalog_db_present = false` yields a warning check carrying a `FetchCatalog`
  action; a ready machine with the catalog present does not (and stays "Ready").
- `target_entry_count = Some(0)` yields a warning check carrying a `RunDiscovery`
  action; `Some(n>0)` does not; `None` is reported as undetermined, never as zero.
- Neither check pushes an otherwise-ready machine to a failing verdict; `doctor`
  (no `--fix`) still exits 0.

## Tier 2 (Windows, out of CI, stated)

- `doctor --fix` on a machine without npcap, in a net-enabled build: confirming the
  npcap action fetches the vendor's signed installer from the official location and
  launches it; nothing is stored in a fragcap artifact.
- `doctor --fix` on a machine without npcap, in a default build: confirming opens the
  official download page.
- `doctor --fix` not elevated: confirming relaunches `doctor --fix` elevated; the
  parent reports the handoff and stops.
- `doctor --fix` with the extcap integration not registered: confirming runs
  `extcap install` at the chosen scope.

## Gate

Before proposing the change:

```
cargo xtask ci
```

which runs fmt, clippy (all features), the workspace tests, the conventions linter,
the dependency-direction check, and the license check. `cargo xtask spec` must pass
for the specification lock-step (P-11). The npcap license determination and the
constitution rule-2 amendment must be present as a `changelog.d/` decisions fragment
(SC-005) and the constitution version bumped to 1.3.0.
