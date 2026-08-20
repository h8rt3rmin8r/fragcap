# Feature Specification: Capture scope and truthful narration

**Feature Branch**: `064-capture-scope`

**Created**: 2026-08-20

**Status**: Implemented

**Input**: Scope written output to the target, count what is excluded, and make
the run say truthfully what it is doing while it does it. Closes issues #184
(P0) and #185.

## Context

fragcap's headline claim is process attribution, and a user reasonably reads
that as "the capture I get back is my target's traffic". The first real
end-to-end run returned a file that was **91.1 percent other processes'
traffic**.

This is a gap against fragcap's own specification, not a feature request.
Section 12.3 says the narrowed kernel filter "is a performance optimization and
never the sole determinant of what is captured. Userspace attribution runs on
every packet regardless of the installed filter, and the capture scope decision
is made there." Section 11.5 says out-of-scope packets "are filtered as a scope
decision, not an attribution failure, and are counted separately." Neither the
filtering nor the counter exists. The kernel half of section 12.2 was built and
works; the userspace half of 12.3 and 11.5 was not.

#185 is the same run's other half: the last thing the operator saw before
sixteen minutes of silence was a line whose meaning is inverted from how it
reads. Fixing the gate while leaving that line printed beside the new counters
would ship a run that is correct and still reads as broken.

## Evidence

From the 2026-08-20 run: `fragcap capture --launch --direction both --max-bytes
250mb --mode file --target 1`, sixteen minutes wall clock.

| process | packets | share of file |
| --- | --- | --- |
| `com.docker.backend.exe` | 14,108 | 80.0% |
| `claude.exe` | 704 | 10.4% |
| **`AngelLegion.exe`** (the target) | **1,246** | **8.9%** |
| `chrome.exe` | 139 | 0.2% |
| (unattributed) | 45 | 0.1% |
| eight others | 185 | 0.4% |

The reported summary said `attributed 18184`, which reads as "attributed to the
game" and means "resolved to some process on this machine".

**The noise is one contiguous window, and the kernel filter is not at fault.**
First target packet at t+20.7s; last non-target packet at t+22.5s; after that the
file is pure target traffic for the remaining 15.8 minutes. Phase-two narrowing
engaged about 1.8 seconds after the target's first socket appeared, which is the
1 second attribution refresh plus the 2 second debounce behaving exactly as
specified. The whole loss is the bootstrap window before the target touches the
network, which for a launcher-mediated title is tens of seconds and cannot be
shortened. Section 12.2 anticipated the magnitude: reconnaissance recorded "a
single unrelated background process accounted for up to 94 percent of captured
bytes", and this run reproduced that to within one point.

**The information needed to make the decision is already on every packet.**
Exactly 1,246 written packet comments carry a `role=` and a `stage=` key, and
that is precisely the target's count. `CapturedPacket::attribution` carries
`role: Option<Arc<str>>` and `stage: Option<StageId>`
(`crates/fragcap-core/src/attribution.rs:78`), stamped by
`RoleStampingAttributor::resolve` (`crates/fragcap/src/session.rs:668`) from the
session's binding snapshot. What is missing is a predicate that reads it.

**There is exactly one gate, and it tests three things, none of them scope.**
`crates/fragcap-core/src/pipeline/mod.rs:1024` is the only point between the
bounded buffer and the sinks. The only production `WriteGate` is `SessionGate`
(`crates/fragcap/src/session.rs:931`), which tests `ts < admit_from` (watch
time), `ts >= admit_until` (out of window), and the volume bound. Everything
else is written.

**`(enforced)` is not true at the packet level.**
`crates/fragcap-cli/src/orchestrator.rs:154` prints `scope: direction both,
roles all (enforced), loopback false`. `--roles` reaches
`CaptureSession::new_scoped` and gates **which stages trigger acquisition**,
never which packets are retained. `--help` is more accurate than the runtime
line: "The roles to capture, comma-separated. Scopes which stages trigger."

### The blast radius is small, and that was measured rather than hoped

