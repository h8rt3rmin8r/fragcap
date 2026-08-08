// SPDX-License-Identifier: Apache-2.0

//! The fixture corpus of specification section 25.3: its generator, its drift
//! check, and the assertions that hold it to its own description.
//!
//! The generator is the readable record of what each fixture contains. The
//! `.pcap` is its output, and a committed binary nobody can read is a test
//! input nobody can review.
//!
//! Regenerate with:
//!
//! ```text
//! FRAGCAP_UPDATE_FIXTURES=1 cargo test -p fragcap-capture --test corpus
//! ```
//!
//! Then read the diff. A regenerated fixture whose diff nobody looked at is the
//! same defect as a golden file updated without looking, which specification
//! section 25.4 names for the goldens and which applies here for the same
//! reason.
//!
//! Without the variable this target checks instead of writing, and runs as part
//! of the ordinary gate, so drift is caught by `cargo xtask ci` rather than by
//! remembering to run something.

use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};

use fragcap_capture::pcap::PcapReader;
use fragcap_core::link::LinkType;
use fragcap_core::packet::RawPacket;
use fragcap_core::parse::{HeaderParser, InterfaceAddrs, ParseOutcome};

// ---------------------------------------------------------------------------
// Constants. Every byte of the corpus traces to one of these, which is what
// FR-032a means by no ambient input: no clock, no filesystem, no environment.
// ---------------------------------------------------------------------------

/// The instant every fixture starts at, in seconds since the Unix epoch. A
/// constant rather than a clock reading, because a fixture that varies between
/// runs is a golden comparison that cannot be trusted.
const BASE_SECS: u32 = 1_700_000_000;

/// Payload filler. One byte, repeated, chosen to be obvious in a hex dump and
/// to be nothing: no fixture payload carries meaning, so no fixture payload can
/// carry an account identifier or a session token.
const FILLER: u8 = 0xa5;

/// Locally administered, so they cannot collide with a real manufacturer's
/// assignment.
const MAC_LOCAL: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x01];
const MAC_PEER: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x02];

const HOST_V4: Ipv4Addr = Ipv4Addr::new(192, 0, 2, 10);
const PEER_V4: Ipv4Addr = Ipv4Addr::new(198, 51, 100, 5);
const LOOPBACK_V4: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

const ETHERTYPE_IPV4: u16 = 0x0800;
const ETHERTYPE_IPV6: u16 = 0x86dd;
const ETHERTYPE_VLAN: u16 = 0x8100;
const IPPROTO_TCP: u8 = 6;
const IPPROTO_UDP: u8 = 17;
const IPPROTO_ICMP: u8 = 1;
const EXT_HOP_BY_HOP: u8 = 0;
const EXT_DEST_OPTS: u8 = 60;
const NO_NEXT_HEADER: u8 = 59;

const MAX_FIXTURE_BYTES: usize = 64 * 1024;
const MAX_CORPUS_BYTES: usize = 256 * 1024;

fn host_v6() -> Ipv6Addr {
    Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x10)
}

fn peer_v6() -> Ipv6Addr {
    Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 5)
}

/// Whether this run writes the corpus rather than checking it.
fn updating() -> bool {
    std::env::var_os("FRAGCAP_UPDATE_FIXTURES").is_some()
}

fn fixtures_dir() -> PathBuf {
    // The manifest directory rather than the working directory, so the path
    // holds however the test is invoked.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
}

// ---------------------------------------------------------------------------
// Frame and file construction.
// ---------------------------------------------------------------------------

fn filler(n: usize) -> Vec<u8> {
    vec![FILLER; n]
}

fn ethernet(ether_type: u16, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(14 + payload.len());
    out.extend_from_slice(&MAC_PEER);
    out.extend_from_slice(&MAC_LOCAL);
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
    /// Override the header length nibble, for the malformed fixture.
    bad_ihl: bool,
}

impl V4 {
    fn new(src: Ipv4Addr, dst: Ipv4Addr, proto: u8) -> Self {
        V4 {
            src,
            dst,
            proto,
            ident: 0,
            frag_offset: 0,
            more_fragments: false,
            bad_ihl: false,
        }
    }
}

fn ipv4(h: &V4, payload: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; 20];
    out[0] = if h.bad_ihl { 0x44 } else { 0x45 };
    out[2..4].copy_from_slice(&((20 + payload.len()) as u16).to_be_bytes());
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

