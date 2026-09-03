// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use fragcap::deep_capture::{
    ClassificationReason, ClassificationSummary, DetectionState, InspectabilityState,
    ProtocolClassification, TrafficFamily, CLASSIFICATION_SCHEMA_VERSION,
};

#[test]
fn every_published_family_has_one_identified_full_or_metadata_cell() {
    let full = [
        TrafficFamily::Http1,
        TrafficFamily::Https,
        TrafficFamily::Http2,
        TrafficFamily::WebSocket,
        TrafficFamily::Sse,
        TrafficFamily::Grpc,
        TrafficFamily::Http3,
    ];
    let metadata = [
        TrafficFamily::GenericTcp,
        TrafficFamily::NonHttpTls,
        TrafficFamily::Socks5Tcp,
        TrafficFamily::Socks5Udp,
        TrafficFamily::GenericUdp,
        TrafficFamily::Quic,
    ];

    for family in full {
        let value = ProtocolClassification::new(
            family,
            DetectionState::Identified,
            InspectabilityState::Full,
            None,
        )
        .expect("published semantic family is valid");
        assert_eq!(value.schema_version(), CLASSIFICATION_SCHEMA_VERSION);
    }
    for family in metadata {
        ProtocolClassification::new(
            family,
            DetectionState::Identified,
            InspectabilityState::MetadataOnly,
            None,
        )
        .expect("published transport family is valid");
    }
}

#[test]
fn unknown_unsupported_and_failed_are_pairwise_distinct() {
    let unknown = ProtocolClassification::new(
        TrafficFamily::Unknown,
        DetectionState::Unknown,
        InspectabilityState::Unavailable,
        Some(ClassificationReason::NotReached),
    )
    .unwrap();
    let unsupported = ProtocolClassification::new(
        TrafficFamily::Quic,
        DetectionState::Unsupported,
        InspectabilityState::Unavailable,
        Some(ClassificationReason::UnsupportedVersion),
    )
    .unwrap();
    let failed = ProtocolClassification::new(
        TrafficFamily::Http1,
        DetectionState::Failed,
        InspectabilityState::MetadataOnly,
        Some(ClassificationReason::ParserFailed),
    )
    .unwrap();

    assert_ne!(unknown, unsupported);
    assert_ne!(unknown, failed);
    assert_ne!(unsupported, failed);
    assert_eq!(unknown.detection().as_str(), "unknown");
    assert_eq!(unsupported.detection().as_str(), "unsupported");
    assert_eq!(failed.detection().as_str(), "failed");
}

#[test]
fn invalid_combinations_are_refused_instead_of_normalized() {
    assert!(ProtocolClassification::new(
        TrafficFamily::Unknown,
        DetectionState::Unknown,
        InspectabilityState::Full,
        None,
    )
    .is_err());
    assert!(ProtocolClassification::new(
        TrafficFamily::Http1,
        DetectionState::Failed,
        InspectabilityState::Full,
        Some(ClassificationReason::ParserFailed),
    )
    .is_err());
    assert!(ProtocolClassification::new(
        TrafficFamily::Unrouted,
        DetectionState::Unknown,
        InspectabilityState::PacketOnly,
        None,
    )
    .is_err());
    for (reason, inspectability) in [
        (ClassificationReason::NotRouted, InspectabilityState::Full),
        (ClassificationReason::NotReached, InspectabilityState::Full),
        (
            ClassificationReason::EncryptedOpaque,
            InspectabilityState::Full,
        ),
        (
            ClassificationReason::CertificatePinned,
            InspectabilityState::Full,
        ),
        (
            ClassificationReason::ClientAuthRequired,
            InspectabilityState::Full,
        ),
        (
            ClassificationReason::UnsupportedVersion,
            InspectabilityState::Full,
        ),
        (
            ClassificationReason::ParserFailed,
            InspectabilityState::Full,
        ),
        (
            ClassificationReason::Truncated,
            InspectabilityState::Unavailable,
        ),
        (
            ClassificationReason::WriterFailed,
            InspectabilityState::Unavailable,
        ),
    ] {
        assert!(ProtocolClassification::new(
            TrafficFamily::Http1,
            DetectionState::Identified,
            inspectability,
            Some(reason),
        )
        .is_err());
    }
}

