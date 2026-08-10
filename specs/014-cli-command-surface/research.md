# Research: CLI Command Surface

Phase 0 decisions. Each records what was chosen, why, and the alternatives
weighed against the constitution, the architecture of record, and the slice
scope.

## D-a. Argument parser: clap 4 (derive), on fragcap-cli only

**Decision**: use `clap` 4 with the `derive` feature, declared on `fragcap-cli`
alone.

**Rationale**: specification section 17.2 fixes a grammar of short and long flags,
per-command defaults, repeatable options, subcommands, and `-h`/`-V`. `clap`
derive produces exactly that from typed structs, including the help text, at a
fraction of the code a hand-rolled parser needs. The crate sits at the top of the
dependency graph: nothing depends on `fragcap-cli` (P-3, checked by `cargo xtask
deps`), so a large graph there never reaches core or the mid-level crates. MSRV
1.82 is satisfied by clap 4.x (verified in the `msrv` gate).

**Alternatives considered**: `lexopt`/`pico-args` (tiny, but the help text the
spec prints verbatim would be hand-maintained and drift from the flags); a
hand-rolled `std::env::args` parser (zero deps but reimplements subcommand
dispatch, defaults, and help formatting, the "harder half written anyway"
pattern the project rejects); `argh` (derive, smaller, but weaker on the
repeatable-option and subcommand-help shapes section 17.2 needs).

## D-b. Interrupt handling: ctrlc, on fragcap-cli only

**Decision**: use `ctrlc` to install a console-interrupt handler that sets an
`AtomicBool` the `SessionDriver` observes.

**Rationale**: section 17.4 makes an operator interrupt during capture an exit-0
success (the capture completes and the output is valid). Reaching that outcome
requires catching the interrupt and driving the session to a clean
`StopReason::Interrupt`, not letting the process be killed. The standard library
has no portable console-interrupt hook. `ctrlc` is small, widely used, uses
`SetConsoleCtrlHandler` on Windows, and is MSRV-friendly.

**Alternatives considered**: a raw `SetConsoleCtrlHandler` via `windows-sys` in
the CLI (adds a platform binding and `unsafe` to a crate that otherwise has
neither, for no gain over `ctrlc`); ignoring interrupts (fails section 17.4).

## D-c. Session and pipeline compose side by side, not nested

**Decision**: the `Pipeline` runs on its own thread owning the packet stream and
the shared attributor; a `SessionDriver` thread owns the `CaptureSession`. They
connect through a `StopHandle` and a published binding snapshot, not by routing
packets through the session.

**Rationale**: `Pipeline::run` owns the bounded drop-oldest buffer and the
per-interface capture threads and never surfaces individual packets; routing every
captured packet back out to `CaptureSession::on_packet` would fight that design
and reintroduce a per-packet cross-thread hop the pipeline exists to avoid. The
session is a pure control brain (process events, stage matching, stop conditions);
it belongs beside the pipeline, driving it, not inside its packet path. This is
also what keeps both testable offline: the pipeline over `ReplaySource` +
`ScriptedAttributor`, the session over `ScriptedWatcher`.

**Alternatives considered**: a single thread that pumps packets through the
session before the sinks (serializes the acquisition path and duplicates the
buffer); extending `Pipeline` to expose a per-packet callback (widens a stable
core API for a CLI concern and risks the P-3 seam).

## D-d. Role and stage stamping via a FlowAttributor decorator in the facade

**Decision**: add `RoleStampingAttributor` to the facade `session` module, a
`FlowAttributor` that wraps the real attributor, holds a published `pid ->
(role, stage)` snapshot sourced from the session's bindings, and applies
`Attribution::with_role`/`with_stage` after the inner `resolve`. Add
`CaptureSession::role_bindings()` so the orchestrator can build the snapshot.

**Rationale**: `Attribution` already carries `role: Option<Arc<str>>` and `stage:
Option<StageId>` with builder methods, so stamping populates existing fields, it
does not change a type. A decorator is still just a `FlowAttributor` with no
packet acquisition, so P-3 holds. The facade `session` module is the correct home:
it already bridges session and attribution (both above the sibling crates), and
`fragcap-attr` must not learn about profiles or stages. The snapshot is published
by swapping an `Arc` under a short-held lock on rare writes (a process start or
exit), while per-packet reads clone the inner `Arc`; this needs no new dependency
(`arc-swap` is not pulled into the facade for this).