fn ipv6(src: Ipv6Addr, dst: Ipv6Addr, next: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; 40];
    out[0] = 0x60;
    out[4..6].copy_from_slice(&(payload.len() as u16).to_be_bytes());
    out[6] = next;
    out[7] = 64;
    out[8..24].copy_from_slice(&src.octets());
    out[24..40].copy_from_slice(&dst.octets());
    out.extend_from_slice(payload);
    out
}

/// One eight-octet extension header naming what follows.
fn extension(next: u8) -> Vec<u8> {
    let mut out = vec![0u8; 8];
    out[0] = next;
    out
}

fn tcp(src_port: u16, dst_port: u16) -> Vec<u8> {
    let mut out = vec![0u8; 20];
    out[0..2].copy_from_slice(&src_port.to_be_bytes());
    out[2..4].copy_from_slice(&dst_port.to_be_bytes());
    out[12] = 0x50;
    out
}

fn udp(src_port: u16, dst_port: u16, payload_len: usize) -> Vec<u8> {
    let mut out = vec![0u8; 8];
    out[0..2].copy_from_slice(&src_port.to_be_bytes());
    out[2..4].copy_from_slice(&dst_port.to_be_bytes());
    out[4..6].copy_from_slice(&((8 + payload_len) as u16).to_be_bytes());
    out
}

/// A capture under construction: link type plus timestamped frames.
struct Capture {
    link_type: u32,
    records: Vec<(u32, u32, Vec<u8>)>,
}

impl Capture {
    fn new() -> Self {
        Capture {
            link_type: 1,
            records: Vec::new(),
        }
    }

    /// Append a frame at `offset_micros` after the base instant.
    fn at(&mut self, offset_micros: u32, frame: Vec<u8>) -> &mut Self {
        self.records.push((
            BASE_SECS + offset_micros / 1_000_000,
            offset_micros % 1_000_000,
            frame,
        ));
        self
    }

    /// Serialize as little-endian microsecond-resolution classic pcap.
    fn to_pcap(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&0xa1b2_c3d4u32.to_le_bytes());
        out.extend_from_slice(&2u16.to_le_bytes());
        out.extend_from_slice(&4u16.to_le_bytes());
        out.extend_from_slice(&0i32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&65_535u32.to_le_bytes());
        out.extend_from_slice(&self.link_type.to_le_bytes());
        for (secs, micros, frame) in &self.records {
            out.extend_from_slice(&secs.to_le_bytes());
            out.extend_from_slice(&micros.to_le_bytes());
            out.extend_from_slice(&(frame.len() as u32).to_le_bytes());
            out.extend_from_slice(&(frame.len() as u32).to_le_bytes());
            out.extend_from_slice(frame);
        }
        out
    }
}

/// Nanoseconds since the epoch for an offset from the base instant, for writing
/// script windows that agree with the capture they describe.
fn at_nanos(offset_micros: u64) -> i64 {
    BASE_SECS as i64 * 1_000_000_000 + (offset_micros as i64) * 1_000
}

// ---------------------------------------------------------------------------
// The eight fixtures.
// ---------------------------------------------------------------------------

/// A fixture and the script that describes it.
struct Fixture {
    name: &'static str,
    pcap: Vec<u8>,
    script: String,
    /// Addresses the capturing host holds, for the condition assertions.
    local: Vec<IpAddr>,
}

fn corpus() -> Vec<Fixture> {
    vec![
        tcp_session(),
        udp_gameplay(),
        ipv6_mixed(),
        fragmented(),
        loopback(),
        malformed(),
        port_reuse(),
        burst(),
    ]
}

/// Ordinary TCP flow, both directions.
fn tcp_session() -> Fixture {
    let mut c = Capture::new();
    for i in 0..6u32 {
        let outbound = i % 2 == 0;
        let (src, dst, sp, dp) = if outbound {
            (HOST_V4, PEER_V4, 51_000, 443)
        } else {
            (PEER_V4, HOST_V4, 443, 51_000)
        };
        let mut body = tcp(sp, dp);
        body.extend_from_slice(&filler(16));
        c.at(
            i * 5_000,
            ethernet(
                ETHERTYPE_IPV4,
                &ipv4(&V4::new(src, dst, IPPROTO_TCP), &body),
            ),
        );
    }
    Fixture {
        name: "tcp-session",
        pcap: c.to_pcap(),
        script: concat!(
            "# Ordinary TCP flow, both directions. One owner throughout.\n",
            "flow tcp 192.0.2.10:51000 198.51.100.5:443 always owner 4242 game.exe\n",
            "endpoint tcp 192.0.2.10:51000\n",
        )
        .to_string(),
        local: vec![IpAddr::V4(HOST_V4)],
    }
}

