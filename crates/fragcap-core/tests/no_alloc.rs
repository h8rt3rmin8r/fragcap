// SPDX-License-Identifier: Apache-2.0

//! FR-004 and SC-003: parsing performs no heap allocation, asserted rather
//! than intended.
//!
//! A separate integration binary because a global allocator is installed per
//! binary. Putting one in the unit test build would measure the test harness
//! as well as the parser, and the signal would be lost in the noise.
//!
//! The counter is thread-local rather than a global atomic for the same
//! reason: the harness runs tests concurrently, and a global counter would
//! record whatever else happened to be running. A thread-local counts only the
//! allocations made on the thread doing the parsing, which is exactly the
//! claim under test.
//!
//! The cell is const-initialized and has no destructor. A thread-local whose
//! first access initializes lazily can itself allocate, which would have the
//! allocator reenter the counter it is trying to update.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use fragcap_core::link::LinkType;
use fragcap_core::parse::{HeaderParser, InterfaceAddrs};

thread_local! {
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.with(|n| n.set(n.get() + 1));
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.with(|n| n.set(n.get() + 1));
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

fn allocations() -> usize {
    ALLOCATIONS.with(|n| n.get())
}

const ETHERTYPE_IPV4: u16 = 0x0800;
const ETHERTYPE_IPV6: u16 = 0x86dd;
const IPPROTO_TCP: u8 = 6;
const IPPROTO_UDP: u8 = 17;

const LOCAL_V4: Ipv4Addr = Ipv4Addr::new(192, 0, 2, 10);
const PEER_V4: Ipv4Addr = Ipv4Addr::new(198, 51, 100, 5);
const OTHER_V4: Ipv4Addr = Ipv4Addr::new(203, 0, 113, 7);

fn local_v6() -> Ipv6Addr {
    Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x10)
}

fn peer_v6() -> Ipv6Addr {
    Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 5)
}

// Frame builders. Duplicated from the crate's own test helpers rather than
// exported, because exporting test-only builders from the library would put
// them in the public surface for the sake of one test binary.

fn ethernet(ether_type: u16, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(14 + payload.len());
    out.extend_from_slice(&[0x02, 0, 0, 0, 0, 1, 0x02, 0, 0, 0, 0, 2]);
    out.extend_from_slice(&ether_type.to_be_bytes());
    out.extend_from_slice(payload);
    out
}

struct V4 {
    src: Ipv4Addr,
    dst: Ipv4Addr,
    proto: u8,
    ident: u16,
    frag_offset: u16,
    more_fragments: bool,
    option_words: u8,
}

impl Default for V4 {
    fn default() -> Self {
        V4 {
            src: LOCAL_V4,
            dst: PEER_V4,
            proto: IPPROTO_TCP,
            ident: 0,
            frag_offset: 0,
            more_fragments: false,
            option_words: 0,
        }
    }
}

fn ipv4(h: V4, payload: &[u8]) -> Vec<u8> {
    let words = 5 + h.option_words;
    let header_len = words as usize * 4;
    let mut out = vec![0u8; header_len];
    out[0] = 0x40 | words;
    // Consistent with the payload, so the corpus exercises the extent rule
    // rather than the unset-length fallback.
    out[2..4].copy_from_slice(&((header_len + payload.len()) as u16).to_be_bytes());
    out[4..6].copy_from_slice(&h.ident.to_be_bytes());
    let flags = if h.more_fragments { 0x2000u16 } else { 0 };
    out[6..8].copy_from_slice(&(flags | (h.frag_offset & 0x1fff)).to_be_bytes());
    out[8] = 64;
    out[9] = h.proto;
    out[12..16].copy_from_slice(&h.src.octets());
    out[16..20].copy_from_slice(&h.dst.octets());
    out.extend_from_slice(payload);
    out
}

fn ipv6(next: u8, tail: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; 40];
    out[0] = 0x60;
    out[4..6].copy_from_slice(&(tail.len() as u16).to_be_bytes());
    out[6] = next;
    out[7] = 64;
    out[8..24].copy_from_slice(&local_v6().octets());
    out[24..40].copy_from_slice(&peer_v6().octets());
    out.extend_from_slice(tail);
    out
}

fn extension(next: u8) -> Vec<u8> {
    let mut out = vec![0u8; 8];
    out[0] = next;
    out
}

fn tcp(src: u16, dst: u16) -> Vec<u8> {
    let mut out = vec![0u8; 20];
    out[0..2].copy_from_slice(&src.to_be_bytes());
    out[2..4].copy_from_slice(&dst.to_be_bytes());
    out[12] = 0x50;
    out
}

