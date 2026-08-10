// SPDX-License-Identifier: Apache-2.0

//! Live packet acquisition, over the platform capture driver.
//!
//! Compiled only under `--features live` on Windows. Absent everywhere else
//! rather than stubbed: a stub that compiles and returns nothing is a source
//! that claims to be capturing and is not, which is the one failure mode
//! constitution P-9 rules out entirely.
//!
//! # What this is and is not
//!
//! It is an adapter. Every method below maps one `pcap` call onto one
//! [`PacketSource`] obligation, and the mapping is deliberately boring. The two
//! places it is not boring are worth reading before changing anything here.
//!
//! **Timeouts are not failures.** `pcap` reports a read timeout as
//! `Error::TimeoutExpired`, and the capture loop must treat it as "nothing
//! arrived" rather than as an error, or a quiet interface would end the run.
//!
//! **Device loss is determined by looking, not by reading an error string.**
//! `pcap::Error` has thirteen variants and none of them names a device that has
//! gone away; a removed adapter surfaces as the general `PcapError(String)`.
//! Matching on that string would work until a driver update or a non-English
//! locale changed the text, and would then silently downgrade a lost device to
//! an unmodelled backend failure. So on a terminal error this module
//! re-enumerates and asks whether its interface is still there. That is an
//! observation. See slice S09 plan decision D-5.
//!
//! # What P-1 permits, and what this module must never do
//!
//! The capture driver is npcap's NDIS driver, which specification section 19.2
//! permits explicitly. The `pcap` crate additionally exposes packet
//! transmission on an active capture. **fragcap never transmits.** That is not
//! a matter of discipline here: `cargo xtask lint` fails if any fragcap source
//! names the transmit call, so the claim is checked rather than promised.

pub mod driver;
pub mod enumerate;
pub mod route;

use std::time::Duration;

use fragcap_core::error::SourceError;
use fragcap_core::filter::FilterProgram;
use fragcap_core::interface::InterfaceRecord;
use fragcap_core::link::LinkType;
use fragcap_core::packet::{Payload, RawPacket, Timestamp};
use fragcap_core::stats::SourceStats;
use fragcap_core::traits::PacketSource;

pub use driver::detect_driver;
pub use enumerate::enumerate;

/// Section 12.2 phase one: admit IPv4 and IPv6 and nothing else.
///
/// Deliberately permissive. No attribution exists when capture starts, so
/// narrowing here would discard traffic in the kernel with no way to know what
/// was lost, which is the discard-without-a-counter that constitution P-4
/// forbids. Phases two and three, which compile a narrowed filter from the
/// attribution map, belong to slice S13.
pub const BOOTSTRAP_FILTER: &str = "ip or ip6";

/// How a live handle is opened.
#[derive(Clone, Copy, Debug)]
pub struct LiveOptions {
    /// Bytes retained per frame. The original on-wire length is recorded
    /// separately, so truncation stays self-describing.
    pub snaplen: u32,
    /// Whether to place the adapter in promiscuous mode.
    pub promiscuous: bool,
    /// How long a read waits before reporting that nothing arrived.
    pub read_timeout: Duration,
}

impl LiveOptions {
    /// Options whose read timeout matches the pipeline that will drive them.
    ///
    /// The correct way to build these, and the reason it exists is that
    /// `next_packet`'s timeout argument cannot be honoured by this backend:
    /// libpcap fixes the read timeout when the handle is activated and offers
    /// no way to change it afterwards. If the two disagree, the handle's value
    /// wins and stop latency follows it, so a source opened with a thirty
    /// second timeout would delay a requested stop by thirty seconds however
    /// short the pipeline's own timeout is.
    ///
    /// Passing [`crate::live::LiveOptions::for_pipeline`] the value from
    /// `PipelineConfig::read_timeout` makes them agree by construction. Raised
    /// in review of pull request 12.
    pub fn for_pipeline(read_timeout: Duration) -> Self {
        LiveOptions {
            read_timeout,
            ..LiveOptions::default()
        }
    }

