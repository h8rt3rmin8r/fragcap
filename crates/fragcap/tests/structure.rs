// SPDX-License-Identifier: Apache-2.0

//! Does the file satisfy pcapng's own structural rules?
//!
//! This is the check that reaches outside the writer's assumptions. The
//! validator below walks the file the way a conforming reader would, by
//! declared block lengths, and calls none of the writer's encoding functions.
//! A writer verified only by its own encoder has proven that two functions
//! agree, which is not what specification section 13.1 promises.
//!
//! It is a test, not a capability. fragcap writes pcapng and reads classic
//! pcap; nothing here is exported, and nothing outside this file may depend on
//! it.

mod common;

use common::{render, CORPUS};
use fragcap_core::interface::InterfaceId;

const SECTION_HEADER: u32 = 0x0A0D_0D0A;
const INTERFACE_DESCRIPTION: u32 = 0x0000_0001;
const INTERFACE_STATISTICS: u32 = 0x0000_0005;
const ENHANCED_PACKET: u32 = 0x0000_0006;
const OPT_END_OF_OPT: u16 = 0;

fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

struct Block {
    block_type: u32,
    offset: usize,
    body: Vec<u8>,
}

/// Walk a pcapng file by its declared lengths.
///
/// Every failure here is stated as what a reader would experience, because that
/// is the population the format has to satisfy.
fn walk(name: &str, buf: &[u8]) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < buf.len() {
        assert!(
            i + 12 <= buf.len(),
            "{name}: block at {i} is too short to carry its own framing"
        );
        let block_type = u32_at(buf, i);
        let total = u32_at(buf, i + 4) as usize;

        assert!(
            total >= 12,
            "{name}: block at {i} declares {total} bytes, less than its framing"
        );
        assert_eq!(
            total % 4,
            0,
            "{name}: block at {i} is not a whole number of 32-bit words"
        );
        assert!(
            i + total <= buf.len(),
            "{name}: block at {i} declares {total} bytes, past the end of the file"
        );

        let trailing = u32_at(buf, i + total - 4) as usize;
        assert_eq!(
            total, trailing,
            "{name}: block at {i} disagrees with itself about its length, \
             so a reader walking backwards would go astray"
        );

        blocks.push(Block {
            block_type,
            offset: i,
            body: buf[i + 8..i + total - 4].to_vec(),
        });
        i += total;
    }
    assert_eq!(
        i,
        buf.len(),
        "{name}: the block walk did not consume the file exactly"
    );
    assert!(
        !blocks.is_empty(),
        "{name}: a capture has at least a header"
    );
    blocks
}

/// Walk an option list, asserting alignment and termination.
///
/// `prefix` is the fixed part of the block body, before the options begin.
fn check_options(name: &str, what: &str, body: &[u8], prefix: usize) -> Vec<(u16, Vec<u8>)> {
    let mut found = Vec::new();
    if body.len() == prefix {
        return found; // No options at all is legal.
    }
    assert!(
        body.len() >= prefix + 4,
        "{name}: {what} has a truncated option list"
    );

    let mut i = prefix;
    let mut terminated = false;
    while i + 4 <= body.len() {
        let code = u16_at(body, i);
        let len = u16_at(body, i + 2) as usize;
        if code == OPT_END_OF_OPT {
            assert_eq!(len, 0, "{name}: {what} terminator carries a value");
            terminated = true;
            i += 4;
            break;
        }
        assert!(
            i + 4 + len <= body.len(),
            "{name}: {what} option {code} declares {len} bytes, past the block"
        );
        found.push((code, body[i + 4..i + 4 + len].to_vec()));
        let padded = len + (4 - len % 4) % 4;
        assert_eq!(
            (i + 4 + padded) % 4,
            0,
            "{name}: {what} option {code} leaves the next option misaligned"
        );
        i += 4 + padded;
    }
    assert!(
        terminated,
        "{name}: {what} option list is not terminated with opt_endofopt"
    );
    assert_eq!(
        i,
        body.len(),
        "{name}: {what} has trailing bytes after its option list"
    );
    found
}

