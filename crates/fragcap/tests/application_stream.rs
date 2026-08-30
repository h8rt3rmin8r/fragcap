// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use std::io::Write;

use bytes::Bytes;
use fragcap::deep_capture::{
    read_application_prefix, ApplicationArtifactLease, ApplicationStreamStatus,
};
use fragcap_proxy::{
    ApplicationEvent, ApplicationEventKind, BodyDirection, BodyOutcome, BodyRepresentation,
    BodySegment, MetadataBlock, MetadataField, MetadataKind, ProtocolVersion,
};

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
    std::thread::sleep(std::time::Duration::from_millis(20));
    let live = read_application_prefix(&path).unwrap();
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
                observed_len: 64,
                bytes: Bytes::new(),
                outcome: BodyOutcome::IntentionallyOmitted,
            }),
        ));
        if disposition == fragcap_proxy::EventDisposition::QueueFull {
            queue_full += 1;
        }
    }
    assert!(queue_full > 0);
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
    for body in complete
        .records
        .iter()
        .filter(|record| record["type"] == "http.body_segment")
    {
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