    /// The read timeout this handle was activated with, which is the one that
    /// actually governs how long a read blocks.
    pub fn read_timeout(&self) -> Duration {
        self.read_timeout
    }
}

impl Default for LiveOptions {
    fn default() -> Self {
        LiveOptions {
            // The whole frame. An operator who wants less says so, and their
            // own invocation records the choice, which is the form of scoping
            // P-9 permits.
            snaplen: 65_535,
            // Off by default. Promiscuous mode captures frames addressed to
            // other hosts, which is a broader observation than attributing this
            // machine's own processes requires.
            promiscuous: false,
            // Deliberately the same value as the pipeline's own default, and a
            // test below pins them together so that changing one without the
            // other fails rather than silently widening stop latency.
            read_timeout: fragcap_core::pipeline::DEFAULT_READ_TIMEOUT,
        }
    }
}

/// One open handle on one interface.
pub struct LiveSource {
    /// Behind a cell because `PacketSource::stats` takes `&self` while
    /// `pcap::Capture::stats` needs `&mut`. The alternative was caching the
    /// last reading and returning it from `stats`, which would report a stale
    /// number as a current one. `RefCell` is `Send` when its contents are, and
    /// `PacketSource` needs `Send` and not `Sync`, so nothing is given up.
    handle: std::cell::RefCell<pcap::Capture<pcap::Active>>,
    /// The interface's platform name, kept so that a lost device can be named
    /// and so that re-enumeration can look for it.
    name: String,
    link: LinkType,
    read_timeout: Duration,
}

impl LiveSource {
    /// Open a handle, install the bootstrap filter, and start delivering.
    ///
    /// The filter goes on before the first packet is read, per FR-036: the
    /// capture must never be broader than section 12.2 phase one describes,
    /// including for the first few frames.
    pub fn open(record: &InterfaceRecord, options: LiveOptions) -> Result<Self, SourceError> {
        let name = record.name.to_string();

        let inactive =
            pcap::Capture::from_device(name.as_str()).map_err(|e| SourceError::Backend {
                detail: format!("{name}: {e}"),
            })?;

        let timeout_ms = i32::try_from(options.read_timeout.as_millis()).unwrap_or(i32::MAX);
        let handle = inactive
            .snaplen(options.snaplen as i32)
            .promisc(options.promiscuous)
            .timeout(timeout_ms)
            .open()
            .map_err(|e| SourceError::Backend {
                detail: format!("{name}: {e}"),
            })?;

        let link = LinkType::from_code(handle.get_datalink().0 as u16);
        let mut source = LiveSource {
            handle: std::cell::RefCell::new(handle),
            name,
            link,
            read_timeout: options.read_timeout,
        };
        source.install_filter(BOOTSTRAP_FILTER)?;
        Ok(source)
    }

    /// The interface's platform name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The read timeout this handle was activated with.
    ///
    /// This is the value that actually bounds how long a read blocks, and
    /// therefore how long a requested stop takes to be observed. See
    /// [`PacketSource::next_packet`] on this type.
    pub fn configured_timeout(&self) -> Duration {
        self.read_timeout
    }

    fn install_filter(&mut self, program: &str) -> Result<(), SourceError> {
        // `true` optimizes the compiled program. It changes how the filter is
        // evaluated, never which packets match, so it cannot alter what is
        // observed.
        self.handle
            .borrow_mut()
            .filter(program, true)
            .map_err(|e| SourceError::FilterRejected {
                detail: format!("{}: {e}", self.name),
            })
    }

    /// Classify a terminal backend error by looking rather than by guessing.
    ///
    /// See this module's documentation and plan decision D-5. If enumeration
    /// itself fails we cannot tell, and the answer is the unmodelled backend
    /// failure rather than the more specific claim.
    fn classify(&self, detail: String) -> SourceError {
        let still_present = match pcap::Device::list() {
            Ok(devices) => devices.iter().any(|d| d.name == self.name),
            Err(_) => return SourceError::Backend { detail },
        };
        if still_present {
            SourceError::Backend { detail }
        } else {
            SourceError::DeviceLost { detail }
        }
    }
}

