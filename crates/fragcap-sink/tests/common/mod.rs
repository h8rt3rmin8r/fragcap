// SPDX-License-Identifier: Apache-2.0

//! Shared helpers for the transport integration tests: a minimal pcapng block
//! walker and a packet builder. The walker is deliberately independent of the
//! writer it checks, so it can catch a malformed stream rather than reproduce
//! the writer's own assumptions.

#![allow(dead_code)]

use fragcap_core::interface::InterfaceId;
use fragcap_core::packet::{CapturedPacket, Payload, RawPacket, Timestamp};

/// pcapng block type codes (specification section 13.2).
pub const SHB: u32 = 0x0A0D_0D0A;
pub const IDB: u32 = 0x0000_0001;
pub const EPB: u32 = 0x0000_0006;
pub const ISB: u32 = 0x0000_0005;

/// One parsed pcapng block: its type and its body (the bytes between the
/// leading type/length header and the trailing length).
#[derive(Clone, Debug)]
pub struct Block {
    pub block_type: u32,
    pub body: Vec<u8>,
}

/// Walk a pcapng byte stream into blocks, asserting the length framing is
/// self-consistent. Panics (fails the test) on any malformed framing.
pub fn walk(bytes: &[u8]) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut off = 0usize;
    while off + 8 <= bytes.len() {
        let block_type = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
        let len = u32::from_le_bytes(bytes[off + 4..off + 8].try_into().unwrap()) as usize;
        assert!(
            len >= 12 && len.is_multiple_of(4) && off + len <= bytes.len(),
            "block length {len} at offset {off} is out of range (total {})",
            bytes.len()
        );
        let body = bytes[off + 8..off + len - 4].to_vec();
        let trailing =
            u32::from_le_bytes(bytes[off + len - 4..off + len].try_into().unwrap()) as usize;
        assert_eq!(trailing, len, "trailing length must equal leading length");
        blocks.push(Block { block_type, body });
        off += len;
    }
    assert_eq!(off, bytes.len(), "trailing bytes after the last block");
    blocks
}

/// The captured payloads of every Enhanced Packet Block, in order.
pub fn epb_payloads(blocks: &[Block]) -> Vec<Vec<u8>> {
    blocks
        .iter()
        .filter(|b| b.block_type == EPB)
        .map(|b| {
            // EPB body: interface_id(4) ts_high(4) ts_low(4) cap_len(4)
            // orig_len(4) then the captured bytes.
            let cap_len = u32::from_le_bytes(b.body[12..16].try_into().unwrap()) as usize;
            b.body[20..20 + cap_len].to_vec()
        })
        .collect()
}

/// Assert a stream is a valid pcapng preamble: a Section Header Block first,
/// then exactly `expected_ifaces` Interface Description Blocks before any
/// packet, matching what every other consumer of the same capture declares.
pub fn assert_valid_pcapng_stream(bytes: &[u8], expected_ifaces: usize) {
    let blocks = walk(bytes);
    assert!(!blocks.is_empty(), "stream has no blocks");
    assert_eq!(
        blocks[0].block_type, SHB,
        "first block must be a Section Header Block"
    );
    let idbs = blocks
        .iter()
        .take_while(|b| b.block_type != EPB)
        .filter(|b| b.block_type == IDB)
        .count();
    assert_eq!(
        idbs, expected_ifaces,
        "expected {expected_ifaces} interface blocks before the first packet"
    );
}

/// Build a captured packet with a distinct payload, on the default interface.
pub fn packet(ts_nanos: i64, payload: &[u8]) -> CapturedPacket {
    let raw = RawPacket::new(
        Timestamp::from_nanos(ts_nanos),
        Payload::from(payload.to_vec()),
        payload.len() as u32,
    );
    CapturedPacket::from_raw(raw, InterfaceId::default())
}

/// A run of `n` packets, each with a distinct one-byte-varied payload of
/// `size` bytes so ordering and identity are checkable.
pub fn packets(n: usize, size: usize) -> Vec<CapturedPacket> {
    (0..n)
        .map(|i| {
            let mut payload = vec![(i & 0xff) as u8; size];
            if let Some(first) = payload.first_mut() {
                *first = (i & 0xff) as u8;
            }
            if size >= 2 {
                payload[1] = ((i >> 8) & 0xff) as u8;
            }
            packet(1_000 + i as i64, &payload)
        })
        .collect()
}

/// The payloads of a packet run, for comparison against what a consumer read.
pub fn expected_payloads(pkts: &[CapturedPacket]) -> Vec<Vec<u8>> {
    pkts.iter().map(|p| p.data.as_ref().to_vec()).collect()
}