- **`--process` captures stamp role and stage too.** The committed golden
  `crates/fragcap-cli/tests/goldens/capture.jsonl` is produced by `capture
  --process game.exe` and every one of its 24 records carries
  `"role":"target","stage":"target"`. So one predicate covers both target
  selection paths.
- **The CLI capture goldens are 100 percent target traffic.** All 24 records are
  `game.exe`. A default-scoped run over them is therefore byte-identical, and
  the goldens do not move.
- **The corpus tests attach no gate.** `Pipeline::run` takes `Option<Arc<dyn
  WriteGate>>` and `crates/fragcap/tests/corpus_pipeline.rs` passes `None`
  except in one test that deliberately passes a reject-everything gate. Scope
  lives in `SessionGate`, which only the CLI capture path constructs, so the
  fixture corpus is untouched.

### #185: the line that means its opposite

Two identical emit sites, `orchestrator.rs:277` (prerecorded) and `:684` (live):

```rust
let endpoints = stamper_reader.as_ref().map(|s| s.active_endpoints().len()).unwrap_or(0);
emitter.event(&Event::FilterNarrowed { endpoints });
emitter.progress(&format!("filter narrowed to {endpoints} endpoint(s)"));
```

Both read the count once, at acquisition. Three problems, in the order they bite:

1. **The wording inverts the meaning.** "Narrowed to 0" reads as "fragcap has
   narrowed its listening down to zero targets", which sounds like the run gave
   up. It means the opposite: zero endpoints means no narrowing has happened and
   fragcap is still capturing everything on the wire.
2. **The sample is taken at the worst possible instant.** On a `--launch` run
   acquisition happens when the process starts, many seconds before the title
   touches the network, so the answer is structurally zero.
3. **It is never updated.** The transition that actually matters, the moment
   capture stops being machine-wide and becomes target-only, is invisible. On the
   observed run the filter narrowed at t+22.5s and the terminal's final state
   described a moment that had been obsolete for sixteen minutes.

The zero itself is not a bug. Zero endpoints at acquisition is correct, and
`FilterManager` deliberately never installs an empty program.

## Clarifications

### Session 2026-08-20

- Q: What is the default scope? -> A: **`target`, the scoped form.** Recorded
  operator decision, and #184's own recommendation: scoped output is what the
  tool claims and what a user expects. This is a user-visible default change and
  belongs in the release notes.
- Q: #184 proposes `--scope target|profile|all`. With `--roles` defaulting to
  `all`, do `target` and `profile` differ? -> A: **Yes, and only when `--roles`
  is narrowed.** `target` admits a packet whose bound role is in the `--roles`
  set; `profile` admits any packet bound to any profile stage regardless of
  `--roles`; `all` admits everything. With the default `--roles all` the first
  two coincide, which is correct rather than redundant. Making `target` consult
  `--roles` is also what finally makes the `(enforced)` claim true, so the two
  defects close together.
- Q: #184 item 4 asks that a target packet dropped for scope be distinguishable
  from a background packet dropped for scope. -> A: **Two counters, not one.**
  `scope_discarded` counts packets attributed to a process no profile stage
  binds, which are confidently out of scope. `scope_unresolved_discarded` counts
  packets with no attribution at all, which *might* have been the target's and
  were dropped because the socket table had not yet published. Folding them
  would hide a real loss behind an intended one, which is the P-4 failure this
  slice exists to fix.
- Q: Where does the narrowing transition get observed, given that the control
  thread that performs it lives in `fragcap-core` and the emitter lives in the
  CLI? -> A: **In the CLI's own `drive` loop.** `spawn_pipeline` runs the
  pipeline on its own thread and the CLI thread then sits in `drive`
  (`orchestrator.rs:302`), holding the stamper `Arc`. It can poll
  `active_endpoints().len()` and emit on change with no cross-crate plumbing and
  no new channel. A core-to-CLI event channel was rejected as a larger change
  for the same observation.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The capture contains the target's traffic (Priority: P1)

An operator names a target, captures for sixteen minutes, and opens the result.
Today 91 percent of it belongs to processes they did not name.

