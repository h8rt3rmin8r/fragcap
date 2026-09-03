# Research: Complete Native Calibration Matrix

## Decision 1: Store the missing case dimensions on each compatibility row

**Decision**: Add nullable `routing_strategy`, `address_family`, and `protocol_family` columns with closed token constraints, then require all applicable fields on newly created observed rows.

**Rationale**: Existing fields already hold launch case, backend name and version, fragcap version, target version, provenance, and stale state. Keeping the missing dimensions on the same row preserves P-10 and makes every historical claim self-contained.

**Alternatives considered**: A separate calibration-runs table would create a second join and lifecycle for evidence. Reusing free-form `proxy_mode` would not distinguish routing strategy from backend configuration and would leave address family and protocol untyped. Backfilling old rows would invent facts.

## Decision 2: Put exact applicability in `fragcap-targets`

**Decision**: Define a typed current case and a fact applicability result in the crate that owns compatibility facts. Provide deterministic latest-applicable selection for prerequisite keys.

**Rationale**: CLI-only matching has already allowed launch-case-only eligibility. A shared authority prevents target detail, facade preflight, and tests from developing different meanings of current evidence.

**Alternatives considered**: Keeping matching in the CLI repeats policy outside the model owner. Moving stored vocabulary into the facade would invert the existing dependency direction.

## Decision 3: Use S120 traffic families as the protocol authority

**Decision**: Map the exhaustive S120 classification families into a closed storage vocabulary. Add `routing` for the operator-selected route-only case and `not-applicable` for protocol-independent fact rows.

**Rationale**: S120 already defines the durable distinction between HTTP versions, HTTP/3, streaming semantics, SOCKS transport forms, generic transports, QUIC, and unsupported states. A second coarser protocol list would recreate the ambiguity S120 removed.

**Alternatives considered**: Retaining the old `protocol-behavior` values alone merges HTTP/1, HTTP/2, SSE, and gRPC, and merges HTTPS with HTTP/3. Free-form strings would make typo-driven cross-promotion possible.

## Decision 4: Keep reachability and TLS as the effect phases

**Decision**: Add a selected protocol to calibration plans rather than adding a third session effect phase. Reachability stays trust-free; TLS retains explicit trust authorization. Both filter positive protocol facts to the selected family.

**Rationale**: The two phases represent distinct effect boundaries, not the full protocol vocabulary. Protocol selection is a case dimension orthogonal to those effects.

**Alternatives considered**: One phase per protocol would duplicate lifecycle code and obscure the only security-relevant distinction, whether trust may change.

## Decision 5: Treat legacy rows as visible, stale-for-eligibility evidence

**Decision**: Preserve legacy rows byte-for-value, classify their applicability as `legacy-incomplete`, and render that state. Never rewrite their stored stale bit or source.

**Rationale**: This preserves history while ensuring a row missing a newly required dimension cannot authorize a new exact case.

**Alternatives considered**: Deleting legacy rows loses evidence. Marking them stale in storage mutates history. Assuming defaults silently promotes evidence across dimensions.

## Decision 6: Target version is exact when both sides possess trustworthy evidence

**Decision**: Persist target version when the selected target supplies it. A known current target version requires equality. If the current target has no trustworthy version, a versioned historical row remains visible but cannot be promoted as exact current evidence.

**Rationale**: Issue #317 requires version evidence where available and prohibits promotion without proof. Unknown cannot equal known.

**Alternatives considered**: Ignoring target version permits stale game-build results. Requiring it unconditionally would make targets without a local build clue impossible to calibrate.
