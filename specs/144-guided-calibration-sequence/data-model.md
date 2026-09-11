# Data Model: Bounded Guided Calibration Sequence

S144 adds transient orchestration values and additive presentation fields only. It does not change SQLite, compatibility-fact, bundle artifact, manifest, or public facade schemas.

## GuidedCalibrationSequence

| Field | Meaning | Rule |
| --- | --- | --- |
| target stable id | Durable target authority | Fixed after registration or setup, re-resolved before every attempt |
| requested protocols | Explicit operator candidates | Sorted, deduplicated, concrete, and never treated as evidence |
| observed protocols | Eligible final-client candidates accumulated from terminal reports | Sorted, deduplicated, and distinct from requested candidates |
| attempted cases | Exact cases whose session executor was entered | Append-only in memory and checked before every attempt |
| next attempt number | One-based effectful session order | Increases only when a distinct selected case is delegated |
| maximum attempts | Closed supported bound | Fourteen in the current vocabulary |
| explicit bundle root | Optional operator-selected first destination | Later attempts use deterministic siblings |

## ExactAttemptedCase

| Field | Meaning | Rule |
| --- | --- | --- |
| launch case | Exact cold launch topology | Copied from the current S139 step |
| routing strategy | Child-environment route | Fixed by the current guided proposal |
| address family | IPv4 guided default | Fixed by the current guided proposal |
| phase | Reachability or TLS | Must match protocol kind |
| protocol | Routing or one concrete protocol | Routing only for reachability; concrete only for TLS |

Equality covers every field. The set prevents repeated effects and proves finite execution.

## AttemptProgress

| Field | Meaning | Rule |
| --- | --- | --- |
| attempt and maximum | Ordered location in the finite sequence | Present for selected and terminal attempt guidance |
| action, status, reason | Stable decision vocabulary | Never inferred from display prose |
| selected case | Phase, protocol, and launch case | Exact current S139 step |
| requested protocols | Original normalized request | Stable for the invocation |
| observed protocols | Accumulated eligible observations | May grow after sessions |
| completed protocols | Candidates with current positive facts | Recomputed from a fresh proposal |
| remaining protocols | Candidates still proposed or deferred | Recomputed from a fresh proposal |
| next command | Stateless continuation or ordinary Deep Capture | Present only when current authority supports it |

## AttemptBundle

| Attempt | Explicit `--bundle` behavior | Default behavior |
| --- | --- | --- |
| 1 | Exact supplied path | Existing unique session path |
| 2 through 14 | Deterministic sibling containing attempt, phase, and protocol | Existing unique session path |

Every destination remains bound into its own S134 plan and must pass the existing empty-directory validation.

## State Transitions

```text
front-door-ready
  -> refresh-authority
    -> refused-or-warm-stop
    -> no-work -> ordinary-readiness-check -> complete
    -> select-unattempted-case
      -> repeated-case-stop
      -> emit-selected-progress
      -> build-and-confirm-current-plan
        -> decline-or-input-stop
        -> execute-session
          -> terminal-failure-stop
          -> accumulate-eligible-candidates
          -> refresh-authority
            -> attempted-fact-missing-stop
            -> next-unattempted-case
            -> complete
```

No transition carries authorization from one attempt to another. No transition after a stop begins another session.

## Validation Rules

- Attempt identity is complete and duplicate-free.
- Attempt count cannot exceed the closed supported bound.
- Candidate sets are deterministic and exclude routing and not-applicable.
- Only current S139 steps may be selected.
- Current positive facts, not exit status or observations, authorize progression.
- Explicit later bundle names use only a checked numeric attempt and fixed enum strings.
- A collision remains a pre-effect refusal under existing bundle validation.
- Terminal coverage is computed from fresh current facts and the accumulated candidate union.