/// Sustained UDP flow at gameplay cadence.
fn udp_gameplay() -> Fixture {
    let mut c = Capture::new();
    // Sixteen milliseconds is a sixty hertz tick, which is the cadence the
    // focal titles send at.
    for i in 0..24u32 {
        let outbound = i % 2 == 0;
        let (src, dst, sp, dp) = if outbound {
            (HOST_V4, PEER_V4, 30_000, 5_055)
        } else {
            (PEER_V4, HOST_V4, 5_055, 30_000)
        };
        let mut body = udp(sp, dp, 32);
        body.extend_from_slice(&filler(32));
        c.at(
            i * 16_000,
            ethernet(
                ETHERTYPE_IPV4,
                &ipv4(&V4::new(src, dst, IPPROTO_UDP), &body),
            ),
        );
    }
    Fixture {
        name: "udp-gameplay",
        pcap: c.to_pcap(),
        script: concat!(
            "# Sustained UDP at a sixty hertz tick. A UDP entry names no remote\n",
            "# endpoint, because the UDP socket table carries none.\n",
            "flow udp 192.0.2.10:30000 * always owner 4242 game.exe\n",
            "endpoint udp 192.0.2.10:30000\n",
        )
        .to_string(),
        local: vec![IpAddr::V4(HOST_V4)],
    }
}

/// IPv6 with extension header chains.
fn ipv6_mixed() -> Fixture {
    let mut c = Capture::new();
    // Two bare, two behind chains, so the fixture exercises both the presence
    // and the absence of extension headers.
    let mut bare = tcp(51_000, 443);
    bare.extend_from_slice(&filler(16));
    c.at(
        0,
        ethernet(
            ETHERTYPE_IPV6,
            &ipv6(host_v6(), peer_v6(), IPPROTO_TCP, &bare),
        ),
    );

    let mut chained = extension(EXT_DEST_OPTS);
    chained.extend_from_slice(&extension(IPPROTO_TCP));
    chained.extend_from_slice(&tcp(51_000, 443));
    chained.extend_from_slice(&filler(16));
    c.at(
        5_000,
        ethernet(
            ETHERTYPE_IPV6,
            &ipv6(host_v6(), peer_v6(), EXT_HOP_BY_HOP, &chained),
        ),
    );

    let mut inbound = tcp(443, 51_000);
    inbound.extend_from_slice(&filler(16));
    c.at(
        10_000,
        ethernet(
            ETHERTYPE_IPV6,
            &ipv6(peer_v6(), host_v6(), IPPROTO_TCP, &inbound),
        ),
    );

    let mut inbound_chained = extension(IPPROTO_UDP);
    inbound_chained.extend_from_slice(&udp(5_055, 30_000, 16));
    inbound_chained.extend_from_slice(&filler(16));
    c.at(
        15_000,
        ethernet(
            ETHERTYPE_IPV6,
            &ipv6(peer_v6(), host_v6(), EXT_HOP_BY_HOP, &inbound_chained),
        ),
    );

    Fixture {
        name: "ipv6-mixed",
        pcap: c.to_pcap(),
        script: concat!(
            "# IPv6, half of it behind extension header chains.\n",
            "flow tcp [2001:db8::10]:51000 [2001:db8::5]:443 always owner 4242 game.exe\n",
            "flow udp [2001:db8::10]:30000 * always owner 4242 game.exe\n",
            "endpoint tcp [2001:db8::10]:51000\n",
            "endpoint udp [2001:db8::10]:30000\n",
        )
        .to_string(),
        local: vec![IpAddr::V6(host_v6())],
    }
}

