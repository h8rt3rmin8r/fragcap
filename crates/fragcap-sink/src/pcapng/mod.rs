// SPDX-License-Identifier: Apache-2.0

//! The pcapng writer of specification sections 13.1 through 13.4.
//!
//! Writes the four blocks of section 13.2, carrying attribution in the Enhanced
//! Packet Block `opt_comment` where every reader already displays it. That
//! choice is settled in section 13.3 and is the practical form of constitution
//! P-5: a format carrying more at the cost of a plugin is a worse format here,
//! because the value of attribution is realized in tooling that already exists.
//!
//! Two properties are worth knowing before reading the code.
//!
//! **It reads no clock.** Every byte is a function of the packets, the
//! interface declarations, and the statistics snapshot. The Interface
//! Statistics Block timestamp comes from the last packet written, not from the
//! current time, because a writer that reads a clock produces goldens that pass
//! once and fail forever after.
//!
//! **It writes little-endian on every host.** pcapng permits either and
//! declares the choice, so both are valid; only one produces the same capture
//! on every machine.

pub(crate) mod block;
pub mod interface;

use std::io::Write;

use fragcap_core::interface::InterfaceId;
use fragcap_core::packet::CapturedPacket;
use fragcap_core::stats::CaptureStats;
use fragcap_core::traits::Sink;
use fragcap_core::SinkError;

use crate::annotation::Annotation;
use crate::error::WriteError;
use block::{block_type, opt, write_block, Options};
use interface::{DeclaredInterface, InterfaceDeclaration};

/// The application name written to `shb_userappl`.
pub const USER_APPL: &str = concat!("fragcap/", env!("CARGO_PKG_VERSION"));

/// The annotation profile version declared in the Section Header Block comment.
///
/// Versions the grammar, not the crate. A change to which keys exist, what they
/// mean, or how values are encoded bumps this; adding a key consumers may
/// ignore does not.
pub const PROFILE_COMMENT: &str = "fragcap:profile=0.1.0";

/// `if_tsresol` value: 6, meaning microseconds. Specification section 12.7.
const TSRESOL_MICROS: u8 = 6;

/// Writes packets and their attribution as pcapng.
#[derive(Debug)]
pub struct PcapngWriter<W: Write> {
    out: W,
    interfaces: Vec<DeclaredInterface>,
    /// Whether any packet block has been written. Once true, no further
    /// interface may be declared.
    wrote_a_packet: bool,
}

impl<W: Write> PcapngWriter<W> {
    /// Begin a capture, writing the Section Header Block immediately.
    ///
    /// Written here rather than lazily so it is always the first thing in the
    /// file, including in a capture that never receives a packet.
    pub fn new(mut out: W) -> Result<Self, WriteError> {
        let mut body = Vec::new();
        body.extend_from_slice(&0x1A2B_3C4Du32.to_le_bytes()); // byte-order magic
        body.extend_from_slice(&1u16.to_le_bytes()); // major version
        body.extend_from_slice(&0u16.to_le_bytes()); // minor version
        body.extend_from_slice(&(-1i64).to_le_bytes()); // section length, unspecified

        let mut options = Options::new();
        options.push_str(opt::COMMENT, PROFILE_COMMENT)?;
        options.push_str(opt::SHB_USERAPPL, USER_APPL)?;
        body.extend_from_slice(&options.finish());

        write_block(&mut out, block_type::SECTION_HEADER, &body)?;
        Ok(PcapngWriter {
            out,
            interfaces: Vec::new(),
            wrote_a_packet: false,
        })
    }

