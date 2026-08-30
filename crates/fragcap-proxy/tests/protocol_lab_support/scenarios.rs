// SPDX-License-Identifier: Apache-2.0

use super::{
    CaseKind, FixtureFidelity, OutputExpectation, ProtocolFamily, Scenario, CASES, PROTOCOLS,
};

const SYNTHETIC_PAYLOAD: &[u8] = b"fragcap.test/s103/synthetic-payload";

pub fn matrix() -> Vec<Scenario> {
    PROTOCOLS
        .into_iter()
        .flat_map(|protocol| {
            CASES.into_iter().map(move |case| Scenario {
                protocol,
                case,
                payload: SYNTHETIC_PAYLOAD,
                fidelity: if case == CaseKind::Positive {
                    FixtureFidelity::RealLoopback
                } else {
                    FixtureFidelity::DeterministicModel
                },
                packet_truth: OutputExpectation::Unavailable,
                raw_observations: OutputExpectation::Available,
                projection: OutputExpectation::Unavailable,
                key_log: OutputExpectation::Unavailable,
                cleanup: if case == CaseKind::CleanupFailure {
                    OutputExpectation::ExplicitFailure
                } else {
                    OutputExpectation::Available
                },
            })
        })
        .collect()
}

pub fn outcome_code(case: CaseKind) -> &'static str {
    match case {
        CaseKind::Positive => "completed",
        CaseKind::Refusal => "policy-refused",
        CaseKind::Malformed => "malformed-input",
        CaseKind::Timeout => "operation-timeout",
        CaseKind::Cancellation => "cancelled",
        CaseKind::Disconnect => "peer-disconnected",
        CaseKind::CleanupFailure => "cleanup-failed",
    }
}

pub fn fixture_is_synthetic(payload: &[u8]) -> bool {
    payload.starts_with(b"fragcap.test/")
        && !payload.windows(5).any(|value| value == b"BEGIN")
        && !payload.windows(13).any(|value| value == b"Authorization")
}

#[allow(dead_code)]
fn _protocol_exhaustive(protocol: ProtocolFamily) -> ProtocolFamily {
    protocol
}
