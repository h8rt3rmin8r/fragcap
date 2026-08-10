// SPDX-License-Identifier: Apache-2.0

//! The behavioral seams, transcribed from specification section 8.5.
//!
//! These are the part of the surface intended to survive to 1.0.0 unchanged. A
//! change to one of them is a change to the architecture of record and requires
//! the deviation process rather than a local edit.
//!
//! Two constraints bind every trait here.
//!
//! **Constitution P-3.** [`PacketSource`] and [`FlowAttributor`] stay separate.
//! Neither names the other in any signature. That separation is what makes the
//! testing strategy in specification section 25 work at all: a replay source
//! and a scripted attributor make the whole pipeline testable offline, with no
//! capture driver, no elevation, and no game running.
//!
//! **Dyn compatibility.** Section 8.6 puts these on three threads and fans out
//! to a heterogeneous set of sinks chosen at runtime, so all four behavioral
//! traits must be usable as trait objects. `Sink::finish` taking `self:
//! Box<Self>` exists for exactly that reason. Adding a generic method to any of
//! them would break the pipeline; a test below fails if one does.

use std::sync::mpsc::Receiver;
use std::time::Duration;

use crate::attribution::Attribution;
use crate::error::{AttrError, SinkError, SourceError};
use crate::filter::FilterProgram;
use crate::flow::{Endpoint, FlowKey, OwnedEndpoint};
use crate::link::LinkType;
use crate::packet::{CapturedPacket, RawPacket, Timestamp};
use crate::process::{ProcessEvent, ProcessRecord};
use crate::stats::{CaptureStats, SourceStats};

/// Acquires packets. Implemented by a live backend in slice S09 and by a replay
/// source in slice S04.
///
/// Contains no attribution logic, per P-3.
///
/// `Send` because specification section 12.1 requires one capture thread per
/// interface, so the pipeline moves a source onto a thread it did not create
/// it on.
///
/// **This bound is a recorded deviation.** Specification section 8.5 declares
/// the trait without it, and slice S08 relied on its absence: that slice
/// acquired on the calling thread and spawned only the sink thread, precisely
/// so a trait intended to reach 1.0.0 unchanged did not have to change to make
/// one slice work. Multi-interface capture ends that arrangement, because there
/// is no arrangement of one thread that reads several handles without either
/// this bound or a second buffer that section 12.4 does not allow. Added by
/// S09 and promoted to specification section 29.
pub trait PacketSource: Send {
    /// The next packet, or `None` if the timeout elapsed with nothing to
    /// report. A timeout is not an error, which is why `None` is distinct from
    /// [`SourceError::Timeout`]: the latter is for backends that signal it as a
    /// failure.
    fn next_packet(&mut self, timeout: Duration) -> Result<Option<RawPacket>, SourceError>;

    /// Install a capture filter. Slice S13 owns when this is called.
    fn set_filter(&mut self, filter: &FilterProgram) -> Result<(), SourceError>;

    /// What the backend reports about itself, relayed unaltered.
    fn stats(&self) -> SourceStats;

    /// The link layer encapsulation this source produces.
    fn link_type(&self) -> LinkType;
}

