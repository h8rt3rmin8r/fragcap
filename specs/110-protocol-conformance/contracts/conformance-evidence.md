# Contract: Native HTTP and TLS Conformance Evidence v1

## Files

- `conformance/native-http-tls/matrix-v1.json` is the immutable required-row declaration.
- `conformance/native-http-tls/report-v1.json` is the normalized committed observation.
- `conformance/native-http-tls/analyzer.pcapng` is the synthetic analyzer input.
- `conformance/native-http-tls/tls-keylog.log` is the synthetic standard key-log input.

## Required row semantics

A required row passes only when all conditions hold:

1. Its id is unique and declared in the matrix.
2. Its client and origin references resolve to distinct role-correct implementation identities.
3. Its expected and observed values match exactly.
4. Every named executable test ran and passed on the row's required tier.
5. Every named artifact assertion ran and passed.
6. Its status is exactly `pass`.

Skip, ignored, filtered-out, not-run, duplicate, missing, unexpected-pass, and unexpected-failure states are failures. They never contribute to protocol, client, origin, artifact, or tier coverage.

## Implementation independence

Coverage counts unique `driver_lineage` values, not display names or ids. A protocol meets the independent-peer rule only with at least two passing required client lineages and two passing required origin lineages.

## Artifact reconciliation

Integrated rows must validate:

- application JSON Lines framing, trailer, counts, loss, connection and stream identity;
- HAR entries, omissions, sizes, timings, body facts, and correlation extensions;
- client-facing TLS key-log syntax and upstream key-log exclusion;
- packet-flow correlation state and exact unavailable or ambiguous reasons;
- proxy lifecycle and cleanup lifecycle readable prefixes, trailers, gaps, and counts;
- cleanup summary derivation from cleanup lifecycle;
- resource journal terminal obligations;
- manifest version 2 role authority, sensitivity, completeness, loss, correlation, and omissions.

## Analyzer gate

The dedicated analyzer tier must invoke an unmodified `tshark` executable with the committed pcapng input and the TLS key-log preference. It must record the exact TShark version, exit successfully, emit a nonzero frame count, and match every field assertion declared by the matrix. Absence of the executable is a gate failure.

## Sanitization

Committed evidence must reject capability proofs, proxy authorization values, Basic credentials, private key blocks, current-user absolute paths, non-loopback endpoints, and nondeterministic timestamps or ephemeral ports. Public synthetic certificates and synthetic key-log secrets are allowed only in the analyzer fixture and must be labeled synthetic.
