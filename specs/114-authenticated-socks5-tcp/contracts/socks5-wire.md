# Contract: SOCKS5 TCP Wire Behavior

## Negotiation

The server accepts version 5 greetings and selects only method `0x02`. It sends `0x05 0xff` when the method is absent. All reads are exact, bounded, and deadline-controlled.

## Authentication

The server accepts RFC 1929 version 1 with username `fragcap` and the current session capability password. It responds with version 1 and status zero on success, or status one on every refusal. No request byte is parsed before success.

## CONNECT

The server accepts command 1 with reserved byte zero and address types IPv4 (`0x01`), domain (`0x03`), and IPv6 (`0x04`). BIND and UDP ASSOCIATE receive command-not-supported. Unknown address types receive address-type-not-supported.

Domain bytes must be a valid destination name under the existing authority grammar. Resolution is proxy-owned and every result is policy checked.

## Replies

Success is sent only after upstream connect and carries the upstream socket's actual local address. Failures use the most specific truthful RFC 1928 result supported by observed error evidence. A failure never claims a tunnel.

## Forwarding

Client and upstream bytes are unchanged and ordered. Each direction has a fixed buffer. EOF half-closes its destination writer; the other direction continues. Idle timeout, cancellation, I/O failure, or bounded shutdown terminates the relay with exact byte counts and one terminal result.
