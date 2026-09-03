// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use std::io::Write;

use bytes::Bytes;
use fragcap::deep_capture::{
    read_application_prefix, ApplicationArtifactLease, ApplicationStreamStatus,
};
use fragcap_proxy::{
    ApplicationEvent, ApplicationEventKind, BodyDirection, BodyOutcome, BodyRepresentation,
    BodySegment, GenericStreamChunk, GenericStreamDirection, GenericStreamOutcome,
    GenericStreamProvenance, GrpcMessage, MetadataBlock, MetadataField, MetadataKind,
    ProtocolVersion, StreamingEvent, StreamingOutcome,
};

#[test]
fn generic_stream_chunks_preserve_provenance_and_omission() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("generic.jsonl");
    let mut lease = ApplicationArtifactLease::open(&path, "session-116", 8).unwrap();
    let sink = lease.sink();
    for chunk in [
        GenericStreamChunk {
            direction: GenericStreamDirection::ClientToUpstream,
            provenance: GenericStreamProvenance::TlsDecrypted,
            offset: 0,
            observed_len: 3,
            bytes: Bytes::from_static(b"abc"),
            outcome: GenericStreamOutcome::Complete,
        },
        GenericStreamChunk {
            direction: GenericStreamDirection::UpstreamToClient,
            provenance: GenericStreamProvenance::TlsDecrypted,
            offset: 0,
            observed_len: 5,
            bytes: Bytes::new(),
            outcome: GenericStreamOutcome::RetentionLimit,
        },
    ] {
        assert_eq!(
            sink.try_emit(ApplicationEvent::now(
                "session-116",
                7,
                None,
                None,
                ApplicationEventKind::GenericStreamChunk(chunk),
            )),
            fragcap_proxy::EventDisposition::Accepted
        );
    }
    drop(sink);
    lease.finish().unwrap();
    let complete = read_application_prefix(&path).unwrap();
    assert_eq!(complete.status, ApplicationStreamStatus::Complete);
    let chunks = complete
        .records
        .iter()
        .filter(|record| record["type"] == "generic.stream_chunk")
        .collect::<Vec<_>>();
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0]["provenance"], "tls-decrypted");
    assert_eq!(chunks[0]["payload"], "YWJj");
    assert_eq!(chunks[1]["outcome"], "retention-limit");
    assert!(chunks[1].get("payload").is_none());
    let trailer = complete.records.last().unwrap();
    assert_eq!(trailer["generic_stream_bytes_observed"], 8);
    assert_eq!(trailer["generic_stream_bytes_retained"], 3);
    assert_eq!(trailer["generic_stream_bytes_omitted"], 5);
}

#[test]
fn generic_stream_queue_pressure_counts_observed_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("generic-pressure.jsonl");
    let mut lease = ApplicationArtifactLease::open(&path, "session-116-pressure", 1).unwrap();
    let sink = lease.sink();
    let mut queue_full = 0_u64;
    for offset in 0..10_000 {
        let disposition = sink.try_emit(ApplicationEvent::now(
            "session-116-pressure",
            9,
            None,
            None,
            ApplicationEventKind::GenericStreamChunk(GenericStreamChunk {
                direction: GenericStreamDirection::ClientToUpstream,
                provenance: GenericStreamProvenance::TcpPlaintext,
                offset: offset * 8,
                observed_len: 8,
                bytes: Bytes::from_static(b"pressure"),
                outcome: GenericStreamOutcome::Complete,
            }),
        ));
        queue_full += u64::from(disposition == fragcap_proxy::EventDisposition::QueueFull);
    }
    assert!(queue_full > 0);
    assert_eq!(
        sink.accounting().generic_stream_bytes_queue_dropped,
        queue_full * 8
    );
    drop(sink);
    lease.finish().unwrap();
    let stream = read_application_prefix(&path).unwrap();
    assert_eq!(stream.status, ApplicationStreamStatus::Complete);
    let gap = stream
        .records
        .iter()
        .find(|value| value["type"] == "application.gap")
        .unwrap();
    assert_eq!(gap["generic_stream_bytes_queue_dropped"], queue_full * 8);
    assert_eq!(
        stream.records.last().unwrap()["generic_stream_bytes_queue_dropped"],
        queue_full * 8
    );
}

