# Contract: Events, Snapshots, and Results

## Event ordering

The coordinator emits typed events in lifecycle order. Existing CLI JSON event names and fields remain the compatibility baseline. The CLI renderer maps typed values without reclassification.

Plan presentation is a caller responsibility. Authorization asserts that the identified prepared plan was reviewed. After effects begin, event-delivery failure is accumulated and never prevents later stop, fact, cleanup, artifact, or terminal attempts. Delivery sequence numbers make gaps explicit.

## Observation and failure retention

Every observation accepted before interruption or failure survives into the terminal snapshot. Failures remain in chronological order and do not mask later failures. No missing or malformed correlation anchor becomes a reached-client observation. Silence remains inconclusive unless affirmative evidence supports a narrower outcome.

## Fact results

The coordinator derives fact candidates only from supported observations and attempts them independently. Each result is appended, skipped with reason, or failed. Routing does not imply propagation. Launcher HTTPS does not imply final-client CA acceptance. Only correlated final-client evidence supports final-client compatibility facts.

## Cleanup results

Every owned or applicable resource has exactly one recorded result, including resources whose cleanup budget was already exhausted. A failure or uncertain release prevents `Complete` and includes remediation where available.

## Immutable snapshot

After fact and cleanup attempts, the coordinator freezes one terminal snapshot. Compatibility, fact-write, cleanup, and manifest content are rendered from this snapshot. Bundle policy cannot inspect mutable CLI state or reclassify the run.

## Artifact authority

Artifact writes are independently reported. The manifest is attempted last. If a late write fails, already written files retain the snapshot they were derived from, the missing or failed artifact is explicit in `TerminalReport`, and the report remains final authority. No artifact is rewritten to predict a failure that had not yet occurred.

## Outcome rules

- `Complete`: operation succeeded and every required fact, cleanup, artifact, and event obligation succeeded or was explicitly not applicable.
- `Partial`: useful evidence exists but one or more required obligations failed or remain uncertain.
- `Failed`: the intended operation did not produce useful session evidence, with all cleanup and failure truth retained.
- `Interrupted`: caller or system interruption ended execution, with earlier evidence and subsequent cleanup retained.

Omission, failure, and cleanup reason codes are stable integration values. Diagnostic prose can evolve without changing classification.
