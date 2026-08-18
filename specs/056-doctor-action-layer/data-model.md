# Phase 1 Data Model: doctor gains an action layer (--fix)

The data model is small and lives entirely in `fragcap-cli::doctor`. It extends the
existing `Check`/`Report` types additively and adds the action-layer value types.
Nothing here is persisted; these are in-memory values produced by the pure classifier
and consumed by the `--fix` driver.

## Extended: Check (doctor/mod.rs)

The existing `Check` gains one optional field. Every existing constructor
(`Check::ok/warn/skip/fail`) keeps its signature and default the new field to
`None`, so existing call sites and tests are unmodified.

| Field | Type | Notes |
| --- | --- | --- |
| section | &'static str | unchanged |
| name | &'static str | unchanged |
| detail | String | unchanged |
| status | Status | unchanged |
| remediation | Option<String> | unchanged (human-readable) |
| action | Option<Action> | NEW: the structured, machine-facing remediation |

**Invariant (FR-004)**: a check that carries an `action` also carries a
`remediation` describing the same step; the two are constructed together (a helper
that sets both), so they cannot drift. A check with `action: None` is informational
or has no automatable remedy.

## Extended: Inputs (doctor/mod.rs)

One new field, gathered by the thin probe, so the two new checks stay pure.

| Field | Type | Notes |
| --- | --- | --- |
| ... existing fields ... | | unchanged |
| target_entry_count | Option<usize> | NEW: number of registered target entries read from local.db; `None` when the count could not be determined (never presented as zero, P-9) |

`catalog_db_present: bool` already exists and is reused by the new catalog check.

## New: Action (doctor/action.rs)

The structured counterpart of a remediation string: what `--fix` can do for a check.

| Field | Type | Notes |
| --- | --- | --- |
| kind | ActionKind | which remediation this is |
| label | String | the human sentence `--fix` prints before performing it |
| net_required | bool | true when the primary form needs the `net` capability |
| degraded | bool | true when only the no-network fallback is available in this build |

## New: ActionKind (doctor/action.rs)

A closed enum, one variant per finding in the action catalog. The enum is what binds
an action to its performing code and makes the mapping exhaustive and testable.

- `ObtainNpcap` - npcap absent
- `RelaunchNpcapInstaller` - WinPcap API mode off
- `RelaunchElevated` - not elevated
- `InstallExtcap { scope: ExtcapScope }` - extcap not registered
- `FetchCatalog` - catalog store missing
- `RunDiscovery` - no target entries

`ExtcapScope` is `User | Machine` (the choice the operator makes when confirming the
extcap action), reusing the scope concept already in `ExtcapInstallArgs`.

## New: ActionOutcome (doctor/action.rs)

The honest result of attempting one action (P-9, FR-011).

- `Performed` - the action ran to success
- `Skipped` - the operator declined it
- `Degraded` - a capability-limited fallback ran (e.g. opened the page instead of
  fetching), reported as what actually happened, not as success of the primary form
- `Failed { reason: String }` - the action was attempted and failed; never reported
  as performed

## New: offered-actions selection (doctor/action.rs, pure)

A pure function `offered_actions(report: &Report, caps: Capabilities) -> Vec<Action>`:

- Walks the report in order and collects each check's `action` (skipping `None`).
- Applies capability degradation: when an action's `net_required` is true and
  `caps.net` is false, the returned `Action` is marked `degraded` with a label
  naming the fallback. A degraded `FetchCatalog` is surfaced as guidance, not a
  confirm prompt (FR-016).
- Returns the actions in report order, except that a `RelaunchElevated` action is
  moved to the front when present, so escalation precedes privilege-gated work
  (FR-014).

**Invariant (FR-003, SC-003)**: the output is a subset of the actions carried by
checks in `report`. There is no other source of actions; an `ActionKind` whose check
is absent from the report is never in the output. This is the load-bearing safety
property and is asserted directly.

`Capabilities { net: bool }` is a small value the shell fills from compile-time
features, injected so the selection logic is tested both ways without cfg in the
test.

## New: ActionConfirm seam (the confirm trait)

| Item | Shape | Notes |
| --- | --- | --- |
| trait ActionConfirm | `fn confirm(&self, action: &Action) -> bool` | yes/no for one action |
| ConsoleConfirm | reads stdin yes/no | the real path (mirrors `prompt_socket_holder`) |
| ScriptedConfirm | fixed or scripted answers | tests; drives the confirm loop with no console |
| YesConfirm | always true | the `--yes` pre-confirm implementation |

## New: the --fix driver (doctor/fix.rs)

Not a data type but the orchestration over the above:

1. Gate: refuse on `--json` or non-terminal stdout (exit 2), before anything.
2. Run the classifier, print the report (the existing render).
3. `offered = offered_actions(report, caps)`; if empty, state nothing to fix, exit 0.
4. For each offered action: confirm (via the seam / `--yes`), and on yes perform it,
   recording an `ActionOutcome`; on a RelaunchElevated that is confirmed, hand off
   and stop.
5. Re-run the classifier, print the updated verdict, and return the appropriate exit.

## State transitions

There is no persisted state machine. The only transition is the machine's readiness,
observed twice: the initial report, and the post-action re-run whose verdict reflects
what the confirmed actions changed. An action's lifecycle is
offered -> (confirmed | declined) -> (performed | degraded | failed | skipped),
captured by `ActionOutcome`.