#[test]
fn version_two_stream_is_live_binary_safe_and_trailer_complete() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("application.jsonl");
    let mut lease = ApplicationArtifactLease::open(&path, "session-105", 8).unwrap();
    let sink = lease.sink();
    assert_eq!(
        sink.try_emit(ApplicationEvent::now(
            "session-105",
            7,
            Some(3),
            Some(ProtocolVersion::Http2),
            ApplicationEventKind::Metadata(MetadataBlock::http2(
                MetadataKind::Request,
                vec![
                    MetadataField {
                        name: b":method".to_vec(),
                        value: b"POST".to_vec(),
                        original_index: 0,
                        sensitive: false,
                    },
                    MetadataField {
                        name: b":path".to_vec(),
                        value: b"/play?mode=a&mode=b".to_vec(),
                        original_index: 1,
                        sensitive: false,
                    },
                ],
                vec![
                    MetadataField {
                        name: b"x-binary".to_vec(),
                        value: vec![0, 0xff],
                        original_index: 2,
                        sensitive: false,
                    },
                    MetadataField {
                        name: b"cookie".to_vec(),
                        value: b"sid=one; sid=two".to_vec(),
                        original_index: 3,
                        sensitive: true,
                    },
                ],
            )),
        )),
        fragcap_proxy::EventDisposition::Accepted
    );
    assert_eq!(
        sink.try_emit(ApplicationEvent::now(
            "session-105",
            7,
            Some(3),
            Some(ProtocolVersion::Http2),
            ApplicationEventKind::Body(BodySegment {
                direction: BodyDirection::Request,
                representation: BodyRepresentation::Raw,
                offset: 0,
                observed_len: 3,
                bytes: Bytes::from_static(b"abc"),
                outcome: BodyOutcome::Complete,
            }),
        )),
        fragcap_proxy::EventDisposition::Accepted
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    let live = loop {
        let prefix = read_application_prefix(&path).unwrap();
        if prefix.records.len() >= 3 || std::time::Instant::now() >= deadline {
            break prefix;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    assert_eq!(live.schema_version, 2);
    assert_eq!(live.status, ApplicationStreamStatus::Incomplete);
    assert!(live.records.len() >= 3);
    drop(sink);
    lease.finish().unwrap();
    let complete = read_application_prefix(&path).unwrap();
    assert_eq!(complete.status, ApplicationStreamStatus::Complete);
    assert_eq!(
        complete.records.last().unwrap()["type"],
        "application.trailer"
    );
    let metadata = complete
        .records
        .iter()
        .find(|record| record["type"] == "http.metadata")
        .unwrap();
    assert_eq!(metadata["query"].as_array().unwrap().len(), 2);
    assert_eq!(metadata["cookies"].as_array().unwrap().len(), 2);
    assert_eq!(
        metadata["unavailable"],
        serde_json::json!(["hpack-wire-bytes", "compressed-cross-name-order"])
    );
}

#[test]
fn version_two_serializes_reserved_streaming_protocol_families() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("streaming.jsonl");
    let mut lease = ApplicationArtifactLease::open(&path, "session-106", 8).unwrap();
    let sink = lease.sink();
    assert_eq!(
        sink.try_emit(ApplicationEvent::now(
            "session-106",
            2,
            Some(5),
            Some(ProtocolVersion::Http2),
            ApplicationEventKind::Streaming(StreamingEvent::GrpcMessage(GrpcMessage {
                direction: BodyDirection::Response,
                sequence: 1,
                compressed: false,
                declared_len: 3,
                payload: b"abc".to_vec(),
                encoding: None,
                payload_omitted: false,
                outcome: StreamingOutcome::Complete,
            })),
        )),
        fragcap_proxy::EventDisposition::Accepted
    );
    drop(sink);
    lease.finish().unwrap();
    let stream = read_application_prefix(&path).unwrap();
    assert_eq!(stream.status, ApplicationStreamStatus::Complete);
    let header = &stream.records[0];
    assert!(header["exports"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("grpc")));
    let message = stream
        .records
        .iter()
        .find(|value| value["type"] == "grpc.message")
        .unwrap();
    assert_eq!(message["declared_len"], 3);
    assert_eq!(message["payload"], "YWJj");
    let trailer = stream.records.last().unwrap();
    assert_eq!(trailer["streaming_bytes_observed"], 3);
    assert_eq!(trailer["streaming_bytes_retained"], 3);
    assert_eq!(trailer["streaming_bytes_truncated"], 0);
    assert_eq!(trailer["streaming_records_by_outcome"]["complete"], 1);
}

#[test]
fn streaming_queue_loss_is_counted_without_blocking_the_producer() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("streaming-pressure.jsonl");
    let mut lease = ApplicationArtifactLease::open(&path, "session-stream-pressure", 1).unwrap();
    let sink = lease.sink();
    let mut queue_full = 0_u64;
    for sequence in 0..10_000 {
        let disposition = sink.try_emit(ApplicationEvent::now(
            "session-stream-pressure",
            3,
            Some(7),
            Some(ProtocolVersion::Http2),
            ApplicationEventKind::Streaming(StreamingEvent::GrpcMessage(GrpcMessage {
                direction: BodyDirection::Response,
                sequence,
                compressed: false,
                declared_len: 8,
                payload: b"retained".to_vec(),
                encoding: None,
                payload_omitted: false,
                outcome: StreamingOutcome::Complete,
            })),
        ));
        queue_full += u64::from(disposition == fragcap_proxy::EventDisposition::QueueFull);
    }
    assert!(queue_full > 0);
    assert_eq!(
        sink.accounting().streaming_bytes_queue_dropped,
        queue_full * 8
    );
    drop(sink);
    lease.finish().unwrap();
    let stream = read_application_prefix(&path).unwrap();
    assert_eq!(stream.status, ApplicationStreamStatus::Complete);
    let gap = stream
        .records
        .iter()
        .find(|value| value["type"] == "application.gap")
        .unwrap();
    assert_eq!(gap["streaming_bytes_queue_dropped"], queue_full * 8);
    assert_eq!(
        stream.records.last().unwrap()["streaming_bytes_queue_dropped"],
        queue_full * 8
    );
}

