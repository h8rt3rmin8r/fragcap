// SPDX-License-Identifier: Apache-2.0

//! TCP streaming (specification 14.3, 14.4; user stories US3, US4). All tier 1:
//! in-process loopback clients, no external analyzer.

mod common;

use std::io::Read;
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use common::{assert_valid_pcapng_stream, epb_payloads, expected_payloads, packets, walk};

use fragcap_core::stats::CaptureStats;
use fragcap_core::traits::Sink;
use fragcap_core::LinkType;
use fragcap_sink::{
    ConsumerReport, Format, InterfaceSpec, RotatingFileSink, RotationPolicy, SinkFactory,
    StreamSink, TcpAcceptor,
};

const SNAP: u32 = 262_144;

fn factory() -> SinkFactory {
    SinkFactory::new(
        Format::Pcapng,
        vec![InterfaceSpec::new("eth0", LinkType::ETHERNET, SNAP)],
    )
}

fn bind(queue: usize, timeout: Duration) -> (StreamSink, SocketAddr) {
    // A stalled consumer is unblocked by the sink's own stop flag (on the
    // backpressure timeout and at finish), so the disconnect reason is
    // deterministic and finish is bounded regardless of the timeout.
    let acceptor = TcpAcceptor::bind("127.0.0.1:0").expect("bind");
    let addr = acceptor.local_addr();
    let sink = StreamSink::with_settings(factory(), Box::new(acceptor), queue, timeout);
    (sink, addr)
}

fn wait_registered(sink: &StreamSink, n: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while sink.active_consumers() < n {
        assert!(Instant::now() < deadline, "consumers did not register");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn read_all(mut stream: TcpStream) -> Vec<u8> {
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).expect("read to end");
    bytes
}

fn reports(handle: &std::sync::Arc<std::sync::Mutex<Vec<ConsumerReport>>>) -> Vec<ConsumerReport> {
    handle.lock().unwrap().clone()
}

#[test]
fn a_single_client_receives_a_valid_stream() {
    let (mut sink, addr) = bind(1024, Duration::from_secs(5));
    let client = TcpStream::connect(addr).expect("connect");
    wait_registered(&sink, 1);

    let pkts = packets(6, 64);
    for p in &pkts {
        sink.write(p).expect("write");
    }
    Box::new(sink)
        .finish(&CaptureStats::default())
        .expect("finish");

    let bytes = read_all(client);
    assert_valid_pcapng_stream(&bytes, 1);
    assert_eq!(epb_payloads(&walk(&bytes)), expected_payloads(&pkts));
}

#[test]
fn a_mid_capture_joiner_receives_only_later_packets() {
    let (mut sink, addr) = bind(1024, Duration::from_secs(5));

    let first = TcpStream::connect(addr).expect("connect first");
    wait_registered(&sink, 1);

    let pkts = packets(8, 64);
    for p in &pkts[..4] {
        sink.write(p).expect("write");
    }

    // A second consumer connects mid-capture.
    let second = TcpStream::connect(addr).expect("connect second");
    wait_registered(&sink, 2);

    for p in &pkts[4..] {
        sink.write(p).expect("write");
    }
    Box::new(sink)
        .finish(&CaptureStats::default())
        .expect("finish");

    let first_bytes = read_all(first);
    let second_bytes = read_all(second);

    // The first consumer saw the whole capture.
    assert_valid_pcapng_stream(&first_bytes, 1);
    assert_eq!(epb_payloads(&walk(&first_bytes)), expected_payloads(&pkts));

    // The second consumer got its own header and only the packets after it
    // connected, never an earlier one.
    assert_valid_pcapng_stream(&second_bytes, 1);
    assert_eq!(
        epb_payloads(&walk(&second_bytes)),
        expected_payloads(&pkts[4..])
    );
}

#[test]
fn two_clients_each_receive_the_full_stream() {
    let (mut sink, addr) = bind(1024, Duration::from_secs(5));
    let a = TcpStream::connect(addr).expect("connect a");
    let b = TcpStream::connect(addr).expect("connect b");
    wait_registered(&sink, 2);

    let pkts = packets(10, 64);
    for p in &pkts {
        sink.write(p).expect("write");
    }
    Box::new(sink)
        .finish(&CaptureStats::default())
        .expect("finish");

    for client in [a, b] {
        let bytes = read_all(client);
        assert_valid_pcapng_stream(&bytes, 1);
        assert_eq!(epb_payloads(&walk(&bytes)), expected_payloads(&pkts));
    }
}

#[test]
fn a_stalled_consumer_is_isolated_and_its_drops_are_counted() {
    // A large disconnect timeout so the stalled consumer is not disconnected
    // mid-run; a small queue so its backpressure bites quickly.
    let (mut sink, addr) = bind(4, Duration::from_secs(30));
    let handle = sink.reports_handle();

    // The stalled consumer connects and never reads.
    let _slow = TcpStream::connect(addr).expect("connect slow");
    wait_registered(&sink, 1);

    // A file sink attached to the same run must be wholly unaffected by the
    // stalled network consumer (specification 14.4, SC-004).
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("also.fcapng");
    let mut file_sink =
        RotatingFileSink::create(&file_path, RotationPolicy::None, factory()).expect("file sink");

    let pkts = packets(100, 4096);
    let started = Instant::now();
    for p in &pkts {
        // Both writes are non-blocking and never fail from the pipeline's view,
        // even though one consumer has stopped reading entirely.
        sink.write(p).expect("stream write");
        file_sink.write(p).expect("file write");
    }
    // Capture was never stalled by the dead consumer: 100 writes complete fast.
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the stalled consumer did not stall capture"
    );

    Box::new(file_sink)
        .finish(&CaptureStats::default())
        .expect("file finish");
    Box::new(sink)
        .finish(&CaptureStats::default())
        .expect("finish");

    // The file sink got every packet, in order, with none lost.
    let file_bytes = std::fs::read(&file_path).unwrap();
    assert_valid_pcapng_stream(&file_bytes, 1);
    assert_eq!(epb_payloads(&walk(&file_bytes)), expected_payloads(&pkts));

    // The stalled consumer had packets dropped on its own connection, counted.
    let rs = reports(&handle);
    assert_eq!(rs.len(), 1, "the stalled consumer reported");
    assert_eq!(rs[0].offered, 100, "every packet was offered to it");
    assert!(rs[0].dropped > 0, "the stalled consumer dropped packets");
    // Per-consumer accounting is honest: what it wrote never exceeds what its
    // queue accepted (offered minus the backpressure drops).
    assert!(
        rs[0].written <= rs[0].offered - rs[0].dropped,
        "written ({}) exceeds accepted ({} - {})",
        rs[0].written,
        rs[0].offered,
        rs[0].dropped
    );
}

