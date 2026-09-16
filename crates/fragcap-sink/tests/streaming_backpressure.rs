// SPDX-License-Identifier: Apache-2.0

//! Deterministic per-consumer backpressure and disconnect-on-timeout
//! (specification 14.4, user story US4), using an in-process stalled connection
//! rather than a real socket. A real transport's kernel buffering hides
//! backpressure until megabytes have queued, which makes the timing
//! nondeterministic; a controlled stalled writer makes the streaming sink's own
//! logic exact.

mod common;

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use common::{assert_valid_pcapng_stream, epb_payloads, expected_payloads, packets, walk};

use fragcap_core::stats::CaptureStats;
use fragcap_core::traits::Sink;
use fragcap_core::LinkType;
use fragcap_sink::{
    Acceptor, ConnShutdown, Connection, DisconnectReason, Format, InterfaceSpec, RotatingFileSink,
    RotationPolicy, SinkFactory, Stopper, StreamSink,
};

const SNAP: u32 = 262_144;

#[derive(Clone, Default)]
struct StallControl {
    armed: Arc<AtomicBool>,
    blocked: Arc<AtomicBool>,
    released: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
}

/// Accept the complete header before arming, then acknowledge a blocked write.
struct StallWriter {
    control: StallControl,
}

impl Write for StallWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if !self.control.armed.load(Ordering::Acquire) {
            return Ok(buf.len());
        }
        self.control.blocked.store(true, Ordering::Release);
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if self.control.released.load(Ordering::Acquire) {
                return Ok(buf.len());
            }
            if self.control.stop.load(Ordering::Acquire) || Instant::now() >= deadline {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "stopped",
                ));
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct StallShutdown(Arc<AtomicBool>);

impl ConnShutdown for StallShutdown {
    fn shutdown(&self) {
        self.0.store(true, Ordering::Release);
    }
}

/// Yields exactly one stalled connection, then blocks until stopped.
struct OneStallAcceptor {
    yielded: AtomicBool,
    stop: Arc<AtomicBool>,
    control: StallControl,
}

impl OneStallAcceptor {
    fn new() -> Self {
        OneStallAcceptor {
            yielded: AtomicBool::new(false),
            stop: Arc::new(AtomicBool::new(false)),
            control: StallControl::default(),
        }
    }
}

impl Acceptor for OneStallAcceptor {
    fn accept(&mut self) -> Option<Connection> {
        if self.stop.load(Ordering::Acquire) {
            return None;
        }
        if !self.yielded.swap(true, Ordering::AcqRel) {
            return Some(Connection {
                id: "stall#0".to_string(),
                writer: Box::new(StallWriter {
                    control: self.control.clone(),
                }),
                shutdown: Box::new(StallShutdown(Arc::clone(&self.control.stop))),
            });
        }
        while !self.stop.load(Ordering::Acquire) {
            std::thread::sleep(Duration::from_millis(10));
        }
        None
    }

    fn stopper(&self) -> Stopper {
        let stop = Arc::clone(&self.stop);
        Arc::new(move || stop.store(true, Ordering::Release))
    }

    fn describe(&self) -> String {
        "stall".to_string()
    }
}

fn factory() -> SinkFactory {
    SinkFactory::new(
        Format::Pcapng,
        vec![InterfaceSpec::new("eth0", LinkType::ETHERNET, SNAP)],
    )
}

#[test]
fn a_stalled_consumer_is_disconnected_after_the_timeout_with_drops_counted() {
    let timeout = Duration::from_millis(200);
    let acceptor = OneStallAcceptor::new();
    let control = acceptor.control.clone();
    let mut sink = StreamSink::with_settings(factory(), Box::new(acceptor), 2, timeout);
    let handle = sink.reports_handle();

    // Wait for the stalled consumer to register.
    let deadline = Instant::now() + Duration::from_secs(5);
    while sink.active_consumers() < 1 {
        assert!(Instant::now() < deadline, "consumer did not register");
        std::thread::sleep(Duration::from_millis(5));
    }

    control.armed.store(true, Ordering::Release);
    // Offer packets until the sink disconnects the stalled consumer.
    let pkts = packets(500, 64);
    let deadline = Instant::now() + timeout * 20;
    for p in &pkts {
        sink.write(p).expect("write never blocks or fails");
        if sink.active_consumers() == 0 {
            break;
        }
        assert!(Instant::now() < deadline, "consumer was not disconnected");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        sink.active_consumers(),
        0,
        "the stalled consumer was disconnected on the backpressure timeout"
    );

    Box::new(sink)
        .finish(&CaptureStats::default())
        .expect("finish");

    let reports = handle.lock().unwrap().clone();
    assert_eq!(reports.len(), 1, "the stalled consumer reported");
    assert_eq!(
        reports[0].reason,
        DisconnectReason::Timeout,
        "the disconnect reason is the backpressure timeout"
    );
    assert!(reports[0].dropped > 0, "packets were dropped for it");
    assert_eq!(reports[0].offered, reports[0].written + reports[0].dropped);
}