#[test]
fn streaming_scope_omission_is_explicit_and_retains_no_payload() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("streaming-omitted.jsonl");
    let mut lease = ApplicationArtifactLease::open(&path, "session-stream-omitted", 2).unwrap();
    let sink = lease.sink();
    assert_eq!(
        sink.try_emit(ApplicationEvent::now(
            "session-stream-omitted",
            3,
            Some(7),
            Some(ProtocolVersion::Http2),
            ApplicationEventKind::Streaming(StreamingEvent::GrpcMessage(GrpcMessage {
                direction: BodyDirection::Response,
                sequence: 1,
                compressed: false,
                declared_len: 8,
                payload: Vec::new(),
                encoding: None,
                payload_omitted: true,
                outcome: StreamingOutcome::IntentionallyOmitted,
            })),
        )),
        fragcap_proxy::EventDisposition::Accepted
    );
    drop(sink);
    lease.finish().unwrap();
    let stream = read_application_prefix(&path).unwrap();
    assert_eq!(stream.status, ApplicationStreamStatus::Complete);
    let message = stream
        .records
        .iter()
        .find(|value| value["type"] == "grpc.message")
        .unwrap();
    assert_eq!(message["outcome"], "intentionally-omitted");
    assert_eq!(message["declared_len"], 8);
    assert_eq!(message["retained_len"], 0);
    assert!(message.get("payload").is_none());
    let trailer = stream.records.last().unwrap();
    assert_eq!(trailer["streaming_bytes_observed"], 8);
    assert_eq!(trailer["streaming_bytes_retained"], 0);
    assert_eq!(trailer["streaming_bytes_truncated"], 8);
    assert_eq!(
        trailer["streaming_records_by_outcome"]["intentionally-omitted"],
        1
    );
}