**Why this priority**: it is the product's headline claim failing on its first
real exercise, and it is filed P0.

**Independent Test**: run a capture, then parse the written file by the `proc=`
field of every packet.

**Acceptance Scenarios**:

1. **Given** a capture under the default scope, **When** the written file is
   parsed by `proc=`, **Then** every packet belongs to the named target.
2. **Given** `--scope all`, **When** the same capture runs, **Then** today's
   behavior is reproduced exactly.
3. **Given** a bootstrap window in which the target has not yet opened a socket,
   **When** background traffic arrives, **Then** none of it reaches the file.

---

### User Story 2 - Nothing is discarded without being counted (Priority: P1)

A capture that threw away 15,181 packets must say so on the way out.

**Why this priority**: equal to US1. P-4 is not "a counter may exist" but
"nothing escaped the accounting", and a scope gate without a counter would
replace visible noise with invisible loss, which is worse.

**Independent Test**: run a capture over traffic containing both target and
non-target packets and reconcile the counters.

**Acceptance Scenarios**:

1. **Given** any capture, **When** it completes, **Then** every packet excluded
   on scope grounds has advanced a named counter.
2. **Given** the session's counters, **When** they are summed, **Then** the
   watch-time, out-of-window, scope, and unresolved-scope discards equal the
   pipeline's capture-wide `gate_dropped`.
3. **Given** the pipeline conservation identity, **When** it is evaluated per
   sink, **Then** it still holds.
4. **Given** a packet dropped for scope with no attribution, **When** the
   summary is read, **Then** it is counted separately from one dropped with a
   non-profiled attribution.
5. **Given** the human summary and the `--json` stream, **When** both are read,
   **Then** both carry the scope counters.

---

### User Story 3 - The summary says what it means (Priority: P2)

`attributed 18184` reads as "attributed to the game" and means "resolved to some
process". A per-process breakdown would have made the defect visible on the
first run instead of on inspection in Wireshark.

**Why this priority**: reporting rather than correctness, but it is what would
have caught US1 without a manual file autopsy.

**Acceptance Scenarios**:

1. **Given** a completed capture, **When** the summary is read, **Then** the
   count of packets attributed to the target is distinguishable from the count
   resolved to any process.
2. **Given** a run that admitted packets, **When** the summary is read, **Then**
   it carries a per-image breakdown of what was written.
3. **Given** the scope line, **When** it is read, **Then** either `(enforced)`
   is true of packet retention or the word is gone.

---

### User Story 4 - The run narrates its phase truthfully (Priority: P1)

The last line before sixteen minutes of silence said `filter narrowed to 0
endpoint(s)`, which means the opposite of how it reads and was obsolete two
seconds later.

**Why this priority**: it is the operator's only window into a long run, and it
currently misinforms at the one moment they read it.

**Acceptance Scenarios**:

1. **Given** a run before the first narrowing, **When** the operator reads the
   terminal, **Then** it says capture is machine-wide and names what is being
   waited for.
2. **Given** the first successful narrowing, **When** it happens, **Then** a
   line is emitted from that transition naming the target and the endpoint
   count.
3. **Given** `--json`, **When** the filter narrows more than once, **Then** an
   event is emitted per narrowing, not one sample at acquisition.
4. **Given** any human output, **When** it is read, **Then** the word
   "endpoint(s)" does not appear as a bare count.

---

### Edge Cases

- **The setup race is the one genuine risk.** A target packet whose socket the
  table has not yet published carries no attribution and would be rejected
  rather than merely mislabeled. Retention (section 11.4) covers teardown; the
  triggered refresh covers setup and demonstrably worked on the observed run,
  where the target's first packet at t+20.7s was already attributed. The
  separate `scope_unresolved_discarded` counter is what keeps this visible
  rather than silent, and a non-zero value on a real capture is a signal to
  investigate rather than an expected outcome.
- **A run with no profile and no target** (`--process`) still stamps role and
  stage, verified against the committed golden, so it is scoped like any other.
