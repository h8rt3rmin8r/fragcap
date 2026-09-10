# Data Model: Guided Calibration Case Proposals

## Calibration Proposal Request

| Field | Meaning | Validation |
| --- | --- | --- |
| target | One already resolved stored target | Existing `TargetEntry`; no second identity form |
| process snapshot | Present image names or explicit unavailable state | Cold only when complete |
| backend | Native backend name and version | Both non-empty |
| fragcap version | Product version used by fact applicability | Non-empty |
| target version | Exact target build clue when available | Preserved as optional |
| routing override | Exact requested routing strategy | Defaults to child environment; other currently declared strategies refuse |
| family override | Exact loopback family | Defaults to IPv4; IPv4 and IPv6 accepted |
| protocol candidates | Concrete observed or explicitly selected protocols | Routing and not-applicable are limitations, not attempts |
| facts | Existing append-only compatibility rows for the selected target | Read-only |

## Declared Calibration Topology

| Kind | Required identity | Valid shape |
| --- | --- | --- |
| Steam | Numeric `steam:` anchor, `steam.exe`, one client | Exactly one Windows launch entry with absent role or role `client` |
| Direct | One client image | Exactly one Windows launch entry with absent role or role `client` |
| Publisher | Ordered stages | At least two entries, all roles present and unique, first `launcher`, last `client`, no conflicting image roles |

Malformed, missing, and multiply plausible declarations produce a limitation carrying the preserved candidate images. They never become a topology kind.

## Process Snapshot

The snapshot has two states:

- **Complete**: a case-insensitive set of present process image names. Every required topology image absent proves cold.
- **Unavailable**: a stable limitation plus caller detail. It proves neither warm nor cold.

Repeated image names do not change the result. Declared spelling is retained in output.

## Launch Readiness

```text
topology unavailable
  -> unavailable limitation, zero steps

topology known + snapshot unavailable
  -> unavailable limitation, zero steps

topology known + any required image present
  -> operator action(warm case, cold case, all declared images), zero steps

topology known + every required image absent
  -> ready(cold case)
```

The publisher warm case is `publisher-launcher-game-start-clean-warm` only when the root is present and the terminal client is absent. Every other non-cold publisher snapshot is `publisher-launcher-warm`.

## Evidence Assessment

Each fact class and exact case produces one reason:

| Reason | Condition |
| --- | --- |
| conflict | Current exact rows have more than one distinct value |
| negative | Current exact rows agree, but not on the class's positive value |
| stale | No current exact row; an otherwise exact row is explicitly stale |
| legacy-incomplete | No current or stale exact row; a relevant row lacks required dimensions |
| context-mismatch | Relevant rows exist only for another exact context |
| missing | No relevant row exists |

Positive routing is `reached-client`. Positive protocol inspectability is `full`. An unconflicted positive produces no step.

## Proposal Step

Each step carries a phase, one complete `CompatibilityCase`, and the evidence reason. Reachability uses protocol `routing`. Protocol candidates use the existing TLS calibration phase because the shipped calibration state machine has reachability and TLS as its two bounded operations.

## Deferred Protocol

When reachability is not currently established, each valid protocol candidate is retained with `reachability-required`. Deferred protocols are sorted by their stable closed-set token and are never represented as runnable work.

## State Transitions

```text
request
  -> validate context and overrides
  -> derive topology
  -> evaluate complete snapshot
  -> ready | operator action | unavailable
  -> assess exact routing evidence
  -> route step + deferred protocols | protocol assessment
  -> deterministic proposal
```

No transition performs or authorizes an external effect.
