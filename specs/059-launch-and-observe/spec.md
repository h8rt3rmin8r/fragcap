# Feature Specification: Launch-and-observe promotion

**Feature Branch**: `059-launch-and-observe`

**Created**: 2026-08-18

**Status**: Draft

**Input**: User description: "S059 launch-and-observe capture and capture-time promotion of unsure/no-authored targets (issue #152)."

## Context

The interactive `targets add` prompt asks whether the executable the operator
pointed at is the process that holds the gameplay sockets. Two of its three
answers, `no` and `unsure`, deliberately record no socket holder: they store an
unresolved launch chain (`socket_holder: "unresolved"`) rather than fabricate a
client the tool never observed (P-9). Slice S055 shipped the promotion
mechanism, `Store::promote_target_launch`, but deferred the capture-time
trigger: a stored target whose launch chain is unresolved names no capturable
client, so `capture` refused it before any run could observe one.

This slice closes that gap. A target with an unresolved launch chain becomes
capturable in an observe mode built from the executable the operator did record.
When the run observes a dominant socket-holding process, the target is promoted:
its launch chain is rewritten to that resolved client and its fidelity is raised.
When the run observes nothing, the target is left exactly as it was, because
promoting on no observation would be the fabrication P-9 forbids.

It extends the shared stored-target resolution seam S058 extracted
(`commands/target_resolve.rs`), so both the `capture` command and the Wireshark
`extcap` path reach the new behavior through one implementation.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Capturing an unsure target and promoting it (Priority: P1)

An operator registered a game with `targets add` but answered `no` or `unsure`
when asked whether the executable they pointed at holds the sockets. The target
is stored with an unresolved launch chain. Later they run a capture against it.
The capture observes the actual socket-holding process, writes the capture, and
promotes the stored target so that the next capture against it addresses the
observed client directly.

**Why this priority**: This is the whole feature: it turns a target the operator
could register but not capture into one they can capture and that improves itself
by being captured. Without it, `no`/`unsure` targets are dead ends.

**Independent Test**: Register a target with `--socket-holder no`, run an offline
capture over a fixture whose process tree spawns a child socket holder, and
confirm the run succeeds, writes attributed packets, and rewrites the stored
target to a resolved client at raised fidelity.

**Acceptance Scenarios**:

1. **Given** a stored target with an unresolved launch chain and an observed
   executable, **When** a capture runs against it and a dominant socket-holding
   process is observed, **Then** the capture succeeds and the stored target's
   launch chain is rewritten to the observed client with fidelity raised to
   verified.
2. **Given** the same stored target, **When** a capture runs against it and no
   socket-holding process is observed, **Then** the capture completes without
   error and the stored target is left unchanged (still unresolved).

---

### User Story 2 - Observe-mode capture writes a normal file (Priority: P1)

An operator captures an unsure target and gets a normal, well-formed capture
file with the socket-holding process's traffic attributed, exactly as they would
for a fully resolved target. The observe machinery is invisible in the output.

**Why this priority**: The promotion is a side benefit; the capture itself must
be a first-class capture. If observe mode degraded the output, the feature would
not be usable for its primary purpose (capturing the game).

**Independent Test**: Run the observe-mode capture to a pcapng and a JSON Lines
sink and confirm the attributed packets carry the socket holder's process, role,
and stage, and that the completion summary and file trailers are unchanged in
shape from a normal capture.

**Acceptance Scenarios**:

1. **Given** an observe-mode capture that acquires the target, **When** it
   finishes, **Then** the written files attribute the socket holder's packets and
   the completion summary reports the same counters a resolved-target capture
   reports.
2. **Given** an observe-mode capture, **When** the completion summary and the
   golden output files are compared, **Then** no new per-image tally leaks into
   them.

---

### User Story 3 - Extcap resolves unsure targets without promoting (Priority: P2)

An operator selects an unsure target in the Wireshark extcap configuration
dialog. The capture streams to the analyzer normally. The extcap path shares the
same resolution, so it too can capture an unsure target; it does not perform the
store write-back (it is a streaming bridge, not the store owner).