#[test]
fn streaming_parse_failure_survives_payload_omission() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("streaming-malformed-omitted.jsonl");
    let mut lease = ApplicationArtifactLease::open(&path, "session-malformed-omitted", 2).unwrap();
    let sink = lease.sink();
    assert_eq!(
        sink.try_emit(ApplicationEvent::now(
            "session-malformed-omitted",
            4,
            Some(8),
            Some(ProtocolVersion::Http2),
            ApplicationEventKind::Streaming(StreamingEvent::GrpcMessage(GrpcMessage {
                direction: BodyDirection::Response,
                sequence: 1,
                compressed: false,
                declared_len: 3,
                payload: Vec::new(),
                encoding: None,
                payload_omitted: true,
                outcome: StreamingOutcome::Malformed,
            })),
        )),
        fragcap_proxy::EventDisposition::Accepted
    );
    drop(sink);
    lease.finish().unwrap();
    let stream = read_application_prefix(&path).unwrap();
    let message = stream
        .records
        .iter()
        .find(|value| value["type"] == "grpc.message")
        .unwrap();
    assert_eq!(message["outcome"], "malformed");
    assert_eq!(message["payload_omitted"], true);
    assert!(message.get("payload").is_none());
}

#[test]
fn reader_accepts_a_pre_s106_complete_version_two_stream() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("pre-s106.jsonl");
    let text = concat!(
        "{\"type\":\"application.header\",\"schema_version\":2,\"sequence\":0,\"session_id\":\"older\"}\n",
        "{\"type\":\"application.trailer\",\"schema_version\":2,\"sequence\":1,\"session_id\":\"older\",",
        "\"writer_status\":\"complete\",\"writer_failures\":0,\"accepted_records\":0,",
        "\"written_records\":0,\"dropped_records\":0,\"serialized_bytes\":0,",
        "\"records_by_type\":{},\"body_bytes_observed\":0,\"body_bytes_retained\":0,",
        "\"body_bytes_truncated\":0,\"body_bytes_queue_dropped\":0,",
        "\"body_retained_bytes_queue_dropped\":0}\n"
    );
    std::fs::write(&path, text).unwrap();
    assert_eq!(
        read_application_prefix(&path).unwrap().status,
        ApplicationStreamStatus::Complete
    );
}

#[test]
fn reader_distinguishes_legacy_unknown_and_torn_prefixes() {
    let temp = tempfile::tempdir().unwrap();
    let legacy = temp.path().join("legacy.jsonl");
    std::fs::write(
        &legacy,
        b"{\"type\":\"application.header\",\"manifest_version\":1}\n",
    )
    .unwrap();
    let value = read_application_prefix(&legacy).unwrap();
    assert_eq!(value.schema_version, 1);
    assert_eq!(value.status, ApplicationStreamStatus::Incomplete);

    let unknown = temp.path().join("unknown.jsonl");
    std::fs::write(
        &unknown,
        b"{\"type\":\"application.header\",\"schema_version\":99}\n",
    )
    .unwrap();
    assert_eq!(
        read_application_prefix(&unknown).unwrap().status,
        ApplicationStreamStatus::UnknownVersion
    );

    let torn = temp.path().join("torn.jsonl");
    let mut file = std::fs::File::create(&torn).unwrap();
    file.write_all(b"{\"type\":\"application.header\",\"schema_version\":2}\n{\"type\":")
        .unwrap();
    let value = read_application_prefix(&torn).unwrap();
    assert_eq!(value.records.len(), 1);
    assert_eq!(value.status, ApplicationStreamStatus::Incomplete);
}

