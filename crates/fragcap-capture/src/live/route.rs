// SPDX-License-Identifier: Apache-2.0

//! Which interface carries the default route.
//!
//! Specification section 12.1's second precedence step needs this, and libpcap
//! does not answer it: `pcap_findalldevs` reports addresses and flags, not
//! routing.
//!
//! The answer here is to ask the operating system's own routing table the only
//! question it will answer without a platform API, which is "if I were to send
//! to this address, which of my addresses would you send it from". Binding a UDP
//! socket and calling `connect` performs exactly that lookup and binds the local
//! end to the result, which `local_addr` then reports.
//!
//! **`connect` on a UDP socket transmits nothing.** There is no handshake, no
//! datagram, and nothing reaches the wire. It is a route lookup with a side
//! effect on a socket this function then throws away, which is why it is
//! compatible with a tool whose entire posture is that it observes rather than
//! acts.
//!
//! The alternative was `GetBestRoute2` through `windows-sys`. It would be the
//! direct answer and would add a platform dependency plus a second major version
//! of `windows-sys` to the graph, since `pcap` pins the 0.36 line. Three lines
//! of `std::net` answer the same question on every target section 28 has in
//! view, which is worth more than directness here.

use std::net::{IpAddr, UdpSocket};

/// Addresses used only as a routing question. Nothing is sent to either.
///
/// Documentation-range addresses from RFC 5737 and RFC 3849, chosen so that a
/// reader auditing this file can confirm at a glance that no real host is
/// involved. They are off-link on any ordinary machine, which is the only
/// property the lookup needs.
const V4_PROBE: &str = "192.0.2.1:9";
const V6_PROBE: &str = "[2001:db8::1]:9";

/// The source address the routing table would choose for an off-link
/// destination, or `None` when the machine has no route to one.
///
/// `None` is a real answer rather than a failure: a machine with no default
/// route has no interface for section 12.1's second step to select, and the
/// selection then declines rather than capturing on nothing.
pub fn default_route_source() -> Option<IpAddr> {
    // IPv4 first, because a machine with both usually carries game traffic on
    // it, and because a machine with only IPv6 still gets an answer below.
    probe("0.0.0.0:0", V4_PROBE).or_else(|| probe("[::]:0", V6_PROBE))
}

fn probe(bind: &str, destination: &str) -> Option<IpAddr> {
    let socket = UdpSocket::bind(bind).ok()?;
    socket.connect(destination).ok()?;
    socket.local_addr().ok().map(|addr| addr.ip())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Deliberately not asserting that an address is found. A build machine may
    // legitimately have no route, and a test that failed there would be
    // asserting something about the runner rather than about fragcap.
    //
    // What is asserted is that the function answers rather than hanging or
    // panicking, and that whatever it returns is an address this machine could
    // plausibly own.
    #[test]
    fn the_lookup_answers_without_panicking() {
        match default_route_source() {
            Some(IpAddr::V4(v4)) => {
                assert!(
                    !v4.is_multicast(),
                    "a routing table never chooses a multicast source"
                );
            }
            Some(IpAddr::V6(v6)) => {
                assert!(!v6.is_multicast());
            }
            None => {
                // A machine with no route. Nothing to assert, and nothing
                // wrong.
            }
        }
    }

    #[test]
    fn a_destination_that_cannot_be_routed_yields_no_answer() {
        // The unspecified address is not a routable destination anywhere, so
        // this exercises the `None` path on a machine that does have a route.
        assert_eq!(probe("0.0.0.0:0", "0.0.0.0:0"), None);
    }
}