/// IP fragmentation, first and subsequent.
fn fragmented() -> Fixture {
    let mut c = Capture::new();
    let ident = 4242u16;

    // The initial fragment carries the transport header, and therefore the
    // ports every later fragment is attributed by. Its length field describes
    // the whole reassembled datagram rather than this fragment: eight header
    // bytes plus 1472 here, 1480 in the middle, and 400 in the last. Declaring
    // only this fragment's worth made the fixture an internally malformed
    // datagram rather than the valid fragmentation it claims to be, which
    // review of pull request 7 caught.
    const FRAGMENTED_UDP_PAYLOAD: usize = 1_472 + 1_480 + 400;
    let mut first = udp(30_000, 5_055, FRAGMENTED_UDP_PAYLOAD);
    first.extend_from_slice(&filler(1_472));
    let mut h = V4::new(HOST_V4, PEER_V4, IPPROTO_UDP);
    h.ident = ident;
    h.more_fragments = true;
    c.at(0, ethernet(ETHERTYPE_IPV4, &ipv4(&h, &first)));

    // A middle fragment: offset past zero, more still coming, no transport
    // header of its own.
    let mut mid = V4::new(HOST_V4, PEER_V4, IPPROTO_UDP);
    mid.ident = ident;
    mid.frag_offset = 185;
    mid.more_fragments = true;
    c.at(1_000, ethernet(ETHERTYPE_IPV4, &ipv4(&mid, &filler(1_480))));

    // The last fragment, which is what forgets the datagram.
    let mut last = V4::new(HOST_V4, PEER_V4, IPPROTO_UDP);
    last.ident = ident;
    last.frag_offset = 370;
    c.at(2_000, ethernet(ETHERTYPE_IPV4, &ipv4(&last, &filler(400))));

    Fixture {
        name: "fragmented",
        pcap: c.to_pcap(),
        script: concat!(
            "# One fragmented datagram: initial, middle, last. Nothing is\n",
            "# reassembled; the later fragments resolve from what the first said.\n",
            "flow udp 192.0.2.10:30000 * always owner 4242 game.exe\n",
            "endpoint udp 192.0.2.10:30000\n",
        )
        .to_string(),
        local: vec![IpAddr::V4(HOST_V4)],
    }
}

/// Local traffic, direction ambiguity.
fn loopback() -> Fixture {
    let mut c = Capture::new();
    for i in 0..4u32 {
        let (sp, dp) = if i % 2 == 0 {
            (51_000, 8_080)
        } else {
            (8_080, 51_000)
        };
        let mut body = tcp(sp, dp);
        body.extend_from_slice(&filler(8));
        c.at(
            i * 2_000,
            ethernet(
                ETHERTYPE_IPV4,
                &ipv4(&V4::new(LOOPBACK_V4, LOOPBACK_V4, IPPROTO_TCP), &body),
            ),
        );
    }
    Fixture {
        name: "loopback",
        pcap: c.to_pcap(),
        script: concat!(
            "# Both endpoints local, so section 12.6's rule returns two answers\n",
            "# and the direction is left undetermined. Section 12.6 resolves it\n",
            "# from the attributed endpoint in a later slice.\n",
            "#\n",
            "# The entry is written low endpoint first, which is not arbitrary.\n",
            "# When both endpoints are local there is no local one in the usual\n",
            "# sense, so the parser picks the position by a canonical ordering\n",
            "# (slice S03 decision D-5) and a script has to agree with it or it\n",
            "# matches nothing. Port 8080 sorts below 51000, so 8080 is local.\n",
            "flow tcp 127.0.0.1:8080 127.0.0.1:51000 always owner 4242 game.exe\n",
            "endpoint tcp 127.0.0.1:8080\n",
        )
        .to_string(),
        local: vec![IpAddr::V4(LOOPBACK_V4)],
    }
}

/// Truncated and invalid headers.
///
/// The pcap records are well-formed; it is the packets inside them that are
/// not. The reader's own skip causes are exercised against byte arrays built in
/// its unit tests, because a committed file that is a broken capture would
/// confuse every other tool that opens the corpus.
fn malformed() -> Fixture {
    let mut c = Capture::new();

    // An encapsulation fragcap does not parse.
    c.at(0, ethernet(ETHERTYPE_VLAN, &filler(40)));

    // A transport that is neither TCP nor UDP.
    c.at(
        1_000,
        ethernet(
            ETHERTYPE_IPV4,
            &ipv4(&V4::new(HOST_V4, PEER_V4, IPPROTO_ICMP), &filler(20)),
        ),
    );

    // A network header whose own fields contradict each other.
    let mut bad = V4::new(HOST_V4, PEER_V4, IPPROTO_TCP);
    bad.bad_ihl = true;
    c.at(2_000, ethernet(ETHERTYPE_IPV4, &ipv4(&bad, &tcp(1, 2))));

    // A frame cut off inside its transport header.
    let short = ipv4(&V4::new(HOST_V4, PEER_V4, IPPROTO_TCP), &tcp(1, 2)[..2]);
    c.at(3_000, ethernet(ETHERTYPE_IPV4, &short));

    // A well-formed IPv6 packet that legitimately carries no transport.
    c.at(
        4_000,
        ethernet(
            ETHERTYPE_IPV6,
            &ipv6(host_v6(), peer_v6(), NO_NEXT_HEADER, &[]),
        ),
    );

    Fixture {
        name: "malformed",
        pcap: c.to_pcap(),
        script: concat!(
            "# Nothing here parses to a flow, so nothing here has an owner.\n",
            "# The script exists because every fixture has one, and its emptiness\n",
            "# is the statement.\n",
        )
        .to_string(),
        local: vec![IpAddr::V4(HOST_V4), IpAddr::V6(host_v6())],
    }
}

