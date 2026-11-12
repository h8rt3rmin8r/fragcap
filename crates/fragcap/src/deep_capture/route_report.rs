// SPDX-License-Identifier: Apache-2.0

//! Shareable, bounded projection of one direct-route bundle.

use std::{collections::BTreeMap, fs, io, path::Path};

use serde_json::{json, Value};

use super::{read_lifecycle_prefix, read_process_trace, read_route_prefix, LifecycleStreamStatus};

const MAX_SUMMARY_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Default)]
struct FlowSummary {
    protocol: Option<String>,
    pid: Option<u64>,
    process_instance_id: Option<String>,
    first_packet_at_ns: Option<i64>,
    first_packet_clock_error_ns: Option<u64>,
    last_route_event_at_ns: Option<u64>,
    route_clock_error_ns: Option<u64>,
    proxy_attempted: bool,
    proxy_connected: bool,
    proxy_refused: bool,
    reply_generated: bool,
    reply_sent: bool,
    reply_failed: bool,
    client_ack: bool,
    returned_data_acknowledged: bool,
    client_to_upstream: u64,
    upstream_to_client: u64,
    client_datagrams: u64,
    peer_datagrams: u64,
    terminal: Option<String>,
    helper_pair: Option<(String, String)>,
}

struct ProcessTraceSource {
    state: &'static str,
    completeness: Option<&'static str>,
    events: u64,
}

fn process_trace_source(path: &Path) -> io::Result<ProcessTraceSource> {
    if fs::metadata(path)?.len() > MAX_SUMMARY_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "process trace exceeds report byte limit",
        ));
    }
    let bytes = fs::read(path)?;
    if bytes.len() as u64 > MAX_SUMMARY_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "process trace exceeds report byte limit",
        ));
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let mut lines = text.lines().peekable();
    let mut records = 0_u64;
    let mut trailer = false;
    let mut session_id = None;
    while let Some(line) = lines.next() {
        if records >= 65_536 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "process trace exceeds report record limit",
            ));
        }
        let value: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) if lines.peek().is_none() && !text.ends_with('\n') => break,
            Err(error) => return Err(io::Error::new(io::ErrorKind::InvalidData, error)),
        };
        if value["sequence"].as_u64() != Some(records) || trailer {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "process trace sequence is invalid",
            ));
        }
        if records == 0 {
            if value["type"] != "process-trace.header" {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "process trace header is missing",
                ));
            }
            if value["schema_version"] != 1 {
                return Ok(ProcessTraceSource {
                    state: "unknown-version",
                    completeness: None,
                    events: 0,
                });
            }
            session_id = value["session_id"].as_str().map(str::to_owned);
            if session_id.is_none() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "process trace session is missing",
                ));
            }
        } else if value["type"] == "process-trace.header" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "process trace has a second header",
            ));
        }
        if value["session_id"].as_str() != session_id.as_deref() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "process trace record session differs",
            ));
        }
        if value["type"] == "process-trace.trailer" {
            if value["session_id"].as_str() != session_id.as_deref() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "process trace trailer session differs",
                ));
            }
            trailer = true;
        }
        records += 1;
    }
    if records == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "process trace is empty",
        ));
    }
    if trailer && text.ends_with('\n') {
        let summary = read_process_trace(text).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "process trace trailer is invalid",
            )
        })?;
        if summary.records != records {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "process trace record count differs",
            ));
        }
        return Ok(ProcessTraceSource {
            state: "complete",
            completeness: Some(summary.completeness),
            events: records - 2,
        });
    }
    Ok(ProcessTraceSource {
        state: "crash-prefix",
        completeness: None,
        events: records - 1,
    })
}

fn bounded_json(path: &Path) -> io::Result<Value> {
    if fs::metadata(path)?.len() > MAX_SUMMARY_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "report input exceeds byte limit",
        ));
    }
    serde_json::from_slice(&fs::read(path)?).map_err(io::Error::other)
}