#[test]
fn summaries_conserve_each_classification_axis() {
    let values = [
        ProtocolClassification::new(
            TrafficFamily::Http2,
            DetectionState::Identified,
            InspectabilityState::Full,
            None,
        )
        .unwrap(),
        ProtocolClassification::new(
            TrafficFamily::NonHttpTls,
            DetectionState::Identified,
            InspectabilityState::EncryptedOpaque,
            Some(ClassificationReason::EncryptedOpaque),
        )
        .unwrap(),
        ProtocolClassification::new(
            TrafficFamily::Unrouted,
            DetectionState::Unknown,
            InspectabilityState::PacketOnly,
            Some(ClassificationReason::NotRouted),
        )
        .unwrap(),
    ];
    let summary = ClassificationSummary::from_classifications(values.iter(), 2);

    assert_eq!(summary.observations, 3);
    assert_eq!(summary.unclassified_lost, 2);
    assert_eq!(summary.detection_total(), 3);
    assert_eq!(summary.inspectability_total(), 3);
    assert_eq!(summary.by_reason.get("not-routed"), Some(&1));
}

#[test]
fn native_reason_mapping_preserves_required_distinctions() {
    let cases = [
        ("unrouted", "packet-only", Some("not-routed"), "not-routed"),
        (
            "unknown",
            "unknown",
            Some("proxy-not-reached"),
            "not-reached",
        ),
        ("tls", "opaque", None, "encrypted-opaque"),
        (
            "https",
            "unknown",
            Some("certificate-pinned"),
            "certificate-pinned",
        ),
        (
            "https",
            "unknown",
            Some("client-certificate-required"),
            "client-auth-required",
        ),
        (
            "quic",
            "unknown",
            Some("quic-alpn-unsupported"),
            "unsupported-version",
        ),
        ("socks5", "metadata-only", Some("udp-association"), "none"),
        (
            "http",
            "metadata-only",
            Some("http-protocol-failed"),
            "parser-failed",
        ),
        (
            "http",
            "metadata-only",
            Some("http-response-head-invalid"),
            "parser-failed",
        ),
        (
            "http",
            "metadata-only",
            Some("http-response-head-incomplete"),
            "parser-failed",
        ),
        ("http", "full", Some("retention-limit"), "truncated"),
        (
            "http",
            "full",
            Some("application-writer-failed"),
            "writer-failed",
        ),
    ];

    for (protocol, inspectability, raw_reason, expected) in cases {
        let value =
            ProtocolClassification::from_proxy_evidence(protocol, inspectability, raw_reason);
        if expected == "none" {
            assert_eq!(value.family(), TrafficFamily::Socks5Udp);
            assert_eq!(value.reason(), None);
        } else {
            assert_eq!(
                value.reason().map(ClassificationReason::as_str),
                Some(expected)
            );
        }
    }
}

#[test]
fn published_schema_contains_the_complete_vocabulary() {
    let schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/schema/deep-capture-classification.v1.json"
    ))
    .unwrap();
    let values = |axis: &str| {
        schema["properties"][axis]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        values("family"),
        [
            "http1",
            "https",
            "http2",
            "websocket",
            "sse",
            "grpc",
            "generic-tcp",
            "non-http-tls",
            "socks5-tcp",
            "socks5-udp",
            "generic-udp",
            "quic",
            "http3",
            "unrouted",
            "unknown",
        ]
    );
    assert_eq!(
        values("detection"),
        ["identified", "unknown", "unsupported", "failed"]
    );
    assert_eq!(
        values("inspectability"),
        [
            "full",
            "metadata-only",
            "decrypted-unknown",
            "encrypted-opaque",
            "packet-only",
            "unavailable",
        ]
    );
}
