**2026-08-08** Recorded for promotion to specification section 29: section 12.5
requires that subsequent IP fragments be attributed by their fragment
identifier and address pair, which presupposes a memory it does not describe.
Slice S03 defines one: a fixed 256 entry table, drop-oldest, holding the
protocol and ports the first fragment carried, with the eviction counted and
the entry removed when the datagram's last fragment is observed. It is bounded
by entry count rather than by age because an age bound needs a clock, and a
clock in `fragcap-core` is a platform surface constitution P-2 excludes.

**2026-08-08** Recorded for promotion to specification section 29: section 12.6
defines three of the four combinations of endpoint locality and is silent on
the fourth, a packet with neither endpoint on the capturing host. S03 makes it
a counted rejection producing no flow key, on the reasoning that section 8.4
defines the key's local field as the endpoint on the capturing host, so putting
an arbitrary endpoint there would assert something untrue. It would also buy
nothing: such a packet has no local socket, so no socket table lookup could
ever resolve the key. The practical consequence is that a stale or empty
interface address set announces itself once per packet instead of yielding a
capture full of keys that resolve to nothing.

**2026-08-08** Recorded for promotion to specification section 29: the residual
mis-attribution risk from IPv4 fragment identifier reuse. The identifier is
sixteen bits, and a host that fragments heavily can reuse one for the same
address pair and protocol before the earlier table entry has been removed.
Removing an entry when its datagram's last fragment is observed shortens the
window and the 256 entry bound caps it, but neither eliminates it, and it is
not detectable from the capture so it cannot be counted. It is stated rather
than claimed away, which is what constitution P-9 requires when P-4's mechanism
does not reach.

**2026-08-08** Corrected in `fragcap-core`: link type code 0 was documented as
having no link layer header. That is code 101's property. Code 0 is BSD
loopback encapsulation, which prefixes a four byte host-order address family
value. The error was harmless while nothing parsed and would have had S03's
parser read an IP version nibble out of an address family field, rejecting
every loopback frame and attributing the failure to the wrong cause. Recorded
because a comment that misdescribes an observation is the kind of small
inaccuracy a later parser inherits.
