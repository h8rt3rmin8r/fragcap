# Contract: Target-Scoped Routing

## Preparation

Preparation is side-effect free and produces one immutable route plan. The plan declares strategy kind, availability, symbolic secret sources, exact destinations, verification rules, and cleanup obligations.

## Authorization

Authorization names the exact session plan id. Any plan mismatch refuses before proxy, trust, route, or launch effects.

## Application

The routing adapter applies only an implemented strategy. For child environment it resolves session proxy values into a secret-bearing launch environment without mutating parent or machine-wide state.

## Verification

The lease returns one of:

- `reached-socket-owner`
- `not-reached`
- `escaped-tree`
- `ambiguous`
- `unavailable`
- `not-attempted`

The outcome cites observed connection, process, and flow anchors where available.

## Cleanup

Every applied effect has one terminal cleanup disposition. Future target-owned configuration must compare current content to the authorized replacement before restoring exact original bytes.
