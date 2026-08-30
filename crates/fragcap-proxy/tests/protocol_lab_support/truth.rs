// SPDX-License-Identifier: Apache-2.0

use super::{outcome_code, CaseKind, ProtocolFamily, Scenario};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TruthLedger {
    pub protocol: ProtocolFamily,
    pub case: CaseKind,
    pub attempts: u64,
    pub terminal: &'static str,
    pub owned_tasks: u64,
}

impl TruthLedger {
    pub fn from_scenario(scenario: &Scenario) -> Self {
        Self {
            protocol: scenario.protocol,
            case: scenario.case,
            attempts: 1,
            terminal: outcome_code(scenario.case),
            owned_tasks: 0,
        }
    }

    pub fn conserves(&self) -> bool {
        self.attempts == 1 && !self.terminal.is_empty() && self.owned_tasks == 0
    }
}
