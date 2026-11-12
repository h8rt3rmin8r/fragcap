// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use std::net::SocketAddr;

use fragcap::deep_capture::{
    build_process_trace, build_route_debug_report, read_route_prefix, CaptureProcessEvidence,
    LifecycleWriter, ProcessTraceInput, RouteEvent, RouteEventKind, RouteEventLease,
    RouteRecordDisposition,
};

#[test]
fn selected_tcp_stream_has_exact_flow_and_terminal_outcome() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("route.jsonl");
    let mut lease = RouteEventLease::open(&path, "session-test", 16).unwrap();
    let sink = lease.sink();
    let local: SocketAddr = "192.0.2.10:49152".parse().unwrap();
    let remote: SocketAddr = "198.51.100.20:443".parse().unwrap();
    assert_eq!(
        sink.record(RouteEvent {
            flow_id: 1,
            at_ns: 100,
            clock_error_ns: Some(2),
            kind: RouteEventKind::Selected {
                protocol: "tcp",
                local,
                remote,
                pid: 42,
                process_instance_id: "process-42-100".into(),
                endpoint_id: 9,
                first_packet_at_ns: 90,
                first_packet_clock_error_ns: 2,
                tcp_flags: Some(2),
                packet_flags: 0,
                ip_checksum_valid: false,
                if_idx: 0,
                sub_if_idx: 0,
                tcp_checksum_valid: false,
            },
        }),
        RouteRecordDisposition::Accepted
    );
    sink.record(RouteEvent {
        flow_id: 1,
        at_ns: 101,
        clock_error_ns: Some(2),
        kind: RouteEventKind::ProxyConnected {
            helper_local: "127.0.0.1:41000".parse().unwrap(),
            proxy_local: "127.0.0.1:42000".parse().unwrap(),
        },
    });
    sink.record(RouteEvent {
        flow_id: 1,
        at_ns: 102,
        clock_error_ns: Some(2),
        kind: RouteEventKind::Transfer {
            client_to_upstream: 5,
            upstream_to_client: 8,
        },
    });
    sink.record(RouteEvent {
        flow_id: 1,
        at_ns: 103,
        clock_error_ns: Some(2),
        kind: RouteEventKind::Terminal {
            reason: "transferred",
        },
    });
    let outcome = lease.finish().unwrap();
    assert_eq!(outcome.dropped_events, 0);
    assert_eq!(outcome.written_events, 4);
    let prefix = read_route_prefix(&path).unwrap();
    assert_eq!(prefix.events.len(), 4);
    assert!(prefix.complete());
    assert_eq!(prefix.events[0].flow_id, 1);
    assert_eq!(prefix.events[2].kind, "transfer");
    assert_eq!(prefix.events[2].fields["client_to_upstream"], 5);
}

#[test]
fn crash_prefix_does_not_claim_completion() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("route.jsonl");
    let header = serde_json::json!({
        "type": "lifecycle.header",
        "schema_version": 1,
        "stream": "route",
        "session_id": "session-test"
    });
    std::fs::write(&path, format!("{header}\n")).unwrap();
    let prefix = read_route_prefix(&path).unwrap();
    assert!(!prefix.complete());
    assert!(prefix.events.is_empty());
    let report = build_route_debug_report(temp.path()).unwrap();
    assert_eq!(report["route"]["state"], "crash-prefix");
    assert_eq!(report["route"]["complete"], false);
}

#[test]
fn invalid_and_missing_report_inputs_remain_distinct() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("route.jsonl"), b"not-json\n").unwrap();
    std::fs::write(temp.path().join("compatibility.json"), b"{").unwrap();
    let report = build_route_debug_report(temp.path()).unwrap();
    assert_eq!(report["route"]["state"], "invalid");
    assert_eq!(report["proxy"]["state"], "unavailable");
    assert_eq!(report["compatibility"]["state"], "invalid");
    assert_eq!(report["source"]["manifest_source_state"], "unavailable");
}

