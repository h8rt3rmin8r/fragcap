# Contract: IPv6 Parity

## Listener Contract

One prepared session authorizes one exact loopback socket. The native runtime must bind that socket or fail. It may not substitute another family, wildcard, port, or interface.

## Route Contract

- IPv4: `http://fragcap:<capability>@127.0.0.1:<port>` and equivalent SOCKS URL.
- IPv6: `http://fragcap:<capability>@[::1]:<port>` and equivalent SOCKS URL.
- Capability material remains redacted from debug and evidence.

## Authority Contract

Accepted forms include `example.test:443`, `192.0.2.1:443`, `[2001:db8::1]:443`, and scoped local literals with a decimal index. A scope is socket-local and excluded from TLS server name and HTTP authority forwarding.

## Race Contract

Allowed canonical candidates are finite and family-interleaved. Attempt zero starts immediately. Attempt `n` becomes eligible after `n * 250 ms`, bounded by the one connect deadline. First success wins. Dropping all other attempt futures closes any loser sockets before the winning stream is returned to application code.

## Evidence Contract

Observed sockets retain their family and scope. Canonical identity is used only for security comparison, ownership, deduplication, flow identity, and correlation. A successful record names the actual selected peer. A failed record carries no selected peer.

## Doctor Contract

Doctor emits separate `Deep Capture / IPv4 loopback listener` and `Deep Capture / IPv6 loopback listener` checks in both human and newline-delimited JSON reports. Each is based on its own exact ephemeral bind attempt.
