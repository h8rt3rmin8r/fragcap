// SPDX-License-Identifier: Apache-2.0

//! Real finite socket exchanges supply endpoint identities. Packet frames and
//! owned platform/client events are declared fixtures, not Npcap observations.

#![cfg(feature = "deep-capture")]

mod common;

use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use common::SharedBuf;
use fragcap::core::SourceError;
use fragcap::deep_capture::{
    build_process_trace, CaptureProcessEvidence, ProcessTraceInput, StageTransition,
    StageTransitionKind,
};
use fragcap::{
    AttributorConfig, CaptureSession, DeclaredNames, DeclaredTable, FlowRegistry, InterfaceAddrs,
    InterfaceDeclaration, InterfaceId, JsonLinesWriter, LinkType, PacketSource, PayloadMode,
    PcapngWriter, Pipeline, PipelineConfig, ProcessEvent, Profile, RawPacket,
    RoleStampingAttributor, SessionConfig, SessionGate, SocketTable, SocketTableAttributor,
    SocketTableEntry, SourceBinding, SourceStats, TestClock, Timestamp,
};
use fragcap_core::filter::FilterProgram;
use fragcap_core::process::WatcherReport;
use fragcap_core::traits::FlowAttributor;
use serde_json::Value;

const PLATFORM: u32 = 42;
const CLIENT: u32 = 43;
const PROXY: u32 = 99;
const REQUEST: &[u8] = b"GET /owned HTTP/1.1\r\nHost: fixture.invalid\r\n\r\n";
const RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK";

fn at(nanos: i64) -> Timestamp {
    Timestamp::from_nanos(nanos)
}

fn real_exchanges(ip: IpAddr) -> (SocketAddr, Vec<SocketAddr>) {
    let listener = TcpListener::bind(SocketAddr::new(ip, 0)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let proxy = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut peers = Vec::new();
        while peers.len() < 2 {
            assert!(Instant::now() < deadline, "finite loopback accept deadline");
            let (mut stream, peer) = match listener.accept() {
                Ok(value) => value,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(1));
                    continue;
                }
                Err(error) => panic!("loopback accept: {error}"),
            };
            // Windows can inherit the listener's nonblocking mode on accept.
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut request = vec![0; REQUEST.len()];
            stream.read_exact(&mut request).unwrap();
            assert_eq!(request, REQUEST);
            stream.write_all(RESPONSE).unwrap();
            peers.push(peer);
        }
        peers
    });
    let mut endpoints = Vec::new();
    for _ in 0..2 {
        let mut stream = TcpStream::connect_timeout(&proxy, Duration::from_secs(2)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        endpoints.push(stream.local_addr().unwrap());
        stream.write_all(REQUEST).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();
        assert_eq!(response, RESPONSE);
    }
    assert_eq!(server.join().unwrap(), endpoints);
    (proxy, endpoints)
}

fn frame(src: SocketAddr, dst: SocketAddr, payload: &[u8], nanos: i64) -> RawPacket {
    let mut tcp = vec![0_u8; 20];
    tcp[..2].copy_from_slice(&src.port().to_be_bytes());
    tcp[2..4].copy_from_slice(&dst.port().to_be_bytes());
    tcp[12] = 0x50;
    tcp.extend_from_slice(payload);
    let mut bytes = Vec::new();
    match (src.ip(), dst.ip()) {
        (IpAddr::V4(src), IpAddr::V4(dst)) => {
            bytes.extend_from_slice(&2_u32.to_le_bytes());
            let mut ip = vec![0_u8; 20];
            ip[0] = 0x45;
            ip[2..4].copy_from_slice(&((20 + tcp.len()) as u16).to_be_bytes());
            ip[8] = 64;
            ip[9] = 6;
            ip[12..16].copy_from_slice(&src.octets());
            ip[16..20].copy_from_slice(&dst.octets());
            bytes.extend_from_slice(&ip);
        }
        (IpAddr::V6(src), IpAddr::V6(dst)) => {
            // Windows DLT_NULL uses AF_INET6=23. The parser accepts it directly.
            bytes.extend_from_slice(&23_u32.to_le_bytes());
            let mut ip = vec![0_u8; 40];
            ip[0] = 0x60;
            ip[4..6].copy_from_slice(&(tcp.len() as u16).to_be_bytes());
            ip[6] = 6;
            ip[7] = 64;
            ip[8..24].copy_from_slice(&src.octets());
            ip[24..40].copy_from_slice(&dst.octets());
            bytes.extend_from_slice(&ip);
        }
        _ => panic!("one family per loopback exchange"),
    }
    bytes.extend_from_slice(&tcp);
    let length = bytes.len() as u32;
    RawPacket::new(at(nanos), bytes.into(), length)
}

