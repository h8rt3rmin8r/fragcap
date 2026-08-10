# Agent guide (provider-agnostic)

This is the canonical, agent-neutral instruction file for this repository. Any
coding agent that reads `AGENTS.md` (Codex, Cursor, opencode, and others)
should treat this as the source of truth. Claude Code reads it through
`CLAUDE.md`, which imports this file.

These instructions OVERRIDE any default behavior. Follow them exactly.

## What fragcap is

fragcap is a passive, process-attributed network capture tool for Windows,
written in Rust. Packet capture is a solved problem; attribution is not.
Standard tooling captures at the network driver, below the socket layer, where
the association between a packet and the process that produced it has already
been discarded. fragcap reconstructs that association for game clients launched
indirectly through platform and publisher launchers, and writes it into an
extended pcapng profile that unmodified analyzers still read as ordinary
pcapng.

It observes. It does not modify traffic, and it does not reach inside the
processes it names. That distinction is the whole security posture, and
principle P-1 makes it absolute.

## Reference documents

Read these before acting. They are ordered by authority.

- **Constitution** (governing principles, versioned):
  `.specify/memory/constitution.md`
- **Master specification** (architecture of record):
  `docs/fragcap-specification.md`. Every feature traces to it. Section
  references in the constitution and in slice specs point here.
- **Specification outline** (a map of the above, useful for navigation):
  `docs/fragcap-spec-outline.md`
- **Slice ordering and dependencies**: `docs/plans/README.md`
- **Repository mechanical rules**: `CONVENTIONS.md`
- **Contributor workflow**: `CONTRIBUTING.md`
- **Active feature directory**: recorded in `.specify/feature.json`
  (`feature_directory`). Read that feature's `plan.md` before implementing; it
  carries the technologies, project structure, shell commands, and
  slice-specific context for the current work.

## Current state

Slices S01 through S11 are complete. The Cargo workspace exists with the eight
crates from the architecture of record, a task runner carrying the repository's
own checks, and six workflow files. `fragcap-core` carries the type and trait
vocabulary from specification sections 8.4 and 8.5, a `parse` module
implementing sections 12.5 and 12.6, a `pipeline` module implementing
sections 8.6 and 12.4, a `duration` module carrying the literal grammar three
later slices share, and, since S11, a `process` module carrying the process tree
of section 10. `fragcap-profile` carries section 15 in full: the schema, the
validation set, and the resolution order. `fragcap-capture` reads classic pcap
and replays it as a `PacketSource`. `fragcap-attr` answers attribution from a
declared script and, since S10, from the operating system socket table, and
since S11 watches process start and exit through Event Tracing for Windows.
`fragcap-sink` writes both output formats: pcapng carrying attribution in packet
comments, and JSON Lines. `fixtures/` holds the committed corpus of section 25.3
and, since S06, a golden per fixture per format.

**fragcap attributes flows to processes.** S10 filled in specification section
11. `SocketTableAttributor` snapshots the socket table, joins captured flows
against it by 5-tuple, keeps a closing connection's tail attributed through a
thirty second retention window, and publishes each snapshot as an immutable
value every capture thread reads without locking. Every attributor before it
answered from a text file a test wrote.

**The join's order is total, and that is load-bearing rather than tidy.**
Competing entries rank by exactness, then by the latest socket creation instant
at or before the packet, then by a declared tiebreak whose only job is to make
the order total. An implementation that iterates the platform's rows and takes
the first hit passes an ordinary test and produces answers that change between
runs over identical traffic; the permutation test in `index.rs` is what fails
it. Do not replace that test with one that resolves a single unambiguous flow.

**A socket created after a packet cannot have owned it**, and that filter is
the only mechanism available that distinguishes the previous owner of a reused
port from the current one. Both tables are therefore read by owning module
rather than by owning process identifier, which is the class that carries a
creation instant. Appendix D attributes the timestamp to TCP alone; it is on
both, and the correction is in the S10 decisions fragment. It matters more for
UDP, whose key is the local endpoint alone.

**A retained answer is marked, and the window's origin is exact.** Retention
runs from the instant an endpoint was last observed present in a table, not
from the refresh that noticed it gone; those differ by up to one interval, and
measuring from the later one would make thirty seconds silently thirty-one. A
retained answer can be wrong in exactly one way, a port reassigned inside the
window, which is why `Fidelity::Retained` exists and why widening the window
quietly is a P-9 problem rather than a tuning question.

