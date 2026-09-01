# Data Model: Authenticated SOCKS5 TCP Routing

## SocksGreeting

- `version`: must be 5
- `methods`: one to 255 advertised method octets
- Selected method is username/password (`0x02`) or no-acceptable-methods (`0xff`)

## SocksAuthentication

- `version`: must be 1
- `username`: one-byte length plus bounded octets, must equal `fragcap`
- `password`: one-byte length plus bounded capability text
- State: `accepted`, `malformed`, or `refused`
- Secret fields are never emitted

## SocksConnectRequest

- `version`: 5
- `command`: CONNECT only
- `reserved`: zero
- `address`: IPv4, IPv6, or validated domain
- `port`: nonzero TCP port
- `dns_owner`: `proxy` for domain, `not-required` for literal addresses

## SocksReply

- `result`: exact SOCKS5 reply code
- `bound_address`: upstream local address after success, unspecified address for refusal
- State transition: a tunnel exists only after a success reply is fully written

## SocksTunnel

- Existing `session_id` and `connection_id`
- Client peer and proxy local endpoint
- Requested authority and address form
- Classification: `http`, `tls`, or `tcp-opaque`
- Client-to-upstream and upstream-to-client byte counts
- Open and close timestamps
- Terminal: complete, refused, timed out, cancelled, protocol error, transport error, or forced shutdown

## SocksProtocolAccounting

- Negotiations, authentication accepted/refused
- CONNECT requested/succeeded/refused
- Address forms by type
- DNS attempted/failed and policy refused
- Classified HTTP/TLS/opaque
- Directional forwarded bytes
- Parse, timeout, cancellation, transport failure, and observation loss totals

## SocksRoute

- Endpoint shared with HTTP proxy
- Scheme `socks5h`
- Username fixed to `fragcap`
- Password derived from the current capability
- Secret-bearing value available only to effect adapters

## State Transitions

```text
accepted -> greeting -> authentication -> request -> upstream -> reply -> classified -> forwarding -> terminal
             |              |              |          |        |                         |
             +--------------+--------------+----------+--------+-------------------------+-> refused/failed
```

No transition before authenticated request may perform DNS, connect, classification, or forwarding work.