- **`--scope all` must reproduce today's behavior exactly**, so an operator
  correlating the target against the rest of the machine, or debugging
  attribution itself, keeps that ability.
- **The offline replay path** opens the gate window from `i64::MIN` and has no
  live socket table. Its attributions come from a script. Scope must behave
  identically there or the CLI capture goldens move, which would mean the
  predicate is reading something other than the stamped role and stage.
- **A packet with no flow key** was never attempted for attribution and is a
  distinct state from attempted-and-failed (the S02 precedent). It carries no
  attribution and therefore falls to the unresolved counter, not the confident
  one.

## Requirements *(mandatory)*

### Functional Requirements

**The scope decision (#184)**

- **FR-001**: The write gate MUST make a scope decision on every packet, in
  userspace, independent of any installed kernel filter (specification section
  12.3).
- **FR-002**: `--scope` MUST accept `target`, `profile`, and `all`, defaulting
  to `target`.
- **FR-003**: Under `target`, a packet MUST be admitted when its attribution
  carries a bound stage or role whose role is within the `--roles` set.

  The role set reaches the gate through the gate's own configuration. It is held
  today on `CaptureSession` as `allowed_roles` and is not on the structure the
  gate is built from, so this requirement implies plumbing across two types; the
  plan names it.
- **FR-004**: Under `profile`, a packet MUST be admitted when its attribution
  carries any bound stage or role, regardless of `--roles`.
- **FR-005**: Under `all`, the gate MUST behave exactly as it does today, and a
  capture under `--scope all` MUST be byte-identical to one taken before this
  slice.
- **FR-006**: The scope decision MUST NOT depend on the kernel filter's state,
  so the bootstrap window no longer determines what reaches the file.

**The accounting (#184, P-4)**

- **FR-007**: A packet excluded on scope grounds whose attribution names a
  process no stage binds MUST advance `scope_discarded`.
- **FR-008**: A packet excluded on scope grounds carrying no attribution at all
  MUST advance a distinct `scope_unresolved_discarded`, because it might have
  been the target's and was dropped for a reason the operator needs to see.
- **FR-009**: The session's four discard reasons (watch-time, out-of-window,
  scope, unresolved-scope) MUST sum to the number of times the gate refused a
  packet, asserted by a test at the gate.

  That refusal count *is* the pipeline's capture-wide `gate_dropped`, which is
  incremented once per refusal at the single call site in `fragcap-core`. Stated
  as a gate-local invariant rather than a cross-crate one because that is where
  it is testable: `SessionGate` never sees `gate_dropped`, so a requirement
  phrased across the boundary could not be checked without inventing a seam.
- **FR-010**: The pipeline conservation identity MUST continue to hold per sink.
- **FR-011**: Both scope counters MUST appear in the human completion summary
  and in the `--json` event stream.

**The reporting (#184 item 5)**

- **FR-012**: The summary MUST distinguish packets attributed to the target from
  packets resolved to any process.
- **FR-013**: The summary MUST carry a per-image breakdown of what was written,
  from the `holder_tally` the pipeline already accumulates.
- **FR-014**: The scope line MUST NOT claim `(enforced)` unless role enforcement
  is true of packet retention. Under the default scope it becomes true; the
  requirement is that the line and the behavior agree.

**The narration (#185)**

- **FR-015**: The filter-narrowed line MUST be emitted from the narrowing
  transition, not from a one-shot read at acquisition.
- **FR-016**: Before the first narrowing, human output MUST say that capture is
  currently machine-wide and name what it is waiting for.
- **FR-017**: On the first narrowing, human output MUST say that capture has
  become target-only, and how many connections matched, in words that carry
  their own meaning.

  Softened during implementation from "name the target". The target's image is
  already printed on the `stage matched: <role> pid <n> <image>` line
  immediately above, so the operator has it in view; carrying a second copy into
  this line would mean threading the narrator through `apply_event` and its
  three callers to duplicate a name already on screen. The narrator keeps a
  field for it so a caller that does have the image can say it.
- **FR-018**: `Event::FilterNarrowed` MUST be emitted on each actual narrowing
  with its count, so a machine consumer receives the series rather than one
  sample of zero.
- **FR-019**: Human output MUST NOT present a bare "endpoint(s)" count. The
  operator is being told how many of their own sockets are being watched.
- **FR-020**: Subsequent set changes MUST NOT produce a line per change; they
  are debounced, or reported only in the structured stream.

**The observe-mode interaction (found during implementation)**

- **FR-021**: A run whose target was resolved in observe mode (slice S059, an
  unresolved launch chain being promoted to its observed socket holder) MUST NOT
  be scoped to that target, and MUST report that it is not.

  Found by running the tests, not by the analyze gate, and it is the sharpest
  interaction in this slice. S059 promotes an unresolved target to the process
  it observes holding the sockets, and the observation is `holder_tally`, which
  counts **only packets the write gate admitted** (asserted by an existing S059
  test). Scoping such a run to its target would therefore starve the mechanism
  that decides what the target is: the gate would reject everything unbound, the
  tally would be empty, the file would contain nothing, and nothing would be
  promoted. Measured before the fix: `retained 0` on a run that captured 24
  packets and attributed all of them.

  A run that does not yet know its target cannot scope to it. The scope widens
  to `all` for that run only, and the run says so, because an operator who asked
  for a scoped file and received an unscoped one must be told and told why
  (P-9). The scope they asked for applies to every run after the promotion.

### Out of scope

- **OOS-001**: The live status display and the wider visual pass (#186). It
  depends on this slice's counters and is sequenced after it.
- **OOS-002**: Directional output filtering. `--direction` is still recorded and
  not enforced, and its existing warning stays.
- **OOS-003**: Shortening the bootstrap window. It cannot be shortened below the
  time a launcher-mediated title takes to reach the network, which is the whole
  reason the userspace decision is the fix.
- **OOS-004**: Retroactive filtering of already-written output. Every observed
  noise packet resolves to a non-profiled process at the moment it reaches the
  gate, so an active gate removes 100 percent of the observed noise and no
  second pass is needed.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On a real capture under the default scope, the share of written
  packets belonging to the named target is 100 percent, verified by parsing the
  `proc=` field of every packet, against 8.9 percent before.

  This is the only check that exercises the defect as filed, and it needs npcap,
  an installed title, and a live run. The *mechanism* is proven without any of
  that by the replay path, which drives the same gate over scripted attributions
  (SC-002), so a live run that cannot be performed weakens this claim to
  "demonstrated by replay, not confirmed in the field" rather than blocking the
  slice. Which of the two happened must be stated plainly at completion.
- **SC-002**: Every excluded packet is counted, and the session's four discard
  reasons sum to `gate_dropped`.
- **SC-003**: The pipeline conservation identity holds per sink, unchanged.
- **SC-004**: A capture under `--scope all` is byte-identical to one taken
  before this slice.
- **SC-005**: The committed CLI capture goldens are byte-identical under the new
  default, because every packet in them is the target's.
- **SC-006**: A `--json` run emits a `filter.narrowed` event per narrowing, not
  one at acquisition.
- **SC-007**: No human output contains "filter narrowed to 0 endpoint(s)" or any
  bare endpoint count.
- **SC-008**: `cargo xtask ci` is green.
- **SC-009**: An observe-mode run still retains packets and still promotes its
  target, and warns that its scope was widened. The S059 promotion tests pass
  unchanged.

## Assumptions

- The binding snapshot is the authority on what belongs to the capture. It is
  what stamps `role=` and `stage=` into output today, so scoping on it means the
  file's contents and the file's own annotations cannot disagree.
- The observed run's composition is representative of the defect but not of
  every machine. The acceptance is expressed as "every written packet is the
  target's", which does not depend on the ratio.
- `holder_tally` counts only gate-admitted packets (asserted by an existing S059
  test), so after this slice it becomes a breakdown of the file rather than of
  the wire. That is the desired meaning for FR-013 and must be stated where the
  summary renders it.