**The pipeline no longer locks per packet.** S08 held the attributor behind a
mutex and said in `pipeline/mod.rs` that the lock was not the destination; S10
is where it went. `FlowAttributor` gained `Sync` through the deviation process,
the pipeline holds `Arc<dyn FlowAttributor>`, and `Pipeline::new` kept its
`Box` parameter because `Arc<dyn T>` is constructible from one, so no caller
changed.

**The socket table backend has actually run**, which the live capture source of
S09 still has not. A real socket was opened, found in this machine's real
socket table, attributed to the process that opened it, then closed and
observed to survive as a retained attribution. That was possible because the
backend needs no capture driver and no elevation: the IP Helper API ships with
the operating system. Its feature is `socket-table` and deliberately not
`live`, and folding the two together would make attribution unavailable to
anyone without an npcap software development kit it never calls.

**Nothing in this project opens a process handle.** Image names come from a
toolhelp enumeration, which returns them in the snapshot. `OpenProcess` with
query-limited rights would also satisfy P-1, and the point of choosing
otherwise is that a handle request is a thing a reviewer has to check while
having no handle is a thing `cargo xtask lint` can assert. It now does, for
`OpenProcess`, `ReadProcessMemory`, and `WriteProcessMemory`, case
insensitively.

**The section 25.1 claim is now demonstrated rather than asserted.**
`crates/fragcap/tests/pipeline.rs` reads a fixture, parses every packet, and
resolves every flow, with no capture driver, no elevated privilege, and no game.
Every slice from here is testable the day it is written.

**There is a pipeline, and the loss counters carry real values.** S08 composes
a source, the parser, an attributor, and any number of sinks across two threads
with a bounded drop-oldest buffer between them, and
`crates/fragcap/tests/corpus_pipeline.rs` runs the whole corpus through it with
both writers attached, reproducing the committed goldens. The
`CaptureStats` a writer receives is now the run's own rather than a snapshot a
test composed by hand.

The assertion that matters is conservation, not reachability: for every sink,
received plus `buffer_dropped` plus refusals equals `packets_captured`. It is
checked in every pipeline test, and a discard path added later with no counter
fails there rather than passing quietly. Prefer extending it over adding a
counter-specific test.

**A failed sink is retired, not fatal.** Every packet after the failure
advances `sink_dropped` for that sink, which is what section 12.4 already
defines the counter as; the run ends only when every sink has retired. This
reverses the first answer the slice wrote down, and the reasoning for the
reversal is in the S08 decisions fragment. Do not relitigate it without reading
that first.

**The pipeline runs one capture thread per interface.** S09 added `Send` to
`PacketSource` through the deviation process, which is what S08 predicted it
would have to. All capture threads feed the single bounded buffer of section
12.4; there is no second buffer and no multiplexing source, and a proposal to
add either should read the S09 clarification session first.

**Selection is a pure decision over a value, and that is load-bearing.**
`fragcap-core::interface::select` takes an inventory and returns the chosen
interfaces plus a named reason for every interface it passed over. It opens
nothing, so the whole section 12.1 precedence is tested on any machine. The
accounting invariant, that chosen plus passed-over equals the inventory, is
asserted for every case: capturing on the wrong interface produces a run that
exits zero and contains nothing, which is invisible unless the decision is
reported.

**Both writers now record more than one interface.** S06's blanket refusal of a
second is replaced by the narrower rule that was actually needed: every
interface must be declared before the first packet, because section 13.3 settles
the annotation `iface` key from the interface count and a written block cannot
be revised. A single-interface capture is byte-identical to what S06 and S07
produced, checked against the committed goldens.

Note that the two writers differ on the single-interface case and the difference
is deliberate: pcapng omits the `iface` key, JSON Lines always writes it. S09's
specification initially claimed both omit it, and the goldens caught that
during implementation.

**Loss accounting is per-interface where the cause is.** `CaptureStats` holds
one backend report per interface and computes the capture-wide view, so a kernel
drop names the driver buffer that is undersized. `buffer_dropped` and
`sink_dropped` stay capture-wide, because the buffer and the sinks are.

**A retired interface is not a lost packet.** A capture thread that fails
retires its interface, the run continues on the others, and the report names the
interface and the reason. It advances no drop counter: nothing was observed and
then discarded, and counting it as loss would report packets that were never
observed as packets that were thrown away.