    /// Declare an interface, returning the identifier assigned to it.
    ///
    /// Identifiers are assigned in declaration order from zero, which is how
    /// pcapng identifies interfaces, so the writer's numbering and the file's
    /// cannot disagree. Declaring the same interface twice produces two
    /// identifiers; deduplicating would silently repoint packets the caller
    /// attributed to the second.
    /// Every interface must be declared before the first packet is written.
    /// Section 13.3 decides the annotation `iface` key from whether the capture
    /// holds more than one, and a block already written cannot be revised, so
    /// the question has to be answerable once rather than per packet.
    pub fn declare_interface(&mut self, decl: &InterfaceDeclaration) -> Result<u32, WriteError> {
        if self.wrote_a_packet {
            return Err(WriteError::InterfaceAfterPacket);
        }
        let mut body = Vec::new();
        body.extend_from_slice(&decl.link_type.code().to_le_bytes());
        body.extend_from_slice(&0u16.to_le_bytes()); // reserved
        body.extend_from_slice(&decl.snap_len.to_le_bytes());

        let mut options = Options::new();
        options.push_str(opt::IF_NAME, &decl.name)?;
        options.push_u8(opt::IF_TSRESOL, TSRESOL_MICROS)?;
        body.extend_from_slice(&options.finish());

        write_block(&mut self.out, block_type::INTERFACE_DESCRIPTION, &body)?;
        self.interfaces.push(DeclaredInterface {
            name: decl.name.clone(),
            ..DeclaredInterface::default()
        });
        Ok((self.interfaces.len() - 1) as u32)
    }

    /// How many interfaces have been declared.
    pub fn interface_count(&self) -> usize {
        self.interfaces.len()
    }

    /// Write one packet against a declared interface.
    fn write_packet(
        &mut self,
        interface_id: u32,
        packet: &CapturedPacket,
    ) -> Result<(), WriteError> {
        let idx = interface_id as usize;
        if idx >= self.interfaces.len() {
            return Err(WriteError::UndeclaredInterface { id: interface_id });
        }

        let micros = to_micros(packet.ts.as_nanos())?;

        // Section 13.3 writes `iface` only in a multi-interface capture. The
        // decision is safe to make here now because `declare_interface` refuses
        // to run after this point, so the count cannot change under packets
        // already written. That refusal is the whole reason this is a lookup
        // rather than a constant; see `WriteError::InterfaceAfterPacket`.
        let name = if self.interfaces.len() > 1 {
            Some(self.interfaces[idx].name.as_ref())
        } else {
            None
        };
        let annotation = Annotation::from_packet(packet, name).encode();

        let data = packet.data.as_ref();
        let mut body = Vec::with_capacity(data.len() + 64);
        body.extend_from_slice(&interface_id.to_le_bytes());
        body.extend_from_slice(&((micros >> 32) as u32).to_le_bytes());
        body.extend_from_slice(&((micros & 0xFFFF_FFFF) as u32).to_le_bytes());
        // Both lengths exactly as recorded. A file that contradicts itself is
        // reported by whatever reads it, not repaired here: repairing would
        // hide a defect in whatever produced it.
        body.extend_from_slice(&(data.len() as u32).to_le_bytes());
        body.extend_from_slice(&packet.orig_len.to_le_bytes());
        body.extend_from_slice(data);
        body.extend_from_slice(&vec![0u8; block::padding_for(data.len())]);

        let mut options = Options::new();
        options.push_str(opt::COMMENT, &annotation)?;
        body.extend_from_slice(&options.finish());

        write_block(&mut self.out, block_type::ENHANCED_PACKET, &body)?;
        self.interfaces[idx].last_ts_micros = Some(micros);
        self.wrote_a_packet = true;
        Ok(())
    }

    /// Write the Interface Statistics Blocks and consume the writer.
    fn write_statistics(mut self, stats: &CaptureStats) -> Result<(), WriteError> {
        for idx in 0..self.interfaces.len() {
            let ts = self.interfaces[idx].last_ts_micros.unwrap_or(0);

            let mut body = Vec::new();
            body.extend_from_slice(&(idx as u32).to_le_bytes());
            body.extend_from_slice(&((ts >> 32) as u32).to_le_bytes());
            body.extend_from_slice(&((ts & 0xFFFF_FFFF) as u32).to_le_bytes());

            let mut options = Options::new();
            // This interface's own report, not the capture-wide sum. Each
            // handle has its own driver buffer, and an Interface Statistics
            // Block that carried the total would tell anyone summing the blocks
            // that the capture lost several times what it lost.
            //
            // An interface with no report is written as zeroes rather than
            // omitted: it was declared, so it was watched, and saying nothing
            // about it would be a gap where a measurement belongs.
            let reported = stats
                .source_for(InterfaceId::new(idx as u32))
                .unwrap_or_default();
            options.push_u64(opt::ISB_IFRECV, reported.received)?;
            options.push_u64(opt::ISB_IFDROP, reported.interface_dropped)?;
            options.push_u64(opt::ISB_OSDROP, reported.kernel_dropped)?;

            // Correct because there is exactly one interface: a capture-wide
            // `SourceStats` is that interface's stats. With two, copying this
            // snapshot into each block would report every packet twice to
            // anyone summing them, which is why a second interface is refused
            // rather than written with numbers that do not mean what the
            // format says they mean.
            //
            // fragcap's own losses have no standard field. Omitting them would
            // satisfy section 13.2 as written and violate P-4, which is the
            // more important of the two; putting them in `isb_osdrop` would
            // report a fragcap loss as an operating system loss, which P-9
            // forbids. The comment carries them in the grammar the rest of the
            // file already uses.
            options.push_str(
                opt::COMMENT,
                &format!(
                    "fragcap:buffer_dropped={};sink_dropped={}",
                    stats.buffer_dropped, stats.sink_dropped
                ),
            )?;
            body.extend_from_slice(&options.finish());

            write_block(&mut self.out, block_type::INTERFACE_STATISTICS, &body)?;
        }
        self.out.flush()?;
        Ok(())
    }
}

