# Contract: Routing Policy and Decision Evidence

## Plan projection

Every prepared plan exposes:

- `policy_version`: `1`
- `operator_rules`: canonical ordered rule strings
- `infrastructure`: exact listener-derived exclusions
- `environment_variables`: the uppercase and lowercase proxy variables owned by the plan
- `dns_matching`: `requested-authority-before-resolution`
- `resolved_address_policy`: `evaluate-every-answer-every-attempt`
- `fallback`: `none`

## Decision record

Each localized record exposes:

- requested destination
- outcome: `proxied`, `bypassed`, `infrastructure`, `refused`, or `undetermined`
- authority: `operator-policy`, `session-infrastructure`, `proxy-destination-policy`, or `evidence-reconciliation`
- canonical matching rule or null
- stable reason
- proxy loss delta, which is always zero for `bypassed`

## Summary

The summary exposes counts for all five outcomes. Their sum reconciles the authority's localized decisions and explicitly unlocalized observations. It does not claim that traffic absent from both proxy and packet/process evidence was observed.

## Compatibility

Raw `proxy.jsonl`, `application.jsonl`, pcapng, and JSON Lines packet formats remain unchanged. Plan, compatibility, and manifest fields are additive.
