// SPDX-License-Identifier: Apache-2.0

//! Pre-effect inspection scope and exact game-owner decisions.

pub use fragcap_proxy::{InspectionAction, InspectionDecision, InspectionReason, InspectionScope};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestedInspectionScope {
    Auto,
    ExactGame,
    LaunchChain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedInspectionScope {
    pub requested: RequestedInspectionScope,
    pub effective: InspectionScope,
    pub policy_id: &'static str,
}

impl RequestedInspectionScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::ExactGame => "exact-game",
            Self::LaunchChain => "launch-chain",
        }
    }

    /// Resolve before effects. The caller establishes whether exact game
    /// ownership and uninspected forwarding are available for this plan.
    pub const fn resolve(self, game_only_available: bool) -> Option<ResolvedInspectionScope> {
        let effective = match self {
            Self::Auto if game_only_available => InspectionScope::ExactGame,
            Self::Auto | Self::LaunchChain => InspectionScope::LaunchChain,
            Self::ExactGame if game_only_available => InspectionScope::ExactGame,
            Self::ExactGame => return None,
        };
        Some(ResolvedInspectionScope {
            requested: self,
            effective,
            policy_id: match effective {
                InspectionScope::ExactGame => "exact-game-v1",
                InspectionScope::LaunchChain => "launch-chain-v1",
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_is_frozen_before_effects_and_explicit_game_requires_relay_availability() {
        assert_eq!(
            RequestedInspectionScope::Auto
                .resolve(false)
                .unwrap()
                .effective,
            InspectionScope::LaunchChain
        );
        assert_eq!(
            RequestedInspectionScope::Auto
                .resolve(true)
                .unwrap()
                .effective,
            InspectionScope::ExactGame
        );
        assert!(RequestedInspectionScope::ExactGame.resolve(false).is_none());
        assert_eq!(
            RequestedInspectionScope::ExactGame
                .resolve(true)
                .unwrap()
                .policy_id,
            "exact-game-v1"
        );
    }
}
