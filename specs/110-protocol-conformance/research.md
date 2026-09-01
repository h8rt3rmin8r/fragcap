# Research: Native Protocol Conformance

## R-1: A closed data matrix is the gate

**Decision**: Store a versioned matrix and normalized report, then compute required coverage and row outcomes from those records.

**Rationale**: Test discovery alone is open-ended. A renamed or deleted test can disappear without proving that the required protocol row disappeared. A closed matrix makes absence, duplication, skip state, implementation lineage, standards, versions, and tier ownership reviewable and mechanically enforceable.

**Rejected**: Treat all existing tests as the matrix. They do not name expected versus observed outcomes or prove that the required set is complete.

## R-2: Independence is protocol-driver lineage

**Decision**: Count two peers as independent only when their protocol driver is separately implemented or comes from a distinct library lineage. Configuration variants and aliases do not count.

**Rationale**: The acceptance criterion is intended to catch self-consistent defects. Two wrappers around the same helper reproduce the same parser and serializer mistakes.

**Rejected**: Count TLS version, sync versus async scheduling, or two function names as independent implementations.

## R-3: Standards are attached to each row

**Decision**: Map rows to RFC 9110 and RFC 9112 for HTTP semantics and HTTP/1.1, RFC 9113 for HTTP/2, RFC 6455 and RFC 8441 for WebSocket, the HTML Server-Sent Events processing model, the gRPC HTTP/2 protocol contract, and TLS 1.2 and TLS 1.3 standards.

**Rationale**: HTTP conformance includes both shared semantics and version-specific messaging. The RFC Editor describes HTTP/1.1 as message syntax, parsing, and connection management, while HTTP/2 defines a distinct framing layer. WebSocket over HTTP/2 is separately standardized by RFC 8441. Sources: [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html), [RFC 9112](https://www.rfc-editor.org/rfc/rfc9112.html), [RFC 9113](https://www.rfc-editor.org/rfc/rfc9113.html), [RFC 6455](https://www.rfc-editor.org/rfc/rfc6455.html), [RFC 8441](https://www.rfc-editor.org/rfc/rfc8441.html), and [gRPC guides](https://grpc.io/docs/guides/).

## R-4: Production readers validate integrated evidence

**Decision**: Generate bounded application observations through the shipped facade and verify every artifact with the same reader, schema, or reconciliation logic used by product code.

**Rationale**: A parallel test-only JSON interpretation could agree with its own generator while production readers fail. The matrix records which production authority validated each assertion.

**Rejected**: Compare only text snapshots or assert that files exist.

## R-5: TShark is a dedicated required CI tier

**Decision**: Run TShark as an external analyzer in one supported Ubuntu CI job, pass the committed key-log file through the standard TLS preference, require nonzero packets and declared protocol fields, and treat missing TShark as failure.

**Rationale**: TShark is Wireshark's command-line analyzer and documents capture-file reading and field extraction. It establishes interoperability with an unmodified analyzer without making Wireshark a product dependency. Source: [TShark manual](https://www.wireshark.org/docs/man-pages/tshark.html).

**Rejected**: Use fragcap's own pcapng parser as proof of Wireshark compatibility, or silently skip when the analyzer is absent.

## R-6: Evidence is normalized, synthetic, and secret-free

**Decision**: Commit deterministic row results and analyzer fixtures. Strip timestamps, ephemeral ports, temporary paths, capability proofs, authorization fields, private keys, and live host identity before comparison.

**Rationale**: Reviewable evidence must not leak session capability or user material and must not churn on every run. Exact version identities and semantic outcomes remain.

**Rejected**: Commit a real operator bundle or raw runtime logs.

## R-7: No new product package

**Decision**: Use existing exact-pinned libraries and separately implemented wire harnesses. Add no product runtime dependency.

**Rationale**: S110 validates the stack selected in S102. Changing that stack during conformance would move the subject under test. If a test-only library is later proven necessary, it requires an explicit dependency decision before adoption.

## R-8: Correct the stale transport assignment

**Decision**: S110 closes only #305. Generic transport support remains under milestone 3 issues #310 through #318.

**Rationale**: Issue #305 and master specification section 28 define the milestone 2 conformance gate. The S109 prose calling #305 generic transports is internally contradictory and must not expand scope.
