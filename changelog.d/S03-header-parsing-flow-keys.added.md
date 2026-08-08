`fragcap-core` parses link, network, and transport headers into a flow key and
a direction, per specification sections 12.5 and 12.6. Ethernet, raw IP, and
BSD loopback encapsulations; IPv4 and IPv6 including the extension header
chain; TCP and UDP. Zero-copy, and asserted allocation-free under a counting
allocator rather than merely intended.

This is the first behavior in the workspace. Everything before it declared
shapes.

The accounting is the half worth reading. Twelve named rejection causes, each
with its own counter and each separated from the others exactly where the
remedy differs: a short header means raise the snapshot length, a malformed
header means a broken sender or a defect here, an unsupported EtherType means
unexpected traffic, an unsupported link type means an unexpected capture
backend. The enumeration is closed, so adding a way to decline without adding a
counter does not compile. No parse outcome is a drop, and a test asserts that
every parse counter leaves both drop totals at zero.

Two cases are reported rather than resolved, on purpose. Loopback traffic has a
local source and a local destination, so section 12.6's rule returns two
answers; fragcap produces the flow key, leaves the direction undetermined, and
counts it, because guessing would be right half the time with no indication of
which half. A packet with no local endpoint at all produces no flow key,
because a flow key's local field is defined as the endpoint on the capturing
host and there is not one.

IP fragments are attributed without reassembly, from a 256 entry table of what
each datagram's first fragment said. fragcap does not reassemble and will not:
doing it during capture would destroy the on-wire fidelity that makes the
capture worth taking.

Reads are bounded by the datagram's extent rather than by the captured frame.
The two differ in both directions and each needs its own answer. A declared
length longer than the capture is truncation, usually a snapshot length, and
the capture wins. A declared length shorter than the capture means the frame
carries bytes that are not the datagram, because Ethernet pads anything below
sixty bytes, and the declared length wins. A declared length of zero is neither
and is not an error: large send offload leaves the field for the adapter to
fill in after the capture point, which is ordinary for outbound traffic
captured on the sending host.
