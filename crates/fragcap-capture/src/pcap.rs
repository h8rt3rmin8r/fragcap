// SPDX-License-Identifier: Apache-2.0

//! Classic pcap reading, for the fixture corpus of specification section 25.3.
//!
//! A pcap file is a twenty-four byte header followed by records, each a
//! sixteen byte record header and exactly its captured length of packet data.
//! That is the whole format. fragcap writes pcapng, which is a much larger
//! format; this reads the small one, because the corpus is written in it and
//! because a reader with no dependencies is one fewer thing between a
//! contributor and a running test.
//!
//! # What the magic number decides
//!
//! Byte order and timestamp resolution are both properties of the file, read
//! from its magic number and never from the host. A file written on a
//! big-endian machine must yield the same packets on a little-endian one, and a
//! nanosecond capture read as microseconds would report timestamps a thousand
//! times too small while staying entirely plausible.
//!
//! # What is counted, and what is still delivered
//!
//! Four ways a record is not what the file said it would be, each with its own
//! counter, per constitution P-4. Two of them mean the bytes are not there and
//! reading stops. The other two mean the file contradicts itself about bytes
//! that are present, and the record is delivered anyway, because P-9 does not
//! permit withholding an observation on the strength of a header field being
//! wrong. Reconciling the contradiction by adjusting a length would be worse
//! still: it would hide a defect in whatever wrote the file.

use std::fmt;

use fragcap_core::error::SourceError;
use fragcap_core::link::LinkType;
use fragcap_core::packet::{Payload, RawPacket, Timestamp};

/// The file header, and the smallest thing that can be a pcap file.
const FILE_HEADER_LEN: usize = 24;
/// Every record header, whatever the byte order.
const RECORD_HEADER_LEN: usize = 16;

/// Microsecond-resolution pcap, read in the order that yields this value.
const MAGIC_MICROS: u32 = 0xa1b2_c3d4;
/// Nanosecond-resolution pcap. Not exotic: it is what a modern capture writes
/// when asked for full resolution.
const MAGIC_NANOS: u32 = 0xa1b2_3c4d;

/// How to read the file's multi-byte fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Order {
    Little,
    Big,
}

impl Order {
    fn u32(self, b: [u8; 4]) -> u32 {
        match self {
            Order::Little => u32::from_le_bytes(b),
            Order::Big => u32::from_be_bytes(b),
        }
    }
}

/// What the record header's fractional timestamp field counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Unit {
    Micros,
    Nanos,
}

impl Unit {
    /// Convert a fraction to nanoseconds. Lossless in both directions: the core
    /// timestamp is nanoseconds, which is finer than either unit here, so
    /// nothing rounds. See slice S02 decision D-2.
    fn to_nanos(self, fraction: u32) -> u32 {
        match self {
            Unit::Micros => fraction.saturating_mul(1_000),
            Unit::Nanos => fraction,
        }
    }
}

/// One counter per way a record was not delivered as the file described it.
///
/// Constitution P-4: a reader that quietly drops a record turns a damaged
/// fixture into a passing test over fewer packets than intended, which is the
/// failure hardest to notice and most corrosive to every test built on it.
///
/// The two `skipped` causes are the file failing to supply bytes. The other two
/// are the file describing present bytes wrongly, and those records are still
/// delivered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReplayStats {
    /// The file ended part way through a record header, or through data whose
    /// declared length a complete file of this size could have held. Indicates
    /// a truncated transfer or a partial write. Not delivered.
    pub truncated_record: u64,
    /// A record declared more data than the whole file contains, so no reading
    /// position could satisfy it. Indicates a corrupted length field rather
    /// than a short file. Not delivered, and reading stops rather than
    /// resynchronizing on whatever follows.
    pub impossible_length: u64,
    /// A record's captured length exceeds its original on-wire length, which
    /// cannot be true of any real capture. Delivered, with both lengths exactly
    /// as recorded.
    pub caplen_exceeds_wire: u64,
    /// A record's captured length exceeds the snapshot length the file
    /// declares. The file contradicts itself; the bytes are real. Delivered.
    pub caplen_exceeds_snaplen: u64,
}