/// Resolves a flow to the process that owns it. Implemented against the socket
/// table in slice S10.
///
/// Contains no packet acquisition, per P-3.
///
/// **`Sync` is a recorded deviation.** Specification section 8.5 declares this
/// trait with neither bound, and S08 needed neither: it held the attributor
/// behind a mutex and locked it once per packet, which was correct for a
/// pipeline with one capture thread and no publication mechanism.
///
/// Specification section 11.6 ends that arrangement. It requires the control
/// thread to publish an immutable snapshot atomically and the capture threads
/// to read it without locking, and there is no arrangement of a `Send`-only
/// trait that several threads share without a lock somewhere. S08 deferred the
/// mechanism to S10 by name, on the reasoning that building it earlier would
/// fix the snapshot's shape before anything knew what a socket table snapshot
/// costs to publish. Added by S10 and promoted to specification section 29.
///
/// The same reasoning S09 used for [`PacketSource`] applies to the size of the
/// change: a bound that every existing implementor already satisfies is a far
/// smaller commitment than a method on a surface intended to reach 1.0.0.
pub trait FlowAttributor: Send + Sync {
    /// The process owning this flow at the instant the packet was observed, if
    /// it can be determined. `None` means attempted and unresolved: the packet
    /// is retained and marked, per P-4, never dropped.
    ///
    /// `at` is the packet's own timestamp, not the present moment, and the
    /// distinction is load-bearing rather than pedantic. Specification section
    /// 11.4: "capture and socket table observation are not synchronized. A
    /// connection closing produces final packets that may be processed after
    /// the socket has left the table." The question is therefore always who
    /// owned this flow *then*, and an implementation that answered about now
    /// would misattribute the tail of every connection and every reused port.
    ///
    /// Slice S04 added this parameter. It was omitted in S02, on the reasoning
    /// that a socket table is already current so the instant is implicit; that
    /// reasoning does not survive section 11.4, and the omission was found in
    /// review of pull request 7. Recorded for promotion to specification
    /// section 29.
    fn resolve(&self, key: &FlowKey, at: Timestamp) -> Option<Attribution>;

    /// Re-read the underlying table.
    ///
    /// **`&self` is a recorded deviation.** Specification section 8.5 declared
    /// this `&mut self`, and slices S10 through S14 relied on that: the mutable
    /// attributor lived on a control thread of its own (the CLI `RefreshDriver`)
    /// and the capture pipeline resolved against a separate read-only view
    /// cloned from it, because a `&mut` method cannot be called through the
    /// `Arc<dyn FlowAttributor>` the capture threads share. Specification section
    /// 8.6 places the socket-table refresh on the pipeline control thread, and
    /// there is no arrangement of a `&mut self` refresh that the control thread
    /// can drive through the same shared pointer the capture threads resolve
    /// through. An implementor with refresh-mutable state carries it behind
    /// interior mutability that the lock-free `resolve` path (section 11.6) never
    /// touches. Changed by slice 015 and promoted to specification section 29.
    fn refresh(&self) -> Result<(), AttrError>;

    /// Whether a refresh is due or has been requested, so the control thread can
    /// gate [`Self::refresh`] on the section 11.2 cadence without this crate
    /// naming the schedule type that lives in `fragcap-attr` (P-2).
    ///
    /// Defaults to `false`: an attributor with nothing to re-read is never asked
    /// to refresh, which is correct for the scripted and stub attributors and for
    /// the read-only resolver. Added by slice 015 with the `refresh` change and
    /// promoted to specification section 29.
    fn wants_refresh(&self) -> bool {
        false
    }

    /// Endpoints currently believed active, including the retention window in
    /// specification section 11.4.
    fn active_endpoints(&self) -> Vec<Endpoint>;

    /// The same active endpoints, each paired with the process identifier that
    /// owns it when the source can supply one.
    ///
    /// Specification section 12.2 narrows the kernel filter to endpoints
    /// belonging to profiled processes, a decision the owning identifier is
    /// needed to make; [`Self::active_endpoints`] has dropped it. A decorator
    /// that holds the profiled-process set (the session's role-stamping
    /// attributor) filters this by owner and reports the result through
    /// `active_endpoints`.
    ///
    /// Defaults to mapping [`Self::active_endpoints`] to unowned endpoints, so an
    /// attributor that does not track ownership needs no change and a consumer
    /// that does not filter by owner sees the same endpoints as before. Added by
    /// slice 015 and promoted to specification section 29.
    fn active_endpoints_owned(&self) -> Vec<OwnedEndpoint> {
        self.active_endpoints()
            .into_iter()
            .map(OwnedEndpoint::unowned)
            .collect()
    }
}

/// Watches process lifecycle. Implemented over ETW kernel providers in slice
/// S11.
///
/// Nothing here requires a process handle. Ancestry comes from creation-time
/// events, which is what P-1 requires.
pub trait ProcessWatcher: Send {
    /// A stream of lifecycle events. Each call yields an independent receiver,
    /// so an implementor holds its senders behind interior mutability.
    fn subscribe(&self) -> Receiver<ProcessEvent>;

