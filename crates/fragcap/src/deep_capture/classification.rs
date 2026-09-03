// SPDX-License-Identifier: Apache-2.0

//! Stable classification of native Deep Capture protocol evidence.

use std::collections::BTreeMap;
use std::fmt;

/// First published classification vocabulary.
pub const CLASSIFICATION_SCHEMA_VERSION: u32 = 1;

/// Published traffic family identified from retained evidence.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TrafficFamily {
    Http1,
    Https,
    Http2,
    WebSocket,
    Sse,
    Grpc,
    GenericTcp,
    NonHttpTls,
    Socks5Tcp,
    Socks5Udp,
    GenericUdp,
    Quic,
    Http3,
    Unrouted,
    Unknown,
}

impl TrafficFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Http1 => "http1",
            Self::Https => "https",
            Self::Http2 => "http2",
            Self::WebSocket => "websocket",
            Self::Sse => "sse",
            Self::Grpc => "grpc",
            Self::GenericTcp => "generic-tcp",
            Self::NonHttpTls => "non-http-tls",
            Self::Socks5Tcp => "socks5-tcp",
            Self::Socks5Udp => "socks5-udp",
            Self::GenericUdp => "generic-udp",
            Self::Quic => "quic",
            Self::Http3 => "http3",
            Self::Unrouted => "unrouted",
            Self::Unknown => "unknown",
        }
    }

    /// Map one native proxy label without guessing from port numbers.
    pub fn from_proxy_label(value: &str) -> Self {
        match value {
            "http" | "http/1.1" => Self::Http1,
            "https" | "connect" => Self::Https,
            "http2" | "h2" => Self::Http2,
            "websocket" => Self::WebSocket,
            "sse" => Self::Sse,
            "grpc" => Self::Grpc,
            "tcp" | "tcp-opaque" => Self::GenericTcp,
            "tls" | "tls-protocol-unknown" => Self::NonHttpTls,
            "socks5" | "socks5-connect" => Self::Socks5Tcp,
            "socks5-udp" => Self::Socks5Udp,
            "udp" | "generic-udp" => Self::GenericUdp,
            "quic" => Self::Quic,
            "http3" | "h3" => Self::Http3,
            "unrouted" => Self::Unrouted,
            _ => Self::Unknown,
        }
    }
}

/// Evidence state of protocol detection.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DetectionState {
    Identified,
    Unknown,
    Unsupported,
    Failed,
}

impl DetectionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Identified => "identified",
            Self::Unknown => "unknown",
            Self::Unsupported => "unsupported",
            Self::Failed => "failed",
        }
    }
}

/// Highest application evidence boundary actually observed.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum InspectabilityState {
    Full,
    MetadataOnly,
    DecryptedUnknown,
    EncryptedOpaque,
    PacketOnly,
    Unavailable,
}

impl InspectabilityState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::MetadataOnly => "metadata-only",
            Self::DecryptedUnknown => "decrypted-unknown",
            Self::EncryptedOpaque => "encrypted-opaque",
            Self::PacketOnly => "packet-only",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Stable reason category. Detailed owning records retain their raw reason.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ClassificationReason {
    NotRouted,
    NotReached,
    EncryptedOpaque,
    CertificatePinned,
    ClientAuthRequired,
    UnsupportedVersion,
    ParserFailed,
    Truncated,
    WriterFailed,
}

impl ClassificationReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotRouted => "not-routed",
            Self::NotReached => "not-reached",
            Self::EncryptedOpaque => "encrypted-opaque",
            Self::CertificatePinned => "certificate-pinned",
            Self::ClientAuthRequired => "client-auth-required",
            Self::UnsupportedVersion => "unsupported-version",
            Self::ParserFailed => "parser-failed",
            Self::Truncated => "truncated",
            Self::WriterFailed => "writer-failed",
        }
    }

    pub fn from_raw(value: &str) -> Option<Self> {
        if value == "proxy-not-reached" || value == "not-reached" {
            Some(Self::NotReached)
        } else if value == "not-routed" || value == "packet-only-unsupported" {
            Some(Self::NotRouted)
        } else if value.contains("pinning") || value == "certificate-pinned" {
            Some(Self::CertificatePinned)
        } else if value.contains("client-certificate") || value.contains("client-auth") {
            Some(Self::ClientAuthRequired)
        } else if value.contains("unsupported-version") || value.contains("alpn-unsupported") {
            Some(Self::UnsupportedVersion)
        } else if value.contains("parse") || value.contains("protocol-failed") {
            Some(Self::ParserFailed)
        } else if value.contains("retention-limit") || value == "truncated" {
            Some(Self::Truncated)
        } else if value.contains("writer") || value.contains("storage-failed") {
            Some(Self::WriterFailed)
        } else if value.contains("opaque") || value == "encrypted-opaque" {
            Some(Self::EncryptedOpaque)
        } else {
            None
        }
    }
}