impl ReplayStats {
    /// Records the file could not supply. Computed from the two named causes,
    /// never stored, so it cannot drift from them.
    ///
    /// Deliberately excludes the two delivered causes. Adding a count of
    /// records handed over with a complaint to a count of records never handed
    /// over at all would produce a number meaning nothing.
    pub fn skipped(&self) -> u64 {
        self.truncated_record.saturating_add(self.impossible_length)
    }

    /// Whether the file was read whole.
    pub fn read_whole_file(&self) -> bool {
        self.skipped() == 0
    }

    /// Every counter in a fixed order, so a test can assert exactly one moved.
    #[cfg(test)]
    pub(crate) fn counters(&self) -> [u64; 4] {
        [
            self.truncated_record,
            self.impossible_length,
            self.caplen_exceeds_wire,
            self.caplen_exceeds_snaplen,
        ]
    }
}

impl fmt::Display for ReplayStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "truncated={} impossible={} caplen>wire={} caplen>snaplen={}",
            self.truncated_record,
            self.impossible_length,
            self.caplen_exceeds_wire,
            self.caplen_exceeds_snaplen
        )
    }
}

/// Decodes a classic pcap file held in memory.
///
/// Whole-file rather than streaming, deliberately. Fixtures are capped at
/// 64 KiB, so the memory is irrelevant, and a reader with no buffering has no
/// partial-read path to get wrong. The live source in S09 shares nothing with
/// this and has entirely different constraints.
pub struct PcapReader {
    data: Vec<u8>,
    cursor: usize,
    order: Order,
    unit: Unit,
    link_type: LinkType,
    snaplen: u32,
    stats: ReplayStats,
    /// Set once a cause stops reading, so a caller that keeps asking gets
    /// nothing rather than a resynchronization attempt.
    stopped: bool,
}

impl fmt::Debug for PcapReader {
    /// Written by hand rather than derived: the derived form would print the
    /// whole file, which is unreadable in a panic message and enormous in a
    /// failing assertion.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PcapReader")
            .field("bytes", &self.data.len())
            .field("cursor", &self.cursor)
            .field("order", &self.order)
            .field("unit", &self.unit)
            .field("link_type", &self.link_type)
            .field("snaplen", &self.snaplen)
            .field("stopped", &self.stopped)
            .field("stats", &self.stats)
            .finish()
    }
}

impl PcapReader {
    /// Parse the file header and prepare to read records.
    ///
    /// Fails on anything that is not a pcap file. That is a terminal condition
    /// rather than a counter, because there is no capture to account for.
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, SourceError> {
        if data.len() < FILE_HEADER_LEN {
            return Err(SourceError::Backend {
                detail: format!(
                    "not a capture file: {} bytes is shorter than a {FILE_HEADER_LEN} byte header",
                    data.len()
                ),
            });
        }

        let magic = [data[0], data[1], data[2], data[3]];
        // Read both ways and let the file decide. Never consult the host's own
        // byte order: the same file must yield the same packets anywhere.
        let (order, unit) = match (u32::from_le_bytes(magic), u32::from_be_bytes(magic)) {
            (MAGIC_MICROS, _) => (Order::Little, Unit::Micros),
            (MAGIC_NANOS, _) => (Order::Little, Unit::Nanos),
            (_, MAGIC_MICROS) => (Order::Big, Unit::Micros),
            (_, MAGIC_NANOS) => (Order::Big, Unit::Nanos),
            _ => {
                return Err(SourceError::Backend {
                    detail: format!(
                        "not a capture file: magic {:02x}{:02x}{:02x}{:02x} is not a pcap magic",
                        magic[0], magic[1], magic[2], magic[3]
                    ),
                })
            }
        };

        let snaplen = order.u32([data[16], data[17], data[18], data[19]]);
        let link_code = order.u32([data[20], data[21], data[22], data[23]]);
        // The registry codes are sixteen bit in practice; a file declaring more
        // is truncated to what the type can hold rather than rejected, because
        // an unparseable link type is the parser's business to count and
        // refusing the file would make that counter unreachable through a
        // fixture.
        let link_type = LinkType::from_code(link_code as u16);

        Ok(PcapReader {
            data,
            cursor: FILE_HEADER_LEN,
            order,
            unit,
            link_type,
            snaplen,
            stats: ReplayStats::default(),
            stopped: false,
        })
    }

    /// The link layer encapsulation the file declares its frames carry.
    pub fn link_type(&self) -> LinkType {
        self.link_type
    }

