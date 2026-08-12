A `watch` subcommand captures a target by identity, launch-agnostic (issue #77,
watch mode). It arms the process watcher and sinks and captures the first process
matching an executable name plus an optional path anchor
(`--exe`/`--path`/`--path-regex`), however and wherever it was started, with no
authored profile and no managed launch, which is what makes a modded install
launched from a mod manager, a standalone title, and every non-storefront game
capturable at all. Where `tap` matches an executable name only, `watch` adds the
path anchor that distinguishes a modded install and a `--wait` acquisition
timeout. Watch mode also attaches to a target already running when it starts: the
process watcher's query-only startup snapshot is now folded into the capture
session at arm (a new `CaptureSession::apply_snapshot`), so an already-running
process is acquired without a later start event, and the offline process-script
grammar gains a `snapshot` line so this is tier-1 testable. The S027
runtime-observation provider resolves the identity over the snapshot to report the
honest observed answer naming the already-running process, while the session
remains the single acquisition authority. A watch that never sees its target gives
up at the acquisition timeout with the existing `StopReason::AcquisitionTimeout`
and its discard accounting surfaced (P-4). Watch mode's output is byte-identical
to an equivalent single-stage profile capture. The master specification (sections
7.1 and 10.5) now names watch mode as the default launch-agnostic path, and the
glossary gains a `watch mode` entry. No dependency is added.