fn udp(src: u16, dst: u16, len: u16) -> Vec<u8> {
    let mut out = vec![0u8; 8];
    out[0..2].copy_from_slice(&src.to_be_bytes());
    out[2..4].copy_from_slice(&dst.to_be_bytes());
    out[4..6].copy_from_slice(&len.to_be_bytes());
    out
}

/// Every supported combination and every rejection cause.
///
/// The corpus is deliberately the whole contract, not the happy path. An
/// allocation on an error path is still an allocation on the capture thread,
/// and error paths are where a tempting `format!` gets written.
fn corpus() -> Vec<(LinkType, Vec<u8>)> {
    let mut long_chain = ipv6(0, &[]);
    for _ in 0..9 {
        long_chain.extend_from_slice(&extension(0));
    }
    long_chain.extend_from_slice(&tcp(1, 2));

    let mut bad_ihl = ethernet(ETHERTYPE_IPV4, &ipv4(V4::default(), &tcp(1, 2)));
    bad_ihl[14] = 0x44;

    vec![
        // Supported combinations.
        (
            LinkType::ETHERNET,
            ethernet(ETHERTYPE_IPV4, &ipv4(V4::default(), &tcp(51000, 443))),
        ),
        (
            LinkType::ETHERNET,
            ethernet(
                ETHERTYPE_IPV4,
                &ipv4(
                    V4 {
                        proto: IPPROTO_UDP,
                        ..V4::default()
                    },
                    &udp(30000, 5055, 8),
                ),
            ),
        ),
        (
            LinkType::ETHERNET,
            ethernet(
                ETHERTYPE_IPV4,
                &ipv4(
                    V4 {
                        option_words: 3,
                        ..V4::default()
                    },
                    &tcp(51000, 443),
                ),
            ),
        ),
        (
            LinkType::ETHERNET,
            ethernet(ETHERTYPE_IPV6, &ipv6(IPPROTO_TCP, &tcp(51000, 443))),
        ),
        (
            LinkType::ETHERNET,
            ethernet(ETHERTYPE_IPV6, &ipv6(IPPROTO_UDP, &udp(30000, 5055, 8))),
        ),
        (
            LinkType::ETHERNET,
            ethernet(ETHERTYPE_IPV6, &{
                // The whole chain is the payload, so the declared payload
                // length covers it. Appending after the builder would declare
                // a length that excludes what was appended.
                let mut tail = extension(60);
                tail.extend_from_slice(&extension(IPPROTO_TCP));
                tail.extend_from_slice(&tcp(51000, 443));
                ipv6(0, &tail)
            }),
        ),
        (LinkType::RAW, ipv4(V4::default(), &tcp(51000, 443))),
        (LinkType::RAW, ipv6(IPPROTO_TCP, &tcp(51000, 443))),
        (LinkType::NULL, {
            let mut p = 2u32.to_le_bytes().to_vec();
            p.extend_from_slice(&ipv4(V4::default(), &tcp(51000, 443)));
            p
        }),
        (LinkType::NULL, {
            let mut p = 24u32.to_be_bytes().to_vec();
            p.extend_from_slice(&ipv6(IPPROTO_TCP, &tcp(51000, 443)));
            p
        }),
        // Fragments: initial, subsequent, last.
        (
            LinkType::ETHERNET,
            ethernet(
                ETHERTYPE_IPV4,
                &ipv4(
                    V4 {
                        proto: IPPROTO_UDP,
                        ident: 4242,
                        more_fragments: true,
                        ..V4::default()
                    },
                    &udp(30000, 5055, 800),
                ),
            ),
        ),
        (
            LinkType::ETHERNET,
            ethernet(
                ETHERTYPE_IPV4,
                &ipv4(
                    V4 {
                        proto: IPPROTO_UDP,
                        ident: 4242,
                        frag_offset: 185,
                        more_fragments: true,
                        ..V4::default()
                    },
                    &[0xab; 16],
                ),
            ),
        ),
        (
            LinkType::ETHERNET,
            ethernet(
                ETHERTYPE_IPV4,
                &ipv4(
                    V4 {
                        proto: IPPROTO_UDP,
                        ident: 4242,
                        frag_offset: 370,
                        ..V4::default()
                    },
                    &[0xab; 16],
                ),
            ),
        ),
        // Every rejection cause.
        (LinkType::from_code(108), ethernet(ETHERTYPE_IPV4, &[0; 40])),
        (LinkType::ETHERNET, ethernet(0x8100, &[0; 40])),
        (LinkType::NULL, {
            let mut p = 99u32.to_le_bytes().to_vec();
            p.extend_from_slice(&[0x45; 40]);
            p
        }),
        (LinkType::RAW, vec![0x50; 40]),
        (LinkType::ETHERNET, bad_ihl),
        (LinkType::ETHERNET, ethernet(ETHERTYPE_IPV6, &long_chain)),
        (LinkType::ETHERNET, ethernet(ETHERTYPE_IPV6, &ipv6(59, &[]))),
        (
            LinkType::ETHERNET,
            ethernet(
                ETHERTYPE_IPV4,
                &ipv4(
                    V4 {
                        proto: 1,
                        ..V4::default()
                    },
                    &[0; 20],
                ),
            ),
        ),
        (
            LinkType::ETHERNET,
            ethernet(
                ETHERTYPE_IPV4,
                &ipv4(
                    V4 {
                        proto: IPPROTO_UDP,
                        ..V4::default()
                    },
                    &udp(1, 2, 4),
                ),
            ),
        ),
        (
            LinkType::ETHERNET,
            ethernet(ETHERTYPE_IPV4, &ipv4(V4::default(), &tcp(1, 2)[..2])),
        ),
        (
            LinkType::ETHERNET,
            ethernet(
                ETHERTYPE_IPV4,
                &ipv4(
                    V4 {
                        proto: IPPROTO_UDP,
                        ident: 777,
                        frag_offset: 185,
                        more_fragments: true,
                        ..V4::default()
                    },
                    &[0; 16],
                ),
            ),
        ),
        (
            LinkType::ETHERNET,
            ethernet(
                ETHERTYPE_IPV4,
                &ipv4(
                    V4 {
                        src: OTHER_V4,
                        dst: PEER_V4,
                        ..V4::default()
                    },
                    &tcp(1, 2),
                ),
            ),
        ),
        // Loopback, the ambiguous case.
        (
            LinkType::ETHERNET,
            ethernet(
                ETHERTYPE_IPV4,
                &ipv4(
                    V4 {
                        src: Ipv4Addr::LOCALHOST,
                        dst: Ipv4Addr::LOCALHOST,
                        ..V4::default()
                    },
                    &tcp(51000, 8080),
                ),
            ),
        ),
    ]
}