/// Invalid combination of otherwise known classification labels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidClassification {
    pub code: &'static str,
}

impl fmt::Display for InvalidClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code)
    }
}

impl std::error::Error for InvalidClassification {}

/// Validated schema version 1 classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtocolClassification {
    family: TrafficFamily,
    detection: DetectionState,
    inspectability: InspectabilityState,
    reason: Option<ClassificationReason>,
}

impl ProtocolClassification {
    pub fn new(
        family: TrafficFamily,
        detection: DetectionState,
        inspectability: InspectabilityState,
        reason: Option<ClassificationReason>,
    ) -> Result<Self, InvalidClassification> {
        let value = Self {
            family,
            detection,
            inspectability,
            reason,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn schema_version(self) -> u32 {
        CLASSIFICATION_SCHEMA_VERSION
    }

    pub fn family(self) -> TrafficFamily {
        self.family
    }

    pub fn detection(self) -> DetectionState {
        self.detection
    }

    pub fn inspectability(self) -> InspectabilityState {
        self.inspectability
    }

    pub fn reason(self) -> Option<ClassificationReason> {
        self.reason
    }

    fn validate(self) -> Result<(), InvalidClassification> {
        if self.family == TrafficFamily::Unknown && self.detection != DetectionState::Unknown {
            return Err(InvalidClassification {
                code: "unknown-family-requires-unknown-detection",
            });
        }
        if self.family == TrafficFamily::Unrouted
            && !(self.detection == DetectionState::Unknown
                && self.inspectability == InspectabilityState::PacketOnly
                && self.reason == Some(ClassificationReason::NotRouted))
        {
            return Err(InvalidClassification {
                code: "unrouted-requires-packet-only-reason",
            });
        }
        if self.inspectability == InspectabilityState::Full
            && self.detection != DetectionState::Identified
        {
            return Err(InvalidClassification {
                code: "full-requires-identified-protocol",
            });
        }
        if self.inspectability == InspectabilityState::PacketOnly
            && self.family != TrafficFamily::Unrouted
        {
            return Err(InvalidClassification {
                code: "packet-only-requires-unrouted-family",
            });
        }
        if self.detection == DetectionState::Unsupported
            && self.reason != Some(ClassificationReason::UnsupportedVersion)
        {
            return Err(InvalidClassification {
                code: "unsupported-requires-version-reason",
            });
        }
        if self.reason == Some(ClassificationReason::UnsupportedVersion)
            && self.detection != DetectionState::Unsupported
        {
            return Err(InvalidClassification {
                code: "unsupported-version-requires-unsupported-detection",
            });
        }
        if self.detection == DetectionState::Failed
            && self.reason != Some(ClassificationReason::ParserFailed)
        {
            return Err(InvalidClassification {
                code: "failed-requires-parser-reason",
            });
        }
        if self.reason == Some(ClassificationReason::ParserFailed)
            && self.detection != DetectionState::Failed
        {
            return Err(InvalidClassification {
                code: "parser-failed-requires-failed-detection",
            });
        }
        if self.detection == DetectionState::Failed
            && self.inspectability == InspectabilityState::Full
        {
            return Err(InvalidClassification {
                code: "failed-cannot-be-full",
            });
        }
        if (self.inspectability == InspectabilityState::EncryptedOpaque)
            != (self.reason == Some(ClassificationReason::EncryptedOpaque))
        {
            return Err(InvalidClassification {
                code: "encrypted-opaque-state-and-reason-must-agree",
            });
        }
        if self.reason == Some(ClassificationReason::NotRouted)
            && self.family != TrafficFamily::Unrouted
        {
            return Err(InvalidClassification {
                code: "not-routed-requires-unrouted-family",
            });
        }
        if self.reason == Some(ClassificationReason::NotReached)
            && !(self.detection == DetectionState::Unknown
                && self.inspectability == InspectabilityState::Unavailable)
        {
            return Err(InvalidClassification {
                code: "not-reached-requires-unknown-unavailable",
            });
        }
        if matches!(
            self.reason,
            Some(
                ClassificationReason::CertificatePinned | ClassificationReason::ClientAuthRequired
            )
        ) && !(self.detection == DetectionState::Identified
            && matches!(
                self.family,
                TrafficFamily::Https | TrafficFamily::NonHttpTls | TrafficFamily::Quic
            )
            && matches!(
                self.inspectability,
                InspectabilityState::MetadataOnly | InspectabilityState::Unavailable
            ))
        {
            return Err(InvalidClassification {
                code: "tls-boundary-reason-requires-identified-tls-family",
            });
        }
        if matches!(
            self.reason,
            Some(ClassificationReason::Truncated | ClassificationReason::WriterFailed)
        ) && (self.detection != DetectionState::Identified
            || matches!(
                self.inspectability,
                InspectabilityState::PacketOnly | InspectabilityState::Unavailable
            ))
        {
            return Err(InvalidClassification {
                code: "retention-reason-requires-retained-identified-evidence",
            });
        }
        Ok(())
    }

    /// Classify one raw native proxy observation while retaining its raw labels elsewhere.
    pub fn from_proxy_evidence(protocol: &str, inspectability: &str, reason: Option<&str>) -> Self {
        let family = TrafficFamily::from_proxy_label(protocol);
        let stable_reason = reason.and_then(ClassificationReason::from_raw);
        let detection = if stable_reason == Some(ClassificationReason::UnsupportedVersion) {
            DetectionState::Unsupported
        } else if stable_reason == Some(ClassificationReason::ParserFailed) {
            DetectionState::Failed
        } else if matches!(family, TrafficFamily::Unknown | TrafficFamily::Unrouted) {
            DetectionState::Unknown
        } else {
            DetectionState::Identified
        };
        let inspectability = match inspectability {
            "full" => InspectabilityState::Full,
            "metadata-only" => InspectabilityState::MetadataOnly,
            "protocol-unknown" => InspectabilityState::DecryptedUnknown,
            "opaque" | "encrypted-opaque" => InspectabilityState::EncryptedOpaque,
            "packet-only" => InspectabilityState::PacketOnly,
            _ => InspectabilityState::Unavailable,
        };
        let stable_reason = if inspectability == InspectabilityState::EncryptedOpaque {
            Some(ClassificationReason::EncryptedOpaque)
        } else {
            stable_reason
        };
        Self::new(family, detection, inspectability, stable_reason).unwrap_or(Self {
            family: TrafficFamily::Unknown,
            detection: DetectionState::Unknown,
            inspectability: InspectabilityState::Unavailable,
            reason: None,
        })
    }
}

/// Conserved projection over retained classifications.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ClassificationSummary {
    pub observations: u64,
    pub by_family: BTreeMap<&'static str, u64>,
    pub by_detection: BTreeMap<&'static str, u64>,
    pub by_inspectability: BTreeMap<&'static str, u64>,
    pub by_reason: BTreeMap<&'static str, u64>,
    pub unclassified_lost: u64,
}

impl ClassificationSummary {
    pub fn from_classifications<'a>(
        values: impl IntoIterator<Item = &'a ProtocolClassification>,
        unclassified_lost: u64,
    ) -> Self {
        let mut summary = Self {
            unclassified_lost,
            ..Self::default()
        };
        for value in values {
            summary.record(value);
        }
        summary
    }

    pub fn record(&mut self, value: &ProtocolClassification) {
        self.observations += 1;
        *self.by_family.entry(value.family().as_str()).or_default() += 1;
        *self
            .by_detection
            .entry(value.detection().as_str())
            .or_default() += 1;
        *self
            .by_inspectability
            .entry(value.inspectability().as_str())
            .or_default() += 1;
        if let Some(reason) = value.reason() {
            *self.by_reason.entry(reason.as_str()).or_default() += 1;
        }
    }

    pub fn detection_total(&self) -> u64 {
        self.by_detection.values().sum()
    }

    pub fn inspectability_total(&self) -> u64 {
        self.by_inspectability.values().sum()
    }
}