**A profile cannot exist unvalidated, and being wrong well is the deliverable.**
S05 filled in `fragcap-profile`. `Profile::parse` returns either a validated
profile or every diagnostic found, and there is no other constructor, so section
15.4's requirement that validation run before every capture cannot be forgotten
by a later caller. Nothing on a diagnostic path uses `?`: a profile with four
mistakes reports four, which is what section 15.4 asks for and what an author
working against a game update needs.

Two of its checks exist because the failures they catch are invisible. A stage
bound to the wrong process among several sharing an image name, and a
`capture.roles` entry naming a role nothing declares, both produce a run that
exits zero, writes a well-formed file, and contains no gameplay. That is the
configuration-side form of the loss P-4 forbids: every packet lost, none
counted. The ambiguity decision is therefore exact rather than approximate, by a
reachability walk over the two patterns. Three further checks were added beyond
the section 15.4 list for the same reason and are recorded in the S05 decisions
fragment as candidates for promotion.

**Unknown keys in a profile are refused rather than ignored**, and the schema
version is what makes that safe. An author who writes `payloads = false`
intending `payload = false` is told, rather than handed a capture containing
contents they meant to exclude. Do not relax this to be helpful; read the S05
decisions fragment first.

**A packet with no flow key advances neither attribution counter.** Never
attempted is not attempted and failed, and `AttributionState` has kept the two
apart since S02. S07's corpus helper conflated them and put a wrong count into
the `malformed` golden, which stood for a whole slice because the goldens were
self-consistent and the definition lived in another crate. S08 found it by
driving the same writers from the pipeline. Corrected in both.

Attribution fidelity is carried on `Attribution`, not derived from whether an
attribution exists. S06 initially derived it and review caught that every
golden was claiming a live socket-table hit for a resolution that came from a
text file.

Two placements are load-bearing and worth not relitigating. The parser lives in
`fragcap-core` rather than `fragcap-capture`, because the capture thread that
calls it belongs to the pipeline, which specification section 8.2 places in
core; the other way round would invert section 8.3. The end-to-end test lives in
the `fragcap` facade rather than in either backend crate, because the facade is
the only crate that legitimately depends on both, and a dev-dependency between
capture and attribution would create exactly the edge P-3 exists to prevent
while slipping past `cargo xtask deps` unnoticed.

**Fixtures are generated, not hand-made.** The generator in
`crates/fragcap-capture/tests/corpus.rs` is the readable record of what each
one contains, and a drift check in the ordinary gate fails if a committed file
stops matching it. Regenerate with `FRAGCAP_UPDATE_FIXTURES=1 cargo test -p
fragcap-capture --test corpus`, then read the diff. See `fixtures/README.md`.

**Dependency inventory.** The workspace has three runtime dependencies, one
optional runtime dependency, and one dev-dependency, and the distinction is load-bearing rather than bookkeeping.

| Crate | Kind | Added by | Why |
| --- | --- | --- | --- |
| `bytes` | runtime | S02 | Reference-counted payload clones |
| `pcap` | runtime, optional | S09 | The capture driver binding, behind the `live` feature |
| `toml-span` | runtime | S05 | Profile parsing with byte spans on every value |
| `regex` | runtime | S05 | Compiles the `path_regex` match predicate |
| `arc-swap` | runtime | S10 | Lock-free publication of the attribution snapshot |
| `windows-sys` | runtime, optional | S10 | The IP Helper socket table, behind the `socket-table` feature |
| `serde_json` | dev only | S07 | Parses every line the JSON writer emits, in tests |

S03, S04, S06, and S08 added none. The parser is arithmetic over a byte slice, a
pcap file is a header and a run of records, the attribution script format is
deliberately trivial, and pcapng is length-prefixed binary over a byte sink.

S08 is the one worth spelling out, because a concurrency crate is the obvious
reach and it would not have helped. Section 12.4 needs bounded, drop-oldest, and
a producer that never waits, together. The standard library's channels are
either unbounded or blocking, and their non-blocking form fails rather than
evicting, which is drop-newest. A third-party bounded channel has the same two
shapes and would still leave the eviction to be written by hand. The buffer is
therefore a `VecDeque` behind a `Mutex` and a `Condvar`, and a proposal to add
`crossbeam` or an async runtime here should say which of those three properties
it thinks the dependency supplies.