#[test]
fn parsing_the_whole_corpus_allocates_nothing() {
    // Everything that may allocate happens here, before the measurement.
    // Building the address set once is permitted; the requirement is that
    // parsing allocates nothing per packet.
    let corpus = corpus();
    let mut parser = HeaderParser::new(InterfaceAddrs::new([
        IpAddr::V4(LOCAL_V4),
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        IpAddr::V6(local_v6()),
    ]));
    // Touch the thread-local once so its own initialization, if any, is not
    // attributed to the parser.
    let _ = allocations();

    let before = allocations();
    for (link, frame) in &corpus {
        let _ = parser.parse(*link, frame);
    }
    let after = allocations();

    assert_eq!(
        after - before,
        0,
        "parsing {} frames performed {} allocations",
        corpus.len(),
        after - before
    );
}

#[test]
fn the_counting_allocator_actually_counts() {
    // Without this, a broken allocator that never increments would make the
    // test above pass for the wrong reason, which is the failure mode a
    // measurement harness is most prone to.
    let before = allocations();
    let v: Vec<u8> = Vec::with_capacity(4096);
    let after = allocations();
    assert!(
        after > before,
        "the harness must observe an allocation it knows happened"
    );
    drop(v);
}

#[test]
fn every_frame_in_the_corpus_reaches_a_defined_outcome() {
    // The corpus is only evidence for the allocation claim if it exercises
    // what it says it does. A frame that stopped parsing at the link layer
    // because a builder changed would silently shrink the coverage.
    let mut parser = HeaderParser::new(InterfaceAddrs::new([
        IpAddr::V4(LOCAL_V4),
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        IpAddr::V6(local_v6()),
    ]));
    let corpus = corpus();
    for (link, frame) in &corpus {
        parser.parse(*link, frame);
    }
    let stats = parser.stats();
    assert_eq!(
        stats.rejected(),
        12,
        "the corpus must reach all twelve rejection causes exactly once each"
    );
    assert_eq!(stats.direction_ambiguous, 1);
    assert_eq!(stats.fragment_evicted, 0);
    for (name, count) in [
        ("unsupported_link_type", stats.unsupported_link_type),
        ("unsupported_ether_type", stats.unsupported_ether_type),
        (
            "unsupported_address_family",
            stats.unsupported_address_family,
        ),
        ("unsupported_ip_version", stats.unsupported_ip_version),
        ("malformed_network_header", stats.malformed_network_header),
        ("extension_chain_too_long", stats.extension_chain_too_long),
        ("no_next_header", stats.no_next_header),
        ("unsupported_transport", stats.unsupported_transport),
        (
            "malformed_transport_header",
            stats.malformed_transport_header,
        ),
        ("short_header", stats.short_header),
        ("unmatched_fragment", stats.unmatched_fragment),
        ("no_local_endpoint", stats.no_local_endpoint),
    ] {
        assert_eq!(count, 1, "{name} was not exercised exactly once");
    }
}
