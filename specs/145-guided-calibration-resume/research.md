# Research: Durable Guided Calibration Resume

## Decision 1: Persist in the existing local target store

The workflow belongs to one durable target and must share the effective `local.db`
selection already carried by calibration guidance. Add one schema-versioned table to
the existing SQLite store. This preserves one explicit storage authority, atomic
updates, foreign-key cleanup, and the repository's established migration behavior.

Rejected alternatives:

- A JSON sidecar creates a second persistence and locking path.
- Bundle-local state disappears when the operator does not choose an explicit bundle
  and would mix progress with evidence artifacts.
- Compatibility facts cannot represent intent, pause state, or in-flight execution
  without corrupting their evidence meaning.
- Cleanup journals describe system-effect obligations and cannot authorize workflow
  progression.

## Decision 2: Use an explicit store-local positive integer identifier

SQLite assigns the workflow identifier inside the creation transaction. The resume
command always includes `--local-db`, so global uniqueness adds no safety value. A
positive integer is easy to validate, quote, report, and use without another entropy
or identifier dependency.

`--resume` joins the existing mutually exclusive target-input group and conflicts
with `--protocol`. Operational session bounds may be supplied again because the new
S134 plan binds them before effects.

## Decision 3: Bind the complete target authority, not a weak fingerprint

The checkpoint stores the stable identifier plus the target row's handle, name,
anchor, install root, and launch declaration. Resume re-resolves by stable identifier
and compares those fields exactly. Explicit fields remain reviewable and avoid adding
hash construction as another durable contract.

Any mismatch refuses the workflow. Automatic rebasing would let old intent silently
authorize a changed topology.

## Decision 4: Store protocol sets as canonical JSON arrays

Requested, observed, completed, and remaining sets are ordered by the existing
protocol token and serialized as JSON arrays. Reads parse every token through the
closed `CompatibilityProtocol` vocabulary, reject routing and not-applicable where a
concrete candidate is required, sort, deduplicate, and reject non-canonical storage.

This uses the `serde_json` runtime dependency already present in `fragcap-targets` and
adds no lockfile package.

## Decision 5: Checkpoint at execution boundaries with optimistic revision

Creation writes `ready`, revision one, and attempt ordinal zero before any attempt.
The immediate pre-delegation transaction writes `in-flight`, the next ordinal, phase,
protocol, and exact S144 case key. Its bounded canonical history preserves no-repeat
authority across processes. A terminal transaction writes the fresh coverage and either `ready`,
`paused`, `completed`, or `refused`. Every update includes the expected revision and
increments it exactly once.

If a process dies after the in-flight commit, resume reports interruption and rebuilds
current state. The row is not an effect journal and supplies no cleanup authority.

## Decision 6: Model operator work as bounded pause reasons

The closed pause vocabulary is `login`, `eula`, `gameplay`, `shutdown`,
`interrupted`, `authorization`, and `failure`. The first five can be selected
explicitly with `--pause-for` on `--resume`; authorization and failure are terminal
classifications written by the sequence.

The explicit pause operation updates one row and emits guidance. It never calls the
Deep Capture executor. Warm state maps to shutdown, missing post-session positive
evidence maps to gameplay, and an interrupted session maps to interrupted. The
software does not inspect or automate the target UI.

## Decision 7: Preserve fresh authorization and current facts

A checkpoint contains no plan identifier, response, secret, capability, endpoint,
certificate, trust state, effect obligation, or claimed compatibility result. Resume
reuses only target-bound candidate intent, bounded exact attempt history, and the
durable attempt ordinal. Current
facts determine completion, current process state determines readiness, existing
lifecycle recovery determines effect safety, and a new plan determines authorization.

## Decision 8: Keep S145 below final parent completion

S145 resolves cross-process pause and resume for the existing exact Steam and direct
topologies. Non-Steam topology authoring, ambiguous candidate selection, advanced
override policy, and the final guided-completion gate remain in parent issue #380.