S05 is the other one worth spelling out, because it added two and the obvious
choice was not available. `toml` declares Rust 1.85 against this workspace's
1.82 minimum, and pinning it to `~1.0` does not help: `toml_parser` resolves to
1.1.3 underneath and declares 1.85 too. `toml-span` declares 1.70, brings one
transitive crate, has no serde in its graph, and carries the byte spans the
diagnostics are built on. A serde-derived deserializer was never available on
its own terms either: it returns the first error and stops, and section 15.4
requires every problem in one report. `regex` is taken with default features off
because the engine that validates a `path_regex` must be the engine that
evaluates it in S12, and because `aho-corasick` and `memchr` accelerate scanning
large haystacks while a haystack here is one image path. `regex-lite` was
rejected for reduced Unicode support against paths that can carry non-ASCII.

The `exe` glob matcher stays hand-rolled despite both, and the pairing only
looks inconsistent: section 15.4 needs glob intersection, every glob crate
answers glob matching, and a dependency would leave the harder half to be
written anyway. A proposal to replace it should say how it decides whether two
patterns can match one name.

`toml-span` does not implement TOML datetimes, which its own documentation
states and which the S05 analyze gate caught contradicting that slice's first
requirement. No key in schema version 1 has a datetime type, so the divergence
is confined to profiles that are invalid anyway; it is pinned by a test rather
than left in prose.

S09 is the third worth spelling out, and it broke the project's usual pattern of
adding nothing. The alternative to a dependency here is not arithmetic over a
byte slice, as it was in S03 and S06, but a C ABI whose struct layouts must be
transcribed by hand with nothing checking them against the header. A wrong
offset in the packet header yields plausible timestamps that are wrong, which is
the P-9 failure no test over synthetic data catches. `pcap` is MIT or Apache-2.0
across its whole graph and declares Rust 1.64.

Two things about it are worth keeping in working memory. **`libloading` is
pinned to the 0.8 line by `pcap`, and `libloading` 0.9 declares Rust 1.88**, so
taking it directly would break `cargo xtask msrv` in a check most contributors
cannot run locally. And **`pcap` can transmit, and fragcap never does**: `cargo
xtask lint` fails if any fragcap source names a transmit call, so the P-1
argument is mechanical rather than remembered.

S10 added two and both need their argument kept.

**`arc-swap` supplies one property: a reader that a writer cannot block.**
Specification section 11.6 requires the capture thread to read the current
attribution snapshot without locking while the control thread replaces it. The
tempting alternative is `RwLock<Arc<Index>>`, and it is a lock: a reader can
block behind a writer, and the reader here is the acquisition path section 11.6
exists to keep unblocked. It would satisfy a test and not the requirement,
which is worse than failing both because it looks like the requirement was met.
A hand-rolled `AtomicPtr` is correct and needs a reclamation scheme in `unsafe`,
in a workspace that has none outside a platform binding. A proposal to drop it
should answer whether a reader may be blocked at all, not whether a read lock is
fast enough.

Note that it adds **two** packages, not one: it has a build dependency on
`rustversion`. The planning research said one, from reading an empty
`[dependencies]` table without looking at `[build-dependencies]`. Recorded
because an audit that makes that mistake under-reports every proc macro in the
graph.

**`windows-sys` is pinned to 0.36 because `pcap` already resolves it there**, so
it adds no package to `Cargo.lock` at all. Taking the current line would put a
second complete `windows-sys` tree in the graph for declarations that have not
changed. If `pcap` later requires a newer line the graph gains a second copy,
which is Cargo working correctly rather than a defect.

Two further things about it. It is **unrelated to npcap**: the IP Helper API
ships with the operating system, which is why its feature is `socket-table`
rather than `live` and why that backend runs on a machine with no capture
driver. And the alternative to it is the same C ABI transcription S09 rejected:
a wrong offset in `MIB_TCPROW_OWNER_MODULE` yields a plausible process
identifier that is wrong.

S07's writer is hand-rolled and its `serde_json` is test-only on purpose:
verification is worth more the less it shares with what it verifies. Anything
proposing to move it into `[dependencies]` is changing that argument and should
say so.

`fragcap-core` may depend only on crates named in the allowlist in
`xtask/src/deps.rs`, which is checked mechanically. Note that `cargo xtask
deps` ignores `[dev-dependencies]` by design, so a dev-dependency on a sibling
crate would pass the gate; S06 and S07 both keep their corpus tests in the
`fragcap` facade for that reason.

The remote is `origin`, at `https://github.com/h8rt3rmin8r/fragcap`. S01
integrated through pull request #1.

Two things are scaffolded but not exercised, and must not be reported as
passing checks:

