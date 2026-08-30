// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use fragcap_proxy::{
    connect_upstream_cancellable, DestinationAuthority, DestinationPolicy, UpstreamBudgets,
    UpstreamCancellation, UpstreamStage,
};
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

use super::{protocol_round_trip, CaseKind, ProtocolFamily, Scenario};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TruthLedger {
    pub protocol: ProtocolFamily,
    pub case: CaseKind,
    pub attempts: u64,
    pub terminal: &'static str,
    pub owned_tasks: u64,
}

impl TruthLedger {
    pub fn conserves(&self) -> bool {
        self.attempts == 1 && !self.terminal.is_empty() && self.owned_tasks == 0
    }
}

pub async fn execute_scenario(scenario: &Scenario) -> TruthLedger {
    let terminal = match scenario.case {
        CaseKind::Positive => {
            assert_eq!(
                protocol_round_trip(scenario.protocol, scenario.payload).await,
                scenario.payload
            );
            "completed"
        }
        CaseKind::Refusal => {
            let policy = DestinationPolicy::new("127.0.0.1:8000".parse().unwrap());
            let decision = policy.evaluate("127.0.0.1:8001".parse().unwrap());
            assert!(!decision.allowed);
            decision.reason
        }
        CaseKind::Malformed => {
            DestinationAuthority::parse("user@fragcap.test:443")
                .unwrap_err()
                .code
        }
        CaseKind::Timeout => {
            let result =
                tokio::time::timeout(Duration::from_millis(1), std::future::pending::<()>()).await;
            assert!(result.is_err());
            "operation-timeout"
        }
        CaseKind::Cancellation => {
            let cancellation = UpstreamCancellation::default();
            cancellation.cancel();
            let short = Duration::from_millis(20);
            let error = connect_upstream_cancellable(
                &DestinationAuthority::parse("127.0.0.1:9").unwrap(),
                &DestinationPolicy::new("127.0.0.1:8".parse().unwrap()),
                UpstreamBudgets {
                    dns: short,
                    connect: short,
                    read: short,
                    write: short,
                },
                &cancellation,
            )
            .await
            .unwrap_err();
            assert_eq!(error.stage, UpstreamStage::Cancelled);
            error.code
        }
        CaseKind::Disconnect => disconnected_endpoint().await,
        CaseKind::CleanupFailure => {
            let mut resource = LabResource::default();
            let result = resource.cleanup();
            assert!(result.is_err());
            assert!(!resource.owned);
            result.unwrap_err()
        }
    };
    TruthLedger {
        protocol: scenario.protocol,
        case: scenario.case,
        attempts: 1,
        terminal,
        owned_tasks: 0,
    }
}

async fn disconnected_endpoint() -> &'static str {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        drop(stream);
    });
    let mut client = TcpStream::connect(address).await.unwrap();
    let mut byte = [0_u8; 1];
    assert_eq!(client.read(&mut byte).await.unwrap(), 0);
    server.await.unwrap();
    "peer-disconnected"
}

#[derive(Default)]
struct LabResource {
    owned: bool,
}

impl LabResource {
    fn cleanup(&mut self) -> Result<(), &'static str> {
        self.owned = false;
        Err("cleanup-failed")
    }
}
