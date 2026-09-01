# Contract: SOCKS5 Evidence And Routing

## Route

One `ProxyRoute` carries both the existing authenticated HTTP URL and an authenticated `socks5h` URL derived from the same endpoint and capability. Child environment routing resolves:

- `HTTP_PROXY` and `HTTPS_PROXY` to the HTTP URL
- `ALL_PROXY` to the SOCKS URL
- `NO_PROXY` to the empty session value
- `FRAGCAP_PROXY_AUTHORIZATION` to the existing HTTP authorization value

No debug or presentation surface reveals either secret-bearing URL or authorization value.

## Typed Events

The existing application and proxy lifecycle streams carry SOCKS negotiation, authentication outcome, CONNECT request and result, address form, DNS ownership, classification, directional bytes, and terminal state. Events use the existing session and connection identifiers and finite queue accounting.

## Correlation

The existing connection-open descriptor and exact open/close window remain the correlation authority. SOCKS events do not create a second connection identifier or infer process ownership.

## Declared Limits

Opaque TCP remains metadata-only. Non-HTTP TLS semantics, raw TCP payload evidence, UDP ASSOCIATE, generic UDP, QUIC/HTTP3, and complete IPv6 parity remain explicitly deferred to their owning issues.