**Why this priority**: The shared seam means extcap gets observe-mode resolution
for free, and the feature would be inconsistent if extcap rejected a target
`capture` accepts. Promotion write-back is out of extcap's remit, so it is
deliberately excluded there rather than duplicated.

**Independent Test**: Resolve an unsure target through the extcap capture path and
confirm it produces the same observe-mode profile and a valid stream, with no
store mutation.

**Acceptance Scenarios**:

1. **Given** an unsure target selected in extcap, **When** the capture runs, **Then**
   it resolves the same observe-mode profile as `capture` and streams normally.
2. **Given** an extcap capture of an unsure target, **When** it finishes, **Then**
   the stored target is not modified.

---

### Edge Cases

- A stored target whose launch chain is genuinely empty or steamless (names no
  observed executable and carries no Steam anchor) is still refused as before,
  because there is nothing to build an observe-mode profile from.
- A target carrying a Steam anchor is resolved through the existing
  install-layout cascade even when its launch chain is unresolved; the
  observe-mode branch is only for the non-Steam unresolved case.
- Two socket-holding images observed in one run resolve to a single dominant
  image by a deterministic rule, so promotion is never a coin flip.
- A run that acquires the target but captures zero attributed packets observes no
  holder and does not promote.
- The literal `steam://run` launch of an unresolved Steam-anchored target is not
  exercised in continuous integration; it is a Tier 2 path, stated as such.

## Clarifications

### Session 2026-08-18

- Q: What shape must the observe-mode profile take to bind a child socket holder
  without failing validation? A: A two-stage profile: the observed executable as a
  non-terminal launcher stage (matched by `exe`), plus a terminal client stage
  matching processes that descend from the launcher. The client stage's
  `descends_from` predicate is non-empty (so it passes the empty-predicate
  validation) and carries no `exe` (so it cannot trip the ambiguous-image-match
  check). The launcher-stage image itself is captured too, so a run where the
  observed executable is the holder still attributes and promotes.
- Q: How is the dominant socket-holder chosen when a run sees more than one image?
  A: The image with the most attributed packets. Ties break deterministically by an
  ordered per-owner tally, so the same input always yields the same promotion.
- Q: Does the extcap path promote the stored target? A: No. Extcap resolves the same
  observe-mode profile and streams, but the store write-back is the `capture`
  command's responsibility alone.
- Q: What fidelity does a promoted target take? A: Verified. The client was observed
  holding sockets during a real capture, which outranks a heuristic and is outranked
  by an operator's typed assertion.
- Q: What happens on a run that acquires the target but attributes zero packets? A:
  No dominant image is observed, so the target is left unchanged (P-9).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A stored target whose launch chain carries the unresolved marker
  (`socket_holder: "unresolved"`, written by a `no` or `unsure` authoring answer)
  MUST be capturable rather than refused, when it names an observed executable.
- **FR-002**: Resolving such a target MUST synthesize an observe-mode capture
  profile from the stored observed executable. The profile MUST be a valid
  profile (it MUST pass the same validation every profile passes); it MUST NOT be
  an empty-predicate wildcard.
- **FR-003**: The observe-mode profile MUST bind the socket-holding process whether
  it is the observed executable itself or a child process the observed executable
  launched, so that a `no` answer (a different, child process holds the sockets)
  and an `unsure` answer are both capturable.
- **FR-004**: During a capture, the pipeline MUST aggregate, per socket-holding
  process image, the count of attributed packets, so that the run can name the
  dominant socket-holding image. The aggregation MUST be deterministic across runs
  over identical input.
- **FR-005**: The per-image aggregation MUST NOT change any existing counter, any
  golden-pinned completion summary, or any golden-pinned output file (pcapng or
  JSON Lines). It is additive.
- **FR-006**: After a capture against an unresolved target completes, if a dominant
  socket-holding image was observed, the stored target MUST be promoted: its launch
  chain rewritten to a resolved client naming that image, and its fidelity raised
  to verified.