struct DeclaredPackets(VecDeque<RawPacket>);

impl PacketSource for DeclaredPackets {
    fn next_packet(&mut self, _timeout: Duration) -> Result<Option<RawPacket>, SourceError> {
        self.0.pop_front().map(Some).ok_or(SourceError::Closed)
    }
    fn set_filter(&mut self, _filter: &FilterProgram) -> Result<(), SourceError> {
        Ok(())
    }
    fn stats(&self) -> SourceStats {
        SourceStats::default()
    }
    fn link_type(&self) -> LinkType {
        LinkType::NULL
    }
}

#[test]
fn both_families_preserve_real_endpoints_and_owned_platform_client_packet_truth() {
    for ip in ["127.0.0.1".parse().unwrap(), "::1".parse().unwrap()] {
        let (proxy, endpoints) = real_exchanges(ip);
        let profile = Profile::parse(
            r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"fixture","name":"Fixture","platform":"steam","app_id":"42"},"stage":[{"role":"platform","lifecycle":"service","match":{"exe":"steam.exe","path_regex":"(?i)^C:\\\\Steam\\\\steam\\.exe$"}},{"role":"client","lifecycle":"session","terminal":true,"match":{"exe":"client.exe","descends_from":"platform"}}]}"#,
        )
        .unwrap();
        let config = SessionConfig {
            exact_stage_ownership: true,
            ..SessionConfig::default()
        };
        let mut session = CaptureSession::new(profile, config.clone());
        session.require_owned_platform_root("C:\\Steam\\steam.exe");
        session.attach(at(0));
        session.arm_owned_platform_root(PLATFORM, 7, at(5), at(8));
        let starts = vec![
            ProcessEvent::started(PLATFORM, 7, "steam.exe", "fixture", at(6)),
            ProcessEvent::started(CLIENT, PLATFORM, "client.exe", "fixture", at(9)),
        ];
        for event in &starts {
            session.on_process_event(event.clone());
        }
        assert_eq!(session.role_bindings().len(), 2);
        let mut rows = Vec::new();
        let mut packets = VecDeque::new();
        for (index, (peer, pid)) in endpoints.iter().zip([PLATFORM, CLIENT]).enumerate() {
            rows.push(SocketTableEntry::tcp(*peer, proxy, pid).created_at(at(10)));
            rows.push(SocketTableEntry::tcp(proxy, *peer, PROXY).created_at(at(10)));
            let time = 50 + index as i64 * 20;
            packets.push_back(frame(*peer, proxy, REQUEST, time));
            packets.push_back(frame(proxy, *peer, RESPONSE, time + 10));
        }
        let inner = SocketTableAttributor::new(
            Box::new(DeclaredTable::once(SocketTable::new(at(20), rows))),
            Box::new(DeclaredNames::from([
                (PLATFORM, "steam.exe"),
                (CLIENT, "client.exe"),
                (PROXY, "fragcap.exe"),
            ])),
            Arc::new(TestClock::at(at(20))),
            AttributorConfig::default(),
        );
        inner.refresh().unwrap();
        let stamper = RoleStampingAttributor::new(Arc::new(inner));
        stamper.publisher().publish(session.role_bindings());
        let (tee, _receiver) = mpsc::channel();
        let (gate, handle) = SessionGate::new(&config, tee);
        handle.open_from(at(6));
        let registry = Arc::new(FlowRegistry::default());
        let mut pipeline = Pipeline::new(
            vec![SourceBinding::new(
                InterfaceId::new(0),
                Box::new(DeclaredPackets(packets)),
                InterfaceAddrs::new([ip]),
            )],
            Box::new(stamper),
            PipelineConfig::default(),
        )
        .unwrap();
        pipeline.set_write_gate(Arc::new(gate));
        pipeline.set_flow_registry(registry.clone());
        let pcap_bytes = SharedBuf::default();
        let json_bytes = SharedBuf::default();
        let mut pcap = PcapngWriter::new(pcap_bytes.clone()).unwrap();
        pcap.declare_interface(&InterfaceDeclaration::new(
            LinkType::NULL,
            65535,
            "loopback",
        ))
        .unwrap();
        pipeline.add_sink(Box::new(pcap));
        pipeline.add_sink(Box::new(
            JsonLinesWriter::new(json_bytes.clone(), &["loopback"], PayloadMode::MetadataOnly)
                .unwrap(),
        ));
        let report = pipeline.run();
        assert!(report.sink_failures.is_empty());
        assert_eq!(report.stats.packets_captured, 4);
        assert_eq!(report.stats.buffer_dropped, 0);
        assert_eq!(report.stats.sink_dropped, 0);
        assert_eq!(report.stats.gate_dropped, 0);
        assert_eq!(handle.admitted(), 4);
        let json = String::from_utf8(json_bytes.contents()).unwrap();
        let written = json
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .filter(|record| record["pid"].is_number())
            .collect::<Vec<_>>();
        assert_eq!(written.len(), 4);
        assert_eq!(written[0]["pid"], PLATFORM);
        assert_eq!(written[1]["pid"], PLATFORM);
        assert_eq!(written[2]["pid"], CLIENT);
        assert_eq!(written[3]["pid"], CLIENT);
        assert_eq!(written[0]["flow_id"], written[1]["flow_id"]);
        assert_eq!(written[2]["flow_id"], written[3]["flow_id"]);
        assert_ne!(written[0]["flow_id"], written[2]["flow_id"]);
        for packet in &written {
            assert_eq!(packet["dir"], "unknown");
            assert_eq!(packet["attr"], "live");
            assert_ne!(packet["pid"], PROXY);
        }
        let pcap = String::from_utf8_lossy(&pcap_bytes.contents()).into_owned();
        assert_eq!(pcap.matches("pid=42;").count(), 2);
        assert_eq!(pcap.matches("pid=43;").count(), 2);
        assert!(!pcap.contains("pid=99;"));
        let summaries = registry.summaries();
        assert_eq!(summaries.len(), 2);
        for summary in &summaries {
            assert_eq!(summary.observations.len(), 2);
            assert_eq!(summary.unretained_observations, 0);
            assert_eq!(summary.global_unretained_observations, 0);
        }
        let evidence = CaptureProcessEvidence {
            launch_pid: Some(PLATFORM),
            launch_at: Some(at(6)),
            events: starts
                .into_iter()
                .chain([
                    ProcessEvent::Exited {
                        pid: CLIENT,
                        at: at(100),
                    },
                    ProcessEvent::Exited {
                        pid: PLATFORM,
                        at: at(110),
                    },
                ])
                .collect(),
            stage_transitions: [PLATFORM, CLIENT]
                .into_iter()
                .zip(["platform", "client"])
                .map(|(pid, role)| StageTransition {
                    kind: StageTransitionKind::Matched,
                    pid,
                    role: role.into(),
                    stage: Some(role.into()),
                    at: if pid == PLATFORM { at(6) } else { at(9) },
                })
                .collect(),
            watcher_report: Some(WatcherReport::default()),
            watcher_ended: true,
            terminal_state: "complete".into(),
            ..CaptureProcessEvidence::default()
        };
        let trace = build_process_trace(ProcessTraceInput {
            session_id: "s169-loopback-fixture",
            target_id: Some(1),
            target_handle: "fixture",
            launch_case: "steam-protocol-cold",
            evidence: &evidence,
            flows: &summaries,
            globally_unretained_flow_observations: 0,
        });
        assert_eq!(trace.summary.flow_owner_intervals, 2);
        assert_eq!(trace.summary.unresolved_flow_owners, 0);
        let intervals = trace
            .jsonl
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .filter(|record| record["type"] == "socket-owner.interval")
            .collect::<Vec<_>>();
        assert_eq!(intervals[0]["flow_id"], written[0]["flow_id"]);
        assert_eq!(intervals[0]["pid"], PLATFORM);
        assert_eq!(intervals[0]["role"], "platform");
        assert_eq!(intervals[1]["flow_id"], written[2]["flow_id"]);
        assert_eq!(intervals[1]["pid"], CLIENT);
        assert_eq!(intervals[1]["role"], "client");
    }
}