/// The full structural pass over one rendered fixture.
fn validate(name: &str, buf: &[u8]) {
    let blocks = walk(name, buf);

    assert_eq!(
        blocks[0].block_type, SECTION_HEADER,
        "{name}: a capture begins with a Section Header Block"
    );
    assert_eq!(
        u32_at(&blocks[0].body, 0),
        0x1A2B_3C4D,
        "{name}: byte-order magic must read as little-endian on every host"
    );

    let mut declared_interfaces = 0u32;
    for b in &blocks {
        match b.block_type {
            SECTION_HEADER => {
                check_options(name, "section header", &b.body, 16);
            }
            INTERFACE_DESCRIPTION => {
                assert_eq!(
                    u16_at(&b.body, 2),
                    0,
                    "{name}: interface reserved field is not zero"
                );
                check_options(name, "interface description", &b.body, 8);
                declared_interfaces += 1;
            }
            ENHANCED_PACKET => {
                let iface = u32_at(&b.body, 0);
                assert!(
                    iface < declared_interfaces,
                    "{name}: packet block at {} references interface {iface}, \
                     which was not declared before it",
                    b.offset
                );
                let captured = u32_at(&b.body, 12) as usize;
                let padded = captured + (4 - captured % 4) % 4;
                assert!(
                    20 + padded <= b.body.len(),
                    "{name}: packet block at {} declares {captured} captured bytes \
                     that do not fit in the block",
                    b.offset
                );
                check_options(name, "enhanced packet", &b.body, 20 + padded);
            }
            INTERFACE_STATISTICS => {
                let iface = u32_at(&b.body, 0);
                assert!(
                    iface < declared_interfaces,
                    "{name}: statistics block references undeclared interface {iface}"
                );
                check_options(name, "interface statistics", &b.body, 12);
            }
            other => panic!("{name}: unexpected block type {other:#010x}"),
        }
    }
}

#[test]
fn every_fixture_produces_a_structurally_valid_capture() {
    for (name, _) in CORPUS {
        validate(name, &render(name));
    }
}

#[test]
fn every_capture_declares_an_interface_before_its_packets() {
    // Stated separately from the walk because the ordering rule is the one a
    // reader cannot recover from: an identifier with no declaration has no
    // link type, so the packet cannot be dissected at all.
    for (name, _) in CORPUS {
        let buf = render(name);
        let blocks = walk(name, &buf);
        let first_packet = blocks.iter().position(|b| b.block_type == ENHANCED_PACKET);
        let first_iface = blocks
            .iter()
            .position(|b| b.block_type == INTERFACE_DESCRIPTION);
        if let Some(p) = first_packet {
            let i = first_iface
                .unwrap_or_else(|| panic!("{name}: packets present but no interface was declared"));
            assert!(i < p, "{name}: a packet precedes its interface declaration");
        }
    }
}

#[test]
fn every_capture_ends_with_its_statistics() {
    for (name, _) in CORPUS {
        let buf = render(name);
        let blocks = walk(name, &buf);
        assert_eq!(
            blocks.last().unwrap().block_type,
            INTERFACE_STATISTICS,
            "{name}: the loss accounting is the last thing written"
        );
    }
}

#[test]
fn a_writer_dropped_without_finishing_leaves_a_readable_prefix() {
    // Bounding the damage. pcapng is a sequence of self-delimiting blocks, so
    // everything written before the interruption is still a capture. A
    // truncated capture is more useful than an unreadable one.
    use fragcap::{
        CapturedPacket, InterfaceDeclaration, LinkType, Payload, PcapngWriter, RawPacket, Sink,
        Timestamp,
    };

    let mut buf = Vec::new();
    {
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        w.declare_interface(&InterfaceDeclaration::new(
            LinkType::ETHERNET,
            65_535,
            "eth0",
        ))
        .unwrap();
        let raw = RawPacket::new(Timestamp::from_nanos(1_000), Payload::from(vec![0u8; 8]), 8);
        w.write(&CapturedPacket::from_raw(raw, InterfaceId::default()))
            .unwrap();
        // Dropped here, deliberately: no `finish`.
    }

    let blocks = walk("truncated", &buf);
    assert_eq!(blocks.len(), 3, "header, interface, and the packet");
    assert!(
        !blocks.iter().any(|b| b.block_type == INTERFACE_STATISTICS),
        "an unfinished capture has no statistics, and says so by their absence"
    );
}