/// Narrow nanoseconds to the declared microsecond resolution.
///
/// The single lossy conversion in the codebase, which the core `Timestamp`
/// documentation names this slice as the home of, so P-9 compliance has one
/// site to inspect. The loss is declared: the Interface Description Block
/// states the resolution the file actually carries.
///
/// Floors toward negative infinity rather than truncating toward zero, so the
/// conversion preserves ordering. A pre-epoch value is refused: pcapng
/// timestamps are unsigned, and both clamping and wrapping would record a time
/// that was not observed.
fn to_micros(nanos: i64) -> Result<u64, WriteError> {
    if nanos < 0 {
        return Err(WriteError::TimestampBeforeEpoch { nanos });
    }
    Ok(nanos as u64 / 1_000)
}

impl<W: Write + Send> Sink for PcapngWriter<W> {
    /// Writes against the interface the packet says it arrived on.
    ///
    /// S09 gave `CapturedPacket` that field, so this seam no longer has to
    /// assume. A packet naming an interface that was never declared is still
    /// refused rather than written against a fabricated declaration.
    fn write(&mut self, packet: &CapturedPacket) -> Result<(), SinkError> {
        self.write_packet(packet.interface.index(), packet)
            .map_err(Into::into)
    }

    fn flush(&mut self) -> Result<(), SinkError> {
        self.out
            .flush()
            .map_err(WriteError::from)
            .map_err(Into::into)
    }

