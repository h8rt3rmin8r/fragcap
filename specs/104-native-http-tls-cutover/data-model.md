# S104 Data Model

## ProxySessionAccess

A borrowed post-start view over one live native lease. It contains a loopback endpoint, a redacted child launch route, and optional public trust material. It has no serialization, display, equality, clone-to-owned, or artifact representation.

## ProxyLaunchRoute

The exact selected-child proxy route: loopback endpoint, fixed authorization scheme and username, and borrowed capability proof. Its debug form reveals only the endpoint and the presence of authorization. It constructs environment values only at the managed-launch boundary.

## ProxyTrustMaterial

Public certificate DER plus generation, SHA-1 store thumbprint, and SHA-256 provenance fingerprint for the exact authority already owned by the live proxy. DER may be passed to current-user trust; private authority material never leaves the proxy owner.

## HttpConnection

One accepted client endpoint, proxy-local endpoint, session capability generation, monotonic connection id, request ordinal, current upstream authority if reusable, finite budgets, and terminal outcome. It owns every protocol child operation until joined or forcibly accounted.

## HttpMessageHead

Bounded raw start line and ordered header fields plus parsed method/status, target form, authority, version, framing decision, connection tokens, and transformation ledger. Exactly one framing state is valid: none, fixed length, chunked, response-to-HEAD, response-to-CONNECT, or close-delimited where allowed.

## HttpExchangeObservation

Session and connection identifiers, ordinal, client and proxy endpoints, protocol label, method, effective URL, optional status, inspectability, timestamps, transformation names, refusal/failure reason, and loss-accounting snapshot. It deliberately omits later full-header and body artifact claims.

## TlsBoundaryObservation

Boundary kind (`client` or `upstream`), requested authority, SNI when observable, negotiated version and application protocol, certificate identity/fingerprint metadata, resumed flag when observable, and one typed terminal stage. Client and upstream records cannot substitute for each other.

## SessionCertificateState

One generation, in-memory zeroized authority key, public DER and fingerprints, bounded leaf cache, TLS server policy, and exact trust ownership state. Rotation or runtime exit invalidates every leaf and capability.

## ProtocolAccounting

Accepted connections; authenticated and refused requests; HTTP requests and responses; CONNECT attempts; client and upstream TLS outcomes; parse and framing refusals; policy refusals; timeouts; cancellations; observation drops/truncations/projection gaps; completed, failed, and forced tasks. Terminal reconciliation accounts for every accepted connection and admitted observation.

## State Transitions

```text
listener:   configured -> bound -> capability/CA-ready -> accepting -> stopping -> released/residue
request:    head-limited -> authenticated -> framed -> policy-checked -> forwarded/refused -> observed
CONNECT:    authenticated -> authority-checked -> client-200 -> client-TLS -> upstream-TLS -> inner-HTTP -> closed
client TLS: waiting -> client-hello -> identity-issued -> negotiated/failed -> closed/incomplete
upstream:   declared -> resolved -> policy-checked -> connected -> verified/failed -> closed/incomplete
route:      borrowed -> selected-child-applied -> child-ended -> invalidated
trust:      absent/preexisting -> exact-added(optional) -> active -> exact-removed/retained-obligation
```
