# HTTP/TLS Observation Contract

## HTTP/1.1

Every forwarded or refused request produces one ordered native observation containing session id, connection id, request ordinal, endpoints, timestamp, method when parsed, effective URL when derived, protocol, inspectability, and result. Every received final response adds status. Informational responses are relayed and counted without being substituted for the final response.

The transformation ledger names only proxy-required changes, including credential removal and connection-specific field removal. Original bounded head bytes remain raw observation payload where configured. S104 does not project complete headers, cookies, queries, trailers, or bodies into the application schema; those remain #296 and #297.

## CONNECT and HTTPS

CONNECT produces a request observation plus distinct client and upstream TLS boundary observations. Each successful boundary records the requested identity, negotiated TLS version, and ALPN bytes; a failed boundary keeps those facts absent rather than inferred. A complete HTTPS application observation requires all of:

1. authorized CONNECT;
2. client handshake under the exact session leaf;
3. verified upstream handshake for the requested authority;
4. a parsed and forwarded inner HTTP request; and
5. a final HTTP response.

If any item is absent, inspectability and reason reflect the last direct observation. Silence alone is `inconclusive`, never `certificate-pinned`.

## Conservation

```text
accepted client connections
  = completed + failed + forced

admitted raw observations
  = emitted + queued + dropped_oldest
```

Truncation, refusal, unparsed input, and projection gaps are orthogonal named counters. A report with any unexplained difference or required reporting failure is not complete.

## Compatibility

Existing coarse `CompatibilityObservation` fields remain populated from native observations so the v0.8 bundle and fact selectors continue to work. Backend identity changes to `fragcap-native` with the workspace version. No S104 record claims HTTP/2, WebSocket frame, streaming body, key-log, HAR completeness, or cross-artifact correlation support.
