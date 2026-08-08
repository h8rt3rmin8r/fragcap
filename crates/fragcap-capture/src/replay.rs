// SPDX-License-Identifier: Apache-2.0

//! A [`PacketSource`] backed by a recorded capture file.
//!
//! Half of the claim specification section 25.1 makes: with this and a scripted
//! attributor, the pipeline runs with no capture driver, no elevated privilege,
//! and no game. Contains no attribution logic, per constitution P-3, and could
//! not: `fragcap-capture` has no dependency edge to `fragcap-attr`, which
//! `cargo xtask deps` enforces.

use std::fs;
use std::path::Path;
use std::time::Duration;

use fragcap_core::error::SourceError;
use fragcap_core::filter::FilterProgram;
use fragcap_core::link::LinkType;
use fragcap_core::packet::RawPacket;
use fragcap_core::stats::SourceStats;
use fragcap_core::traits::PacketSource;

use crate::pcap::{PcapReader, ReplayStats};

/// Replays a capture file as though it were an interface.
pub struct ReplaySource {
    reader: PcapReader,
    delivered: u64,
    /// The last filter accepted, and never applied. Kept so a test can see
    /// that acceptance happened without mistaking it for application.
    filter: Option<FilterProgram>,
}

impl std::fmt::Debug for ReplaySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReplaySource")
            .field("reader", &self.reader)
            .field("delivered", &self.delivered)
            .field("filtered", &self.filter.is_some())
            .finish()
    }
}

impl ReplaySource {
    /// Open a capture file from disk.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, SourceError> {
        let path = path.as_ref();
        let data = fs::read(path).map_err(|e| SourceError::Backend {
            detail: format!("cannot read {}: {e}", path.display()),
        })?;
        Self::from_bytes(data)
    }

    /// Replay a capture file already in memory.
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, SourceError> {
        Ok(ReplaySource {
            reader: PcapReader::from_bytes(data)?,
            delivered: 0,
            filter: None,
        })
    }

    /// What the reader could not deliver as the file described it.
    ///
    /// Deliberately not folded into [`PacketSource::stats`]. Those counters are
    /// a backend's own report, relayed unaltered, and presenting fragcap's
    /// accounting as the backend's observation is what S02 kept the two types
    /// apart to prevent.
    pub fn replay_stats(&self) -> &ReplayStats {
        self.reader.stats()
    }

    /// The last filter program accepted. Never applied to anything.
    pub fn accepted_filter(&self) -> Option<&FilterProgram> {
        self.filter.as_ref()
    }
}

impl PacketSource for ReplaySource {
    /// The next record, ignoring the timeout.
    ///
    /// A file is never slow, so there is no honest value to wait for and
    /// nothing a timeout could mean.
    ///
    /// Exhaustion is [`SourceError::Closed`], never `Ok(None)`. `Ok(None)`
    /// means the timeout elapsed and the caller should keep going, which over a
    /// finished file is an infinite loop rather than an ending.
    fn next_packet(&mut self, _timeout: Duration) -> Result<Option<RawPacket>, SourceError> {
        match self.reader.next_record() {
            Some(packet) => {
                self.delivered = self.delivered.saturating_add(1);
                Ok(Some(packet))
            }
            None => Err(SourceError::Closed),
        }
    }

    /// Accepts a filter and applies nothing.
    ///
    /// A replay source has no kernel to install a program into. Failing would
    /// break any pipeline that filters before reading; applying it in software
    /// is S13's decision to make, not this slice's. Accepting and recording is
    /// what is left, and saying so here is what stops a caller mistaking
    /// acceptance for application.
    fn set_filter(&mut self, filter: &FilterProgram) -> Result<(), SourceError> {
        self.filter = Some(filter.clone());
        Ok(())
    }

    /// What this source delivered, and nothing else.
    ///
    /// Both drop counts stay zero: there is no kernel and no interface here to
    /// have dropped anything, and reporting the reader's own skips in those
    /// fields would be a false statement about a component that does not exist.
    fn stats(&self) -> SourceStats {
        SourceStats {
            received: self.delivered,
            kernel_dropped: 0,
            interface_dropped: 0,
        }
    }

