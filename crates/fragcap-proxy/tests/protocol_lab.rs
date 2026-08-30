// SPDX-License-Identifier: Apache-2.0

mod protocol_lab_support;

use std::collections::BTreeSet;

use protocol_lab_support::{
    execute_scenario, fixture_is_synthetic, matrix, outcome_code, protocol_round_trip, CaseKind,
    OutputExpectation, CASES, PROTOCOLS,
};

#[tokio::test]
async fn complete_protocol_failure_matrix_is_deterministic_and_conserved() {
    let first = matrix();
    let second = matrix();
    assert_eq!(first, second);
    assert_eq!(first.len(), PROTOCOLS.len() * CASES.len());
    let covered: BTreeSet<_> = first.iter().map(|item| item.protocol).collect();
    assert_eq!(covered.len(), PROTOCOLS.len());
    for scenario in &first {
        assert!(fixture_is_synthetic(scenario.payload));
        let ledger = execute_scenario(scenario).await;
        assert!(ledger.conserves());
        assert_eq!(ledger.terminal, outcome_code(scenario.case));
        assert_eq!(scenario.packet_truth, OutputExpectation::Unavailable);
        assert_eq!(scenario.projection, OutputExpectation::Unavailable);
        assert_eq!(scenario.key_log, OutputExpectation::Unavailable);
        assert_eq!(
            scenario.cleanup == OutputExpectation::ExplicitFailure,
            scenario.case == CaseKind::CleanupFailure
        );
    }
}

#[tokio::test]
async fn positive_transport_families_use_real_bounded_loopback_endpoints() {
    let payload = b"fragcap.test/s103/transport";
    for protocol in PROTOCOLS {
        let echoed = protocol_round_trip(protocol, payload).await;
        assert_eq!(echoed, payload, "{protocol:?}");
    }
}

#[test]
fn every_family_has_every_named_terminal_case() {
    let scenarios = matrix();
    for protocol in PROTOCOLS {
        for case in CASES {
            assert!(scenarios
                .iter()
                .any(|item| item.protocol == protocol && item.case == case));
        }
    }
}