    /// The snapshot length the file declares it was captured under.
    pub fn snaplen(&self) -> u32 {
        self.snaplen
    }

    pub fn stats(&self) -> &ReplayStats {
        &self.stats
    }

    /// The next record, or `None` at the end of the file.
    ///
    /// `None` also follows a cause that stopped reading early. The counters are
    /// how a caller tells the two apart: `stats().skipped()` is zero for a
    /// clean end.
    pub fn next_record(&mut self) -> Option<RawPacket> {
        if self.stopped {
            return None;
        }
        let remaining = self.data.len() - self.cursor;
        if remaining == 0 {
            self.stopped = true;
            return None;
        }
        if remaining < RECORD_HEADER_LEN {
            // A partial record header. The file was cut off mid-record.
            self.stats.truncated_record = self.stats.truncated_record.saturating_add(1);
            self.stopped = true;
            return None;
        }

        let at = self.cursor;
        let secs = self.order.u32([
            self.data[at],
            self.data[at + 1],
            self.data[at + 2],
            self.data[at + 3],
        ]);
        let fraction = self.order.u32([
            self.data[at + 4],
            self.data[at + 5],
            self.data[at + 6],
            self.data[at + 7],
        ]);
        let caplen = self.order.u32([
            self.data[at + 8],
            self.data[at + 9],
            self.data[at + 10],
            self.data[at + 11],
        ]) as usize;
        let orig_len = self.order.u32([
            self.data[at + 12],
            self.data[at + 13],
            self.data[at + 14],
            self.data[at + 15],
        ]);

        let body = at + RECORD_HEADER_LEN;
        let available = self.data.len() - body;

        if caplen > self.data.len() {
            // No file of this size could hold such a record, so this is a
            // corrupted length field rather than a short file. The distinction
            // matters because the remedies differ: one is a bad transfer, the
            // other a bad writer.
            self.stats.impossible_length = self.stats.impossible_length.saturating_add(1);
            self.stopped = true;
            return None;
        }
        if caplen > available {
            // Plausible for a whole file this size, but this one ends first.
            self.stats.truncated_record = self.stats.truncated_record.saturating_add(1);
            self.stopped = true;
            return None;
        }

        // The two causes below describe present bytes wrongly. Both are
        // counted and neither withholds the record. They are not mutually
        // exclusive: a record can be wrong in both ways, and then both move.
        if caplen as u64 > u64::from(orig_len) {
            self.stats.caplen_exceeds_wire = self.stats.caplen_exceeds_wire.saturating_add(1);
        }
        if self.snaplen > 0 && caplen as u64 > u64::from(self.snaplen) {
            self.stats.caplen_exceeds_snaplen = self.stats.caplen_exceeds_snaplen.saturating_add(1);
        }

        self.cursor = body + caplen;
        Some(RawPacket::new(
            Timestamp::from_parts(i64::from(secs), self.unit.to_nanos(fraction)),
            Payload::copy_from_slice(&self.data[body..body + caplen]),
            orig_len,
        ))
    }
}

#[cfg(test)]
pub(crate) mod build {
    //! Byte-level pcap construction for tests.
    //!
    //! Written from the format description in the slice's research rather than
    //! from the reader, deliberately: a builder derived from the reader would
    //! agree with a misreading of the format and prove nothing.

    pub const MAGIC_MICROS_LE: [u8; 4] = [0xd4, 0xc3, 0xb2, 0xa1];
    pub const MAGIC_MICROS_BE: [u8; 4] = [0xa1, 0xb2, 0xc3, 0xd4];
    pub const MAGIC_NANOS_LE: [u8; 4] = [0x4d, 0x3c, 0xb2, 0xa1];
    pub const MAGIC_NANOS_BE: [u8; 4] = [0xa1, 0xb2, 0x3c, 0x4d];

    /// Which way to write multi-byte fields. Must match the magic.
    #[derive(Clone, Copy)]
    pub enum Wide {
        Le,
        Be,
    }

    impl Wide {
        pub fn u32(self, v: u32) -> [u8; 4] {
            match self {
                Wide::Le => v.to_le_bytes(),
                Wide::Be => v.to_be_bytes(),
            }
        }
    }

