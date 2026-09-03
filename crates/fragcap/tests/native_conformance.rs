// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use fragcap::deep_capture::{
    compatibility_fact_candidates, project_application_har, read_application_prefix,
    read_lifecycle_prefix, read_resource_journal, validate_v2, ApplicationArtifactLease,
    ApplicationStreamStatus, CalibrationPhase, CompatibilityObservation, CorrelationState,
    Inspectability, JournalStatus, LifecycleStreamStatus, LifecycleWriter, ProtocolClassification,
    ResourceJournal, ResourceKind, ResourceState, ResourceTransition,
};
use fragcap::profile::FidelityTier;
use fragcap::targets::{
    ClassificationSource, CompatibilityAddressFamily, CompatibilityApplicability,
    CompatibilityCase, CompatibilityEvidenceSource, CompatibilityFact as StoredCompatibilityFact,
    CompatibilityFactKey, CompatibilityLaunchCase, CompatibilityProtocol,
    CompatibilityRoutingStrategy, Store, TargetClassification, TargetEntry,
};
use fragcap_proxy::{
    is_quic_initial, BodyDirection, GrpcObserver, MetadataBlock, MetadataKind, ProtocolVersion,
    SocksAddressType, SocksReplyCode, SseObserver, WebSocketObserver,
};
use serde_json::{json, Value};