#[test]
fn connected_zero_byte_route_is_not_client_reachability() {
    let temp = tempfile::tempdir().unwrap();
    let mut lease =
        RouteEventLease::open(temp.path().join("route.jsonl"), "session-test", 8).unwrap();
    let sink = lease.sink();
    for (index, kind) in [
        RouteEventKind::Selected {
            protocol: "tcp",
            local: "192.0.2.10:49152".parse().unwrap(),
            remote: "198.51.100.20:443".parse().unwrap(),
            pid: 42,
            process_instance_id: "process-42-100".into(),
            endpoint_id: 9,
            first_packet_at_ns: 1,
            first_packet_clock_error_ns: 2,
            tcp_flags: Some(2),
            packet_flags: 0,
            ip_checksum_valid: false,
            if_idx: 2,
            sub_if_idx: 0,
            tcp_checksum_valid: false,
        },
        RouteEventKind::ProxyConnected {
            helper_local: "127.0.0.1:41000".parse().unwrap(),
            proxy_local: "127.0.0.1:42000".parse().unwrap(),
        },
        RouteEventKind::SyntheticReplySent { packets: 1 },
        RouteEventKind::Terminal {
            reason: "socket-closed",
        },
    ]
    .into_iter()
    .enumerate()
    {
        sink.record(RouteEvent {
            flow_id: 1,
            at_ns: index as u64 + 1,
            clock_error_ns: None,
            kind,
        });
    }
    lease.finish().unwrap();
    let report = build_route_debug_report(temp.path()).unwrap();
    assert_eq!(report["flows"][0]["status"], "upstream-connected-zero-byte");
    assert_eq!(report["route"]["client_reachable_flows"], 0);
    assert_eq!(report["route"]["zero_byte_connected_flows"], 1);
}

#[test]
fn route_writer_counts_finite_capacity_loss() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("route.jsonl");
    let mut lease = RouteEventLease::open(&path, "session-test", 4).unwrap();
    let sink = lease.sink();
    for index in 0..10_000u64 {
        sink.record(RouteEvent {
            flow_id: 1,
            at_ns: index,
            clock_error_ns: None,
            kind: RouteEventKind::ProxyAttempt,
        });
    }
    let outcome = lease.finish().unwrap();
    let prefix = read_route_prefix(&path).unwrap();
    assert!(outcome.dropped_events > 0);
    assert_eq!(prefix.dropped_events, Some(outcome.dropped_events));
    assert!(!prefix.complete());
    assert!(prefix.events.len() <= 8_192);
    assert!(std::fs::metadata(path).unwrap().len() < 12 * 1024 * 1024);
}

#[test]
fn report_requires_client_response_and_omits_route_identity() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("route.jsonl");
    let mut lease = RouteEventLease::open(&path, "session-secret", 16).unwrap();
    let sink = lease.sink();
    let events = [
        RouteEventKind::Selected {
            protocol: "tcp",
            local: "192.0.2.10:49152".parse().unwrap(),
            remote: "198.51.100.20:443".parse().unwrap(),
            pid: 42,
            process_instance_id: "process-42-100".into(),
            endpoint_id: 9,
            first_packet_at_ns: 1,
            first_packet_clock_error_ns: 2,
            tcp_flags: Some(2),
            packet_flags: 0,
            ip_checksum_valid: false,
            if_idx: 1,
            sub_if_idx: 0,
            tcp_checksum_valid: false,
        },
        RouteEventKind::ProxyConnected {
            helper_local: "127.0.0.1:41000".parse().unwrap(),
            proxy_local: "127.0.0.1:42000".parse().unwrap(),
        },
        RouteEventKind::SyntheticReplySent { packets: 1 },
        RouteEventKind::Transfer {
            client_to_upstream: 5,
            upstream_to_client: 8,
        },
    ];
    for (index, kind) in events.into_iter().enumerate() {
        sink.record(RouteEvent {
            flow_id: 1,
            at_ns: index as u64 + 1,
            clock_error_ns: Some(2),
            kind,
        });
    }
    lease.finish().unwrap();
    let incomplete = build_route_debug_report(temp.path()).unwrap();
    assert_eq!(
        incomplete["flows"][0]["status"],
        "proxy-connected-without-client-reachability-proof"
    );
    let serialized = incomplete.to_string();
    assert_eq!(incomplete["transformations"].as_array().unwrap().len(), 4);
    for secret in [
        "192.0.2.10",
        "198.51.100.20",
        "process-42-100",
        "session-secret",
    ] {
        assert!(!serialized.contains(secret));
    }
}