fn optional_source<T>(result: io::Result<T>) -> (Option<T>, &'static str) {
    match result {
        Ok(value) => (Some(value), "available"),
        Err(error) if error.kind() == io::ErrorKind::NotFound => (None, "unavailable"),
        Err(_) => (None, "invalid"),
    }
}

fn stream_status(status: LifecycleStreamStatus) -> &'static str {
    match status {
        LifecycleStreamStatus::Complete => "complete",
        LifecycleStreamStatus::CrashPrefix => "crash-prefix",
        LifecycleStreamStatus::UnknownVersion => "unknown-version",
    }
}

/// Derive a source-preserving report with no addresses, process names, payloads,
/// capabilities, certificate material, or raw packet bytes.
pub fn build_route_debug_report(bundle: &Path) -> io::Result<Value> {
    if !bundle.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "bundle directory is missing",
        ));
    }
    let exact_bundle = bundle.canonicalize()?;
    for name in [
        "manifest.json",
        "compatibility.json",
        "cleanup.json",
        "proxy.jsonl",
        "process-trace.jsonl",
        "route.jsonl",
    ] {
        let path = bundle.join(name);
        match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if !metadata.is_file()
                    || metadata.file_type().is_symlink()
                    || !path.canonicalize()?.starts_with(&exact_bundle)
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "report input is not an exact bundle file",
                    ));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    let (manifest, manifest_source_state) =
        optional_source(bounded_json(&bundle.join("manifest.json")));
    let (compatibility, compatibility_source_state) =
        optional_source(bounded_json(&bundle.join("compatibility.json")));
    let (cleanup, cleanup_source_state) =
        optional_source(bounded_json(&bundle.join("cleanup.json")));
    let (proxy, proxy_source_state) =
        optional_source(read_lifecycle_prefix(&bundle.join("proxy.jsonl")));
    let (process_trace, process_trace_source_state) =
        optional_source(process_trace_source(&bundle.join("process-trace.jsonl")));
    let (route, route_source_state) =
        optional_source(read_route_prefix(&bundle.join("route.jsonl")));
    let mut proxy_pairs = BTreeMap::<(String, String), Vec<u64>>::new();
    if let Some(prefix) = &proxy {
        for record in &prefix.records {
            if record["type"] == "proxy.connection-open" {
                let fields = &record["fields"];
                if let (Some(peer), Some(local), Some(connection_id)) = (
                    fields["detail"]["client_peer"].as_str(),
                    fields["detail"]["proxy_local"].as_str(),
                    fields["connection_id"].as_u64(),
                ) {
                    proxy_pairs
                        .entry((peer.into(), local.into()))
                        .or_default()
                        .push(connection_id);
                }
            }
        }
    }
    let mut flows = BTreeMap::<u64, FlowSummary>::new();
    if let Some(prefix) = &route {
        for event in &prefix.events {
            let flow = flows.entry(event.flow_id).or_default();
            flow.last_route_event_at_ns = Some(event.at_ns);
            flow.route_clock_error_ns = event.clock_error_ns;
            match event.kind.as_str() {
                "selected" => {
                    flow.protocol = event.fields["protocol"].as_str().map(str::to_owned);
                    flow.pid = event.fields["pid"].as_u64();
                    flow.process_instance_id = event.fields["process_instance_id"]
                        .as_str()
                        .map(str::to_owned);
                    flow.first_packet_at_ns = event.fields["first_packet_at_ns"].as_i64();
                    flow.first_packet_clock_error_ns =
                        event.fields["first_packet_clock_error_ns"].as_u64();
                }
                "proxy-attempt" => flow.proxy_attempted = true,
                "proxy-connected" => {
                    flow.proxy_connected = true;
                    flow.helper_pair = event.fields["helper_local"]
                        .as_str()
                        .zip(event.fields["proxy_local"].as_str())
                        .map(|(helper, proxy)| (helper.into(), proxy.into()));
                }
                "proxy-refused" => flow.proxy_refused = true,
                "synthetic-reply-generated" => flow.reply_generated = true,
                "synthetic-reply-sent" => flow.reply_sent = true,
                "synthetic-reply-failed" => flow.reply_failed = true,
                "client-response" => flow.client_ack |= event.fields["ack"].as_bool() == Some(true),
                "returned-data-acknowledged" => flow.returned_data_acknowledged = true,
                "transfer" => {
                    flow.client_to_upstream = flow
                        .client_to_upstream
                        .max(event.fields["client_to_upstream"].as_u64().unwrap_or(0));
                    flow.upstream_to_client = flow
                        .upstream_to_client
                        .max(event.fields["upstream_to_client"].as_u64().unwrap_or(0));
                }
                "datagram-transfer" => {
                    flow.client_to_upstream = flow
                        .client_to_upstream
                        .saturating_add(event.fields["client_to_upstream"].as_u64().unwrap_or(0));
                    flow.upstream_to_client = flow
                        .upstream_to_client
                        .saturating_add(event.fields["upstream_to_client"].as_u64().unwrap_or(0));
                    flow.client_datagrams = flow
                        .client_datagrams
                        .saturating_add(event.fields["client_datagrams"].as_u64().unwrap_or(0));
                    flow.peer_datagrams = flow
                        .peer_datagrams
                        .saturating_add(event.fields["peer_datagrams"].as_u64().unwrap_or(0));
                }
                "terminal" => flow.terminal = event.fields["reason"].as_str().map(str::to_owned),
                _ => {}
            }
        }
    }
    let flow_rows: Vec<Value> = flows.into_iter().enumerate().map(|(index, (_, flow))| {
        let client_response = flow.proxy_connected && flow.reply_sent && !flow.reply_failed
            && if flow.protocol.as_deref() == Some("udp") {
                flow.client_datagrams > 0 && flow.peer_datagrams > 0
            } else {
                flow.client_ack
            };
        let bidirectional = client_response
            && (flow.protocol.as_deref() == Some("udp") || flow.returned_data_acknowledged)
            && flow.client_to_upstream > 0
            && flow.upstream_to_client > 0;
        let client_reachable = bidirectional;
        let matching_proxy = flow.helper_pair.as_ref().and_then(|pair| proxy_pairs.get(pair));
        let proxy_connection_id = matching_proxy.and_then(|matches| (matches.len() == 1).then_some(matches[0]));
        let proxy_transfer = proxy_connection_id.and_then(|connection_id| proxy.as_ref().and_then(|prefix| prefix.records.iter().rev().find(|record| {
            record["type"] == "proxy.socks5-transfer" && record["fields"]["connection_id"] == connection_id
        })));
        let proxy_terminal = proxy_connection_id.and_then(|connection_id| proxy.as_ref().and_then(|prefix| prefix.records.iter().rev().find(|record| {
            record["type"] == "proxy.connection-terminal" && record["fields"]["connection_id"] == connection_id
        })));
        let packet = proxy_connection_id.and_then(|connection_id| compatibility.as_ref().and_then(|value| value["observations"].as_array()).and_then(|records| records.iter().find(|record| record["proxy_connection_id"].as_str().and_then(|value| value.parse::<u64>().ok()) == Some(connection_id))));
        let generation_join = packet.and_then(|record| {
            flow.pid.zip(flow.process_instance_id.as_deref()).map(|(pid, instance)| {
                if record["connection_owner"]["pid"].as_u64() == Some(pid)
                    && record["connection_owner"]["process_instance_id"].as_str() == Some(instance)
                {
                    "exact"
                } else {
                    "mismatch"
                }
            })
        });
        json!({
            "flow": format!("flow-{}", index + 1),
            "protocol": flow.protocol,
            "first_packet_at_ns": flow.first_packet_at_ns,
            "first_packet_clock_error_ns": flow.first_packet_clock_error_ns,
            "last_route_event_at_ns": flow.last_route_event_at_ns,
            "last_route_event_clock_error_ns": flow.route_clock_error_ns,
            "status": if bidirectional { "bidirectional-transfer-observed" } else if client_response && flow.protocol.as_deref() == Some("udp") { "client-datagram-response-observed-no-bidirectional-data" } else if client_response { "client-handshake-observed-no-bidirectional-data" } else if flow.proxy_refused { "proxy-refused" } else if flow.reply_failed { "reply-injection-failed" } else if flow.proxy_connected && flow.client_to_upstream == 0 && flow.upstream_to_client == 0 { "upstream-connected-zero-byte" } else if flow.proxy_connected { "proxy-connected-without-client-reachability-proof" } else if flow.proxy_attempted { "proxy-attempt-without-connection" } else { "selected-only" },
            "proxy_connected": flow.proxy_connected,
            "client_ack_observed": flow.client_ack,
            "returned_data_acknowledged": flow.returned_data_acknowledged,
            "client_response_observed": client_response,
            "client_reachability_observed": client_reachable,
            "synthetic_reply_generated": flow.reply_generated,
            "synthetic_reply_sent": flow.reply_sent,
            "client_to_upstream_bytes": flow.client_to_upstream,
            "upstream_to_client_bytes": flow.upstream_to_client,
            "client_datagrams": flow.client_datagrams,
            "peer_datagrams": flow.peer_datagrams,
            "terminal": flow.terminal,
            "proxy_connection_id": proxy_connection_id,
            "proxy_join": if proxy_connection_id.is_some() { "exact-helper-socket" } else if matching_proxy.is_some_and(|matches| matches.len() > 1) { "ambiguous-helper-socket" } else { "unavailable" },
            "proxy_transfer": proxy_transfer.map(|record| json!({"client_to_upstream_bytes":record["fields"]["detail"]["client_to_upstream_bytes"], "upstream_to_client_bytes":record["fields"]["detail"]["upstream_to_client_bytes"]})),
            "proxy_terminal": proxy_terminal.and_then(|record| record["fields"]["detail"]["kind"].as_str()).map(|kind| if kind == "ConnectionTerminal(Complete)" { "complete" } else { "other" }),
            "final_client_generation_proven": generation_join == Some("exact") && packet.is_some_and(|record| record["role"] == "client" && record["process_id"].as_u64() == flow.pid && record["attribution"] == "direct-route-receipt"),
            "process_generation_join": generation_join.unwrap_or("unavailable"),
            "packet_correlation": packet.map(|record| json!({"state":record["correlation_state"],"reason":record["correlation_reason"],"flow_id":record["flow_id"],"observations":record["packet_observations"],"unretained":record["packet_observations_unretained"]})),
        })
    }).collect();
    let route_state = route
        .as_ref()
        .map(|prefix| stream_status(prefix.status))
        .unwrap_or(route_source_state);
    let route_complete = route.as_ref().is_some_and(|prefix| prefix.complete());
    let route_loss = route.as_ref().and_then(|prefix| prefix.dropped_events);
    let proxy_state = proxy
        .as_ref()
        .map(|prefix| stream_status(prefix.status))
        .unwrap_or(proxy_source_state);
    let process_trace_state = process_trace
        .as_ref()
        .map(|source| source.state)
        .unwrap_or(process_trace_source_state);
    let selected_flows = flow_rows.len();
    let bidirectional_flows = flow_rows
        .iter()
        .filter(|flow| flow["status"] == "bidirectional-transfer-observed")
        .count();
    let client_reachable_flows = flow_rows
        .iter()
        .filter(|flow| flow["client_reachability_observed"] == true)
        .count();
    let packet_joined_flows = flow_rows
        .iter()
        .filter(|flow| !flow["packet_correlation"].is_null())
        .count();
    let zero_byte_connected_flows = flow_rows
        .iter()
        .filter(|flow| {
            flow["proxy_connected"] == true
                && flow["client_to_upstream_bytes"] == 0
                && flow["upstream_to_client_bytes"] == 0
        })
        .count();
    Ok(json!({
        "schema_version": 1,
        "kind": "fragcap-direct-route-debug-report",
        "source": {
            "manifest_source_state": manifest_source_state,
            "manifest_state": manifest.as_ref().and_then(|value| value["state"].as_str()),
            "manifest_version": manifest.as_ref().and_then(|value| value["manifest_version"].as_u64()),
            "product_version": manifest.as_ref().and_then(|value| value["product"]["version"].as_str()),
            "proxy_version": manifest.as_ref().and_then(|value| value["proxy"]["version"].as_str()),
            "driver_version": manifest.as_ref().and_then(|value| value["proxy"]["routing_policy"]["direct_socket"]["component_version"].as_str()),
            "route_identity": manifest.as_ref().and_then(|value| value["proxy"]["route_identity"].as_str()),
            "inspection_scope": manifest.as_ref().and_then(|value| value["proxy"]["inspection"]["effective_scope"].as_str()),
            "launch_case": manifest.as_ref().and_then(|value| value["launch"]["case"].as_str()),
            "failure_stages": manifest.as_ref().and_then(|value| value["failures"].as_array()).map(|failures| failures.iter().map(|failure| json!({"stage":failure["stage"],"code":failure["code"]})).collect::<Vec<_>>()),
        },
        "route": {"state":route_state,"complete":route_complete,"dropped_events":route_loss,"events":route.as_ref().map_or(0, |prefix| prefix.events.len()),"selected_flows":selected_flows,"client_reachable_flows":client_reachable_flows,"bidirectional_flows":bidirectional_flows,"zero_byte_connected_flows":zero_byte_connected_flows},
        "proxy": {"state":proxy_state,"events":proxy.as_ref().map_or(0, |prefix| prefix.records.len())},
        "process_trace": {"state":process_trace_state,"completeness":process_trace.as_ref().and_then(|source| source.completeness),"events":process_trace.as_ref().map_or(0, |source| source.events)},
        "packet": {"state":if bundle.join("capture.fcapng").is_file() { "artifact-present" } else { "artifact-unavailable" }, "original_flow_reconciliation":if packet_joined_flows > 0 { "joined-by-proxy-connection" } else { "unavailable" }, "joined_flows":packet_joined_flows},
        "compatibility": {"state":compatibility_source_state,"available":compatibility.is_some(),"calibration_phase":compatibility.as_ref().and_then(|value| value["calibration"]["phase"].as_str()),"calibration_outcome":compatibility.as_ref().and_then(|value| value["calibration"]["outcome"].as_str()),"process_trace":compatibility.as_ref().map(|value| json!({"completeness":value["process_trace"]["completeness"],"unresolved_flow_owners":value["process_trace"]["unresolved_flow_owners"]}))},
        "cleanup": {"state":cleanup_source_state,"available":cleanup.is_some(),"status":manifest.as_ref().and_then(|value| value["cleanup"]["status"].as_str())},
        "flows": flow_rows,
        "transformations": [
            {"source":"route.selected.local,remote,pid,process_instance_id,endpoint_id","operation":"replace-with-session-ordinal","output":"flows[].flow"},
            {"source":"route.proxy-connected.helper_local,proxy_local","operation":"exact-join-then-omit-endpoints","output":"flows[].proxy_connection_id"},
            {"source":"manifest paths, target identity, capability, certificate material, payloads","operation":"omit","output":null}
        ],
        "limitations": ["A WinDivert send success alone does not prove delivery to the client.", "Packet artifact presence does not prove an original diverted packet was retained.", "A crash prefix or lost route event may omit a later outcome."],
    }))
}