    fn finish(self: Box<Self>, stats: &CaptureStats) -> Result<(), SinkError> {
        (*self).write_statistics(stats).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap_core::attribution::{Attribution, Fidelity};
    use fragcap_core::packet::{Payload, RawPacket, Timestamp};
    use fragcap_core::stats::SourceStats;
    use fragcap_core::{Direction, LinkType};

    fn decl(name: &str) -> InterfaceDeclaration {
        InterfaceDeclaration::new(LinkType::ETHERNET, 65_535, name)
    }

    fn packet(ts_nanos: i64, len: usize) -> CapturedPacket {
        let raw = RawPacket::new(
            Timestamp::from_nanos(ts_nanos),
            Payload::from(vec![0x41u8; len]),
            len as u32,
        );
        CapturedPacket::from_raw(raw, InterfaceId::default())
    }

    fn le16(b: &[u8], at: usize) -> u16 {
        u16::from_le_bytes([b[at], b[at + 1]])
    }

    fn le32(b: &[u8], at: usize) -> u32 {
        u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
    }

    /// Find an option value inside a block body, skipping the fixed prefix.
    fn find_option(body: &[u8], prefix: usize, code: u16) -> Option<Vec<u8>> {
        let mut i = prefix;
        while i + 4 <= body.len() {
            let c = le16(body, i);
            let len = le16(body, i + 2) as usize;
            if c == opt::END_OF_OPT {
                return None;
            }
            if c == code {
                return Some(body[i + 4..i + 4 + len].to_vec());
            }
            i += 4 + len + block::padding_for(len);
        }
        None
    }

    // --- section header -----------------------------------------------------

    #[test]
    fn the_section_header_comes_first_and_declares_fragcap() {
        let mut buf = Vec::new();
        PcapngWriter::new(&mut buf).unwrap();

        assert_eq!(le32(&buf, 0), 0x0A0D_0D0A, "section header block type");
        assert_eq!(
            le32(&buf, 8),
            0x1A2B_3C4D,
            "byte-order magic, little-endian"
        );
        assert_eq!(le16(&buf, 12), 1, "major version");
        assert_eq!(le16(&buf, 14), 0, "minor version");
        assert_eq!(
            i64::from_le_bytes(buf[16..24].try_into().unwrap()),
            -1,
            "section length unspecified: the writer streams"
        );

        let body = &buf[8..buf.len() - 4];
        let appl = find_option(body, 16, opt::SHB_USERAPPL).expect("shb_userappl");
        assert_eq!(String::from_utf8(appl).unwrap(), "fragcap/0.2.0");
        let comment = find_option(body, 16, opt::COMMENT).expect("opt_comment");
        assert_eq!(String::from_utf8(comment).unwrap(), PROFILE_COMMENT);
    }

    #[test]
    fn a_capture_with_no_packets_is_still_a_valid_file() {
        let mut buf = Vec::new();
        let w = PcapngWriter::new(&mut buf).unwrap();
        Box::new(w).finish(&CaptureStats::default()).unwrap();
        assert_eq!(le32(&buf, 0), 0x0A0D_0D0A);
        assert_eq!(
            le32(&buf, 4) as usize,
            buf.len(),
            "header is the whole file"
        );
    }

    // --- interface description ----------------------------------------------

    #[test]
    fn an_interface_declares_its_link_type_and_resolution() {
        let mut buf = Vec::new();
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        let shb_len = {
            let mut probe = Vec::new();
            PcapngWriter::new(&mut probe).unwrap();
            probe.len()
        };
        w.declare_interface(&decl("Ethernet 2")).unwrap();

        let idb = &buf[shb_len..];
        assert_eq!(
            le32(idb, 0),
            0x0000_0001,
            "interface description block type"
        );
        let body = &idb[8..idb.len() - 4];
        assert_eq!(le16(body, 0), LinkType::ETHERNET.code(), "link type");
        assert_eq!(le16(body, 2), 0, "reserved field is zero");
        assert_eq!(le32(body, 4), 65_535, "snap length");

        let name = find_option(body, 8, opt::IF_NAME).expect("if_name");
        assert_eq!(String::from_utf8(name).unwrap(), "Ethernet 2");
        let res = find_option(body, 8, opt::IF_TSRESOL).expect("if_tsresol");
        assert_eq!(res, vec![6], "microseconds, per section 12.7");
    }

    #[test]
    fn the_first_interface_is_identifier_zero() {
        let mut buf = Vec::new();
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        assert_eq!(w.declare_interface(&decl("eth0")).unwrap(), 0);
        assert_eq!(w.interface_count(), 1);
    }

    #[test]
    fn a_second_interface_is_accepted_before_the_first_packet() {
        // S06 refused every second interface. S09 replaces that with the
        // narrower rule that was actually needed, because `CapturedPacket` now
        // carries the identifier that made the refusal necessary.
        let mut buf = Vec::new();
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        assert_eq!(w.declare_interface(&decl("eth0")).unwrap(), 0);
        assert_eq!(w.declare_interface(&decl("eth1")).unwrap(), 1);
        assert_eq!(w.interface_count(), 2);
    }

    #[test]
    fn an_interface_declared_after_a_packet_is_refused() {
        // The real constraint. Section 13.3 decides the `iface` key from the
        // interface count, and a block already written cannot be revised, so a
        // late declaration would leave earlier packets without a key they
        // retrospectively needed.
        let mut buf = Vec::new();
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        w.declare_interface(&decl("eth0")).unwrap();
        w.write_packet(0, &packet(0, 4)).unwrap();

        let before = w.out.len();
        assert_eq!(
            w.declare_interface(&decl("eth1")),
            Err(WriteError::InterfaceAfterPacket)
        );
        assert_eq!(w.out.len(), before, "nothing was written for the refusal");
        assert_eq!(w.interface_count(), 1);
    }

    #[test]
    fn every_packet_references_the_declared_interface() {
        let mut buf = Vec::new();
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        let id = w.declare_interface(&decl("eth0")).unwrap();
        w.write_packet(id, &packet(1_000_000, 8)).unwrap();
        w.write_packet(id, &packet(2_000_000, 8)).unwrap();

        let mut ids = Vec::new();
        let mut i = 0;
        while i < buf.len() {
            let ty = le32(&buf, i);
            let len = le32(&buf, i + 4) as usize;
            if ty == block_type::ENHANCED_PACKET {
                ids.push(le32(&buf, i + 8));
            }
            i += len;
        }
        assert_eq!(ids, vec![0, 0]);
    }

    #[test]
    fn no_packet_carries_an_iface_key_in_a_single_interface_capture() {
        let mut buf = Vec::new();
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        w.declare_interface(&decl("eth0")).unwrap();
        w.write_packet(0, &packet(1_000_000, 8)).unwrap();

        let epb = find_block(&buf, block_type::ENHANCED_PACKET).unwrap();
        let body = &epb[8..epb.len() - 4];
        let comment =
            String::from_utf8(find_option(body, 20 + 8, opt::COMMENT).expect("annotation"))
                .unwrap();
        assert!(!comment.contains("iface="), "got {comment}");
    }

    // --- enhanced packet ----------------------------------------------------

    #[test]
    fn a_packet_carries_its_lengths_and_its_annotation() {
        let mut buf = Vec::new();
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        w.declare_interface(&decl("eth0")).unwrap();
        let mut p = packet(1_500_000_000, 60);
        p.direction = Some(Direction::Outbound);
        p.attribution = Some(Attribution::new(7412, "eso64.exe", Fidelity::Live));
        w.write_packet(0, &p).unwrap();

        let epb = find_block(&buf, block_type::ENHANCED_PACKET).expect("a packet block");
        let body = &epb[8..epb.len() - 4];
        assert_eq!(le32(body, 0), 0, "interface identifier");
        let micros = ((le32(body, 4) as u64) << 32) | le32(body, 8) as u64;
        assert_eq!(micros, 1_500_000, "nanoseconds narrowed to microseconds");
        assert_eq!(le32(body, 12), 60, "captured length");
        assert_eq!(le32(body, 16), 60, "original length");

        let comment = find_option(body, 20 + 60, opt::COMMENT).expect("annotation");
        assert_eq!(
            String::from_utf8(comment).unwrap(),
            "fragcap:pid=7412;proc=eso64.exe;dir=out;attr=live"
        );
    }

    #[test]
    fn packet_data_is_padded_without_inflating_the_declared_length() {
        for len in [1usize, 3, 4, 5, 60] {
            let mut buf = Vec::new();
            let mut w = PcapngWriter::new(&mut buf).unwrap();
            w.declare_interface(&decl("eth0")).unwrap();
            w.write_packet(0, &packet(0, len)).unwrap();

            let epb = find_block(&buf, block_type::ENHANCED_PACKET).unwrap();
            let body = &epb[8..epb.len() - 4];
            assert_eq!(
                le32(body, 12) as usize,
                len,
                "captured length excludes padding"
            );
            assert_eq!(epb.len() % 4, 0, "the block occupies whole words");
        }
    }

    #[test]
    fn an_undeclared_interface_is_refused_not_invented() {
        let mut buf = Vec::new();
        let before;
        let err;
        {
            let mut w = PcapngWriter::new(&mut buf).unwrap();
            before = w.out.len();
            err = w.write_packet(0, &packet(0, 4)).unwrap_err();
            assert_eq!(
                w.out.len(),
                before,
                "nothing written for the refused packet"
            );
        }
        assert_eq!(err, WriteError::UndeclaredInterface { id: 0 });
    }

    #[test]
    fn contradictory_lengths_are_written_as_recorded() {
        // S04 settled this for reading: a file that contradicts itself is
        // reported, not repaired. Repairing would hide a defect in whatever
        // produced it.
        let mut buf = Vec::new();
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        w.declare_interface(&decl("eth0")).unwrap();
        let raw = RawPacket::new(Timestamp::from_nanos(0), Payload::from(vec![0u8; 40]), 8);
        w.write_packet(0, &CapturedPacket::from_raw(raw, InterfaceId::default()))
            .unwrap();

        let epb = find_block(&buf, block_type::ENHANCED_PACKET).unwrap();
        let body = &epb[8..epb.len() - 4];
        assert_eq!(le32(body, 12), 40, "captured length as recorded");
        assert_eq!(
            le32(body, 16),
            8,
            "original length as recorded, though smaller"
        );
    }

    #[test]
    fn a_packet_longer_than_the_snap_length_is_written_unrepaired() {
        let mut buf = Vec::new();
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        w.declare_interface(&InterfaceDeclaration::new(LinkType::ETHERNET, 16, "eth0"))
            .unwrap();
        w.write_packet(0, &packet(0, 64)).unwrap();
        let epb = find_block(&buf, block_type::ENHANCED_PACKET).unwrap();
        assert_eq!(le32(&epb[8..], 12), 64);
    }

    // --- timestamps ---------------------------------------------------------

    #[test]
    fn nanoseconds_floor_to_microseconds() {
        assert_eq!(to_micros(0).unwrap(), 0);
        assert_eq!(to_micros(999).unwrap(), 0, "sub-microsecond truncates");
        assert_eq!(to_micros(1_000).unwrap(), 1);
        assert_eq!(to_micros(1_999).unwrap(), 1);
        assert_eq!(to_micros(1_500_000_000).unwrap(), 1_500_000);
    }

    #[test]
    fn the_conversion_preserves_ordering() {
        let mut prev = 0;
        for n in [0i64, 1, 999, 1_000, 1_001, 2_000, 1_000_000] {
            let m = to_micros(n).unwrap();
            assert!(m >= prev, "flooring must not reorder observations");
            prev = m;
        }
    }

    #[test]
    fn a_pre_epoch_timestamp_is_refused_not_clamped() {
        // Clamping would record the observation at a time it did not happen;
        // wrapping would place it half a million years out. Neither is
        // recoverable by a reader.
        assert_eq!(
            to_micros(-1),
            Err(WriteError::TimestampBeforeEpoch { nanos: -1 })
        );
        let mut buf = Vec::new();
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        w.declare_interface(&decl("eth0")).unwrap();
        let before = w.out.len();
        assert!(w.write_packet(0, &packet(-1_000, 4)).is_err());
        assert_eq!(
            w.out.len(),
            before,
            "nothing written for the refused packet"
        );
    }

    // --- interface statistics -----------------------------------------------

    fn stats_with_everything() -> CaptureStats {
        CaptureStats {
            sources: vec![(
                InterfaceId::default(),
                SourceStats {
                    received: 1_000,
                    interface_dropped: 7,
                    kernel_dropped: 13,
                },
            )],
            buffer_dropped: 5,
            sink_dropped: 3,
            ..Default::default()
        }
    }

    #[test]
    fn statistics_carry_every_counter_somewhere_in_the_file() {
        let mut buf = Vec::new();
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        w.declare_interface(&decl("eth0")).unwrap();
        w.write_packet(0, &packet(4_000_000, 8)).unwrap();
        Box::new(w).finish(&stats_with_everything()).unwrap();

        let isb = find_block(&buf, block_type::INTERFACE_STATISTICS).expect("a statistics block");
        let body = &isb[8..isb.len() - 4];
        let u64_at = |code| {
            let v = find_option(body, 12, code).expect("counter option");
            u64::from_le_bytes(v.try_into().unwrap())
        };
        assert_eq!(u64_at(opt::ISB_IFRECV), 1_000);
        assert_eq!(u64_at(opt::ISB_IFDROP), 7);
        assert_eq!(u64_at(opt::ISB_OSDROP), 13);

        let comment =
            String::from_utf8(find_option(body, 12, opt::COMMENT).expect("comment")).unwrap();
        assert_eq!(comment, "fragcap:buffer_dropped=5;sink_dropped=3");
    }

    #[test]
    fn fragcap_losses_are_never_reported_as_operating_system_losses() {
        let mut buf = Vec::new();
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        w.declare_interface(&decl("eth0")).unwrap();
        let s = CaptureStats {
            buffer_dropped: 99,
            sink_dropped: 42,
            ..Default::default()
        };
        Box::new(w).finish(&s).unwrap();

        let isb = find_block(&buf, block_type::INTERFACE_STATISTICS).unwrap();
        let body = &isb[8..isb.len() - 4];
        let osdrop = find_option(body, 12, opt::ISB_OSDROP).unwrap();
        assert_eq!(
            u64::from_le_bytes(osdrop.try_into().unwrap()),
            0,
            "a fragcap buffer drop is not an operating system drop"
        );
    }

    #[test]
    fn the_statistics_timestamp_comes_from_the_last_packet_not_a_clock() {
        let mut buf = Vec::new();
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        w.declare_interface(&decl("eth0")).unwrap();
        w.write_packet(0, &packet(1_000_000, 8)).unwrap();
        w.write_packet(0, &packet(9_000_000, 8)).unwrap();
        Box::new(w).finish(&CaptureStats::default()).unwrap();

        let isb = find_block(&buf, block_type::INTERFACE_STATISTICS).unwrap();
        let body = &isb[8..isb.len() - 4];
        let ts = ((le32(body, 4) as u64) << 32) | le32(body, 8) as u64;
        assert_eq!(ts, 9_000, "the last packet's microseconds, not now");
    }

    #[test]
    fn an_interface_with_no_packets_reports_a_zero_timestamp() {
        let mut buf = Vec::new();
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        w.declare_interface(&decl("eth0")).unwrap();
        Box::new(w).finish(&CaptureStats::default()).unwrap();

        let isb = find_block(&buf, block_type::INTERFACE_STATISTICS).unwrap();
        let body = &isb[8..isb.len() - 4];
        assert_eq!(le32(body, 4), 0);
        assert_eq!(le32(body, 8), 0);
    }

    #[test]
    fn one_statistics_block_per_declared_interface() {
        let mut buf = Vec::new();
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        w.declare_interface(&decl("eth0")).unwrap();
        Box::new(w).finish(&CaptureStats::default()).unwrap();

        let mut count = 0;
        let mut i = 0;
        while i < buf.len() {
            if le32(&buf, i) == block_type::INTERFACE_STATISTICS {
                count += 1;
            }
            i += le32(&buf, i + 4) as usize;
        }
        assert_eq!(count, 1);
    }

    #[test]
    fn the_statistics_totals_are_not_multiplied_across_blocks() {
        // The defect a second interface would have introduced: summing
        // `isb_ifrecv` over the blocks must equal what the source reported,
        // not a multiple of it.
        let mut buf = Vec::new();
        let mut w = PcapngWriter::new(&mut buf).unwrap();
        w.declare_interface(&decl("eth0")).unwrap();
        Box::new(w).finish(&stats_with_everything()).unwrap();

        let mut total = 0u64;
        let mut i = 0;
        while i < buf.len() {
            let len = le32(&buf, i + 4) as usize;
            if le32(&buf, i) == block_type::INTERFACE_STATISTICS {
                let body = &buf[i + 8..i + len - 4];
                let v = find_option(body, 12, opt::ISB_IFRECV).expect("isb_ifrecv");
                total += u64::from_le_bytes(v.try_into().unwrap());
            }
            i += len;
        }
        assert_eq!(total, 1_000, "the capture received 1000 packets, once");
    }

    // --- determinism --------------------------------------------------------

    #[test]
    fn the_same_input_produces_the_same_bytes() {
        let render = || {
            let mut buf = Vec::new();
            let mut w = PcapngWriter::new(&mut buf).unwrap();
            w.declare_interface(&decl("eth0")).unwrap();
            let mut p = packet(1_234_567_000, 40);
            p.attribution = Some(Attribution::new(1, "a.exe", Fidelity::Live));
            p.direction = Some(Direction::Inbound);
            w.write_packet(0, &p).unwrap();
            Box::new(w).finish(&stats_with_everything()).unwrap();
            buf
        };
        assert_eq!(render(), render());
    }

    /// Walk to the first block of a given type.
    fn find_block(buf: &[u8], want: u32) -> Option<Vec<u8>> {
        let mut i = 0;
        while i + 8 <= buf.len() {
            let ty = le32(buf, i);
            let len = le32(buf, i + 4) as usize;
            if ty == want {
                return Some(buf[i..i + len].to_vec());
            }
            i += len;
        }
        None
    }
}
