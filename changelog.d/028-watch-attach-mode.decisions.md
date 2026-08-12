**2026-08-12** Watch mode (issue #77, slice 2 of 4) landed, and four decisions
were recorded, the first because the approved plan's premise changed on contact
with the code. The plan framed watch mode as greenfield, but the capture path
already watched on `run` without a managed launch, already had the acquisition
timeout and a dedicated `StopReason::AcquisitionTimeout`, and already captured a
synthesized identity in `tap`. The corrected scope, confirmed with the operator,
was a new `watch` subcommand adding a path anchor and `--wait`, the one genuine
runtime gap (attach-to-running), and the docs. First, the surface is a `watch`
subcommand rather than a flag on `run` (which already watches) or an extension of
`tap` (whose name reads as a quick exe probe); a named command makes the default
launch-agnostic path discoverable. Second, attach-to-running: the process watcher
already took a query-only toolhelp startup snapshot but the capture path never
applied it, so a game already running at arm was never acquired. A new
`CaptureSession::apply_snapshot` folds the snapshot and matches it, the
orchestrator applies it at arm for both drivers (empty snapshots are a no-op, so
existing goldens are byte-identical and `run`/`tap` gain the same
attach-to-running consistently), and the offline process-script grammar gained a
`snapshot` line so the path is tier-1 testable. The S027 `ObservationProvider` is
wired in `watch` to resolve the identity over the snapshot and report the honest
observed answer naming the already-running process, while the session stays the
single acquisition authority, so the provider and the session do not race for the
acquisition. Third, fidelity: the synthesized identity is `authored` (the operator
typed it, exactly as `tap`'s is), never `observed` (which S027 refuses on a
profile because `observed` is a runtime result, not an author's claim); the
`observed` tier belongs to the provider's answer about a live process, a separate
axis. Fourth, the acquisition timeout is reused, not reinvented: `watch` exposes
`--wait`, and the give-up is the existing named reason with its discard
accounting, so P-4 is satisfied without a new counter. MSRV stays 1.82; no
dependency added.

Two fixes landed from PR review, both about what the toolhelp startup snapshot
can honestly support. First, the snapshot carries only the executable file name,
never the full path, because reading a running process's path means opening a
handle the no-handle P-1 posture declines. A path anchor is matched against the
full path, so it cannot match an already-running process from the snapshot. The
first cut's test fabricated full paths a toolhelp enumeration never provides,
which hid this; the fix makes the behavior honest rather than silent. A path
anchor now disambiguates a target that starts after arm (the start event supplies
the path), an already-running target is attached by executable name alone, and
where a path anchor cannot be checked against an already-running process whose
executable matches, `watch` warns rather than waiting silently until the
acquisition timeout (P-9). The tests and the spec, glossary, and help were
corrected to reflect the file-name-only snapshot. Second, the snapshot is now
folded parent-first: the tree resolves a record's ancestry against nodes already
folded with no retroactive linking, and toolhelp gives no creation-order
guarantee, so a child listed before its parent left its ancestry unresolved and a
`descends_from` stage never bound when attaching to an already-running multi-stage
chain. A stable topological ordering before folding fixes it, verified by a test
that lists a descended client before its launcher.
