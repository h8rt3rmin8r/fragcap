# Contract: Bounded `fragcap calibrate` Sequence

## Invocation

The existing invocation remains authoritative:

```text
fragcap calibrate (<TARGET> | --target <TARGET> | --id <ID>) [OPTIONS]
```

Repeatable `--protocol` values remain explicit measurement candidates. `--authorize-stdin` accepts one exact complete input line for every plan encountered in order. `--bundle <PATH>` names the first effectful attempt bundle.

## Sequence Contract

1. Complete existing target registration, Steam client setup, and optional warm-restart boundaries.
2. Re-resolve the durable target and rebuild the current proposal.
3. If no useful work remains, validate ordinary readiness and emit terminal coverage.
4. Otherwise select the first useful unattempted exact case.
5. Emit selected guidance with one-based attempt and maximum values.
6. Delegate to the existing low-level executor, which emits and confirms one complete plan.
7. Accumulate eligible observed candidates and rebuild current authority.
8. Emit the attempt outcome and continue only after current positive evidence.

Each return to step 2 is a new authority evaluation. No session plan or response survives it.

## Ordering

- Reachability precedes protocol attempts while current routing evidence is absent.
- Protocol candidates use the existing S139 deterministic order after normalized union and deduplication.
- Current positive candidates are skipped without consuming an attempt number.
- The same exact case is never delegated twice in one invocation.

## Authorization Input

Interactive input retains one default-no prompt per effectful attempt. Structured input supplies one exact emitted plan identifier per line:

```text
deep-capture-plan-v1:<first-id>
deep-capture-plan-v1:<second-id>
```

A registration or Steam setup identifier, if needed, occupies its own earlier line and does not confirm a session.

## Bundle Destinations

For `--bundle C:\evidence\game-calibration`:

- attempt 1 uses `C:\evidence\game-calibration`;
- attempt 2 for TLS HTTPS uses `C:\evidence\game-calibration-attempt-02-tls-https`;
- later attempts follow the same fixed pattern.

Existing non-empty-directory refusal applies independently to every path. Default bundle selection remains one unique session path per attempt.

## Stable Guidance Additions

Every `calibration.guidance` event adds nullable attempt fields:

```json
{
  "attempt": 2,
  "maximum_attempts": 14,
  "phase": "tls",
  "protocol": "https"
}
```

Non-attempt setup, warm, ready, and refusal guidance uses null attempt, phase, and protocol values. Existing event identity and coverage fields remain unchanged.

## Stop Contract

The sequence stops before another session on:

- operator decline or closed input;
- invalid structured input or interruption;
- target, topology, process, proposal, fact, or bundle refusal;
- terminal session failure;
- absence of the attempted current positive fact;
- selection of an already attempted exact case;
- exhaustion of useful current work.

The final guidance reports exact requested, observed, completed, and remaining candidates. It includes a continuation only when current authority supports one.

## Exit Compatibility

- Interactive decline and incomplete positive evidence retain clean bounded completion with remaining work visible.
- Usage, invalid structured input, drift, operational errors, and terminal session errors retain their existing exits.
- Previously completed attempts are never rolled back or relabeled because a later attempt stops.
