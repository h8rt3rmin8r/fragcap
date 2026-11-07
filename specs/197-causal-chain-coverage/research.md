# Research: Causal Launch-Chain Coverage

## R1: Packet membership

**Decision**: Add one internal managed-chain packet membership based on exact session-created process generations. Publish its intervals to the existing gate, role stamper, and endpoint filter. Profile roles remain application-stage metadata rather than the full packet eligibility set.

**Rationale**: `CaptureScope::All` includes unrelated machine traffic; current target scope and role intervals omit undeclared descendants. Exact ancestry plus process generation meets #570 and #571 without changing ordinary Capture.

**Alternatives considered**: All-interface capture, requiring every helper in a profile, and PID or basename membership. Each fails scope or unexpected-descendant requirements.

## R2: Early proof and ordering

**Decision**: Reconsider only the existing writer-held head packet while the single section 12.4 buffer holds later packets. A finite proof wait and managed-session byte ceiling bound this path; existing buffer eviction and any byte-cap loss remain counted. Saturation cannot produce a complete-coverage claim.

**Rationale**: The writer drops a packet synchronously when the boolean gate rejects it, before a later process event can establish ownership. Holding only its current head and allowing the existing bounded fan-in to queue later packets preserves order without adding the second buffer rejected in S09. Delayed decisions retain original timestamps across interfaces.

**Alternatives considered**: Immediate irreversible rejection, a second buffer or unlimited all-traffic spool, or out-of-order emission. They lose eligible packets, violate the single-buffer contract or retain unbounded unrelated content, or alter observation order.

## R3: Packet and route lifetime

**Decision**: Keep managed Deep Capture's packet pipeline active after the client observation interval while exact shutdown and route release run, then finalize once at verified release or the finite deadline. Ordinary Capture retains its lifecycle.

**Rationale**: The live pipeline already starts before managed launch but stops and joins at observation end, before release. Extending `--duration` alone changes the observation clock and still fails to reconcile route lifetime.

**Alternatives considered**: Reopen a second packet file, claim proxy-only traffic is complete packet evidence, or prolong client observation. Each changes the requested contract.

## R4: Gate and correlation reasons

**Decision**: Replace the boolean-only gate result with a typed admission or rejection reason and carry it into counters, `FlowOmissionCause`, and application correlation.

**Rationale**: The gate counts watch, window, scope, and unresolved paths, but the pipeline currently labels every rejected flow `ScopeWithheld`. A later reporter cannot reconstruct the actual per-flow reason from aggregate counts.

**Alternatives considered**: Infer reason from counters or retain the blanket label. Both misstate specific connections.

## R5: Managed helper control

**Decision**: The pre-effect plan authorizes exact causal descendants, including undeclared helpers. Acquire bounded process-object receipts for observed starts, prove generation-aware ancestry and path/account/session/receipt, and pass only proven live instances into the existing reverse-depth graceful/force sequence with exit verification.

**Rationale**: Current shutdown paths and receipt capture cover only declared stages. Other descendants are observation obligations even when owned. The field report cannot establish why six helpers lacked receipts, so missing proof must remain distinct from missed owned instances.

**Alternatives considered**: Same-image control, PID-only inference, broader process rights, or treating local route removal as process exit. All would weaken exact authority or report false release.

## R6: Scope facts and resume

**Decision**: Change all `auto` resolvers to full-chain inspection, advance policy identity, and compare resumed stored effective scope, middleware handling, and policy identity to current resolution before effects. Derive attempt progress identity from its exact case.

**Rationale**: Existing compatibility rows already carry these dimensions, so no SQLite column is needed. Current resume re-resolves `auto` without checking stored effective policy, and progress hardcodes old policy IDs.

**Alternatives considered**: Rewrite historical facts, reuse old IDs, or add a duplicate column. These destroy provenance or permit stale qualification.

## R7: Boundaries

**Decision**: #572 direct-socket routing and #574 client-facing TLS refusals remain separate. S197 reports them truthfully under the new default.

**Rationale**: Child-environment routing does not redirect a direct raw socket, and the field TLS categories do not establish their cause. Neither may become a positive inspection fact through S197.
