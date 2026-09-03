<!-- spec-impact: 13.7, 17.2.1, 19, 25, 28.1 -->
Add explicit target-scoped proxy bypass policy for DNS domains, IP addresses,
CIDRs, ports, and IPv6. Bare and leading-dot domains both include descendants
so the reviewed policy matches conventional `NO_PROXY` behavior. Plans and bundles expose canonical
operator rules, exact listener infrastructure, complete child environment
ownership, DNS decision boundaries, and routing-decision accounting.