**Alternatives considered**: mutating `Attribution` inside the pipeline (puts
profile knowledge in core); a new trait method on `FlowAttributor` (widens the
seam for one consumer); adding `arc-swap` to the facade (unneeded for a
rare-write path).

## D-e. Volume bound via a counting tee sink feeding the session

**Decision**: insert a `TeeCountingSink` (an ordinary `Sink`) first in the sink
list; it forwards a retained-packet length to the `SessionDriver` over a channel,
which calls `CaptureSession::on_packet(len)`. The session remains the authority
for `VolumeReached` and its `SessionStats` retained counters.

**Rationale**: because the session no longer sees packets (D-c), the byte and
packet bounds and the session's retained counts need a feed. A tee sink keeps that
feed inside the pipeline's conservation identity (it is just another sink whose
receipts are counted), so nothing it observes escapes P-4 accounting, and the
session stays the single place the six stop conditions live. The completion
summary then reads real counters from both the `PipelineReport` and the session.

**Alternatives considered**: enforcing bounds in the orchestrator tick loop over
raw atomics (leaves `SessionStats.retained` at zero and splits the stop logic
across two places); polling pipeline stats mid-run (the pipeline returns stats
only at the end).

## D-f. doctor is a pure Inputs -> Report classifier over a thin probe

**Decision**: model `doctor` as a pure classifier: an injected `Inputs` struct of
raw environment facts, a set of `fn(&Inputs) -> Check` classifiers producing a
`Report`, and a `Report::exit`. A thin `cfg(windows)`/feature-gated `probe::gather`
fills `Inputs` from the real machine and is not unit-tested.

**Rationale**: this makes every classification and the exit decision testable with
hand-built inputs and goldens, on any target, which is the only way to cover the
matrix section 26.3 implies (npcap present/absent, each option present/absent,
elevated or not, interfaces up/down) without the environment. It mirrors the
project's standing pattern of pushing platform facts to a thin edge and testing
the decision.

**Tracing severity (clarified)**: a missing process-event session is a blocking
fail only when the session is elevated and the session cannot open (attribution is
then degraded); when the tracing capability is not built in, the check is a
non-blocking skip. **npcap options (Licensing rule, clarified)**: loopback capture
support and WinPcap API compatibility mode are separate checks, each naming its
own remediation when absent; `doctor` only detects and never installs.

**Alternatives considered**: a monolithic `doctor` that queries and prints inline
(untestable without the environment, exactly the coverage gap this split closes).

## D-g. Hand-rolled NDJSON events; stream routing without color

**Decision**: define an `Event` enum and emit newline-delimited JSON by hand over
the sink crate's JSON string escaper (promoted to `pub`); route all progress and
events to standard error and capture data to sinks; implement quiet and silent by
gating the human emitter; defer color.

**Rationale**: `serde_json` is dev-only by policy, and the event set is small and
fixed, so hand-rolling over the existing escaper reuses one escaper rather than
adding a fourth, and keeps serde out of the runtime graph. Section 17.6's rule
(diagnostics move to stderr when a sink writes to stdout) is satisfied by always
emitting on stderr. Color is cosmetic; `IsTerminal` gating can add it later
without changing the stream contract, so it is out of scope here. Timestamps are
RFC3339 `Z`, formatted from `SystemTime` by hand (no date crate).

**Alternatives considered**: adding `serde_json` to runtime deps (changes the
test-independence argument the sink crate relies on); adding a color crate
(surface for a cosmetic this slice does not need); a date crate for timestamps
(one small UTC formatter suffices).

## D-h. Size-literal grammar in fragcap-core::size, binary units

**Decision**: add a pure `fragcap-core::size` module parsing an integer plus a
required unit (`b`, `kb`, `mb`, `gb`), binary (1024-based), rejecting zero and a
missing or unknown unit; use it for `--max-bytes` and the size form of `--ring`.

**Rationale**: it mirrors the existing `duration` module (integer plus required
unit, reject zero) so the two literal grammars are consistent, and living in core
(std only, allowlist unchanged) lets the S16 ring slice reuse it rather than
reimplement a size parser in the CLI. Binary units match capture-buffer sizing
conventions.

**Alternatives considered**: a CLI-local size parser (S16 would duplicate it);
decimal (1000-based) units (inconsistent with how buffer and file sizes are
reasoned about); optional unit defaulting to bytes (ambiguous, and `duration`
already sets the required-unit precedent).