#[test]
fn queue_pressure_is_a_gap_and_scope_omission_retains_no_payload() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("application.jsonl");
    let mut lease = ApplicationArtifactLease::open(&path, "session-pressure", 1).unwrap();
    let sink = lease.sink();
    assert_eq!(
        sink.try_emit(ApplicationEvent::now(
            "session-pressure",
            1,
            Some(1),
            Some(ProtocolVersion::Http11),
            ApplicationEventKind::Body(BodySegment {
                direction: BodyDirection::Response,
                representation: BodyRepresentation::Raw,
                offset: 0,
                observed_len: 64,
                bytes: Bytes::new(),
                outcome: BodyOutcome::IntentionallyOmitted,
            }),
        )),
        fragcap_proxy::EventDisposition::Accepted
    );
    std::thread::sleep(std::time::Duration::from_millis(20));
    let mut queue_full = 0_u64;
    for _ in 0..10_000 {
        let disposition = sink.try_emit(ApplicationEvent::now(
            "session-pressure",
            1,
            Some(1),
            Some(ProtocolVersion::Http11),
            ApplicationEventKind::Body(BodySegment {
                direction: BodyDirection::Response,
                representation: BodyRepresentation::Raw,
                offset: 0,
                observed_len: 8,
                bytes: Bytes::from_static(b"retained"),
                outcome: BodyOutcome::Complete,
            }),
        ));
        if disposition == fragcap_proxy::EventDisposition::QueueFull {
            queue_full += 1;
        }
    }
    assert!(queue_full > 0);
    let sink_accounting = sink.accounting();
    assert_eq!(sink_accounting.body_bytes_queue_dropped, queue_full * 8);
    drop(sink);
    lease.finish().unwrap();
    let complete = read_application_prefix(&path).unwrap();
    assert_eq!(complete.status, ApplicationStreamStatus::Complete);
    let gap = complete
        .records
        .iter()
        .find(|record| record["type"] == "application.gap")
        .unwrap();
    assert_eq!(gap["dropped_records"].as_u64(), Some(queue_full));
    assert_eq!(
        gap["body_bytes_queue_dropped"].as_u64(),
        Some(queue_full * 8)
    );
    assert_eq!(
        gap["body_retained_bytes_queue_dropped"].as_u64(),
        Some(queue_full * 8)
    );
    let body_loss = &gap["body_losses"][0];
    assert_eq!(body_loss["proxy_connection_id"], 1);
    assert_eq!(body_loss["http_stream_id"], 1);
    assert_eq!(body_loss["direction"], "response");
    assert_eq!(body_loss["representation"], "raw");
    assert_eq!(body_loss["outcome"], "queue-dropped");
    assert_eq!(body_loss["dropped_records"].as_u64(), Some(queue_full));
    let trailer = complete.records.last().unwrap();
    assert_eq!(
        trailer["body_bytes_queue_dropped"].as_u64(),
        Some(queue_full * 8)
    );
    let omitted: Vec<_> = complete
        .records
        .iter()
        .filter(|record| {
            record["type"] == "http.body_segment" && record["outcome"] == "intentionally-omitted"
        })
        .collect();
    assert_eq!(omitted.len(), 1);
    for body in omitted {
        assert_eq!(body["outcome"], "intentionally-omitted");
        assert!(body.get("payload").is_none());
    }
}

#[test]
fn tampered_trailer_cannot_claim_completeness() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("application.jsonl");
    let mut lease = ApplicationArtifactLease::open(&path, "session-tamper", 2).unwrap();
    lease.finish().unwrap();
    let mut text = std::fs::read_to_string(&path).unwrap();
    text = text.replace("\"written_records\":0", "\"written_records\":99");
    std::fs::write(&path, text).unwrap();
    assert_eq!(
        read_application_prefix(&path).unwrap().status,
        ApplicationStreamStatus::Incomplete
    );
}
