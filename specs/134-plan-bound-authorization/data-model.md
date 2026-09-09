# Data Model: Plan-Bound Deep Capture Authorization

## AuthorizationPlan

One immutable, versioned, canonically serialized review value.

| Field | Meaning | Authorization rule |
| --- | --- | --- |
| schema | Authorization plan contract version | A version change changes the identifier |
| session_id | Pending session identity | Exact and single-use |
| target | Stable id, local row id, handle, and selected name | Any identity change refuses |
| launch | Requested case, observed case, exact stored authority, resolved profile, executable paths, platform root, and dispatch | Any state, declaration, or live resolution change refuses |
| proxy | Backend and product versions, loopback family, route strategy, normalized bypass, environment ownership, and no-fallback boundary | Scope may be realized only inside these fields |
| trust | None, or current-user Root action, SHA-1 thumbprint, SHA-256 fingerprint, validity, and cleanup promise | Exact prepared CA must reach runtime and trust adapter |
| artifacts | Bundle path, packet output, application stream, optional HAR, optional key log, and sensitivity | Selection or path change refuses |
| deadlines | Exact millisecond launch, observation, shutdown, and cleanup bounds | Any bound change refuses |
| facts | Possible append-only compatibility fact families | No undeclared family may be written |
| cleanup | Listener, proxy, route, launch, trust, private material, writers, journal, and retained-evidence obligations | Existing exact lifecycle remains authoritative |
| refusals | No system proxy, no wildcard endpoint, no pinning bypass, no unrelated target, and no silent fallback | A realization that needs widening refuses |
| plan_id | `plan-v1:` plus the canonical plan digest | Derived with this field omitted |

## PreparedNativeAuthority

Process-local ownership of the exact certificate authority named by the authorization plan.

| Field | Visibility | Lifecycle |
| --- | --- | --- |
| generation | Public summary | Created before plan emission, retained through runtime start |
| DER certificate | Internal and trust adapter | Never persisted before authorization |
| SHA-1 thumbprint | Public plan and trust-store identity | Compared exactly through cleanup |
| SHA-256 fingerprint | Public provenance identity | Included in canonical plan |
| private signing key | Secret, zeroizing | In memory until runtime cleanup or decline drop |
| validity | Public plan | Fixed at generation and included in plan id |

## AuthorizationInput

One response associated with one emitted plan.

| Mode | Accepted value | Refusal values |
| --- | --- | --- |
| Interactive | One documented affirmative answer after the complete plan | Negative, unknown, EOF, I/O failure, interrupt |
| Dedicated standard input | Exact complete `plan_id` line | Missing, whitespace-altered, abbreviated, case-changed, stale, replayed, extra data |
| Structured output without dedicated input | None | Usage refusal, no prompt |

## AuthorizationOutcome

| State | Meaning | Effects allowed |
| --- | --- | --- |
| authorized | Exact current plan accepted | Realization may begin |
| declined | Operator answered negatively | None |
| invalid | Response did not match the required form or id | None |
| closed | Input ended before exact approval | None |
| interrupted | Cancellation arrived during authorization | None |
| drifted | Mutable authority no longer matches | None |
| expired | Prepared certificate authority reached its displayed expiry | None |
| realization-failed | Post-authorization listener or adapter preparation stayed within scope but failed | Only already recorded post-authorization cleanup |

## State Transitions

```text
validated request
  -> exact target and launch snapshot
  -> process-local CA preparation
  -> canonical plan and plan_id
  -> pending prior recovery refusal | plan emitted and flushed
  -> declined | invalid | closed | interrupted
  -> exact approval
  -> launch-authority and prepared-authority revalidation
  -> drifted | expired | facade final target revalidation
  -> drifted | exact listener reservation
  -> library Authorization::Approved(plan_id)
  -> existing checked Deep Capture lifecycle
```

No declined branch creates a bundle, listener, route, launch, artifact, fact, journal, trust entry, or cleanup obligation.
