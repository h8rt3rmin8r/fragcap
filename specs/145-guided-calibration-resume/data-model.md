# Data Model: Durable Guided Calibration Resume

## GuidedCalibrationWorkflow

One current checkpoint for a bounded calibration intent.

| Field | Meaning | Invariant |
| --- | --- | --- |
| `id` | Store-local workflow identity | Positive, immutable |
| `record_version` | Checkpoint record contract | Exactly 1 for this build |
| `target_id` | Owning local target row | Foreign key with cascade delete |
| target snapshot fields | Complete S144 sequence authority | Immutable after creation |
| `requested_protocols` | Operator-requested concrete cases | Canonical ordered JSON set |
| `observed_protocols` | Eligible final-client candidates | Canonical ordered JSON set |
| `completed_protocols` | Fresh coverage projection at checkpoint | Canonical ordered JSON set |
| `remaining_protocols` | Fresh coverage projection at checkpoint | Canonical ordered JSON set |
| `attempted_case_keys` | Exact S144 cases that may have entered session work | Canonical ordered bounded JSON set |
| `attempt_ordinal` | Last allocated attempt destination | Integer from 0 through 14 |
| `attempt_phase` | In-flight attempt phase | Present only in-flight |
| `attempt_protocol` | In-flight attempt protocol | Present only in-flight |
| `attempt_key` | Exact current S144 case | Present only in-flight and in attempted history |
| `state` | Current workflow lifecycle | Closed vocabulary |
| `pause_reason` | Exact pause boundary | Present only while paused |
| `revision` | Optimistic concurrency authority | Positive and increments once |
| `created_at` | Creation time | Unix seconds, immutable |
| `updated_at` | Latest checkpoint time | Unix seconds, nondecreasing |

## Lifecycle States

```text
ready -> in-flight -> ready
                   -> paused
                   -> completed
                   -> refused

ready -> paused
paused -> ready
paused -> paused
completed -> completed
in-flight -> paused on explicit resume after interruption classification
```

`refused` and completed target drift are terminal for effects. A completed resume may
still refresh and report current coverage without mutating compatibility evidence.

## Pause Reasons

| Token | Authority |
| --- | --- |
| `login` | Explicit operator selection |
| `eula` | Explicit operator selection |
| `gameplay` | Explicit operator selection or missing required positive evidence |
| `shutdown` | Explicit operator selection or observed warm target |
| `interrupted` | Explicit operator selection, in-flight resume, or interrupted run |
| `authorization` | Declined, invalid, or unavailable authorization input |
| `failure` | Terminal delegated-session failure |

Pause state is not evidence that the named action occurred. It records why the
workflow stopped and what the operator intends to address.

## Target Authority Snapshot

The immutable snapshot retains:

- stable identifier
- handle
- display name
- anchor
- install root
- launch declaration JSON

Resume compares all fields exactly with the freshly resolved target. Row identity is
the foreign-key relationship; stable identity and content protect against replacement
or redirection.

## Protocol Set Invariants

- Only concrete guided protocol tokens are accepted.
- Requested and observed remain distinct.
- Completed and remaining are a checkpointed projection, not new evidence.
- Current compatibility facts are reread and supersede old coverage on every resume.
- Arrays are token-sorted, duplicate-free, and serialized canonically.
- Exact attempted-case keys preserve S144's no-repeat bound across processes. A
  pre-session authorization refusal removes its prospective key but never reuses its
  already allocated bundle ordinal.

## Concurrency

Updates use `WHERE id = ? AND revision = ?`. A successful update writes revision plus
one. Zero changed rows triggers a fresh existence check to distinguish deletion from
concurrent revision drift. No last-writer-wins behavior is permitted.