    impl Wide {
        pub fn u16(self, v: u16) -> [u8; 2] {
            match self {
                Wide::Le => v.to_le_bytes(),
                Wide::Be => v.to_be_bytes(),
            }
        }
    }

    pub struct File {
        pub magic: [u8; 4],
        pub wide: Wide,
        pub snaplen: u32,
        pub link_type: u32,
    }

    impl Default for File {
        fn default() -> Self {
            File {
                magic: MAGIC_MICROS_LE,
                wide: Wide::Le,
                snaplen: 65_535,
                link_type: 1,
            }
        }
    }

    pub fn header(f: &File) -> Vec<u8> {
        let mut out = Vec::with_capacity(24);
        out.extend_from_slice(&f.magic);
        // Two independent two-byte fields, major then minor. Writing them as
        // one four-byte value happens to be right in little-endian and declares
        // version 4.2 in big-endian, which made the big-endian test inputs
        // non-standard captures while the tests still passed, because the
        // reader ignores the version. Raised in review of pull request 7.
        out.extend_from_slice(&f.wide.u16(2)[..]);
        out.extend_from_slice(&f.wide.u16(4)[..]);
        out.extend_from_slice(&f.wide.u32(0)[..]);
        out.extend_from_slice(&f.wide.u32(0)[..]);
        out.extend_from_slice(&f.wide.u32(f.snaplen)[..]);
        out.extend_from_slice(&f.wide.u32(f.link_type)[..]);
        out
    }

