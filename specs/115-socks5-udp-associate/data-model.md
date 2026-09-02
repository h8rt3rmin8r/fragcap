# Data Model: Scoped SOCKS5 UDP Association

## SocksRequest

- command: CONNECT or UDP ASSOCIATE
- address type: IPv4, IPv6, or domain
- authority: validated destination or expected client endpoint

## UdpAssociation

- existing session and connection identifiers
- immutable TCP peer IP
- client UDP port: declared, unlearned, or learned once
- client-facing loopback socket
- fixed IPv4 and IPv6 upstream sockets
- finite peer set of exact socket addresses
- idle deadline, counters, peak peer count, and terminal state

## SocksUdpDatagram

- reserved: exactly zero
- fragment: exactly zero for supported traffic
- destination address form and port
- payload offset and length into the fixed receive buffer

## UdpPeerMapping

- exact normalized remote socket address
- requested address form
- proxy-owned DNS flag
- last successful send instant

## State Transitions

```text
authenticated -> request -> validate claim -> bind sockets -> success reply -> active
                    |              |              |                 |
                    +--------------+--------------+-----------------+-> refused

active -> validate client -> parse -> resolve/policy -> map/send -> await exact reply -> frame/send
   |            |            |             |             |                 |
   +------------+------------+-------------+-------------+-----------------+-> counted drop

active -> control EOF / idle / cancellation / failure -> clear mappings -> drop sockets -> terminal
```