#[test]
fn an_abruptly_closed_consumer_is_reaped_and_others_keep_receiving() {
    let (mut sink, addr) = bind(1024, Duration::from_secs(5));

    let survivor = TcpStream::connect(addr).expect("connect survivor");
    let doomed = TcpStream::connect(addr).expect("connect doomed");
    wait_registered(&sink, 2);

    let pkts = packets(20, 64);
    for p in &pkts[..5] {
        sink.write(p).expect("write");
    }

    // The doomed consumer closes abruptly mid-stream.
    drop(doomed);
    // Offer packets until the sink notices the closed connection and reaps it.
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut i = 5;
    while sink.active_consumers() > 1 {
        if i < pkts.len() {
            sink.write(&pkts[i]).expect("write");
            i += 1;
        } else {
            sink.write(&pkts[pkts.len() - 1]).expect("write");
        }
        assert!(Instant::now() < deadline, "closed consumer was not reaped");
        std::thread::sleep(Duration::from_millis(10));
    }

    // Finish and confirm the survivor is intact.
    Box::new(sink)
        .finish(&CaptureStats::default())
        .expect("finish");
    let bytes = read_all(survivor);
    assert_valid_pcapng_stream(&bytes, 1);
    // The survivor received at least the first five packets in order.
    let got = epb_payloads(&walk(&bytes));
    assert!(
        got.len() >= 5,
        "survivor kept receiving after the peer closed"
    );
    assert_eq!(got[..5], expected_payloads(&pkts)[..5]);
}