#[test]
fn complete_bundle_authorities_reconcile_from_one_synthetic_session() {
    const SESSION_ID: &str = "s110-conformance";
    let directory = tempfile::tempdir().unwrap();
    let application_path = directory.path().join("application.jsonl");
    let mut application = ApplicationArtifactLease::open(&application_path, SESSION_ID, 8).unwrap();
    application.finish().unwrap();
    let application_prefix = read_application_prefix(&application_path).unwrap();
    assert_eq!(application_prefix.status, ApplicationStreamStatus::Complete);
    assert_eq!(application_prefix.classification_schema_version, Some(1));
    assert_eq!(
        application_prefix.classification_status,
        fragcap::deep_capture::ClassificationStreamStatus::Supported
    );
    assert!(application_prefix
        .records
        .iter()
        .all(|record| record["session_id"] == SESSION_ID));
    let trailer = application_prefix.records.last().unwrap();
    assert_eq!(trailer["classification_schema_version"], 1);
    assert_eq!(trailer["classified_records"], 0);
    assert_eq!(trailer["classification_records_lost"], 0);

    let har_path = directory.path().join("http.har");
    let har = project_application_har(&application_path)
        .unwrap()
        .publish(&har_path)
        .unwrap();
    assert_eq!(har.standard_entries, 0);
    assert_eq!(har.partial_entries, 0);

    for stream in ["proxy", "cleanup"] {
        let path = directory.path().join(format!("{stream}.jsonl"));
        let mut writer = LifecycleWriter::create(&path, stream, SESSION_ID).unwrap();
        writer
            .append("conformance.observation", json!({"outcome":"pass"}))
            .unwrap();
        writer.finish().unwrap();
        let prefix = read_lifecycle_prefix(&path).unwrap();
        assert_eq!(prefix.status, LifecycleStreamStatus::Complete);
        assert_eq!(prefix.session_id, SESSION_ID);
    }

    let mut journal = ResourceJournal::create(directory.path(), SESSION_ID, "plan-s110").unwrap();
    for state in [
        ResourceState::Pending,
        ResourceState::Applied,
        ResourceState::CleanupPending,
        ResourceState::Released,
    ] {
        journal
            .append(ResourceTransition::new(
                "synthetic-listener",
                ResourceKind::Proxy,
                "127.0.0.1:0",
                "session:s110-conformance",
                "close-owned-listener",
                state,
                "synthetic conformance transition",
            ))
            .unwrap();
    }
    journal.finish().unwrap();
    let journal_prefix = read_resource_journal(journal.path()).unwrap();
    assert_eq!(journal_prefix.status, JournalStatus::Complete);
    assert_eq!(journal_prefix.session_id, SESSION_ID);
    assert_eq!(journal_prefix.transitions.len(), 4);

    fs::write(
        directory.path().join("cleanup.json"),
        serde_json::to_vec_pretty(&json!({
            "session_id": SESSION_ID,
            "status": "succeeded",
            "released_resources": 1,
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        directory.path().join("correlation.json"),
        serde_json::to_vec_pretty(&json!({
            "session_id": SESSION_ID,
            "application_records": application_prefix.records.len(),
            "connections": 0,
            "state": "complete",
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        directory.path().join("capture.fcapng"),
        include_bytes!("../../../conformance/native-http-tls/analyzer.pcapng"),
    )
    .unwrap();
    fs::write(
        directory.path().join("tls-keylog.log"),
        include_bytes!("../../../conformance/native-http-tls/tls-keylog.log"),
    )
    .unwrap();

    let manifest = json!({
        "$schema": "https://fragcap.dev/schema/deep-capture-manifest.v2.json",
        "manifest_version": 2,
        "product": {"name":"fragcap","version":env!("CARGO_PKG_VERSION")},
        "session_id": SESSION_ID,
        "state": "complete",
        "artifacts": [
            artifact("application-jsonl", "application.jsonl", "primary-evidence", "application-events", None, "sensitive", "application/x-ndjson"),
            artifact("har", "http.har", "derived-projection", "http-projection", Some("application-jsonl"), "sensitive", "application/json"),
            artifact("tls-key-log", "tls-keylog.log", "analyzer-aid", "analyzer-aid", None, "secret-adjacent", "text/plain"),
            artifact("pcapng", "capture.fcapng", "primary-evidence", "packet-truth", None, "ordinary", "application/x-pcapng"),
            artifact("correlation", "correlation.json", "derived-projection", "correlation-summary", Some("application-jsonl"), "ordinary", "application/json"),
            artifact("proxy-lifecycle", "proxy.jsonl", "primary-evidence", "proxy-lifecycle-events", None, "sensitive", "application/x-ndjson"),
            artifact("cleanup-lifecycle", "cleanup.jsonl", "operational-record", "cleanup-lifecycle-events", None, "ordinary", "application/x-ndjson"),
            artifact("cleanup-summary", "cleanup.json", "derived-projection", "cleanup-projection", Some("cleanup-lifecycle"), "ordinary", "application/json"),
            artifact("resource-journal", "resource-journal.jsonl", "operational-record", "resource-ownership-journal", None, "secret-adjacent", "application/x-ndjson"),
            artifact("manifest-v2", "manifest.json", "bundle-index", "bundle-index", None, "ordinary", "application/json")
        ],
        "omissions": [],
    });
    let manifest_path = directory.path().join("manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let manifest: Value = serde_json::from_slice(&fs::read(manifest_path).unwrap()).unwrap();
    validate_v2(&manifest).unwrap();

    let roles = manifest["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["role"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(roles.len(), 10);
    for artifact in manifest["artifacts"].as_array().unwrap() {
        let path = directory.path().join(artifact["path"].as_str().unwrap());
        assert!(path.is_file(), "missing {}", path.display());
        assert!(
            path.metadata().unwrap().len() > 0,
            "empty {}",
            path.display()
        );
    }
    let cleanup: Value =
        serde_json::from_slice(&fs::read(directory.path().join("cleanup.json")).unwrap()).unwrap();
    let correlation: Value =
        serde_json::from_slice(&fs::read(directory.path().join("correlation.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["session_id"], cleanup["session_id"]);
    assert_eq!(manifest["session_id"], correlation["session_id"]);
    assert_eq!(
        correlation["application_records"],
        application_prefix.records.len()
    );
}

fn artifact(
    role: &str,
    path: &str,
    kind: &str,
    owner: &str,
    source_role: Option<&str>,
    sensitivity: &str,
    content_type: &str,
) -> Value {
    json!({
        "role": role,
        "path": path,
        "authority": {"kind":kind,"owner":owner,"source_role":source_role},
        "sensitivity": sensitivity,
        "content_type": content_type,
        "required": true,
        "finalization": "complete",
        "completeness": "complete",
        "loss": {"state":"none"},
        "correlation": if matches!(role, "application-jsonl" | "har" | "pcapng" | "correlation") {
            json!({"state":"complete","records":0})
        } else {
            json!({"state":"not-applicable"})
        },
    })
}

#[test]
fn committed_conformance_report_has_no_required_skip_state() {
    let report: Value = serde_json::from_str(include_str!(
        "../../../conformance/native-http-tls/report-v1.json"
    ))
    .unwrap();
    assert_eq!(report["summary"]["required"], report["summary"]["passed"]);
    for field in ["failed", "skipped", "not_run", "missing", "duplicate"] {
        assert_eq!(report["summary"][field], 0, "{field}");
    }
    assert!(report["rows"]
        .as_array()
        .unwrap()
        .iter()
        .all(|row| row["status"] == "pass"));
}

#[test]
fn native_calibration_matrix_executes_and_persists_every_supported_case() {
    let launches = [
        CompatibilityLaunchCase::SteamProtocolCold,
        CompatibilityLaunchCase::DirectExeCold,
        CompatibilityLaunchCase::PublisherLauncherCold,
    ];
    let families = [
        CompatibilityAddressFamily::Ipv4,
        CompatibilityAddressFamily::Ipv6,
    ];
    let protocols = [
        CompatibilityProtocol::Routing,
        CompatibilityProtocol::Http1,
        CompatibilityProtocol::Https,
        CompatibilityProtocol::Http2,
        CompatibilityProtocol::WebSocket,
        CompatibilityProtocol::Sse,
        CompatibilityProtocol::Grpc,
        CompatibilityProtocol::GenericTcp,
        CompatibilityProtocol::NonHttpTls,
        CompatibilityProtocol::Socks5Tcp,
        CompatibilityProtocol::Socks5Udp,
        CompatibilityProtocol::GenericUdp,
        CompatibilityProtocol::Quic,
        CompatibilityProtocol::Http3,
    ];
    let mut store = Store::open_in_memory().unwrap();
    let target_id = store
        .insert_target(&TargetEntry {
            id: None,
            stable_id: 121,
            handle: "s121-controlled".to_string(),
            name: "S121 controlled target".to_string(),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::User,
            fidelity: FidelityTier::Authored,
            provenance: None,
            anchor: None,
            launch_entries: None,
            install_root: None,
            evidence: None,
            detection_scan: None,
            folder_name: None,
            executable_hint: None,
        })
        .unwrap();
    let mut rows = BTreeSet::new();
    for launch in launches {
        for family in families {
            for protocol in protocols {
                execute_loopback_witness(family, protocol);
                assert!(rows.insert((
                    launch.as_str(),
                    CompatibilityRoutingStrategy::ChildEnvironment.as_str(),
                    family.as_str(),
                    protocol.as_str(),
                )));
                let phase = if protocol == CompatibilityProtocol::Routing {
                    CalibrationPhase::Reachability
                } else {
                    CalibrationPhase::Tls
                };
                let observations = if protocol == CompatibilityProtocol::Routing {
                    vec![controlled_observation(CompatibilityProtocol::Http1)]
                } else {
                    vec![controlled_observation(protocol)]
                };
                let candidates = compatibility_fact_candidates(
                    launch.as_str(),
                    &observations,
                    true,
                    Some(phase),
                    Some(protocol),
                );
                if protocol == CompatibilityProtocol::Routing {
                    assert!(candidates
                        .iter()
                        .any(|fact| fact.key == CompatibilityFactKey::ProxyRouting));
                } else {
                    assert!(candidates.iter().any(|fact| {
                        fact.key == CompatibilityFactKey::ProtocolBehavior
                            && fact.protocol == protocol
                    }));
                    assert!(candidates.iter().any(|fact| {
                        fact.key == CompatibilityFactKey::Inspectability
                            && fact.protocol == protocol
                    }));
                }
                let current = CompatibilityCase {
                    launch_case: launch,
                    proxy_backend: "fragcap-native".to_string(),
                    proxy_backend_version: env!("CARGO_PKG_VERSION").to_string(),
                    routing_strategy: CompatibilityRoutingStrategy::ChildEnvironment,
                    address_family: family,
                    protocol,
                    fragcap_version: env!("CARGO_PKG_VERSION").to_string(),
                    target_version: None,
                };
                for candidate in candidates {
                    let mut fact = StoredCompatibilityFact::new(
                        target_id,
                        candidate.key,
                        candidate.value,
                        CompatibilityEvidenceSource::ObservedRun,
                    )
                    .unwrap();
                    fact.launch_case = Some(launch);
                    fact.proxy_backend = Some("fragcap-native".to_string());
                    fact.proxy_backend_version = Some(env!("CARGO_PKG_VERSION").to_string());
                    fact.routing_strategy = Some(CompatibilityRoutingStrategy::ChildEnvironment);
                    fact.address_family = Some(family);
                    fact.protocol = Some(candidate.protocol);
                    fact.fragcap_version = Some(env!("CARGO_PKG_VERSION").to_string());
                    assert_eq!(
                        fact.applicability(&current),
                        CompatibilityApplicability::Applicable
                    );
                    let fact_id = store.insert_compatibility_fact(&fact).unwrap();
                    let persisted = store
                        .compatibility_facts_for_target(target_id)
                        .unwrap()
                        .into_iter()
                        .find(|stored| stored.id == Some(fact_id))
                        .unwrap();
                    assert_eq!(persisted.launch_case, Some(launch));
                    assert_eq!(persisted.routing_strategy, fact.routing_strategy);
                    assert_eq!(persisted.address_family, Some(family));
                    assert_eq!(persisted.protocol, Some(candidate.protocol));
                    assert_eq!(
                        persisted.applicability(&current),
                        CompatibilityApplicability::Applicable
                    );
                }
            }
        }
    }
    assert_eq!(
        rows.len(),
        launches.len() * families.len() * protocols.len()
    );
}

fn execute_loopback_witness(family: CompatibilityAddressFamily, protocol: CompatibilityProtocol) {
    let bind = match family {
        CompatibilityAddressFamily::Ipv4 => SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        CompatibilityAddressFamily::Ipv6 => SocketAddr::from((Ipv6Addr::LOCALHOST, 0)),
    };
    let listener = TcpListener::bind(bind).unwrap();
    let endpoint = listener.local_addr().unwrap();
    let expected = format!("s121:{}", protocol.as_str()).into_bytes();
    let server_expected = expected.clone();
    let server = thread::spawn(move || {
        let (mut stream, peer) = listener.accept().unwrap();
        assert_eq!(peer.is_ipv4(), endpoint.is_ipv4());
        let mut received = vec![0; server_expected.len()];
        stream.read_exact(&mut received).unwrap();
        assert_eq!(received, server_expected);
        stream.write_all(&received).unwrap();
    });
    let mut client = TcpStream::connect_timeout(&endpoint, Duration::from_secs(2)).unwrap();
    client.write_all(&expected).unwrap();
    let mut echoed = vec![0; expected.len()];
    client.read_exact(&mut echoed).unwrap();
    assert_eq!(echoed, expected);
    server.join().unwrap();
}

fn controlled_observation(protocol: CompatibilityProtocol) -> CompatibilityObservation {
    exercise_protocol_witness(protocol);
    let (label, inspectability) = match protocol {
        CompatibilityProtocol::Http1 => ("http/1.1", "full"),
        CompatibilityProtocol::Https => ("https", "full"),
        CompatibilityProtocol::Http2 => ("h2", "full"),
        CompatibilityProtocol::WebSocket => ("websocket", "full"),
        CompatibilityProtocol::Sse => ("sse", "full"),
        CompatibilityProtocol::Grpc => ("grpc", "full"),
        CompatibilityProtocol::GenericTcp => ("tcp", "protocol-unknown"),
        CompatibilityProtocol::NonHttpTls => ("tls", "protocol-unknown"),
        CompatibilityProtocol::Socks5Tcp => ("socks5-connect", "metadata-only"),
        CompatibilityProtocol::Socks5Udp => ("socks5-udp", "metadata-only"),
        CompatibilityProtocol::GenericUdp => ("generic-udp", "metadata-only"),
        CompatibilityProtocol::Quic => ("quic", "protocol-unknown"),
        CompatibilityProtocol::Http3 => ("h3", "full"),
        CompatibilityProtocol::Routing | CompatibilityProtocol::NotApplicable => {
            panic!("routing has no protocol observation")
        }
    };
    let classification = ProtocolClassification::from_proxy_evidence(label, inspectability, None);
    assert_eq!(classification.family().as_str(), protocol.as_str());
    CompatibilityObservation {
        flow_id: None,
        proxy_connection_id: format!("controlled-{label}"),
        client_peer: None,
        proxy_local: None,
        observed_at: "2026-09-03T00:00:00Z".to_string(),
        process_id: Some(121),
        process_image: Some("fragcap-controlled.exe".to_string()),
        role: Some("client".to_string()),
        attribution: Some("controlled-harness".to_string()),
        packet_observations: 1,
        packet_observations_unretained: 0,
        correlation_state: CorrelationState::FlowOnly,
        correlation_reason: "controlled loopback witness".to_string(),
        protocol: label.to_string(),
        inspectability: match inspectability {
            "full" => Inspectability::Full,
            "metadata-only" => Inspectability::MetadataOnly,
            _ => Inspectability::Inconclusive,
        },
        method: None,
        url: None,
        status: None,
        reason: None,
        classification,
    }
}

fn exercise_protocol_witness(protocol: CompatibilityProtocol) {
    match protocol {
        CompatibilityProtocol::Http1 | CompatibilityProtocol::Https => {
            let metadata = MetadataBlock::http1(
                MetadataKind::Request,
                &[("host".to_string(), b"controlled.invalid".to_vec())],
            )
            .with_http1_request("GET", "/s121", "http://controlled.invalid/s121");
            assert_eq!(metadata.version, ProtocolVersion::Http11);
            assert_eq!(metadata.method.as_deref(), Some(b"GET".as_slice()));
        }
        CompatibilityProtocol::Http2 => {
            let metadata = MetadataBlock::http2(MetadataKind::Request, Vec::new(), Vec::new());
            assert_eq!(metadata.version, ProtocolVersion::Http2);
        }
        CompatibilityProtocol::Http3 => {
            let metadata = MetadataBlock::http3(MetadataKind::Request, Vec::new(), Vec::new());
            assert_eq!(metadata.version, ProtocolVersion::Http3);
        }
        CompatibilityProtocol::WebSocket => {
            let mut observer =
                WebSocketObserver::new(BodyDirection::Response, false, false, 64, 64);
            assert!(!observer.feed(&[0x81, 0x02, b'o', b'k']).is_empty());
        }
        CompatibilityProtocol::Sse => {
            let mut observer = SseObserver::new(64, 64);
            assert!(!observer.feed(b"data: ok\n\n").is_empty());
        }
        CompatibilityProtocol::Grpc => {
            let mut observer = GrpcObserver::new(BodyDirection::Response, 64);
            assert!(!observer.feed(&[0, 0, 0, 0, 2, b'o', b'k']).is_empty());
        }
        CompatibilityProtocol::Socks5Tcp | CompatibilityProtocol::Socks5Udp => {
            assert_eq!(SocksAddressType::Ipv4.as_str(), "ipv4");
            assert_eq!(SocksReplyCode::Succeeded as u8, 0);
        }
        CompatibilityProtocol::Quic => {
            let mut initial = vec![0_u8; 1200];
            initial[0] = 0xc0;
            initial[1..5].copy_from_slice(&[0, 0, 0, 1]);
            assert!(is_quic_initial(&initial));
        }
        CompatibilityProtocol::GenericTcp
        | CompatibilityProtocol::NonHttpTls
        | CompatibilityProtocol::GenericUdp => {}
        CompatibilityProtocol::Routing | CompatibilityProtocol::NotApplicable => {
            panic!("routing has no protocol witness")
        }
    }
}
