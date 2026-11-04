// SPDX-License-Identifier: Apache-2.0

//! Immutable pre-inspection decisions supplied by the session owner.

use std::net::SocketAddr;
use std::time::Duration;

/// The effective, pre-effect inspection scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InspectionScope {
    LaunchChain,
    ExactGame,
}

impl InspectionScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LaunchChain => "launch-chain",
            Self::ExactGame => "exact-game",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InspectionAction {
    Inspect,
    Uninspected,
}

impl InspectionAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inspect => "inspect",
            Self::Uninspected => "uninspected",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InspectionReason {
    LaunchChain,
    VerifiedGame,
    KnownMiddleware,
    PendingTimeout,
    TelemetryUnavailable,
    QueueSaturated,
    StaleSocket,
    ChangedGeneration,
    AmbiguousOwner,
    UnprovenRole,
}

impl InspectionReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LaunchChain => "launch-chain",
            Self::VerifiedGame => "verified-game",
            Self::KnownMiddleware => "known-middleware",
            Self::PendingTimeout => "pending-timeout",
            Self::TelemetryUnavailable => "telemetry-unavailable",
            Self::QueueSaturated => "queue-saturated",
            Self::StaleSocket => "stale-socket",
            Self::ChangedGeneration => "changed-generation",
            Self::AmbiguousOwner => "ambiguous-owner",
            Self::UnprovenRole => "unproven-role",
        }
    }
}

/// One connection's frozen decision. No later report may change its action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InspectionDecision {
    pub requested_scope: &'static str,
    pub scope: InspectionScope,
    pub action: InspectionAction,
    pub reason: InspectionReason,
    pub decided_after_ms: u64,
    pub policy_id: &'static str,
    pub owner_proof: &'static str,
}

impl InspectionDecision {
    pub const fn launch_chain() -> Self {
        Self {
            requested_scope: "launch-chain",
            scope: InspectionScope::LaunchChain,
            action: InspectionAction::Inspect,
            reason: InspectionReason::LaunchChain,
            decided_after_ms: 0,
            policy_id: "launch-chain-v1",
            owner_proof: "explicit-launch-chain",
        }
    }

    pub const fn uninspected(reason: InspectionReason, decided_after_ms: u64) -> Self {
        Self {
            requested_scope: "exact-game",
            scope: InspectionScope::ExactGame,
            action: InspectionAction::Uninspected,
            reason,
            decided_after_ms,
            policy_id: "exact-game-v1",
            owner_proof: "unproven",
        }
    }

    pub const fn verified_game(decided_after_ms: u64) -> Self {
        Self {
            requested_scope: "exact-game",
            scope: InspectionScope::ExactGame,
            action: InspectionAction::Inspect,
            reason: InspectionReason::VerifiedGame,
            decided_after_ms,
            policy_id: "exact-game-v1",
            owner_proof: "client-socket-and-managed-process-instance",
        }
    }

    pub const fn with_requested_scope(mut self, requested_scope: &'static str) -> Self {
        self.requested_scope = requested_scope;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InspectionRequest {
    pub connection_id: u64,
    pub client_peer: SocketAddr,
    pub proxy_local: SocketAddr,
    pub opened_at_ns: u64,
    pub transport: &'static str,
}

/// A platform-neutral source of finite, exact owner decisions.
pub trait InspectionDecider: Send + Sync {
    fn requested_scope(&self) -> &'static str {
        "exact-game"
    }
    fn decide(&self, request: InspectionRequest, budget: Duration) -> InspectionDecision;
}
