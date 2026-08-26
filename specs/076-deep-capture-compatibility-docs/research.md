# Phase 0 Research: Deep Capture Compatibility Documentation

## Target Matrix Source

**Decision**: Generate the target-specific matrix at runtime from the selected
target's local `deep_capture_facts` rows.

**Rationale**: Compatibility varies by launch case, backend, target version,
and freshness. The local store already preserves those dimensions. A static
game list would become stale, encourage guesses, and risk publishing local
test data.

**Alternatives considered**:

- Checked-in title matrix: rejected because it duplicates local evidence,
  cannot remain current, and conflicts with the privacy constraint.
- Catalog-derived verdict: rejected because platform and engine metadata do not
  prove proxy routing or trust behavior.
- One synthesized verdict per target: rejected because conflicting observations
  across launch cases and backends are legitimate evidence.

## Projection Ownership

**Decision**: Put the pure fact-to-matrix projection in `fragcap-targets` and
keep textual rendering in `fragcap-cli`.

**Rationale**: Freshness and ordering are target-domain semantics that future
read-only surfaces can share. Terminal labels and layout remain CLI concerns.
This uses the existing dependency direction and adds no crate edge.

**Alternatives considered**:

- Render `CompatibilityFact` values directly in the CLI: rejected because it
  would make stale-source handling and deterministic ordering presentation-only
  behavior.
- Add a second query model in the store: rejected because no new persistence
  shape or SQL aggregation is required.

## Ordering And Conflicts

**Decision**: Preserve every fact and sort by durable row identity when
available, with fact fields as a total fallback order for unsaved fixture rows.

**Rationale**: Stored row ids preserve chronology. A total fallback makes pure
tests deterministic without assigning semantic precedence to a fact source or
value. No row is selected as the winner.

**Alternatives considered**:

- Group by key and keep newest: rejected because it hides historical and
  launch-specific evidence.
- Rank evidence sources: rejected because source provenance is context, not an
  unconditional truth hierarchy.

## Freshness Semantics

**Decision**: A row is stale when its `stale` marker is true or its evidence
source is `stale-observation`. A non-stale row is current. An empty matrix is
unknown.

**Rationale**: This honors both storage mechanisms already present and prevents
an explicit stale source from appearing current because of an inconsistent
legacy marker. Unknown is a matrix state, not a fabricated fact row.

**Alternatives considered**:

- Derive staleness from timestamps or versions: rejected because the model has
  no universal expiry policy and silent age thresholds would invent meaning.
- Treat imported facts as stale by default: rejected because provenance and
  freshness are independent dimensions in the stored model.

## User-Facing Surface

**Decision**: Add the matrix to `fragcap targets show` and document it on a new
Deep Capture compatibility reference page. Keep the display read-only.

**Rationale**: `targets show` already resolves one local target and presents
its detail. The documentation site needs one stable location for the protocol
table and evidence legend. The existing `targets export` contract describes
target identity documents and is not expanded with local observations.

**Alternatives considered**:

- New `compatibility` command: rejected as command-surface expansion for data
  that naturally belongs to target detail.
- Add facts to target export: rejected because it changes a public schema and
  increases accidental publication risk.
- Probe on view: rejected because read-only inspection must not launch software
  or mutate proxy and trust state.

## Current Traffic Boundaries

**Decision**: Publish the following exact interpretation of shipped behavior.

| Traffic family | Current Deep Capture boundary |
| --- | --- |
| HTTP | HTTP method, URL, and response status are observable through the proxy; optional HAR uses those fields. Headers and bodies are not retained by the current application record. |
| HTTPS | Same HTTP semantics only when traffic reaches the proxy and accepts the local CA. Pinning is not bypassed. An optional proxy-owned key log is an analyzer aid. |
| WebSocket | The HTTP upgrade handshake can appear as HTTP semantics. WebSocket frame records and payload retention are not implemented. |
| Non-HTTP TLS | Proxy connection metadata only. No custom application dissection. |
| QUIC | Not routed or decrypted by the current HTTP proxy path. Capture can still contain UDP packets. |
| UDP | Not handled by the current proxy path. Capture can still contain packets and attribution. |
| Plaintext | Capture preserves packet bytes when payload capture is enabled. Plaintext HTTP follows the HTTP row; arbitrary plaintext protocols receive no generic application dissection. |

**Rationale**: These statements follow the S075 addon and application JSONL
writer rather than proxy-library capability in the abstract.

**Alternatives considered**:

- Describe all mitmproxy-supported features: rejected because fragcap does not
  currently ingest or emit all of them.
- Call HTTP `full payload inspection`: rejected because current records omit
  headers and bodies.

## Privacy

**Decision**: Use only generic placeholders in docs and tests. Do not render
free-form notes or final executable names in the compatibility matrix.

**Rationale**: The required matrix fields are key, value, launch case, source,
and freshness. Notes and executable names can carry local details and add no
necessary compatibility verdict.

**Alternatives considered**:

- Render every stored field: rejected because local provenance can include
  identifiers unnecessary for the user-facing matrix.
