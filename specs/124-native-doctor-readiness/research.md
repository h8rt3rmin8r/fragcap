# Research: Native Deep Capture Doctor Readiness

## R1: Active session identity

**Decision**: Replace PID-only liveness with an opaque per-session lease backed
by a Windows named synchronization object. Keep its handle in the session
adapter and store only the bounded lease identity, PID, and canonical bundle in
the owner record. Tests inject lease state on other platforms.

**Rationale**: PIDs are reusable, and image names do not establish ownership.
The named object exists only while the exact fragcap session generation holds
it. Opening that synchronization object does not open or inspect a target
process.

**Alternatives considered**: A PID plus image is still reusable. Process
creation time would require a process handle or weaker snapshot inference. A
marker file survives crashes and cannot prove liveness. An unlocked token file
has the same problem.

## R2: Inventory authority and bounds

**Decision**: Use one read-only scanner with explicit depth, entry, session,
journal-byte, journal-record, and finding limits. It returns observations and
limitations as data and never creates the session directory. Exact canonical
custom bundle roots remain reachable through owner records because the shipped
CLI supports an operator-selected bundle outside the default root.

**Rationale**: Independent scanners already cause journal errors to collapse
into stale-manifest aggregates. One inventory can preserve unknowns and ensure
the human, JSON, and fix paths see identical facts.

**Alternatives considered**: Extending each aggregate check independently
would retain inconsistent limits and duplicate ownership policy. Treating a
limit as absence would create a false-clean report.

## R3: Resource health classification

**Decision**: Classify every latest journal resource state using the existing
journal parser and recovery plan. Terminal complete history is healthy. A live
matching lease makes nonterminal work active. A dead lease with recoverable
work is stale, a failed latest transition is cleanup-failed, invalid evidence
is unknown, and non-Windows native runtime is unsupported.

**Rationale**: The journal already owns resource identity, state transitions,
and exact recovery eligibility. Health is a diagnostic projection, not a new
lifecycle state machine.

**Alternatives considered**: Filename or port heuristics cannot establish
ownership. Treating every crash prefix as stale loses active-session and
cleanup-failure distinctions.

## R4: Listener and process observations

**Decision**: Associate a listener only when an exact journaled proxy endpoint
and live matching lease agree. A bound endpoint without that ownership is
reported as unrelated occupancy and never as an orphan or cleanup target.
Remove the external proxy process placeholder.

**Rationale**: The native proxy runs inside fragcap, so process discovery is a
legacy abstraction. Port occupancy proves only occupancy.

**Alternatives considered**: Guessing from a port or executable name can target
unrelated software. Reintroducing a sidecar process contradicts the S104 native
cutover.

## R5: Readiness derivation

**Decision**: Give each check an explicit mode scope. Capture readiness derives
from shared and Capture checks. Deep Capture derives from shared, Capture, and
Deep Capture checks because packet capture is part of its evidence contract.
Append stable mode verdict records to JSON and render both in human output.

**Rationale**: One ordered check set prevents format drift while two projections
make the operator boundary explicit. The existing nonzero exit remains the OR
of blocking verdicts.

**Alternatives considered**: Two separately built reports can disagree. A
single global verdict hides which workflow is usable.

## R6: Repair authority

**Decision**: Offers contain the exact existing journal recovery action and are
executed only after the current confirmation gate by the existing recovery
implementation. Re-run the inventory afterward.

**Rationale**: Doctor must not invent a cleanup policy. Exact actions already
exist for trust and process-scoped obligations, while ambiguous artifact and
launch obligations already refuse recovery.

**Alternatives considered**: Broad session-directory deletion can destroy
retained evidence. Duplicating recovery logic risks divergent safety rules.

## R7: Packaging boundary deviation

**Decision**: S124 defines runtime and installed-state diagnostic contracts
needed by packaging, while issue #329 retains installer/archive/offline smoke,
upgrade, repair, uninstall, contents, and size validation.

**Rationale**: The original issue dependency made #321 depend on #329 even
though #329 needs Doctor's stable contracts. Breaking that circular ordering is
an explicit deviation from the issue text and is recorded on issue #321.

**Alternatives considered**: Waiting for #329 leaves packaging without a stable
readiness surface. Pulling packaging into S124 violates the single-slice scope.
