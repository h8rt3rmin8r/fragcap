# Contract: SOCKS5 UDP Wire Behavior

- UDP ASSOCIATE uses SOCKS version 5, command `0x03`, reserved `0x00`, and an IPv4 or IPv6 client endpoint claim. Domain-form client claims are refused because the immutable TCP peer IP is the client identity; IPv4, IPv6, and proxy-resolved domain forms remain supported for datagram destinations.
- A success reply is written only after the relay and fixed upstream sockets exist; BND.ADDR and BND.PORT identify the client-facing relay.
- The TCP control connection remains open and exclusively owns the association.
- Client datagrams use `RSV(2) | FRAG(1) | ATYP(1) | DST.ADDR | DST.PORT | DATA`.
- `RSV != 0`, `FRAG != 0`, truncated fields, invalid domains, port zero destinations, unknown ATYP, and oversized datagrams are dropped with distinct accounting.
- Response datagrams use the same header and name the actual upstream source endpoint.
- Control EOF revokes forwarding before any later datagram can be admitted.
