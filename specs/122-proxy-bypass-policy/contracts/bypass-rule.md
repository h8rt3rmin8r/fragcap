# Contract: Bypass Rule Grammar

## Accepted tokens

| Input | Meaning | Canonical form |
| --- | --- | --- |
| `Example.COM.` | DNS domain including apex and descendants, any port | `.example.com` |
| `.Example.COM` | Same DNS domain semantics | `.example.com` |
| `example.com:443` | DNS domain including descendants, port 443 | `.example.com:443` |
| `192.0.2.10` | Exact IPv4, any port | `192.0.2.10` |
| `192.0.2.10:443` | Exact IPv4, port 443 | `192.0.2.10:443` |
| `2001:db8::10` | Exact IPv6, any port | `2001:db8::10` |
| `[2001:db8::10]:443` | Exact IPv6, port 443 | `[2001:db8::10]:443` |
| `192.0.2.0/24` | IPv4 network, any port | `192.0.2.0/24` |
| `2001:db8::/32` | IPv6 network, any port | `2001:db8::/32` |

Repeated CLI values and comma-delimited tokens are flattened before parsing. Empty elements are invalid.

## Refused tokens

- `*`
- schemes, paths, queries, fragments, or user-info
- non-ASCII DNS names
- empty or malformed DNS labels
- zero, missing, or out-of-range qualified ports
- unbracketed IPv6 with an apparent port
- CIDR with host bits
- port-qualified CIDR
- the exact selected listener endpoint as an operator rule

## Matching

- DNS comparison is ASCII case-insensitive after canonicalization.
- One trailing root dot is ignored.
- `example.com` and `.example.com` both match `example.com` and `a.example.com`, not `notexample.com`.
- Qualified ports match only the named port.
- IP and CIDR use canonical address identity, including IPv4-mapped IPv6 normalization.
- First-input order has no semantic effect; canonical specificity ordering determines the matching rule reported in evidence.
