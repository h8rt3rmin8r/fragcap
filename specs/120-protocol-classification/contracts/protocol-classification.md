# Protocol Classification Contract

## Versioned Record

Every applicable classification is serialized as an additive object:

```json
{
  "classification": {
    "schema_version": 1,
    "family": "http2",
    "detection": "identified",
    "inspectability": "full",
    "reason": null
  }
}
```

The enclosing application and manifest schema versions remain authoritative for their records. Classification schema version 1 defines only the classification object.

## Required Stable Labels

Traffic families:

```text
http1
https
http2
websocket
sse
grpc
generic-tcp
non-http-tls
socks5-tcp
socks5-udp
generic-udp
quic
http3
unrouted
unknown
```

Detection states:

```text
identified
unknown
unsupported
failed
```

Inspectability states:

```text
full
metadata-only
decrypted-unknown
encrypted-opaque
packet-only
unavailable
```

Required outcome reasons:

```text
not-routed
not-reached
encrypted-opaque
certificate-pinned
client-auth-required
unsupported-version
parser-failed
truncated
writer-failed
```

## Authority Rules

- Raw proxy records own transport, TLS, protocol, parser, and refusal evidence.
- Application records own retained metadata, content, transformation, truncation, and writer reconciliation.
- Manifest entries own artifact production, omission, completeness, sensitivity, and severity.
- Compatibility policy owns promotion of observed evidence into durable fact candidates.
- CLI summaries are derived and own no new facts.

One authority may reference another authority's stable classification, but cannot replace or reinterpret its detailed evidence.

## Reader Rules

- Readers accept classification schema version 1.
- Readers reject unsupported future classification versions as classifications.
- Where the enclosing artifact remains readable, readers retain the raw record and report classification as unavailable rather than inventing a downgrade.
- Missing classification on legacy application records remains an explicit legacy absence, not `unknown`.

## Compatibility Eligibility

- Positive protocol behavior requires `identified` and a published supported traffic family.
- Positive inspectability requires `identified` with `full` or `metadata-only`, or an explicit generic transport state with retained evidence.
- Positive TLS trust requires the existing final-client or controlled-harness proof plus a full HTTPS classification.
- Routing and propagation keep their existing correlation and launch-ownership requirements.
- Unknown, unsupported-version, parser-failed, truncated, writer-failed, not-routed, and not-reached cannot independently create positive facts.

## Summary Rules

- Every retained classified observation contributes exactly one detection and one inspectability count.
- A stable reason contributes at most one reason count per classified observation.
- Lost observation content contributes only to `unclassified_lost`.
- Human and JSON output render the same derived summary instance.