- **Two of the four workflows that had never completed now have.** The first
  runs landed during the GitHub incident of 2026-08-06, and `minimum supported
  toolchain`, `core builds without a capture backend`, `platform`, and `audit`
  never acquired a runner, so they were red for that reason rather than a code
  reason. On pull request 10 the first two ran and passed. `platform` and
  `audit` still have not: `audit` is weekly and dispatch-only, and `platform`
  did not trigger on that pull request. Neither should be treated as green
  until watched. S09 gave `platform` real triggers, which makes the next pull
  request the first that can turn it green, and the first that can turn it red
  for a real reason.
- **The minimum-toolchain check now runs for real.** Until S02 it built with
  the pinned toolchain and reported success, which said nothing about the
  declared minimum. It now builds through `rustup run 1.82` and exits 2 when
  that toolchain is absent, so a check that did not run can no longer look like
  one that passed.
- **The npcap SDK acquisition step has now run, and the live source links.**
  Both were first exercised on pull request 12, watched to completion. What that
  proves is that the kit is acquired at build time and that
  `fragcap-capture --features live` compiles and links against `wpcap.lib`.
- **Live capture has still never executed.** The kit supplies the import
  library; `wpcap.dll` ships with the npcap driver, and a binary linked against
  `wpcap.lib` will not start without it. A runner with no npcap installed exits
  with STATUS_DLL_NOT_FOUND before `main`, which is how S09 found this. Tier 2
  tests therefore do not run in continuous integration today, and the workflow
  says so rather than appearing green over nothing. Installing npcap on a runner
  is a licensing decision for the operator.
- **`cargo deny` has never run.** The `audit` workflow owns it and is weekly
  and dispatch-only. S09 added the first dependency with a platform surface and
  a transitive graph worth checking, so the check now has a real subject. Its
  licenses were verified by hand against the allowlist; nobody has watched
  `cargo deny` confirm it. S10 added two more, verified the same way.
- **The socket table backend has run, and it is the exception on this list.**
  S10's tier 2 tests were executed to completion on a Windows developer machine:
  a real socket opened, found in the real socket table, attributed to the
  process that opened it, and then closed and observed as a retained
  attribution. That is a stronger claim than anything else here, and it was
  cheap for one reason worth remembering: the backend needs no capture driver
  and no elevation, so there is no external dependency between the test and the
  machine. Its workflow step is likewise the first in `platform.yml` that can go
  green on a bare runner. **This says nothing about live capture**, which
  remains unexecuted for the reasons above.

## Spec-driven development workflow (spec-kit)

Every slice MUST be spec'd through the spec-kit framework before
implementation. The slice ordering document scopes a slice but never
substitutes for its spec.

The engine is shared and agent-neutral; drive it the same way regardless of
which agent you are:

- Templates: `.specify/templates/` (`spec-template.md`, `plan-template.md`,
  `tasks-template.md`, `checklist-template.md`, `constitution-template.md`)
- Scripts: `.specify/scripts/bash/` (`create-new-feature.sh`, `setup-plan.sh`,
  `setup-tasks.sh`, `check-prerequisites.sh`, `common.sh`)
- Workflow registry: `.specify/workflows/workflow-registry.json`
- Constitution (the gate every phase checks against):
  `.specify/memory/constitution.md`

The full cycle, run end to end per slice:

1. **specify** - create or update the feature spec from the slice intent.
2. **clarify** - resolve underspecified areas; encode answers back into the
   spec.
3. **checklist** - generate a slice-appropriate quality checklist.
4. **plan** - produce design artifacts into the feature directory.
5. **tasks** - generate a dependency-ordered `tasks.md`.
6. **analyze** - non-destructive cross-artifact consistency check. This gate
   MUST pass and MUST NOT be weakened or skipped.
7. **implement** - execute `tasks.md`.
8. **verify** - run the full gate set in the foreground (see below).
9. **commit** - stage only the slice's files, add a changelog fragment under
   `changelog.d/`, and commit. `.specify/feature.json` is local, gitignored
   state; never stage it.

Agents with native spec-kit command wrappers may invoke those. Four surfaces
are installed and all drive the same `.specify/` engine:

| Agent | Command surface |
| --- | --- |
| Claude Code | `.claude/skills/speckit-*` |
| Codex | `.agents/skills/speckit-*` |
| Cursor | `.cursor/skills/speckit-*` |
| opencode | `.opencode/commands/speckit.*` |

Agents without a wrapper should follow the phases above directly against the
templates and scripts. The result is identical; the wrappers are convenience,
not capability.

