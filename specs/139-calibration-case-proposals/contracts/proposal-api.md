# Guided Calibration Proposal API Contract

## Stable boundary

The reviewed `fragcap::deep_capture::api` inventory adds the request, process snapshot, topology kind, launch readiness, limitation, evidence reason, proposal step, deferred protocol, proposal, and pure `propose_calibration` entry point. New enums are non-exhaustive where downstream matching must remain forward compatible.

## Request construction

```text
CalibrationProposalRequest::new(target, backend_name, backend_version, fragcap_version)
    .with_target_version(optional_version)
    .with_process_snapshot(CalibrationProcessSnapshot::complete(images))
    .with_facts(facts)
    .with_protocol_candidates(protocols)
    .with_routing_strategy(optional_exact_override)
    .with_address_family(optional_exact_override)
```

The default snapshot is unavailable, routing is child environment, and family is IPv4. The constructor does not read the target store, process table, filesystem, clock, network, or trust store.

## Proposal invariants

- `Ready` is present only with a supported exact cold launch case.
- `OperatorAction` names one shipped warm case, its cold counterpart, and the complete image set to close normally.
- `Unavailable` has zero runnable steps.
- Any ready proposal lacking current unconflicted `reached-client` routing has exactly one reachability step.
- Protocol steps appear only when current unconflicted routing is positive.
- Protocol steps and deferred protocols are sorted by stable protocol token and contain no duplicates.
- Every step has one complete S121 case and one stable reason.
- Proposal construction never writes or invokes an effect adapter.

## Stable reason tokens

```text
missing
stale
legacy-incomplete
context-mismatch
negative
conflict
reachability-required
```

## Stable limitation tokens

```text
invalid-context
invalid-steam-anchor
missing-launch-declaration
ambiguous-launch-declaration
invalid-publisher-chain
process-inventory-unavailable
unsupported-routing-strategy
unsupported-protocol-candidate
```

Details and candidate lists may expand, but a limitation never becomes a runnable attempt implicitly.

## Compatibility

This additive API does not change existing `SessionConfig`, calibration execution, ordinary eligibility, fact storage, artifacts, or CLI arguments. Later #380 work must consume this contract rather than duplicate its policy.
