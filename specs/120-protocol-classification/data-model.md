# Data Model: Exhaustive Protocol Classification

## ClassificationSchema

- `version`: positive integer, version 1 for S120
- `matrix`: closed set of valid traffic-family, detection, inspectability, and reason combinations

Unsupported future versions are rejected by typed readers. The enclosing raw record remains readable where its artifact schema permits it.

## TrafficFamily

- `http1`
- `https`
- `http2`
- `websocket`
- `sse`
- `grpc`
- `generic-tcp`
- `non-http-tls`
- `socks5-tcp`
- `socks5-udp`
- `generic-udp`
- `quic`
- `http3`
- `unrouted`
- `unknown`

The family describes identified evidence, not an inferred port convention. `unknown` is explicit and never aliases an unsupported family.

## DetectionState

- `identified`: retained evidence identifies a supported family
- `unknown`: retained evidence cannot identify a family
- `unsupported`: retained evidence identifies a family or version outside the supported matrix
- `failed`: a supported parser or protocol operation was attempted and failed

## InspectabilityState

- `full`: supported application semantics were observed
- `metadata-only`: only supported metadata boundaries were observed
- `decrypted-unknown`: decrypted bytes were retained without supported application semantics
- `encrypted-opaque`: encrypted bytes were retained without decryption
- `packet-only`: traffic remained outside the routed proxy evidence path
- `unavailable`: no application evidence boundary was observed

Inspectability does not describe forwarding success or artifact publication.

## OutcomeReason

- `not-routed`
- `not-reached`
- `encrypted-opaque`
- `certificate-pinned`
- `client-auth-required`
- `unsupported-version`
- `parser-failed`
- `truncated`
- `writer-failed`

Raw detailed reason text remains separate. Stable reasons are categories, not replacements.

## ProtocolClassification

- `schema_version`: classification schema version
- `family`: `TrafficFamily`
- `detection`: `DetectionState`
- `inspectability`: `InspectabilityState`
- `reason`: optional `OutcomeReason`

Validation rules:

- Full and metadata-only require an identified supported family.
- Decrypted-unknown requires unknown detection or an identified generic transport boundary.
- Encrypted-opaque requires the matching stable reason.
- Packet-only requires `unrouted` plus `not-routed`.
- Unsupported requires `unsupported-version` or another matrix-declared unsupported boundary.
- Failed requires direct failure evidence such as `parser-failed`.
- `writer-failed` and `truncated` are reported by artifact or retention authorities and cannot rewrite successful protocol detection.

## ClassificationEvidence

- `raw_protocol`: exact native proxy protocol label when present
- `raw_inspectability`: exact native proxy inspectability label when present
- `raw_reason`: exact detailed error or refusal label when present
- `classification`: validated `ProtocolClassification`

## CompatibilityEligibility

- `fact_key`: routing, propagation, TLS trust, protocol behavior, or inspectability
- `required_classification`: explicit predicate over the classification axes
- `required_context`: phase, final-client correlation, controlled-harness identity, or launch ancestry
- `eligible`: derived Boolean

Failures and omissions remain observations even when ineligible for positive fact promotion.

## ClassificationSummary

- `observations`: retained classified observation count
- `by_family`: counts keyed by stable family label
- `by_detection`: counts keyed by stable detection label
- `by_inspectability`: counts keyed by stable inspectability label
- `by_reason`: counts keyed by stable reason label
- `unclassified_lost`: bounded observation-loss count whose missing content cannot be classified

Conservation rule:

```text
sum(by_detection) = observations
sum(by_inspectability) = observations
classified records + unclassified_lost = total accepted observation identities
```
