# Research: Native Blind Relay

## Existing boundaries

- S191's `InspectionDecider` publishes a frozen `Inspect` or `Uninspected` result before native protocol handling. Runtime currently turns `Uninspected` into `inspection-relay-unavailable` before authentication.
- HTTP/1 CONNECT already authenticates and checks destination policy before sending `200`, then creates a Fragcap leaf. The new relay branch belongs at that exact boundary.
- SOCKS5 TCP already has bounded byte-transparent forwarding but emits generic payload chunks. UDP already pins the association and contacted peers, while QUIC Initial enters the session-CA gateway. Both need a decision-controlled non-exporting path.
- Cleartext HTTP/2 presently bridges one authority but accepts only `http` URIs. CONNECT needs a dedicated stream path with exact authority and bounded ownership.
- The native `ConnectionDiagnostic` already records decision and terminal stages. Its exported projection can carry relay totals without inventing application exchanges.

## Alternatives considered

1. **Hostname bypass**: rejected because launcher and game can contact the same origin, and destination identity is not owner proof.
2. **Let a failed TLS inspection fall back to passthrough**: rejected because it changes a frozen decision and creates a false success path.
3. **A second listener for excluded traffic**: rejected because it duplicates authentication, destination, and route-owner obligations.
4. **Single proxy with action-specific forwarding**: selected because it preserves the existing admission and lifecycle authority.

## Open implementation facts

The public CLI remains launch-chain until #561. Tests inject a deterministic decision provider and controlled origin. Installed-product behavior is observed after release; a concrete defect returns as a new issue.