#[test]
fn a_stalled_consumer_is_isolated_and_its_drops_are_counted() {
    // Fresh scenarios, not retries: each must pass on Linux and Windows.
    for scenario in 0..20 {
        let acceptor = OneStallAcceptor::new();
        let control = acceptor.control.clone();
        let mut sink =
            StreamSink::with_settings(factory(), Box::new(acceptor), 4, Duration::from_secs(30));
        let handle = sink.reports_handle();
        let deadline = Instant::now() + Duration::from_secs(5);
        while sink.active_consumers() != 1 {
            assert!(
                Instant::now() < deadline,
                "scenario {scenario}: registration"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("also.fcapng");
        let mut file_sink =
            RotatingFileSink::create(&file_path, RotationPolicy::None, factory()).unwrap();
        let pkts = packets(100, 4096);
        control.armed.store(true, Ordering::Release);
        let first_submission = Instant::now();
        sink.write(&pkts[0]).unwrap();
        file_sink.write(&pkts[0]).unwrap();
        assert!(first_submission.elapsed() < Duration::from_secs(5));
        let deadline = Instant::now() + Duration::from_secs(5);
        while !control.blocked.load(Ordering::Acquire) {
            assert!(
                Instant::now() < deadline,
                "scenario {scenario}: blocked write"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        // The writer cannot drain: four slots accept and the remaining offers
        // necessarily encounter a full queue, independent of kernel buffering.
        let started = Instant::now();
        for packet in &pkts[1..] {
            sink.write(packet).expect("nonblocking stream submission");
            file_sink.write(packet).unwrap();
        }
        assert!(started.elapsed() < Duration::from_secs(5));
        Box::new(file_sink)
            .finish(&CaptureStats::default())
            .unwrap();
        // Drain the in-flight packet and four accepted queue slots. This
        // separates genuine queue refusals from the unwritten terminal tail:
        // an unbounded queue would write all 100 and fail the exact assertion.
        // Timeout and its dropped unwritten tail retain their separate test.
        control.released.store(true, Ordering::Release);
        Box::new(sink).finish(&CaptureStats::default()).unwrap();
        let file_bytes = std::fs::read(file_path).unwrap();
        assert_valid_pcapng_stream(&file_bytes, 1);
        assert_eq!(epb_payloads(&walk(&file_bytes)), expected_payloads(&pkts));
        let reports = handle.lock().unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].offered, 100);
        assert_eq!(
            reports[0].written, 5,
            "one in-flight packet and four queue slots"
        );
        assert_eq!(reports[0].dropped, 95, "only the refused queue offers");
        assert_eq!(reports[0].offered, reports[0].written + reports[0].dropped);
        assert_eq!(reports[0].reason, DisconnectReason::CaptureEnded);
    }
}

#[test]
fn an_idle_stream_with_no_consumer_accepts_every_packet() {
    // A streaming sink with no consumer connected is idle, not failing: write
    // returns success (the pipeline conservation invariant holds) and the sink
    // is not retired.
    let acceptor = OneStallAcceptor::new();
    // Immediately stop so no consumer is ever yielded.
    (acceptor.stopper())();
    let mut sink =
        StreamSink::with_settings(factory(), Box::new(acceptor), 4, Duration::from_secs(1));

    for p in &packets(50, 64) {
        sink.write(p)
            .expect("write succeeds with no consumer connected");
    }
    Box::new(sink)
        .finish(&CaptureStats::default())
        .expect("finish");
}