    /// A record whose four header fields are settable independently, so a test
    /// can build one whose fields contradict each other.
    pub struct Record<'a> {
        pub secs: u32,
        pub fraction: u32,
        pub caplen: Option<u32>,
        pub orig_len: Option<u32>,
        pub payload: &'a [u8],
    }

    impl<'a> Record<'a> {
        pub fn new(payload: &'a [u8]) -> Self {
            Record {
                secs: 1_000_000,
                fraction: 0,
                caplen: None,
                orig_len: None,
                payload,
            }
        }
    }

    pub fn record(wide: Wide, r: &Record<'_>) -> Vec<u8> {
        let caplen = r.caplen.unwrap_or(r.payload.len() as u32);
        let orig_len = r.orig_len.unwrap_or(r.payload.len() as u32);
        let mut out = Vec::with_capacity(16 + r.payload.len());
        out.extend_from_slice(&wide.u32(r.secs)[..]);
        out.extend_from_slice(&wide.u32(r.fraction)[..]);
        out.extend_from_slice(&wide.u32(caplen)[..]);
        out.extend_from_slice(&wide.u32(orig_len)[..]);
        out.extend_from_slice(r.payload);
        out
    }

    /// A whole file with one record carrying `payload`.
    pub fn one(payload: &[u8]) -> Vec<u8> {
        let f = File::default();
        let mut out = header(&f);
        out.extend_from_slice(&record(f.wide, &Record::new(payload)));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::build::{self, Wide};
    use super::*;

    fn read_all(bytes: Vec<u8>) -> (Vec<RawPacket>, ReplayStats) {
        let mut r = PcapReader::from_bytes(bytes).expect("the file opens");
        let mut out = Vec::new();
        while let Some(p) = r.next_record() {
            out.push(p);
        }
        (out, *r.stats())
    }

    fn moved(stats: &ReplayStats) -> Vec<usize> {
        stats
            .counters()
            .iter()
            .enumerate()
            .filter(|(_, c)| **c != 0)
            .map(|(i, _)| i)
            .collect()
    }

    // FR-002 and research R-2. Byte order and resolution come from the file.
    #[test]
    fn all_four_magic_numbers_open() {
        for (magic, wide) in [
            (build::MAGIC_MICROS_LE, Wide::Le),
            (build::MAGIC_MICROS_BE, Wide::Be),
            (build::MAGIC_NANOS_LE, Wide::Le),
            (build::MAGIC_NANOS_BE, Wide::Be),
        ] {
            let f = build::File {
                magic,
                wide,
                ..build::File::default()
            };
            let mut bytes = build::header(&f);
            bytes.extend_from_slice(&build::record(wide, &build::Record::new(&[1, 2, 3, 4])));
            let (packets, stats) = read_all(bytes);
            assert_eq!(packets.len(), 1, "magic {magic:02x?} must yield its record");
            assert_eq!(packets[0].data.as_ref(), &[1, 2, 3, 4]);
            assert_eq!(moved(&stats), Vec::<usize>::new());
        }
    }

    // SC-003. The same capture written four ways is one capture.
    #[test]
    fn the_same_capture_reads_identically_however_it_was_written() {
        let mut sequences = Vec::new();
        for (magic, wide, fraction) in [
            (build::MAGIC_MICROS_LE, Wide::Le, 250_000u32),
            (build::MAGIC_MICROS_BE, Wide::Be, 250_000),
            (build::MAGIC_NANOS_LE, Wide::Le, 250_000_000),
            (build::MAGIC_NANOS_BE, Wide::Be, 250_000_000),
        ] {
            let f = build::File {
                magic,
                wide,
                ..build::File::default()
            };
            let mut bytes = build::header(&f);
            let mut rec = build::Record::new(&[9, 8, 7]);
            rec.secs = 1_700_000_000;
            rec.fraction = fraction;
            bytes.extend_from_slice(&build::record(wide, &rec));
            sequences.push(read_all(bytes).0);
        }
        for other in &sequences[1..] {
            assert_eq!(
                &sequences[0], other,
                "byte order and resolution are properties of the file, not the packets"
            );
        }
        assert_eq!(
            sequences[0][0].ts,
            Timestamp::from_parts(1_700_000_000, 250_000_000)
        );
    }

    // FR-003. A sub-microsecond value must survive a nanosecond file.
    #[test]
    fn a_nanosecond_fraction_is_not_rounded_to_microseconds() {
        let f = build::File {
            magic: build::MAGIC_NANOS_LE,
            ..build::File::default()
        };
        let mut bytes = build::header(&f);
        let mut rec = build::Record::new(&[0]);
        rec.secs = 5;
        rec.fraction = 999;
        bytes.extend_from_slice(&build::record(Wide::Le, &rec));
        let (packets, _) = read_all(bytes);
        assert_eq!(packets[0].ts.as_nanos(), 5_000_000_999);
    }

    #[test]
    fn a_microsecond_fraction_scales_by_a_thousand() {
        let mut bytes = build::header(&build::File::default());
        let mut rec = build::Record::new(&[0]);
        rec.secs = 5;
        rec.fraction = 999;
        bytes.extend_from_slice(&build::record(Wide::Le, &rec));
        let (packets, _) = read_all(bytes);
        assert_eq!(packets[0].ts.as_nanos(), 5_000_999_000);
    }

    // FR-019 and SC-002.
    #[test]
    fn reading_the_same_bytes_twice_yields_identical_sequences() {
        let bytes = build::one(&[4, 5, 6, 7]);
        assert_eq!(read_all(bytes.clone()).0, read_all(bytes).0);
    }

    // FR-006 and FR-005.
    #[test]
    fn payload_bytes_and_the_wire_length_arrive_exactly_as_recorded() {
        let payload: Vec<u8> = (0u8..=255).collect();
        let f = build::File::default();
        let mut bytes = build::header(&f);
        let mut rec = build::Record::new(&payload);
        rec.orig_len = Some(1514);
        bytes.extend_from_slice(&build::record(Wide::Le, &rec));
        let (packets, _) = read_all(bytes);
        assert_eq!(packets[0].data.as_ref(), &payload[..], "bytes were altered");
        assert_eq!(packets[0].orig_len, 1514, "the wire length was not carried");
        assert_eq!(packets[0].captured_len(), 256);
        assert!(packets[0].is_truncated());
    }

    #[test]
    fn a_file_shorter_than_a_header_does_not_open() {
        let e = PcapReader::from_bytes(vec![0; 23]).expect_err("23 bytes is not a pcap file");
        assert!(matches!(e, SourceError::Backend { .. }));
        assert!(e.to_string().contains("shorter than"));
    }

    #[test]
    fn an_unrecognized_magic_does_not_open() {
        let mut bytes = vec![0u8; 24];
        bytes[..4].copy_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
        let e = PcapReader::from_bytes(bytes).expect_err("that is not a pcap magic");
        assert!(e.to_string().contains("magic"));
    }

    #[test]
    fn an_empty_capture_opens_and_yields_nothing() {
        let (packets, stats) = read_all(build::header(&build::File::default()));
        assert!(packets.is_empty());
        assert!(
            stats.read_whole_file(),
            "no packets is not the same as damage"
        );
    }

    // FR-008. A partial record header.
    #[test]
    fn a_partial_record_header_is_a_truncated_record() {
        let mut bytes = build::header(&build::File::default());
        bytes.extend_from_slice(&[0; 9]);
        let (packets, stats) = read_all(bytes);
        assert!(packets.is_empty());
        assert_eq!(moved(&stats), vec![0]);
        assert_eq!(stats.truncated_record, 1);
    }

    #[test]
    fn records_before_a_truncated_one_are_still_delivered() {
        let f = build::File::default();
        let mut bytes = build::header(&f);
        bytes.extend_from_slice(&build::record(Wide::Le, &build::Record::new(&[1, 1, 1])));
        bytes.extend_from_slice(&build::record(Wide::Le, &build::Record::new(&[2, 2, 2])));
        bytes.extend_from_slice(&[0; 5]);
        let (packets, stats) = read_all(bytes);
        assert_eq!(
            packets.len(),
            2,
            "damage at the end must not lose the start"
        );
        assert_eq!(stats.truncated_record, 1);
    }

    // FR-008 again, the data rather than the header. The file must be large
    // enough that the declared length is plausible for a whole file this size,
    // or the impossible-length cause fires first and the two are not being
    // told apart at all.
    #[test]
    fn data_cut_short_is_a_truncated_record() {
        let f = build::File::default();
        let mut bytes = build::header(&f);
        bytes.extend_from_slice(&build::record(Wide::Le, &build::Record::new(&[0; 100])));
        let mut rec = build::Record::new(&[1, 2, 3]);
        rec.caplen = Some(64);
        bytes.extend_from_slice(&build::record(Wide::Le, &rec));
        let (packets, stats) = read_all(bytes);
        assert_eq!(packets.len(), 1, "the complete record before it survives");
        assert_eq!(moved(&stats), vec![0], "cut short, not corrupt");
    }

    #[test]
    fn the_two_undelivered_causes_are_told_apart_by_the_file_size() {
        // 64 bytes is plausible in a large file and impossible in a small one,
        // and the same declared length must therefore reach different causes.
        // This is the assertion that would fail if either rule swallowed the
        // other.
        let f = build::File::default();
        let mut small = build::header(&f);
        let mut rec = build::Record::new(&[1, 2, 3]);
        rec.caplen = Some(64);
        small.extend_from_slice(&build::record(Wide::Le, &rec));
        assert_eq!(moved(&read_all(small).1), vec![1]);
    }

    // FR-009. Distinguished from truncation because no file this size could
    // ever have held it.
    #[test]
    fn a_length_no_file_this_size_could_hold_is_impossible_rather_than_truncated() {
        let f = build::File::default();
        let mut bytes = build::header(&f);
        let mut rec = build::Record::new(&[1, 2, 3]);
        rec.caplen = Some(4_000_000);
        bytes.extend_from_slice(&build::record(Wide::Le, &rec));
        let (packets, stats) = read_all(bytes);
        assert!(packets.is_empty());
        assert_eq!(moved(&stats), vec![1], "a corrupt field, not a short file");
        assert_eq!(stats.impossible_length, 1);
    }

    // FR-010. Delivered, and neither length repaired.
    #[test]
    fn a_captured_length_over_the_wire_length_is_counted_and_still_delivered() {
        let f = build::File::default();
        let mut bytes = build::header(&f);
        let mut rec = build::Record::new(&[7; 40]);
        rec.orig_len = Some(10);
        bytes.extend_from_slice(&build::record(Wide::Le, &rec));
        let (packets, stats) = read_all(bytes);
        assert_eq!(packets.len(), 1, "the bytes are present and real");
        assert_eq!(packets[0].captured_len(), 40);
        assert_eq!(
            packets[0].orig_len, 10,
            "the contradiction is the observation; repairing it would hide it"
        );
        assert_eq!(moved(&stats), vec![2]);
    }

    // FR-011.
    #[test]
    fn a_captured_length_over_the_snapshot_length_is_counted_and_still_delivered() {
        let f = build::File {
            snaplen: 16,
            ..build::File::default()
        };
        let mut bytes = build::header(&f);
        bytes.extend_from_slice(&build::record(Wide::Le, &build::Record::new(&[3; 40])));
        let (packets, stats) = read_all(bytes);
        assert_eq!(packets.len(), 1);
        assert_eq!(moved(&stats), vec![3]);
    }

    #[test]
    fn a_record_wrong_in_two_ways_moves_two_counters() {
        // The delivered causes are properties of a record rather than
        // mutually exclusive classifications, and a record can be wrong in
        // both ways at once. Stated by a test so it is not read as a defect.
        let f = build::File {
            snaplen: 16,
            ..build::File::default()
        };
        let mut bytes = build::header(&f);
        let mut rec = build::Record::new(&[3; 40]);
        rec.orig_len = Some(10);
        bytes.extend_from_slice(&build::record(Wide::Le, &rec));
        let (packets, stats) = read_all(bytes);
        assert_eq!(packets.len(), 1);
        assert_eq!(moved(&stats), vec![2, 3]);
    }

    #[test]
    fn a_zero_snapshot_length_is_treated_as_unset() {
        // Some writers leave it zero. Comparing against it would then fire on
        // every record and drown the counter that matters.
        let f = build::File {
            snaplen: 0,
            ..build::File::default()
        };
        let mut bytes = build::header(&f);
        bytes.extend_from_slice(&build::record(Wide::Le, &build::Record::new(&[3; 40])));
        let (packets, stats) = read_all(bytes);
        assert_eq!(packets.len(), 1);
        assert_eq!(moved(&stats), Vec::<usize>::new());
    }

    // FR-007. Nothing is dropped for being unusual.
    #[test]
    fn a_zero_length_record_is_delivered() {
        let (packets, stats) = read_all(build::one(&[]));
        assert_eq!(packets.len(), 1, "an empty record is well-formed");
        assert_eq!(packets[0].captured_len(), 0);
        assert_eq!(moved(&stats), Vec::<usize>::new());
    }

    #[test]
    fn an_out_of_order_timestamp_is_delivered_in_file_order() {
        let f = build::File::default();
        let mut bytes = build::header(&f);
        for secs in [100u32, 50, 200] {
            let mut rec = build::Record::new(&[0]);
            rec.secs = secs;
            bytes.extend_from_slice(&build::record(Wide::Le, &rec));
        }
        let (packets, _) = read_all(bytes);
        let seen: Vec<i64> = packets
            .iter()
            .map(|p| p.ts.as_nanos() / 1_000_000_000)
            .collect();
        assert_eq!(
            seen,
            vec![100, 50, 200],
            "sorting would be an alteration, not a convenience"
        );
    }

    #[test]
    fn a_link_type_fragcap_cannot_parse_is_read_anyway() {
        let f = build::File {
            link_type: 276,
            ..build::File::default()
        };
        let mut bytes = build::header(&f);
        bytes.extend_from_slice(&build::record(Wide::Le, &build::Record::new(&[1, 2])));
        let mut r = PcapReader::from_bytes(bytes).expect("the file opens");
        assert_eq!(r.link_type(), LinkType::from_code(276));
        assert!(
            r.next_record().is_some(),
            "refusing would make the parser's own counter unreachable"
        );
    }

    #[test]
    fn the_file_header_fields_are_read_in_the_declared_order() {
        let f = build::File {
            magic: build::MAGIC_MICROS_BE,
            wide: Wide::Be,
            snaplen: 262_144,
            link_type: 101,
        };
        let r = PcapReader::from_bytes(build::header(&f)).expect("the file opens");
        assert_eq!(r.snaplen(), 262_144);
        assert_eq!(r.link_type(), LinkType::RAW);
    }

    #[test]
    fn reading_past_the_end_keeps_reporting_nothing() {
        let mut r = PcapReader::from_bytes(build::one(&[1])).expect("the file opens");
        assert!(r.next_record().is_some());
        assert!(r.next_record().is_none());
        assert!(r.next_record().is_none(), "exhaustion is stable");
    }

    #[test]
    fn the_skipped_total_counts_only_undelivered_records() {
        let s = ReplayStats {
            truncated_record: 2,
            impossible_length: 3,
            caplen_exceeds_wire: 100,
            caplen_exceeds_snaplen: 100,
        };
        assert_eq!(s.skipped(), 5, "delivered records are not skipped records");
        assert!(!s.read_whole_file());
        assert!(ReplayStats::default().read_whole_file());
    }
}