#[test]
fn report_joins_exact_proxy_connection_and_packet_outcome() {
    let temp = tempfile::tempdir().unwrap();
    let mut route =
        RouteEventLease::open(temp.path().join("route.jsonl"), "private-session", 32).unwrap();
    let sink = route.sink();
    let kinds = [
        RouteEventKind::Selected {
            protocol: "tcp",
            local: "192.0.2.10:49152".parse().unwrap(),
            remote: "198.51.100.20:443".parse().unwrap(),
            pid: 42,
            process_instance_id: "process-42-100".into(),
            endpoint_id: 9,
            first_packet_at_ns: 100,
            first_packet_clock_error_ns: 2,
            tcp_flags: Some(2),
            packet_flags: 0,
            ip_checksum_valid: false,
            if_idx: 2,
            sub_if_idx: 0,
            tcp_checksum_valid: false,
        },
        RouteEventKind::ProxyConnected {
            helper_local: "127.0.0.1:41000".parse().unwrap(),
            proxy_local: "127.0.0.1:42000".parse().unwrap(),
        },
        RouteEventKind::SyntheticReplyGenerated {
            packets: 1,
            if_idx: 2,
            sub_if_idx: 0,
        },
        RouteEventKind::SyntheticReplySent { packets: 1 },
        RouteEventKind::ClientResponse { ack: true },
        RouteEventKind::ReturnedDataAcknowledged,
        RouteEventKind::Transfer {
            client_to_upstream: 5,
            upstream_to_client: 8,
        },
        RouteEventKind::Terminal {
            reason: "socket-closed",
        },
    ];
    for (index, kind) in kinds.into_iter().enumerate() {
        sink.record(RouteEvent {
            flow_id: 3,
            at_ns: 101 + index as u64,
            clock_error_ns: Some(2),
            kind,
        });
    }
    route.finish().unwrap();
    let mut proxy =
        LifecycleWriter::create(temp.path().join("proxy.jsonl"), "proxy", "private-session")
            .unwrap();
    proxy
        .append(
            "proxy.connection-open",
            serde_json::json!({
                "connection_id": 17, "timestamp_ns": 102,
                "detail": {"client_peer":"127.0.0.1:41000","proxy_local":"127.0.0.1:42000"}
            }),
        )
        .unwrap();
    proxy
        .append(
            "proxy.socks5-transfer",
            serde_json::json!({
                "connection_id": 17, "timestamp_ns": 110,
                "detail": {"client_to_upstream_bytes":5,"upstream_to_client_bytes":8}
            }),
        )
        .unwrap();
    proxy.finish().unwrap();
    std::fs::write(
        temp.path().join("compatibility.json"),
        serde_json::json!({
            "observations":[{"proxy_connection_id":"17","correlation_state":"matched",
                "correlation_reason":"direct-route-client-transfer-original-packet-unavailable",
                "flow_id":null,"packet_observations":0,"packet_observations_unretained":0,
                "role":"client","process_id":42,"attribution":"direct-route-receipt"}]
        })
        .to_string(),
    )
    .unwrap();
    let application_path = temp.path().join("application.jsonl");
    let application = [
        serde_json::json!({"type":"application.header","schema_version":2,"session_id":"private-session","sequence":0}),
        serde_json::json!({"type":"application.correlation","schema_version":2,"session_id":"private-session","sequence":1,"proxy_connection_id":17,
            "connection_owner":{"pid":42,"process_instance_id":"process-42-100","state":"exact","role":"client"}}),
    ];
    std::fs::write(
        &application_path,
        application
            .iter()
            .map(serde_json::Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
    .unwrap();
    let report = build_route_debug_report(temp.path()).unwrap();
    let row = &report["flows"][0];
    assert_eq!(row["status"], "bidirectional-transfer-observed");
    assert_eq!(row["proxy_connection_id"], 17);
    assert_eq!(row["proxy_transfer"]["client_to_upstream_bytes"], 5);
    assert_eq!(row["packet_correlation"]["observations"], 0);
    assert_eq!(row["packet_correlation"]["state"], "matched");
    assert_eq!(row["process_generation_join"], "exact");
    assert_eq!(row["final_client_generation_proven"], true);
    let mut incompatible = application;
    incompatible[1]["connection_owner"]["process_instance_id"] = serde_json::json!("process-42-99");
    std::fs::write(
        &application_path,
        incompatible
            .iter()
            .map(serde_json::Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
    .unwrap();
    let mismatch = build_route_debug_report(temp.path()).unwrap();
    assert_eq!(mismatch["flows"][0]["process_generation_join"], "mismatch");
    assert_eq!(
        mismatch["flows"][0]["final_client_generation_proven"],
        false
    );
    incompatible[1]["sequence"] = serde_json::json!(9);
    std::fs::write(
        &application_path,
        incompatible
            .iter()
            .map(serde_json::Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
    .unwrap();
    let invalid = build_route_debug_report(temp.path()).unwrap();
    assert_eq!(invalid["application"]["state"], "invalid");
    assert_eq!(
        invalid["flows"][0]["process_generation_join"],
        "unavailable"
    );
    let rendered = report.to_string();
    assert!(!rendered.contains("192.0.2.10") && !rendered.contains("process-42-100"));
}

#[test]
fn udp_report_joins_control_connection_and_keeps_relay_identity_private() {
    let temp = tempfile::tempdir().unwrap();
    let mut route =
        RouteEventLease::open(temp.path().join("route.jsonl"), "private-session", 16).unwrap();
    let sink = route.sink();
    for (index, kind) in [
        RouteEventKind::Selected {
            protocol: "udp",
            local: "192.0.2.10:49152".parse().unwrap(),
            remote: "198.51.100.20:443".parse().unwrap(),
            pid: 42,
            process_instance_id: "process-42-100".into(),
            endpoint_id: 9,
            first_packet_at_ns: 100,
            first_packet_clock_error_ns: 2,
            tcp_flags: None,
            packet_flags: 0,
            ip_checksum_valid: false,
            if_idx: 2,
            sub_if_idx: 0,
            tcp_checksum_valid: false,
        },
        RouteEventKind::ProxyConnected {
            helper_local: "127.0.0.1:41000".parse().unwrap(),
            proxy_local: "127.0.0.1:42000".parse().unwrap(),
        },
        RouteEventKind::UdpRelayAssociated {
            relay_local: "127.0.0.1:43000".parse().unwrap(),
            relay_remote: "127.0.0.1:44000".parse().unwrap(),
        },
        RouteEventKind::SyntheticReplySent { packets: 1 },
        RouteEventKind::DatagramTransfer {
            client_to_upstream: 5,
            upstream_to_client: 8,
            client_datagrams: 1,
            peer_datagrams: 1,
        },
    ]
    .into_iter()
    .enumerate()
    {
        sink.record(RouteEvent {
            flow_id: 1,
            at_ns: index as u64 + 100,
            clock_error_ns: Some(2),
            kind,
        });
    }
    route.finish().unwrap();
    let mut proxy =
        LifecycleWriter::create(temp.path().join("proxy.jsonl"), "proxy", "private-session")
            .unwrap();
    proxy
        .append(
            "proxy.connection-open",
            serde_json::json!({"connection_id":22,"timestamp_ns":101,
                "detail":{"client_peer":"127.0.0.1:41000","proxy_local":"127.0.0.1:42000"}}),
        )
        .unwrap();
    proxy.finish().unwrap();
    std::fs::write(
        temp.path().join("compatibility.json"),
        serde_json::json!({"observations":[{"proxy_connection_id":"22",
            "correlation_state":"matched","packet_observations":0}]})
        .to_string(),
    )
    .unwrap();
    let report = build_route_debug_report(temp.path()).unwrap();
    assert_eq!(report["flows"][0]["proxy_connection_id"], 22);
    assert_eq!(report["flows"][0]["packet_correlation"]["state"], "matched");
    assert_eq!(report["flows"][0]["client_reachability_observed"], true);
    assert!(!report.to_string().contains("127.0.0.1:43000"));
}

#[test]
fn empty_udp_datagrams_do_not_prove_payload_reachability() {
    let temp = tempfile::tempdir().unwrap();
    let mut route =
        RouteEventLease::open(temp.path().join("route.jsonl"), "private-session", 16).unwrap();
    let sink = route.sink();
    for (index, kind) in [
        RouteEventKind::Selected {
            protocol: "udp",
            local: "192.0.2.10:49152".parse().unwrap(),
            remote: "198.51.100.20:443".parse().unwrap(),
            pid: 42,
            process_instance_id: "process-42-100".into(),
            endpoint_id: 9,
            first_packet_at_ns: 100,
            first_packet_clock_error_ns: 2,
            tcp_flags: None,
            packet_flags: 0,
            ip_checksum_valid: false,
            if_idx: 2,
            sub_if_idx: 0,
            tcp_checksum_valid: false,
        },
        RouteEventKind::ProxyConnected {
            helper_local: "127.0.0.1:41000".parse().unwrap(),
            proxy_local: "127.0.0.1:42000".parse().unwrap(),
        },
        RouteEventKind::SyntheticReplySent { packets: 1 },
        RouteEventKind::DatagramTransfer {
            client_to_upstream: 0,
            upstream_to_client: 0,
            client_datagrams: 1,
            peer_datagrams: 1,
        },
    ]
    .into_iter()
    .enumerate()
    {
        sink.record(RouteEvent {
            flow_id: 1,
            at_ns: index as u64 + 100,
            clock_error_ns: None,
            kind,
        });
    }
    route.finish().unwrap();
    let report = build_route_debug_report(temp.path()).unwrap();
    assert_eq!(report["route"]["client_reachable_flows"], 0);
    assert_eq!(report["flows"][0]["client_reachability_observed"], false);
    assert_eq!(
        report["flows"][0]["status"],
        "client-datagram-response-observed-no-bidirectional-data"
    );
}

#[test]
fn report_declares_process_trace_source_without_disclosing_private_records() {
    let temp = tempfile::tempdir().unwrap();
    let evidence = CaptureProcessEvidence::default();
    let trace = build_process_trace(ProcessTraceInput {
        session_id: "private-session",
        target_id: Some(42),
        target_handle: "private-target",
        launch_case: "direct-exe-cold",
        evidence: &evidence,
        flows: &[],
        globally_unretained_flow_observations: 0,
    });
    let path = temp.path().join("process-trace.jsonl");
    std::fs::write(&path, &trace.jsonl).unwrap();
    let report = build_route_debug_report(temp.path()).unwrap();
    assert_eq!(report["process_trace"]["state"], "complete");
    assert_eq!(
        report["process_trace"]["completeness"],
        trace.summary.completeness
    );
    assert_eq!(report["process_trace"]["events"], trace.summary.records - 2);
    assert!(!report.to_string().contains("private-target"));
    let prefix = format!("{}\n", trace.jsonl.lines().next().unwrap());
    std::fs::write(&path, &prefix).unwrap();
    let partial = build_route_debug_report(temp.path()).unwrap();
    assert_eq!(partial["process_trace"]["state"], "crash-prefix");
    assert_eq!(partial["process_trace"]["events"], 0);
    std::fs::write(
        &path,
        format!(
            "{prefix}{}\n",
            serde_json::json!({"sequence":1,"type":"process.started","session_id":"other-session"})
        ),
    )
    .unwrap();
    let wrong_session = build_route_debug_report(temp.path()).unwrap();
    assert_eq!(wrong_session["process_trace"]["state"], "invalid");
    std::fs::write(&path, b"broken\n").unwrap();
    let invalid = build_route_debug_report(temp.path()).unwrap();
    assert_eq!(invalid["process_trace"]["state"], "invalid");
    assert_eq!(invalid["process_trace"]["events"], 0);
}