    /// Every process visible now, by query-only enumeration.
    fn snapshot(&self) -> Vec<ProcessRecord>;
}

/// Accepts captured packets and writes them somewhere. Implemented in slices
/// S06, S07, S15, and S16.
pub trait Sink: Send {
    /// Write one packet. Returning [`SinkError::Full`] is counted in
    /// `sink_dropped` by the pipeline rather than aborting the capture.
    fn write(&mut self, packet: &CapturedPacket) -> Result<(), SinkError>;

    /// Flush anything buffered.
    fn flush(&mut self) -> Result<(), SinkError>;

    /// Finish and consume the sink, writing trailing statistics.
    ///
    /// Takes `self: Box<Self>` so a boxed trait object can be consumed, which
    /// is what lets the pipeline own a heterogeneous set of sinks.
    fn finish(self: Box<Self>, stats: &CaptureStats) -> Result<(), SinkError>;
}

/// Protocol dissection.
///
/// Declared with no implementations, deliberately. Specification section 8.5
/// fixes this seam's shape in v0.2.0 so that the eventual dissector layer is
/// not retrofitted against types that were not designed for it.
///
/// The method set is provisional and will be settled by the slice that first
/// implements a dissector. Nothing depends on it yet.
pub trait Dissector {
    /// A stable name, used when annotating output.
    fn name(&self) -> &str;

    /// Whether this dissector recognizes the packet.
    fn claims(&self, packet: &CapturedPacket) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribution::Fidelity;
    use crate::flow::{Direction, Proto};
    use crate::interface::InterfaceId;
    use crate::packet::{AttributionState, Payload, Timestamp};
    use std::net::SocketAddr;
    use std::sync::mpsc;

    fn addr(s: &str) -> SocketAddr {
        s.parse().expect("test address must parse")
    }

    // Stub implementations. Their only job is to prove the seams are
    // expressible without a capture driver, elevation, or a game running, which
    // is the property specification section 25 depends on.

    struct StubSource {
        queued: Vec<RawPacket>,
        stats: SourceStats,
    }

    impl PacketSource for StubSource {
        fn next_packet(&mut self, _timeout: Duration) -> Result<Option<RawPacket>, SourceError> {
            Ok(self.queued.pop())
        }
        fn set_filter(&mut self, _filter: &FilterProgram) -> Result<(), SourceError> {
            Ok(())
        }
        fn stats(&self) -> SourceStats {
            self.stats
        }
        fn link_type(&self) -> LinkType {
            LinkType::ETHERNET
        }
    }

    struct StubAttributor {
        answer: Option<Attribution>,
    }

    impl FlowAttributor for StubAttributor {
        fn resolve(&self, _key: &FlowKey, _at: Timestamp) -> Option<Attribution> {
            self.answer.clone()
        }
        fn refresh(&self) -> Result<(), AttrError> {
            Ok(())
        }
        fn active_endpoints(&self) -> Vec<Endpoint> {
            Vec::new()
        }
    }

    struct StubWatcher;

    impl ProcessWatcher for StubWatcher {
        fn subscribe(&self) -> Receiver<ProcessEvent> {
            let (_tx, rx) = mpsc::channel();
            rx
        }
        fn snapshot(&self) -> Vec<ProcessRecord> {
            Vec::new()
        }
    }

    #[derive(Default)]
    struct StubSink {
        written: Vec<CapturedPacket>,
        finished: bool,
    }

    impl Sink for StubSink {
        fn write(&mut self, packet: &CapturedPacket) -> Result<(), SinkError> {
            self.written.push(packet.clone());
            Ok(())
        }
        fn flush(&mut self) -> Result<(), SinkError> {
            Ok(())
        }
        fn finish(mut self: Box<Self>, _stats: &CaptureStats) -> Result<(), SinkError> {
            self.finished = true;
            Ok(())
        }
    }