impl PacketSource for LiveSource {
    /// # The timeout argument
    ///
    /// **This backend cannot honour it, and says so rather than pretending.**
    /// libpcap fixes the read timeout when the handle is activated and offers
    /// no way to change it on a live handle; reopening would lose the driver's
    /// buffer and the installed filter, which is a worse trade than a longer
    /// read.
    ///
    /// The consequence is that stop latency follows the value passed to
    /// [`LiveSource::open`], not the value passed here. Build the options with
    /// [`LiveOptions::for_pipeline`] so the two agree by construction, and read
    /// [`LiveSource::configured_timeout`] if you need to check.
    ///
    /// Raised in review of pull request 12.
    fn next_packet(&mut self, _timeout: Duration) -> Result<Option<RawPacket>, SourceError> {
        let mut handle = self.handle.borrow_mut();
        match handle.next_packet() {
            Ok(packet) => {
                let header = packet.header;
                // The driver's own timestamp, carried unaltered. Section 12.7:
                // it is applied close to interface arrival and is the most
                // accurate source available.
                let ts = Timestamp::from_parts(
                    header.ts.tv_sec as i64,
                    header.ts.tv_usec as u32 * 1_000,
                );
                Ok(Some(RawPacket::new(
                    ts,
                    Payload::copy_from_slice(packet.data),
                    // What was on the wire, kept separate from what was
                    // retained, so a snapshot length says so.
                    header.len,
                )))
            }
            // Nothing arrived. Not an error, and nothing was lost.
            Err(pcap::Error::TimeoutExpired) => Ok(None),
            Err(pcap::Error::NoMorePackets) => Err(SourceError::Closed),
            Err(e) => Err(self.classify(format!("{}: {e}", self.name))),
        }
    }

    fn set_filter(&mut self, filter: &FilterProgram) -> Result<(), SourceError> {
        self.install_filter(filter.expression())
    }

    fn stats(&self) -> SourceStats {
        // `pcap::Stat` counts from the start of the run to the moment of the
        // call, so this is a copy rather than an accumulation. That distinction
        // is what makes "relayed unaltered" literally true: there is no
        // arithmetic here in which an alteration could hide.
        //
        // A backend that cannot report is reported as zeroes rather than as an
        // error, because `PacketSource::stats` has no failure channel. That is
        // a known weakness of the seam rather than a decision made here.
        match self.handle.borrow_mut().stats() {
            Ok(stat) => SourceStats {
                received: stat.received as u64,
                kernel_dropped: stat.dropped as u64,
                interface_dropped: stat.if_dropped as u64,
            },
            Err(_) => SourceStats::default(),
        }
    }

    fn link_type(&self) -> LinkType {
        self.link
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bootstrap_filter_admits_both_address_families_and_nothing_else() {
        // Section 12.2 phase one, asserted as a string because that is what the
        // backend compiles. Narrowing this is S13's work and changing it here
        // would discard traffic in the kernel with no counter.
        assert_eq!(BOOTSTRAP_FILTER, "ip or ip6");
    }

    #[test]
    fn the_default_snapshot_length_retains_a_whole_frame() {
        assert_eq!(LiveOptions::default().snaplen, 65_535);
    }

    #[test]
    fn promiscuous_mode_is_off_unless_asked_for() {
        assert!(!LiveOptions::default().promiscuous);
    }

    // Review of pull request 12. `next_packet`'s timeout argument cannot be
    // honoured by this backend, so the default must match the pipeline's or a
    // default-configured live capture would have stop latency governed by a
    // number nobody chose. Pinned here so that changing either value alone
    // fails rather than quietly widening it.
    #[test]
    fn the_default_read_timeout_matches_the_pipelines_own() {
        assert_eq!(
            LiveOptions::default().read_timeout,
            fragcap_core::pipeline::DEFAULT_READ_TIMEOUT
        );
    }

    #[test]
    fn options_can_be_built_to_agree_with_a_pipeline() {
        let t = Duration::from_millis(250);
        assert_eq!(LiveOptions::for_pipeline(t).read_timeout(), t);
    }
}
