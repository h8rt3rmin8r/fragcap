# Research: S187 Slow launcher session ownership

## Decision: explicit duration bounds

Raise the session facade's launch ceiling from 120 to 300 seconds and observation ceiling from 300 to 600 seconds. Preserve 30-second launch and 60-second calibration defaults, and require explicit longer values. Validate every deadline before effects instead of applying `min` to caller input. The CLI and facade must use the same limits.

Rationale: #540 requests 120-second calibration and 600-second ordinary observation, while #541 saw a final client after roughly two minutes. A five-minute explicit launch bound covers that case with finite headroom. Alternatives were keeping a 120-second distinct cap, which would not cover the reported late client, and silently clamping to new caps, which violates requested/effective truth.

## Decision: preserve the requested observation span

Give calibration the same separate capture-return allowance already used by ordinary Deep Capture. Today the inner capture reserves five seconds inside the calibration observation deadline, so a nominal 120-second calibration can observe for only 115 seconds. The allowance remains separate from proxy and cleanup budgets.

Rationale: A plan cannot call 120 seconds effective when capture itself is shortened. Alternative of removing the inner return reserve risks an unbounded handoff at the observation edge.

## Decision: stored Steam publisher roles

Reuse ordered stored launch entries with a first `launcher`, optional uniquely named intermediate roles, and one terminal `client`. Generate the platform root as a separate first profile stage, then each declared stage with exact canonical path and declared parent role. Add all declared paths to pre-effect shutdown scope. Treat undeclared dispatch shims and helpers as observed descendants and exit obligations only.

Rationale: The target already supports ordered roles for publisher launches. A parallel Steam-only schema or image-only wildcard would create drift or grant unjustified control. A transient shim need not be declared to become visible in the trace.

## Decision: bounded descendant trace

Retain the exact created root and its causally observed descendants within the existing process-event ceiling, while reporting source loss and projection omission explicitly. Do not infer ancestry from mere PID or image equality when a process generation is ambiguous. A trace row is evidence, not a control grant.

Rationale: The current serializer seeds root, stage matches, and packet owners but drops unseeded publisher helpers even when ETW saw them. Increasing the watcher cap alone would not fix that projection.

## Decision: connection owner truth

At native proxy accept, attempt an exact socket-to-process join under the existing IP Helper attribution authority, then bind it to an observed process instance and immutable connection identity. If socket or process evidence is missing or contradictory, retain unavailable or ambiguous instead of attributing from later final-client proof. Packet ScopeWithheld remains withheld. The independent connection result augments, not rewrites, packet evidence.

Rationale: FlowRegistry correlation occurs at connection close and may have no eligible packet observation before final-client acquisition. Alternatives of using session Basic authentication as process proof or retroactively relabeling withheld packets are unsound.

## Decision: release and recovery

Keep the proxy alive through the existing finite route-owner release window and ETW watch closure. Newly declared stages can be controlled only after exact process-object receipt, path, account, session, creation, and ancestry checks. Unapproved helpers remain exit obligations, including those born during shutdown; unresolved live instances yield partial cleanup and an exact Doctor prerequisite.

Rationale: The existing release mechanism already distinguishes declared owners from helpers and monitors late events. The missing authority is primarily target topology and trace projection. An unconditional kill of descendant PIDs would violate P-1.