    fn link_type(&self) -> LinkType {
        self.reader.link_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pcap::build::{self, Wide};

    fn source(bytes: Vec<u8>) -> ReplaySource {
        ReplaySource::from_bytes(bytes).expect("the file opens")
    }

    fn drain(s: &mut ReplaySource) -> Vec<RawPacket> {
        let mut out = Vec::new();
        loop {
            match s.next_packet(Duration::from_millis(0)) {
                Ok(Some(p)) => out.push(p),
                Ok(None) => panic!("a replay source never times out"),
                Err(SourceError::Closed) => return out,
                Err(e) => panic!("unexpected error: {e}"),
            }
        }
    }

    // FR-015. The distinction that stops a pipeline spinning.
    #[test]
    fn exhaustion_is_closed_and_never_a_timeout() {
        let mut s = source(build::one(&[1, 2, 3]));
        assert!(matches!(s.next_packet(Duration::from_secs(0)), Ok(Some(_))));
        assert_eq!(
            s.next_packet(Duration::from_secs(0)),
            Err(SourceError::Closed)
        );
        assert_eq!(
            s.next_packet(Duration::from_secs(0)),
            Err(SourceError::Closed),
            "a finished file stays finished"
        );
    }

    #[test]
    fn closed_is_terminal_rather_than_recoverable() {
        // A capture loop that treated this as recoverable would spin.
        assert!(!SourceError::Closed.is_recoverable());
    }

    #[test]
    fn packets_arrive_in_file_order() {
        let f = build::File::default();
        let mut bytes = build::header(&f);
        for n in 0u8..5 {
            bytes.extend_from_slice(&build::record(Wide::Le, &build::Record::new(&[n])));
        }
        let mut s = source(bytes);
        let seen: Vec<u8> = drain(&mut s).iter().map(|p| p.data[0]).collect();
        assert_eq!(seen, vec![0, 1, 2, 3, 4]);
    }

    // FR-016 and FR-016a.
    #[test]
    fn statistics_report_what_was_delivered_and_claim_no_backend_drops() {
        let f = build::File::default();
        let mut bytes = build::header(&f);
        for _ in 0..3 {
            bytes.extend_from_slice(&build::record(Wide::Le, &build::Record::new(&[0])));
        }
        let mut s = source(bytes);
        drain(&mut s);
        let stats = s.stats();
        assert_eq!(stats.received, 3);
        assert_eq!(stats.kernel_dropped, 0, "there is no kernel here to drop");
        assert_eq!(stats.interface_dropped, 0);
        assert_eq!(stats.total_dropped(), 0);
    }

    #[test]
    fn reader_skips_stay_out_of_the_backend_report() {
        let f = build::File::default();
        let mut bytes = build::header(&f);
        bytes.extend_from_slice(&build::record(Wide::Le, &build::Record::new(&[1])));
        bytes.extend_from_slice(&[0; 5]);
        let mut s = source(bytes);
        drain(&mut s);
        assert_eq!(s.replay_stats().truncated_record, 1);
        assert_eq!(
            s.stats().total_dropped(),
            0,
            "fragcap's accounting must not be reported as the backend's"
        );
        assert_eq!(s.stats().received, 1);
    }

    // FR-017.
    #[test]
    fn a_filter_is_accepted_and_changes_nothing() {
        let f = build::File::default();
        let mut bytes = build::header(&f);
        for n in 0u8..4 {
            bytes.extend_from_slice(&build::record(Wide::Le, &build::Record::new(&[n])));
        }
        let mut unfiltered = source(bytes.clone());
        let before = drain(&mut unfiltered).len();

        let mut filtered = source(bytes);
        let program = FilterProgram::default();
        assert!(
            filtered.set_filter(&program).is_ok(),
            "acceptance, not failure"
        );
        assert!(filtered.accepted_filter().is_some(), "it was recorded");
        assert_eq!(
            drain(&mut filtered).len(),
            before,
            "a replay source does not filter, and must not appear to"
        );
    }

    #[test]
    fn the_link_type_comes_from_the_file() {
        let f = build::File {
            link_type: 101,
            ..build::File::default()
        };
        let s = source(build::header(&f));
        assert_eq!(s.link_type(), LinkType::RAW);
    }

    #[test]
    fn opening_a_file_that_is_not_a_capture_fails() {
        assert!(ReplaySource::from_bytes(vec![0; 8]).is_err());
    }

    #[test]
    fn opening_a_path_that_does_not_exist_fails_with_the_path_named() {
        let e = ReplaySource::open("fixtures/definitely-not-here.pcap")
            .expect_err("a missing file cannot be replayed");
        assert!(e.to_string().contains("definitely-not-here.pcap"));
    }

    // P-3, and the shape S08 will use.
    #[test]
    fn a_replay_source_is_usable_as_a_packet_source_trait_object() {
        let mut boxed: Box<dyn PacketSource> = Box::new(source(build::one(&[1])));
        assert!(boxed.next_packet(Duration::from_secs(0)).is_ok());
        assert_eq!(boxed.stats().received, 1);
    }
}
