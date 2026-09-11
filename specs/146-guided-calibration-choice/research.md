# Research: Explicit Guided Calibration Choice

## Decision 1: Use one content-derived candidate identity

Each discovery candidate receives `candidate-v1:<64 lowercase hexadecimal characters>`, derived with the repository's existing BLAKE3 dependency over a canonical JSON projection. The projection includes every candidate field that can affect registration or Steam client authoring. Evidence entries are sorted before serialization, so source order does not create a new identity.

The choice identity deliberately excludes the aggregate discovery account and warnings. Those describe the scan, not the candidate. The existing registration and Steam setup plans continue to bind the full current scan and are recomputed after confirmation.

Rejected alternatives:

- List positions change when discovery order changes and can silently redirect a choice.
- Steam app identifiers alone cannot distinguish conflicting metadata for the same app.
- Raw paths do not identify Steam candidates and make the command surface inconsistent.
- Persisting choice rows adds lifecycle and cleanup authority for an ephemeral narrowing operation.

## Decision 2: Treat duplicate canonical candidates as ambiguous corruption

One supplied identity must match exactly one current candidate. If two candidates canonicalize identically, the command reports duplicate authority and refuses. Deduplicating would make discovery accounting and provenance disappear, while selecting the first would reintroduce positional behavior.

## Decision 3: Use one candidate option at one boundary

`--candidate` may narrow either initial discovered-target registration or Steam client setup. The command tracks whether it was consumed and refuses an unused value before workflow creation. A choice cannot be applied twice, and `--resume` conflicts with it.

Stored-target ambiguity continues to use `--id`, which is already a durable exact selector. S146 does not introduce a second identity for stored rows.

## Decision 4: Persist case intent in the existing workflow row

Schema version 12 adds `selected_launch_case`, `routing_strategy`, and `address_family` columns to `calibration_workflows`. The workflow record contract remains version 1 because the columns are additive and have exact backward-compatible defaults. Routing and family are required; launch case remains optional because absence means the current topology authority may infer it. Existing version 11 stores migrate to child-environment, IPv4, and no launch assertion.

The target authority snapshot already captures the registration or Steam authoring result, so the ephemeral candidate digest does not need persistence.

## Decision 5: Make launch selection an assertion, not topology authority

The proposal continues to infer the only safe cold launch case from stored topology and current process state. An explicit launch case must equal that inferred cold case. It cannot convert a direct target into Steam, select a warm case for an effect, or rewrite a launch chain.

This intentionally deviates from the low-level diagnostic surface, which accepts a declared launch case because its operator is directly composing one exact calibration attempt. The guided front door owns discovery and must not let an advanced flag contradict it.

## Decision 6: Expose the closed routing vocabulary but preserve support checks

The guided CLI accepts every existing `CompatibilityRoutingStrategy` token. The proposal remains authoritative and currently admits only child-environment. This lets automation inspect and explicitly assert the dimension without making unimplemented routing executable.

Adding routing implementations belongs to their own slices because each needs scope, plan, evidence, cleanup, and recovery authority.

## Decision 7: Propagate IPv6 through existing S119 authority

`--proxy-family ipv6` maps to the existing `CompatibilityAddressFamily::Ipv6`, enters every proposal case, persists in the workflow, appears in guidance, and sets the low-level `DeepCaptureArgs.proxy_family`. No listener or networking implementation changes are required.

## Decision 8: Emit a dedicated pre-target choice event

Candidate ambiguity can occur before a stored target exists, so `calibration.guidance` cannot truthfully represent it. A new `calibration.choice_required` event carries scope, selector, optional target identifier, and candidate projections. Human output uses the same projections and prints the exact `--candidate` continuation.

## Decision 9: Keep S146 below non-Steam authoring and final completion

S146 resolves discovered target and existing Steam metadata choices. Direct and publisher targets participate when their stored launch declarations are already exact. Rewriting ambiguous non-Steam launch declarations remains separate because it changes durable target topology and needs its own confirmation contract. Parent #380 remains open after S146.