Do not re-point or hand-edit `.specify/integration.json` or
`.specify/init-options.json`. Those record the generated command surfaces and
are regenerated by the spec-kit CLI.

## Skills

Portable skill content is vendored in `.agents/skills/` and committed, with
provenance and integrity hashes in `skills-lock.json`. First-party skills
authored for this repository live in `skills/`.

Codex reads `.agents/skills/` directly. Claude Code and Cursor read their own
directories, populated with machine-local symlinks by the skills CLI; those
symlinks are gitignored because they carry absolute paths. Spec-kit's own
generated `speckit-*` skills are tracked in every surface.

A skill is checked against P-1 before it is vendored. A skill that teaches a
denylisted technique does not land here, whatever else it is useful for.

## Non-negotiables

These restate the constitution's sharpest edges. The constitution is
authoritative; this list is the one to keep in working memory.

- **The technique denylist is absolute.** No packet interception drivers, no
  code injection, no function hooking, no process handles carrying memory-read
  rights against a target, no layered service providers, no executable image
  modification. A slice that appears to need one has been scoped wrong; halt
  and raise it.
- **Any process handle states its access rights explicitly at the call site.**
  A request carrying memory rights fails review.
- **`fragcap-core` takes no platform-specific dependency.** Dependencies flow
  concrete toward abstract, and continuous integration proves core builds for
  a target with no capture backend.
- **Every discard path has a named counter.** A dropped packet that is not
  counted and surfaced is a defect.
- **npcap is never bundled, never downloaded, never installed by fragcap, and
  its SDK is never vendored.** Detection only.
- **Compatibility outranks richness.** Output stays readable by unmodified
  analyzers.
- **A new term gets a glossary entry in the same change that introduces it.**
- **Wrappers stay thin.** A wrapper that needs to parse output means a missing
  capability in Rust.
- **Pinned artifacts change only with a dated decision recorded in
  `CHANGELOG.md`:** `.github/workflows/**`, `rust-toolchain.toml`,
  `release.toml`, `scripts/**`, and release documentation. Write the decision
  as a `changelog.d/<key>.decisions.md` fragment; `CHANGELOG.md` is assembled
  from those fragments at release time, and editing it from a feature branch
  conflicts with every other concurrent pull request. `release.toml` now exists
  (added 2026-08-08 with the release automation), so the rule binds it.
- **All text files are UTF-8 without BOM with LF line endings. No em-dashes or
  en-dashes anywhere, including code comments.**

## Verification discipline

Run verification in the foreground and watch it to completion. Never background
it, never infer a result you did not read.

The gate set, all of which `cargo xtask ci` runs in order:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --locked
cargo xtask lint          # repository conventions, CONVENTIONS.md
cargo xtask deps          # dependency direction, specification section 8.3
cargo xtask license       # per-crate license text for registry publication
```

Two further checks are not in `ci` because they need a target or a toolchain
the runner may not have: `cargo xtask neutral` (constitution P-2) and
`cargo xtask msrv`. Both exit 2 rather than 0 when they cannot run.

The documentation linter and the shell wrapper compliance checkers arrive with
the slices that own them.

**Claims require evidence.** Do not report a slice complete, a test passing, or
a defect fixed without having run the command and read its output. If tests
fail, say so and include the output. If a step was skipped, say that. Reporting
an unverified success is worse than reporting a known failure, because it
removes the operator's ability to trust any other report.

## Deciding versus asking

Default to deciding: enumerate the alternatives, evaluate them against the
constitution, the master specification, and the slice scope, pick the best,
proceed, and record the rationale in the slice.

Halt to the operator only when no option is clearly best on an irreversible or
architecture-defining choice, the slice intent is genuinely ambiguous, or a
constitution conflict needs a human call. A P-1 conflict is always a halt.

## Reconnaissance gate

**Closed.** Open questions Q-1 through Q-6 (specification section 29) are
resolved. The findings are recorded in Appendix D and were applied to the
specification; the protocol that produced them is
`docs/plans/reconnaissance.md`.

Slices S09, S10, and S17 were gated on those answers and are now unblocked.
Q-7 and Q-8 remain open and gate S18.

## Integration workflow

Work integrates through pull requests reviewed by the operator
(`@h8rt3rmin8r`). Never push directly to `main`, and never merge your own pull
request. See `CONTRIBUTING.md` for the full workflow.

Never push, tag, cut a release, or publish a crate without explicit
authorization.
