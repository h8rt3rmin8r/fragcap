# Phase 0 Research: Watch / Attach Mode

Decisions taken under autopilot from the constitution, issue #77, the code on
main, and one operator decision where the approved plan's premise had changed on
contact with the code.

## D-0: The approved plan's premise was partly wrong; scope corrected with the operator

The plan framed watch mode as greenfield ("no such flag; Watching is only an
internal state"). The code already: watches on `run` without a managed launch
(`capture_live` arms and folds events until acquisition); implements the
acquisition timeout (`SessionConfig::acquisition_timeout`, `--wait`) with a
dedicated `StopReason::AcquisitionTimeout`; counts watch-time frames
(`PacketDisposition::Discarded`); and captures by a synthesized identity in `tap`
(exe name only). The operator confirmed the corrected scope: a new `watch`
subcommand adding a path anchor and `--wait`, plus the one genuine runtime gap
(attach-to-running), plus docs.

## D-1: The surface is a new `watch` subcommand

**Decision**: `fragcap watch --exe <glob> [--path <substr>] [--path-regex <re>]
[--wait <dur>] [--duration] [--out] [--sink] [--no-payload]`, dispatched like the
other capture commands.

**Alternatives**: extend `tap` (add `--path`/`--wait`); a `--watch` flag on `run`.
Rejected: `run` already watches without a launch, so a flag there is redundant and
confusing; extending `tap` hides the launch-agnostic-by-identity capability behind
a name (`tap`) that reads as a quick exe-name probe. A named `watch` command makes
the default launch-agnostic path discoverable, which is the whole point of the
slice, and keeps `tap` as the simplest form. (Operator decision.)

## D-2: Attach-to-running mechanism, and the ObservationProvider's load-bearing role

**Decision**: The process watcher already takes a P-1-safe toolhelp startup
snapshot (`EtwWatcher::snapshot`/`snapshot_taken_at`; offline
`ProcessScript::with_snapshot` / `ScriptedWatcher::snapshot`), but the capture
path never applies it. Add `CaptureSession::apply_snapshot(records, at)` that
folds the snapshot into the session tree (via `ProcessTree::apply_snapshot_at`)
and runs the same matching `on_process_event` does, so an already-running match
acquires at arm. The orchestrator applies it at arm in both drivers, before the
acquisition loop. The session is the **single acquisition authority**; there is no
competing acquisition path.

The S027 **ObservationProvider** is wired in the `watch` command over a tree built
from the snapshot: it resolves the identity to an `observed` `Target` naming the
already-running process, which the command surfaces (the honest observed answer,
P-9) and which decides that an attach happened. It does not perform the
acquisition (the session does), so the two do not race; it provides the S027
resolver-blessed observed stamp and the `Target`, consistent with the cascade
being the front door.

**Alternatives**: (a) have the ObservationProvider drive acquisition directly;
(b) skip the ObservationProvider and rely only on session snapshot application.
Rejected: (a) would create two acquisition authorities (the provider and the
session's matching) that must agree, a race the S027 permutation discipline warns
against; (b) ignores the operator's explicit choice to route the attach decision
through the S027 cascade and loses the observed stamp. The chosen split keeps one
acquisition authority and one honest observed answer.

## D-3: Fidelity, and why the identity is authored not observed

**Decision**: The synthesized watch identity profile declares `fidelity:
authored` (exactly as `tap`'s does): the operator authored the identity. S027
refuses `observed` on a profile precisely because `observed` is a runtime result,
not a claim an author makes, so the synthesized document cannot and does not claim
it. The `observed` tier belongs to the ObservationProvider's answer about a live
process, a separate axis carried on the `Target`, not on the definition. The two
are never conflated (P-9).

## D-4: Identity is exe plus an optional path anchor, synthesized like `tap`

**Decision**: `watch` synthesizes a one-stage profile
(`{role: target, lifecycle: session, terminal: true, match: {exe, path_contains?,
path_regex?}}`) and validates it through `Profile::parse`, the same validated
construction `tap` uses (no unvalidated construction). At least one predicate is
required, which `Profile::parse` already enforces (`EmptyMatch`); a non-compiling
`path_regex` is refused with the profile's own diagnostic (`InvalidRegex`). No new
matching code and no new identity vocabulary.

## D-5: The acquisition timeout is reused

**Decision**: `watch` exposes `--wait` (which `tap` lacks), mapped onto
`EffectiveConfig::acquisition_timeout` via a new `effective_config_for_watch`. The
give-up is the existing `StopReason::AcquisitionTimeout` with the watch-time
discard accounting; no new counter is added, and the slice confirms it on the
`watch` surface. An unbounded watch (no `--wait`) runs until interrupted, which is
a deliberate, operator-visible choice.

## D-6: The snapshot travels on CaptureComponents

**Decision**: `CaptureComponents` gains the startup snapshot (`Vec<ProcessRecord>`
plus its instant). The offline builder fills it from `ScriptedWatcher::snapshot`,
the live builder from `EtwWatcher::snapshot`/`snapshot_taken_at`. Both drivers
apply it at arm identically, so the offline path exercises the same
attach-to-running code the live path does and the behavior is CI-testable.

## D-7: Testing is offline and complete

**Decision**: All three user stories are tier-1 testable through the hidden
`OfflineArgs` substrate. Attach-to-running uses `ProcessScript::with_snapshot` (a
process present at arm, no later start event). Wait-for-start uses a start event
after arm. The give-up uses a timeline with no match and `--wait`. Output parity
(SC-004) is checked against an equivalent single-stage profile capture. Live ETW
attach-to-running stays tier-2 (not asserted in CI), consistent with the rest of
the live path.

## Constitution re-check after design

No new dependency, no core edge, no process handle (the snapshot is toolhelp
image names and paths), the give-up named and surfaced, the two fidelity axes
kept separate. No violations introduced.
