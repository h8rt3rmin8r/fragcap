# Data Model: Complete IPv6 Parity

## LoopbackFamily

- `Ipv4`: exact host `127.0.0.1`
- `Ipv6`: exact host `::1`

The family is selected before reservation. There is no automatic runtime fallback because fallback would change the authorized route.

## LoopbackEndpoint

- `address`: exact IPv4 or IPv6 loopback `SocketAddr`

Validation rules:

- The IP must be loopback after mapped-address canonicalization.
- The IP must not be unspecified, wildcard, multicast, or externally reachable.
- The runtime bind address, route URLs, resource target, lifecycle record, and plan address are identical.
- Port zero is permitted only as allocator input; an authorized prepared plan carries the exact reserved port.

## DestinationAuthority

- `host`: DNS name, IPv4 literal, or IPv6 literal
- `scope_id`: optional numeric interface index for a scoped IPv6 literal
- `port`: nonzero transport port
- `tls_identity`: DNS or bare IP identity with no scope

Validation rules:

- IPv6 literals are bracketed.
- A scope is decimal, nonzero, bounded to `u32`, and attached only to link-local or multicast IPv6.
- Credentials, missing ports, empty zones, named zones, and ambiguous colons are refused.

## AddressCandidate

- `observed`: exact resolver or literal socket address
- `canonical`: mapped-address-normalized identity
- `family`: IPv4 or IPv6
- `ordinal`: stable position after family interleaving
- `policy`: allowed or refused with stable reason

Canonical candidates are unique. The observed value remains evidence even when canonical identity is IPv4.

## UpstreamAttemptSet

- `candidates`: finite ordered allowed candidate vector
- `stagger`: 250 ms
- `deadline`: one absolute connect deadline
- `cancellation`: session cancellation token
- `winner`: absent or one selected candidate and connected stream
- `failures`: bounded per-attempt terminal results

State transitions:

```text
planned -> attempting -> won -> losers-cancelled -> returned
                    |-> exhausted -> failed
                    |-> cancelled
                    `-> timed-out
```

Application forwarding begins only after `losers-cancelled`.

## FamilyReadiness

- `family`: IPv4 or IPv6
- `state`: ready, unavailable, failed, or undetermined
- `endpoint`: exact loopback probe address
- `detail`: observed success or failure reason

Each family produces its own Doctor check and JSON record.

## Address-Family Evidence

Existing socket-bearing records retain exact `SocketAddr` text. Where a selected upstream peer was previously implicit, additive fields carry:

- `listener_endpoint`
- `client_endpoint`
- `selected_upstream_peer`
- `upstream_local_endpoint`
- `address_family`

Absent facts remain absent. No record derives a selected peer from the requested authority when no socket connected.