- **FR-007**: If no socket-holding image was observed, the stored target MUST be
  left unchanged. Nothing MUST be written to the store on the strength of no
  observation (P-9).
- **FR-008**: The stored-target resolution seam MUST be a single shared
  implementation reached by both `capture` and `extcap`. The extcap path MUST NOT
  perform the promotion write-back.
- **FR-009**: This slice MUST NOT add a new direct-executable launcher. Live launch
  MUST remain restricted to the existing Steam-anchored launch path. An operator
  may start the game by any means (including manually) and observe-mode captures it.
- **FR-010**: The full promotion decision path (resolve the observe-mode profile,
  aggregate the dominant image, decide to promote or not, write back) MUST be
  verifiable offline over the scripted-attributor fixture pipeline, with no capture
  driver, no elevation, and no game. Only the literal `steam://run` launch is Tier 2
  and MUST be labeled as not exercised in continuous integration.
- **FR-011**: The three new terms introduced by this slice ("launch-and-observe",
  "observed socket-holder", "capture-time promotion") MUST each receive a glossary
  entry in the same change (P-6).
- **FR-012**: The specification's capture and targets command sections (17.2 and
  17.7) MUST be reconciled with the shipped behavior, and `cargo xtask spec` MUST be
  green.
- **FR-013**: `cargo xtask ci` MUST be green (fmt, clippy, test, lint, deps,
  license), and no new dependency MUST be added to the workspace.

### Key Entities

- **Unresolved launch chain**: the stored launch entries a `no` or `unsure`
  authoring answer wrote, carrying `socket_holder: "unresolved"` and the observed
  executable, and naming no client. The precondition for observe-mode resolution.
- **Observe-mode profile**: the validated capture profile synthesized from the
  observed executable, shaped so it binds the socket-holding process (the observed
  executable or a descendant of it).
- **Observed socket-holder**: the process image the run attributed the most packets
  to, the dominant socket-holding image the promotion records.
- **Capture outcome**: what a capture run reports back to its caller: the exit
  result plus, for the `capture` command, the observed socket-holder (if any).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can capture a target they registered with a `no` or
  `unsure` socket-holder answer, where before the capture was refused.
- **SC-002**: After one successful observe-mode capture that saw the game's
  traffic, a second capture against the same target addresses the observed client
  directly (the target reads as resolved).
- **SC-003**: A capture that observes no traffic for the target leaves it exactly
  as registered, so a failed observation never corrupts the stored target.
- **SC-004**: The observe-mode capture's output files and completion summary are
  indistinguishable in shape from a resolved-target capture's (every committed
  golden is reproduced unchanged).
- **SC-005**: The whole promotion decision is demonstrated by an offline test that
  runs without a capture driver, elevation, or a game.

## Assumptions

- The observed executable stored by a `no` answer is the process the operator
  pointed at (a launcher), and the true socket holder is a descendant of it; the
  observed executable stored by an `unsure` answer may be either the holder or its
  ancestor. A two-stage observe-mode profile (the observed executable as a launcher
  stage, plus a terminal client stage matching its descendants) covers the child
  holder case, which is the harder one; the single-holder case where the observed
  executable itself holds the sockets is covered by the same or a companion shape.
- The dominant socket-holder is the image with the most attributed packets in the
  run, resolved by a deterministic tiebreak when two images tie.
- Promotion targets the local store the target was resolved from; the resolved
  target's durable row identifier and that store are both available at write-back
  time.
- Fidelity "verified" is the correct promoted level: the client was observed
  holding sockets during a real capture, which is a stronger claim than a
  heuristic and a weaker one than an operator's typed assertion.

## Dependencies

- Slice S058 (merged): the extracted `commands/target_resolve.rs` resolution seam
  this slice adds a branch to.
- Slice S055 (merged): `Store::promote_target_launch`, the write-back mechanism,
  and `authoring::launch_is_unresolved` / `resolved_client_launch`.
- The scripted-attributor fixture pipeline and the offline capture substrate, for
  the CI-visible proof.