/// Same port, different processes, over time.
fn port_reuse() -> Fixture {
    let mut c = Capture::new();
    // Two sessions on one local port, ten seconds apart. Nothing in the capture
    // distinguishes them; only the script's windows do, which is the point.
    for (session, base) in [(0u32, 0u32), (1, 10_000_000)] {
        for i in 0..3u32 {
            let outbound = i % 2 == 0;
            let (src, dst, sp, dp) = if outbound {
                (HOST_V4, PEER_V4, 51_000, 443)
            } else {
                (PEER_V4, HOST_V4, 443, 51_000)
            };
            let mut body = tcp(sp, dp);
            body.extend_from_slice(&filler(8 + session as usize));
            c.at(
                base + i * 500_000,
                ethernet(
                    ETHERTYPE_IPV4,
                    &ipv4(&V4::new(src, dst, IPPROTO_TCP), &body),
                ),
            );
        }
    }

    let first_end = at_nanos(5_000_000);
    let second_end = at_nanos(20_000_000);
    let script = format!(
        "# One local port, two processes, ten seconds apart. The windows are\n\
         # half-open and abut, so they do not overlap.\n\
         # first window ends at base + 5s, second at base + 20s\n\
         flow tcp 192.0.2.10:51000 198.51.100.5:443 {}..{} owner 100 first.exe\n\
         flow tcp 192.0.2.10:51000 198.51.100.5:443 {}..{} owner 200 second.exe\n\
         endpoint tcp 192.0.2.10:51000\n",
        at_nanos(0),
        first_end,
        first_end,
        second_end,
    );

    Fixture {
        name: "port-reuse",
        pcap: c.to_pcap(),
        script,
        local: vec![IpAddr::V4(HOST_V4)],
    }
}

/// Sustained rate.
///
/// Specification section 25.3 calls this "a sustained rate exceeding buffer
/// capacity". The buffer in section 12.4 holds 65,536 packets, so a fixture
/// that genuinely exceeds it runs to several megabytes and contradicts the same
/// section's requirement that fixtures be small. Backpressure is a relationship
/// between a rate and a capacity rather than a property of a file, so this
/// supplies the rate and S08's test supplies a small capacity. Recorded for
/// promotion to specification section 29.
fn burst() -> Fixture {
    let mut c = Capture::new();
    for i in 0..400u32 {
        let mut body = udp(30_000, 5_055, 8);
        body.extend_from_slice(&filler(8));
        c.at(
            i * 100,
            ethernet(
                ETHERTYPE_IPV4,
                &ipv4(&V4::new(HOST_V4, PEER_V4, IPPROTO_UDP), &body),
            ),
        );
    }
    Fixture {
        name: "burst",
        pcap: c.to_pcap(),
        script: concat!(
            "# Four hundred datagrams a tenth of a millisecond apart. The\n",
            "# capacity half of the backpressure test belongs to S08.\n",
            "flow udp 192.0.2.10:30000 * always owner 4242 game.exe\n",
            "endpoint udp 192.0.2.10:30000\n",
        )
        .to_string(),
        local: vec![IpAddr::V4(HOST_V4)],
    }
}

// ---------------------------------------------------------------------------
// Reading a fixture back.
// ---------------------------------------------------------------------------

fn read_packets(pcap: &[u8]) -> (LinkType, Vec<RawPacket>) {
    let mut r = PcapReader::from_bytes(pcap.to_vec()).expect("a generated fixture must open");
    let link = r.link_type();
    let mut out = Vec::new();
    while let Some(p) = r.next_record() {
        out.push(p);
    }
    assert!(
        r.stats().read_whole_file(),
        "a generated fixture must read whole: {}",
        r.stats()
    );
    (link, out)
}

