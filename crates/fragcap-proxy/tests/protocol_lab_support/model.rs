// SPDX-License-Identifier: Apache-2.0

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProtocolFamily {
    Http1,
    Https,
    Http2,
    StreamingHttp,
    WebSocket,
    Grpc,
    RawTcp,
    NonHttpTls,
    Socks,
    Udp,
    Quic,
}

pub const PROTOCOLS: [ProtocolFamily; 11] = [
    ProtocolFamily::Http1,
    ProtocolFamily::Https,
    ProtocolFamily::Http2,
    ProtocolFamily::StreamingHttp,
    ProtocolFamily::WebSocket,
    ProtocolFamily::Grpc,
    ProtocolFamily::RawTcp,
    ProtocolFamily::NonHttpTls,
    ProtocolFamily::Socks,
    ProtocolFamily::Udp,
    ProtocolFamily::Quic,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaseKind {
    Positive,
    Refusal,
    Malformed,
    Timeout,
    Cancellation,
    Disconnect,
    CleanupFailure,
}

pub const CASES: [CaseKind; 7] = [
    CaseKind::Positive,
    CaseKind::Refusal,
    CaseKind::Malformed,
    CaseKind::Timeout,
    CaseKind::Cancellation,
    CaseKind::Disconnect,
    CaseKind::CleanupFailure,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixtureFidelity {
    RealLoopback,
    DeterministicModel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputExpectation {
    Available,
    Unavailable,
    ExplicitFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Scenario {
    pub protocol: ProtocolFamily,
    pub case: CaseKind,
    pub payload: &'static [u8],
    pub fidelity: FixtureFidelity,
    pub packet_truth: OutputExpectation,
    pub raw_observations: OutputExpectation,
    pub projection: OutputExpectation,
    pub key_log: OutputExpectation,
    pub cleanup: OutputExpectation,
}
