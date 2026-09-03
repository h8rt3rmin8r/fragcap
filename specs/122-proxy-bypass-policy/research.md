# Research: Proxy Bypass and Local-Destination Policy

## Existing authority boundaries

`RoutingPlan` already owns immutable managed-child environment effects. `DestinationPolicy` already canonicalizes IPv4-mapped addresses, refuses the listener, refuses local and non-public destinations, and permits exact controlled-test sockets. The CLI currently supplies an empty uppercase `NO_PROXY`, which clears one ambient variable but does not model policy, lowercase variants, or evidence.

Decision: extend those authorities rather than add a second proxy implementation. The facade owns requested-destination bypass semantics. The proxy owns resolved-address safety.

## Rule grammar

Decision: support repeated or comma-separated conventional tokens with a closed interpretation:

- `example.com` for exact DNS;
- `.example.com` for DNS suffix including the apex;
- `example.com:443` for port-qualified exact DNS;
- `192.0.2.10` and `192.0.2.10:443` for IPv4;
- `2001:db8::10` and `[2001:db8::10]:443` for IPv6;
- `192.0.2.0/24` and `2001:db8::/32` for CIDR.

Rejected alternatives:

- An unrestricted pass-through string cannot produce deterministic matching or evidence.
- A new tagged DSL is clearer internally but would not be an equivalent proxy bypass input and would create needless operator syntax.
- Wildcard `*` is refused because it disables the declared path.
- Port-qualified CIDR is refused because conventional projections do not preserve it consistently.

## Normalization and matching

Decision: lowercase ASCII DNS, strip one trailing root dot, validate labels, canonicalize mapped IPv6 to IPv4, require canonical CIDR network addresses, validate nonzero ports, deduplicate, and sort by canonical text. Exact DNS compares a whole name. Suffix compares the apex or a dot-delimited descendant. IP and CIDR compare canonical address bytes. A port qualifier adds an exact port predicate.

Unicode hostnames are refused because the repository has no IDNA authority and silently inventing one would make scope client-dependent.

## Environment ownership

Decision: write both uppercase and lowercase forms of `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, and `NO_PROXY`. The value for each case variant comes from the same authorized plan. This replaces inherited values rather than attempting to inspect or merge them.

The authorization secret remains only in `FRAGCAP_PROXY_AUTHORIZATION`; no lowercase alias is added because it is a fragcap-owned contract, not a conventional inherited proxy variable.

## Infrastructure projection

Decision: append the exact selected listener authority to the environment projection as a built-in token, separate from operator rules in plan and evidence. The native proxy also refuses the canonical listener socket, which is the enforcement boundary if a client ignores bypass variables. A host-only listener token is deliberately omitted because it would broaden bypass to unrelated services on the same loopback address.

`localhost` is not treated as interchangeable with the listener because its resolution is machine-dependent and broad name bypass could hide unrelated services. Exact loopback aliases are evaluated by their canonical socket identity at the proxy boundary.

## DNS rebinding

Decision: never cache a public-policy conclusion. Bypass evaluates the original requested name. Proxied DNS resolution remains bounded, and each returned address is checked against `DestinationPolicy` before connect. Mixed answers may allow a public candidate but never a private candidate. If no allowed candidate connects, no direct fallback is attempted.

## Evidence

Decision: add canonical operator policy, infrastructure exclusions, environment ownership, and stable matching semantics to the prepared plan and compatibility/bundle metadata. Where the existing route observation can localize a requested destination, report the outcome and reason. When it cannot, report undetermined rather than infer. Bypass never increments proxy loss.

## Dependencies

Decision: implement IP/CIDR parsing and matching with `std::net`; no dependency or lockfile change is justified.