fn outcomes(f: &Fixture) -> Vec<ParseOutcome> {
    let (link, packets) = read_packets(&f.pcap);
    let mut parser = HeaderParser::new(InterfaceAddrs::new(f.local.iter().copied()));
    packets
        .iter()
        .map(|p| parser.parse(link, &p.data))
        .collect()
}

// ---------------------------------------------------------------------------
// The drift check, and regeneration.
// ---------------------------------------------------------------------------

#[test]
fn the_committed_corpus_matches_its_generator() {
    let dir = fixtures_dir();
    let updating = updating();
    if updating {
        std::fs::create_dir_all(&dir).expect("the fixtures directory can be created");
    }

    let mut mismatched = Vec::new();
    for f in corpus() {
        let pcap_path = dir.join(format!("{}.pcap", f.name));
        let script_path = dir.join(format!("{}.script", f.name));

        if updating {
            std::fs::write(&pcap_path, &f.pcap).expect("the fixture can be written");
            std::fs::write(&script_path, f.script.as_bytes()).expect("the script can be written");
            continue;
        }

        match std::fs::read(&pcap_path) {
            Ok(committed) if committed == f.pcap => {}
            Ok(_) => mismatched.push(format!("{}.pcap differs from the generator", f.name)),
            Err(e) => mismatched.push(format!("{}.pcap: {e}", f.name)),
        }
        // The scripts drift as quietly as the captures, and a script that no
        // longer matches its fixture misattributes rather than failing.
        match std::fs::read(&script_path) {
            Ok(committed) if committed == f.script.as_bytes() => {}
            Ok(_) => mismatched.push(format!("{}.script differs from the generator", f.name)),
            Err(e) => mismatched.push(format!("{}.script: {e}", f.name)),
        }
    }

    assert!(
        mismatched.is_empty(),
        "the committed corpus has drifted from its generator:\n  {}\n\
         Regenerate with FRAGCAP_UPDATE_FIXTURES=1 cargo test -p fragcap-capture \
         --test corpus, then read the diff.",
        mismatched.join("\n  ")
    );
}

#[test]
fn generation_is_deterministic() {
    // FR-032a. Two runs in one process must agree, which they cannot if any
    // byte came from a clock, the filesystem, or the environment.
    let a = corpus();
    let b = corpus();
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.name, y.name);
        assert_eq!(x.pcap, y.pcap, "{} is not deterministic", x.name);
        assert_eq!(x.script, y.script, "{} script is not deterministic", x.name);
    }
}

#[test]
fn every_fixture_is_paired_with_a_script_in_both_directions() {
    if updating() {
        // The generator is writing the corpus in a sibling test right now, and
        // the harness runs tests concurrently. Checking the directory mid-write
        // races it. The check run, which is the one that gates, is unaffected.
        return;
    }
    let dir = fixtures_dir();
    let expected: BTreeMap<&str, ()> = corpus().iter().map(|f| (f.name, ())).collect();
    assert_eq!(expected.len(), 8, "section 25.3 names eight fixtures");

    for name in expected.keys() {
        assert!(
            dir.join(format!("{name}.pcap")).is_file(),
            "{name}.pcap is missing"
        );
        assert!(
            dir.join(format!("{name}.script")).is_file(),
            "{name}.script is missing"
        );
    }

    // And nothing extra: a script with no capture means the corpus no longer
    // describes itself.
    for entry in std::fs::read_dir(&dir).expect("the fixtures directory exists") {
        let path = entry.expect("a readable directory entry").path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "pcap" && ext != "script" {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("a fixture has a name");
        assert!(
            expected.contains_key(stem),
            "{} is in the corpus directory but not in the generator",
            path.display()
        );
    }
}

#[test]
fn the_corpus_stays_small_enough_to_review() {
    let mut total = 0usize;
    for f in corpus() {
        assert!(
            f.pcap.len() <= MAX_FIXTURE_BYTES,
            "{} is {} bytes, over the {MAX_FIXTURE_BYTES} byte ceiling",
            f.name,
            f.pcap.len()
        );
        total += f.pcap.len() + f.script.len();
    }
    assert!(
        total <= MAX_CORPUS_BYTES,
        "the corpus is {total} bytes, over the {MAX_CORPUS_BYTES} byte ceiling"
    );
}

// ---------------------------------------------------------------------------
// Privacy.
// ---------------------------------------------------------------------------

fn is_permitted(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            v4.is_loopback()
                // RFC 5737 documentation ranges.
                || o[..3] == [192, 0, 2]
                || o[..3] == [198, 51, 100]
                || o[..3] == [203, 0, 113]
        }
        // RFC 3849 documentation range, plus loopback.
        IpAddr::V6(v6) => v6.is_loopback() || v6.segments()[..2] == [0x2001, 0x0db8],
    }
}

