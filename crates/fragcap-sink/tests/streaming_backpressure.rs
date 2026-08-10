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

use common::packets;

use fragcap_core::stats::CaptureStats;
use fragcap_core::traits::Sink;
use fragcap_core::LinkType;
use fragcap_sink::{
    Acceptor, ConnShutdown, Connection, DisconnectReason, Format, InterfaceSpec, SinkFactory,
    Stopper, StreamSink,
};

const SNAP: u32 = 262_144;

/// A writer that accepts a consumer's header (the first bytes) and then stalls
/// on every packet, honoring a stop flag so the sink can reap it.
struct StallWriter {
    stop: Arc<AtomicBool>,
    accepted: u64,
    header_budget: u64,
}

impl Write for StallWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Let the header through so the encoder builds, then stall.
        if self.accepted < self.header_budget {
            self.accepted += buf.len() as u64;
            return Ok(buf.len());
        }
        loop {
            if self.stop.load(Ordering::Acquire) {
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
}

impl OneStallAcceptor {
    fn new() -> Self {
        OneStallAcceptor {
            yielded: AtomicBool::new(false),
            stop: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Acceptor for OneStallAcceptor {
    fn accept(&mut self) -> Option<Connection> {
        if self.stop.load(Ordering::Acquire) {
            return None;
        }
        if !self.yielded.swap(true, Ordering::AcqRel) {
            let stop = Arc::new(AtomicBool::new(false));
            return Some(Connection {
                id: "stall#0".to_string(),
                writer: Box::new(StallWriter {
                    stop: Arc::clone(&stop),
                    accepted: 0,
                    header_budget: 256,
                }),
                shutdown: Box::new(StallShutdown(stop)),
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
    let mut sink =
        StreamSink::with_settings(factory(), Box::new(OneStallAcceptor::new()), 2, timeout);
    let handle = sink.reports_handle();

    // Wait for the stalled consumer to register.
    let deadline = Instant::now() + Duration::from_secs(5);
    while sink.active_consumers() < 1 {
        assert!(Instant::now() < deadline, "consumer did not register");
        std::thread::sleep(Duration::from_millis(5));
    }

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
    assert!(
        reports[0].written <= reports[0].offered - reports[0].dropped,
        "per-consumer accounting is honest"
    );
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