    // V-7. Each behavioral trait behind a pointer. This test stops compiling if
    // a trait gains a generic method, which would break the pipeline in
    // specification section 8.6.
    #[test]
    fn every_behavioral_trait_is_usable_as_a_trait_object() {
        let _source: Box<dyn PacketSource> = Box::new(StubSource {
            queued: Vec::new(),
            stats: SourceStats::default(),
        });
        let _attributor: Box<dyn FlowAttributor> = Box::new(StubAttributor { answer: None });
        let _watcher: Box<dyn ProcessWatcher> = Box::new(StubWatcher);
        let _sink: Box<dyn Sink> = Box::new(StubSink::default());
    }

    // S09. Section 12.1 puts every source on its own thread, so all four
    // cross-thread traits must be `Send` behind a pointer. Asserted at compile
    // time and here rather than in the pipeline, so that an implementor which
    // stops being `Send` fails at the trait that requires it instead of three
    // layers up in a spawn call whose error message names neither.
    #[test]
    fn every_cross_thread_trait_is_send_behind_a_pointer() {
        fn requires_send<T: Send + ?Sized>(_: &T) {}

        let source: Box<dyn PacketSource> = Box::new(StubSource {
            queued: Vec::new(),
            stats: SourceStats::default(),
        });
        let attributor: Box<dyn FlowAttributor> = Box::new(StubAttributor { answer: None });
        let watcher: Box<dyn ProcessWatcher> = Box::new(StubWatcher);
        let sink: Box<dyn Sink> = Box::new(StubSink::default());

        requires_send(&source);
        requires_send(&attributor);
        requires_send(&watcher);
        requires_send(&sink);
    }

    #[test]
    fn sinks_are_usable_as_a_heterogeneous_collection() {
        // The section 8.6 fan-out: a file sink, a stream sink, and a ring
        // buffer held together and selected at runtime.
        let mut sinks: Vec<Box<dyn Sink>> = vec![
            Box::new(StubSink::default()),
            Box::new(StubSink::default()),
            Box::new(StubSink::default()),
        ];
        let packet = CapturedPacket::from_raw(
            RawPacket::new(Timestamp::from_nanos(1), Payload::from_static(&[0; 4]), 4),
            InterfaceId::default(),
        );
        for sink in sinks.iter_mut() {
            sink.write(&packet).expect("stub sink accepts");
        }
        let stats = CaptureStats::default();
        for sink in sinks {
            sink.finish(&stats).expect("a boxed sink can be consumed");
        }
    }

    #[test]
    fn the_attributor_and_watcher_cross_thread_boundaries() {
        // Both are `Send` because the control thread owns them. Asserting it
        // here means a later change that removes the bound fails a test rather
        // than a distant slice.
        fn assert_send<T: Send + ?Sized>() {}
        assert_send::<dyn FlowAttributor>();
        assert_send::<dyn ProcessWatcher>();
        assert_send::<dyn Sink>();
    }

    // S10. Section 11.6 has every capture thread reading one published
    // attribution snapshot without locking, which requires `Sync` and not only
    // `Send`. Asserted at the trait rather than at the pipeline, so an
    // implementor that stops being `Sync` fails here instead of inside a
    // `spawn` call whose error message names neither.
    #[test]
    fn the_attributor_is_shareable_across_threads() {
        fn requires_sync<T: Sync + ?Sized>(_: &T) {}
        fn assert_sync<T: Sync + ?Sized>() {}

        let attributor: Box<dyn FlowAttributor> = Box::new(StubAttributor { answer: None });
        requires_sync(&attributor);
        assert_sync::<dyn FlowAttributor>();
    }

    // The conversion the pipeline performs. `Arc<dyn T>` is constructible from
    // `Box<dyn T>`, which is what lets `Pipeline::new` keep taking a `Box`
    // while `run` shares an `Arc` across capture threads with no lock. If this
    // stops compiling, every existing caller of `Pipeline::new` has to change.
    #[test]
    fn a_boxed_attributor_converts_into_a_shared_one() {
        use std::sync::Arc;

        let boxed: Box<dyn FlowAttributor> = Box::new(StubAttributor {
            answer: Some(Attribution::new(7, "x.exe", Fidelity::Live)),
        });
        let shared: Arc<dyn FlowAttributor> = Arc::from(boxed);
        let second = Arc::clone(&shared);

        let key = FlowKey::new(Proto::Tcp, addr("192.0.2.1:1"), addr("192.0.2.2:2"));
        assert_eq!(
            second.resolve(&key, Timestamp::from_nanos(0)).unwrap().pid,
            7
        );
    }