#[test]
fn every_address_in_the_corpus_is_documentation_or_loopback() {
    for f in corpus() {
        for outcome in outcomes(&f) {
            if let Some(flow) = outcome.flow() {
                for addr in [flow.local.ip(), flow.remote.ip()] {
                    assert!(
                        is_permitted(addr),
                        "{} carries {addr}, which is neither a documentation \
                         address nor loopback",
                        f.name
                    );
                }
            }
        }
    }
}

#[test]
fn no_fixture_carries_anything_that_reads_as_text() {
    // The drift check is the primary guarantee that fixtures contain only what
    // the generator put there. This is the belt-and-braces pass: an account
    // identifier, a hostname, or a session token would show up as a run of
    // printable characters, and nothing the generator writes does. Stated
    // plainly because it is a heuristic, unlike the drift check.
    for f in corpus() {
        let mut run = 0usize;
        for (i, b) in f.pcap.iter().enumerate() {
            if b.is_ascii_graphic() || *b == b' ' {
                run += 1;
                assert!(
                    run < 8,
                    "{} has a printable run ending at byte {i}, which no \
                     generated fixture should contain",
                    f.name
                );
            } else {
                run = 0;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Each fixture exercises the condition section 25.3 states for it.
// ---------------------------------------------------------------------------

fn named(name: &str) -> Fixture {
    corpus()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("{name} is in the corpus"))
}

#[test]
fn tcp_session_carries_one_flow_in_both_directions() {
    let f = named("tcp-session");
    let keys: std::collections::HashSet<_> = outcomes(&f).iter().filter_map(|o| o.flow()).collect();
    assert_eq!(keys.len(), 1, "both directions must normalize to one key");
    let directions: std::collections::HashSet<_> =
        outcomes(&f).iter().filter_map(|o| o.direction()).collect();
    assert_eq!(directions.len(), 2, "both directions must be present");
}

#[test]
fn udp_gameplay_is_udp_at_a_steady_cadence() {
    let f = named("udp-gameplay");
    let (_, packets) = read_packets(&f.pcap);
    assert!(packets.len() >= 16, "a sustained flow needs sustaining");
    for o in outcomes(&f) {
        assert_eq!(
            o.flow().map(|k| k.proto),
            Some(fragcap_core::flow::Proto::Udp)
        );
    }
    let gaps: Vec<i64> = packets
        .windows(2)
        .map(|w| w[1].ts.nanos_since(w[0].ts))
        .collect();
    for gap in gaps {
        assert!(
            (1_000_000..=50_000_000).contains(&gap),
            "a gameplay cadence gap of {gap} nanoseconds is not gameplay cadence"
        );
    }
}

#[test]
fn ipv6_mixed_carries_extension_header_chains() {
    let f = named("ipv6-mixed");
    let resolved: Vec<_> = outcomes(&f).iter().filter_map(|o| o.flow()).collect();
    assert!(
        resolved.iter().all(|k| k.local.is_ipv6()),
        "every flow here is IPv6"
    );
    // Read the IPv6 next header field directly, at offset 6 of the network
    // header, which begins after the fourteen byte Ethernet header.
    //
    // An earlier version compared captured lengths instead, and review of pull
    // request 7 pointed out that it could not fail: strip every extension
    // header and the fixture still holds differently sized bare TCP and UDP
    // packets, so the assertion passed while the condition it claimed to guard
    // had gone. A check that cannot detect its own regression is worse than no
    // check, because it reads as coverage.
    let (_, packets) = read_packets(&f.pcap);
    let next_headers: Vec<u8> = packets.iter().map(|p| p.data[14 + 6]).collect();
    let chained = next_headers
        .iter()
        .filter(|n| matches!(**n, 0 | 43 | 44 | 51 | 60))
        .count();
    let bare = next_headers
        .iter()
        .filter(|n| matches!(**n, IPPROTO_TCP | IPPROTO_UDP))
        .count();
    assert!(
        chained >= 2,
        "the fixture must carry extension chains, but its next headers are \
         {next_headers:?}"
    );
    assert!(
        bare >= 1,
        "and must carry unchained packets to contrast with"
    );

    // And the walk must actually reach the transport behind them, or the chain
    // is present and never traversed.
    assert_eq!(
        resolved.len(),
        packets.len(),
        "every packet, chained or not, must resolve to a flow"
    );
}

#[test]
fn fragmented_carries_an_initial_and_a_later_fragment() {
    let f = named("fragmented");
    let os = outcomes(&f);
    assert_eq!(os.len(), 3, "initial, middle, and last");
    // Every fragment resolves, which is only possible if the first one was
    // recorded and the later ones matched it without reassembly.
    for (i, o) in os.iter().enumerate() {
        assert!(
            o.flow().is_some(),
            "fragment {i} did not resolve: {:?}",
            o.reject()
        );
    }
    let keys: std::collections::HashSet<_> = os.iter().filter_map(|o| o.flow()).collect();
    assert_eq!(keys.len(), 1, "one datagram is one flow");

    // The UDP length field describes the reassembled datagram, so it must equal
    // the sum of every fragment's IP payload. Asserted because the first
    // version of this fixture declared only the initial fragment's worth and
    // was therefore a malformed datagram that nonetheless parsed, which is the
    // kind of fixture defect that surfaces as a mystery in a later slice.
    let (_, packets) = read_packets(&f.pcap);
    let ip_payload: usize = packets.iter().map(|p| p.captured_len() - 14 - 20).sum();
    let first = &packets[0].data;
    let declared = u16::from_be_bytes([first[14 + 20 + 4], first[14 + 20 + 5]]) as usize;
    assert_eq!(
        declared, ip_payload,
        "the declared UDP length must cover every fragment, not just the first"
    );
}

#[test]
fn loopback_produces_a_key_with_no_direction() {
    let f = named("loopback");
    let os = outcomes(&f);
    assert!(!os.is_empty());
    for o in &os {
        assert!(o.flow().is_some(), "a loopback packet still gets a key");
        assert_eq!(
            o.direction(),
            None,
            "both endpoints are local, so the rule returns two answers"
        );
    }
    let keys: std::collections::HashSet<_> = os.iter().filter_map(|o| o.flow()).collect();
    assert_eq!(keys.len(), 1, "both halves are one conversation");
}

#[test]
fn malformed_reaches_more_than_one_parse_rejection_cause() {
    let f = named("malformed");
    let causes: std::collections::HashSet<_> =
        outcomes(&f).iter().filter_map(|o| o.reject()).collect();
    assert!(
        causes.len() >= 4,
        "the point of this fixture is variety, and it reached only {causes:?}"
    );
    assert!(
        outcomes(&f).iter().all(|o| o.flow().is_none()),
        "nothing here should resolve"
    );
}

#[test]
fn port_reuse_carries_one_flow_across_two_separated_times() {
    let f = named("port-reuse");
    let (_, packets) = read_packets(&f.pcap);
    let keys: std::collections::HashSet<_> = outcomes(&f).iter().filter_map(|o| o.flow()).collect();
    assert_eq!(keys.len(), 1, "the capture alone cannot tell them apart");
    let span = packets
        .last()
        .expect("packets")
        .ts
        .nanos_since(packets[0].ts);
    assert!(
        span >= 10_000_000_000,
        "the two sessions must be far enough apart to be distinct windows"
    );
}

#[test]
fn burst_is_a_sustained_rate() {
    let f = named("burst");
    let (_, packets) = read_packets(&f.pcap);
    assert!(packets.len() >= 200, "a burst needs packets");
    let span = packets
        .last()
        .expect("packets")
        .ts
        .nanos_since(packets[0].ts);
    assert!(
        span <= 100_000_000,
        "a burst is a rate, and {span} nanoseconds is too leisurely"
    );
}

#[test]
fn every_fixture_has_a_non_empty_script() {
    // That each script *parses* is asserted in the facade's pipeline test,
    // which is the only place both crates are legitimately in scope. Adding a
    // dev-dependency here would create exactly the edge between capture and
    // attribution that P-3 and `cargo xtask deps` exist to prevent, even though
    // a dev-dependency would slip past the check unnoticed.
    for f in corpus() {
        assert!(
            !f.script.is_empty(),
            "{} has no script, and every fixture has one",
            f.name
        );
    }
}
