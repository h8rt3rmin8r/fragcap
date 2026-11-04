// SPDX-License-Identifier: Apache-2.0

use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fragcap_proxy::{
    InspectionAction, InspectionDecider, InspectionDecision, InspectionReason, InspectionRequest,
    NativeProxyBackend, NativeProxyConfig,
};

#[derive(Default)]
struct RefuseInspection {
    seen: Mutex<Vec<InspectionRequest>>,
}

impl InspectionDecider for RefuseInspection {
    fn decide(&self, request: InspectionRequest, _budget: Duration) -> InspectionDecision {
        self.seen.lock().unwrap().push(request);
        InspectionDecision::uninspected(InspectionReason::KnownMiddleware, 1)
    }
}

#[test]
fn native_owner_decision_precedes_all_protocol_and_tls_work() {
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        2,
        4096,
        Duration::from_secs(1),
    )
    .unwrap();
    let decider = Arc::new(RefuseInspection::default());
    let mut lease = NativeProxyBackend::new(config)
        .with_inspection_decider(decider.clone())
        .start(Duration::from_secs(1))
        .unwrap();
    let endpoint = lease.endpoint();
    let client = TcpStream::connect(endpoint).unwrap();
    drop(client);
    std::thread::sleep(Duration::from_millis(50));
    let report = lease.cleanup(Duration::from_secs(1));
    assert!(report.is_clean(), "{report:?}");
    let seen = decider.seen.lock().unwrap();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].proxy_local, endpoint);
    let diagnostic = &report.observation.connection_diagnostics[0];
    let decision = diagnostic.inspection.unwrap();
    assert_eq!(decision.action, InspectionAction::Uninspected);
    assert_eq!(decision.reason, InspectionReason::KnownMiddleware);
    assert_eq!(decision.requested_scope, "exact-game");
    assert_eq!(decision.policy_id, "exact-game-v1");
    assert_eq!(decision.owner_proof, "unproven");
    assert!(diagnostic.stages.iter().all(|stage| {
        !matches!(
            stage.stage,
            fragcap_proxy::ConnectionStage::ClientTls
                | fragcap_proxy::ConnectionStage::UpstreamTls
                | fragcap_proxy::ConnectionStage::FirstRequest
        )
    }));
}