    // V-8 and V-2. The whole pipeline shape from specification section 8.6,
    // expressed against these traits with nothing added.
    #[test]
    fn the_section_8_6_pipeline_shape_is_expressible() {
        let local = addr("192.0.2.10:51000");
        let remote = addr("198.51.100.5:443");

        let mut source = StubSource {
            queued: vec![RawPacket::new(
                Timestamp::from_nanos(42),
                Payload::from_static(&[1, 2, 3, 4]),
                1514,
            )],
            stats: SourceStats {
                received: 1,
                kernel_dropped: 0,
                interface_dropped: 0,
            },
        };
        let attributor = StubAttributor {
            answer: Some(Attribution::new(4242, "eso64.exe", Fidelity::Live)),
        };
        let mut sink: Box<dyn Sink> = Box::new(StubSink::default());
        let mut stats = CaptureStats::default();

        // Capture thread: acquire.
        let raw = source
            .next_packet(Duration::from_millis(10))
            .expect("stub source succeeds")
            .expect("one packet was queued");
        let mut packet = CapturedPacket::from_raw(raw, InterfaceId::default());
        stats.packets_captured += 1;

        // Capture thread: header parsing would derive these in slice S03.
        packet.flow = Some(FlowKey::new(Proto::Tcp, local, remote));
        packet.direction = Some(Direction::Outbound);

        // Capture thread: attribution lookup against the control thread's
        // published snapshot.
        if let Some(key) = packet.flow.as_ref() {
            // The packet's own instant, not the present one. Section 11.4.
            packet.attribution = attributor.resolve(key, packet.ts);
        }
        match packet.attribution_state() {
            AttributionState::Resolved => stats.packets_attributed += 1,
            AttributionState::Unresolved => stats.packets_unattributed += 1,
            AttributionState::NotAttempted => {}
        }

        // Sink thread: drain and fan out.
        sink.write(&packet).expect("stub sink accepts");
        stats.set_source(InterfaceId::default(), source.stats());
        sink.finish(&stats).expect("a boxed sink can be consumed");

        assert_eq!(stats.packets_captured, 1);
        assert_eq!(stats.packets_attributed, 1);
        assert_eq!(stats.packets_unattributed, 0);
        assert!(!stats.lost_anything());
        assert!(
            packet.is_truncated(),
            "orig_len survived the whole pipeline"
        );
    }

    #[test]
    fn an_unresolved_attribution_is_counted_and_the_packet_is_kept() {
        // P-4 through the pipeline: attribution fails, the packet is retained
        // and marked rather than dropped, and it lands in its own counter.
        let attributor = StubAttributor { answer: None };
        let mut packet = CapturedPacket::from_raw(
            RawPacket::new(Timestamp::from_nanos(1), Payload::from_static(&[0; 8]), 8),
            InterfaceId::default(),
        );
        packet.flow = Some(FlowKey::new(
            Proto::Udp,
            addr("192.0.2.10:30000"),
            addr("198.51.100.5:5055"),
        ));
        packet.attribution = attributor.resolve(packet.flow.as_ref().unwrap(), packet.ts);

        let mut stats = CaptureStats::default();
        stats.packets_captured += 1;
        assert_eq!(packet.attribution_state(), AttributionState::Unresolved);
        stats.packets_unattributed += 1;

        let mut sink: Box<dyn Sink> = Box::new(StubSink::default());
        sink.write(&packet)
            .expect("an unattributed packet is written");
        assert_eq!(stats.packets_unattributed, 1);
        assert_eq!(stats.fragcap_dropped(), 0, "nothing was dropped");
    }
}
