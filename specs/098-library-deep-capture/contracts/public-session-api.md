# Contract: Public Deep Capture Session API

## Preflight

The public facade accepts typed `SessionConfig` plus side-effect-free resolution capabilities and returns either one `PreparedSession` containing every resolved execution input and immutable `SessionPlan`, or one typed `PreflightRefusal` with stable stage and reason code.

Preflight performs no process spawn, trust mutation, bundle creation, fact append, packet-driver access, or remote request. It resolves a target and launch case exactly once.

## Authorization

The caller presents or otherwise reviews the complete plan. `Authorization::Approved` must carry the exact `PlanId`. Declined, missing, stale, or mismatched authorization reaches a terminal no-effect refusal and cannot call an effect adapter.

## Lifecycle

The API exposes granular checked operations for start, observation, stop, finalization, and cleanup, plus a convenience end-to-end runner. Every operation validates the current state before adapter calls. Invalid ordering returns `InvalidTransition` with operation, actual state, and allowed states. Repeated operations and terminal-session reuse perform zero effects.

Once an effect has started, operational failures are retained in session state. The API returns a `TerminalReport` after all safe independent fact, cleanup, artifact, and terminal-delivery attempts. It does not discard the report in favor of the first error.

## Stable public values

The surface uses documented typed values for configuration, plan, state, deadlines, observations, outcomes, failures, omissions, facts, artifacts, cleanup, event delivery, events, and terminal report. Evolvable enums are non-exhaustive where appropriate. Human strings are diagnostic detail, not machine state.

## Deadline behavior

Configured deadlines are defaulted or capped during preflight and the effective values appear in the plan and report. Blocking adapters receive the relevant remaining budget. A success returned after the deadline is recorded as deadline exceeded. Every independent cleanup adapter is still called once when its budget is exhausted, with an exhausted budget that permits a typed not-attempted result.

Rust cannot safely preempt an arbitrary blocking trait call. Production and third-party adapters must cooperatively honor their budgets; this obligation is part of their public documentation.

## Terminal authority

The in-memory `TerminalReport` is authoritative. Bundle files and events report their own successfully persisted or delivered subset. A fact, artifact, cleanup, or event gap prevents `Complete`, but does not erase observations or change a known operational outcome into an unsupported claim.
