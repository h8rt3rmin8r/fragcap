// SPDX-License-Identifier: Apache-2.0

//! Independent typed terminal populations for operator diagnosis.

use std::time::SystemTime;

pub use fragcap_proxy::{
    ConnectionCauseCounts as ProxyCauseCounts, ConnectionFailureCategory as ProxyFailureCategory,
};

/// Eligibility window of one retained observation. Retention is independent.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceWindow {
    PreClient,
    ProvisionalHandoff,
    Observation,
    OwnerRelease,
    Unavailable,
}

impl EvidenceWindow {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PreClient => "pre-client",
            Self::ProvisionalHandoff => "provisional-handoff",
            Self::Observation => "observation",
            Self::OwnerRelease => "owner-release",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Exact lifecycle cutoffs, independent from authorized maximum durations.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EvidenceWindows {
    pub observation_ended_at: Option<SystemTime>,
    pub owner_release_ended_at: Option<SystemTime>,
}

/// Additive timing qualification of an unchanged version-one observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhaseQualifiedObservation {
    pub observation: super::CompatibilityObservation,
    pub evidence_window: EvidenceWindow,
}

/// Optional phase-qualified terminal collection, preserving the legacy drain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhaseObservationDrain {
    observations: Vec<PhaseQualifiedObservation>,
    status: super::ObservationDrainStatus,
}

impl PhaseObservationDrain {
    pub fn complete(observations: Vec<PhaseQualifiedObservation>) -> Self {
        Self {
            observations,
            status: super::ObservationDrainStatus::Complete,
        }
    }
    pub fn incomplete(
        observations: Vec<PhaseQualifiedObservation>,
        code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            observations,
            status: super::ObservationDrainStatus::Incomplete {
                code: code.into(),
                detail: detail.into(),
            },
        }
    }
    pub fn into_parts(
        self,
    ) -> (
        Vec<PhaseQualifiedObservation>,
        super::ObservationDrainStatus,
    ) {
        (self.observations, self.status)
    }
    pub(crate) fn into_raw(self) -> super::ObservationDrain {
        let raw = self
            .observations
            .into_iter()
            .map(|value| value.observation)
            .collect();
        match self.status {
            super::ObservationDrainStatus::Complete => super::ObservationDrain::complete(raw),
            super::ObservationDrainStatus::Incomplete { code, detail } => {
                super::ObservationDrain::incomplete(raw, code, detail)
            }
        }
    }
}

/// Additive terminal metadata. Window indices match the retained raw snapshot.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TerminalDiagnostics {
    pub proxy: Option<ProxyDiagnostics>,
    pub evidence_windows: EvidenceWindows,
    pub observation_windows: Vec<EvidenceWindow>,
}

/// A bounded terminal connection record containing only a stable code and identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProxyStageRecord {
    pub stage: String,
    pub reached_ms: u64,
    pub completed_ms: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProxyConnectionDiagnostic {
    pub schema_version: u32,
    pub connection_id: u64,
    pub terminal: String,
    pub cause: Option<ProxyFailureCategory>,
    pub code: Option<String>,
    pub stages: Vec<ProxyStageRecord>,
    pub last_reached: String,
    pub last_completed: String,
    pub boundary: Option<String>,
    pub tls_alert: Option<String>,
    pub transport_category: Option<String>,
    pub os_error_code: Option<i32>,
    pub header_timeout_phase: Option<String>,
    pub completed_exchanges: u64,
}

/// Native aggregate counters and independently bounded per-connection evidence.
/// Authentication refusals are a subset of completed connection tasks, not success.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProxyDiagnostics {
    pub accepted_connections: u64,
    pub authenticated_connections: u64,
    pub authentication_refused: u64,
    pub saturated_connections: u64,
    pub completed_connections: u64,
    pub failed_connections: u64,
    pub forced_connections: u64,
    pub live_connections: u64,
    pub incomplete_connections: u64,
    pub http1_exchanges_completed: u64,
    pub http2_streams_completed: u64,
    pub http3_streams_completed: u64,
    pub response_heads: u64,
    pub causes: ProxyCauseCounts,
    pub connections: Vec<ProxyConnectionDiagnostic>,
    pub connection_details_lost: u64,
    pub connection_details_unavailable: u64,
    pub failure_details_lost: u64,
    pub observations_lost: u64,
}

impl ProxyDiagnostics {
    pub(crate) fn from_runtime(value: &fragcap_proxy::RuntimeObservation) -> Self {
        let accounted = value
            .completed_connections
            .saturating_add(value.failed_connections)
            .saturating_add(value.forced_connections)
            .saturating_add(value.live_connections as u64);
        Self {
            accepted_connections: value.accepted_connections,
            authenticated_connections: value.authenticated_connections,
            authentication_refused: value.authentication_refused,
            saturated_connections: value.saturated_connections,
            completed_connections: value.completed_connections,
            failed_connections: value.failed_connections,
            forced_connections: value.forced_connections,
            live_connections: value.live_connections as u64,
            incomplete_connections: value.accepted_connections.saturating_sub(accounted),
            http1_exchanges_completed: value.protocol.http1_exchanges_completed,
            http2_streams_completed: value.protocol.http2_streams_completed,
            http3_streams_completed: value.protocol.http3_streams_completed,
            response_heads: value.protocol.responses,
            causes: value.connection_causes,
            connections: value
                .connection_diagnostics
                .iter()
                .map(|record| ProxyConnectionDiagnostic {
                    schema_version: record.schema_version,
                    connection_id: record.connection_id,
                    terminal: record.terminal.to_string(),
                    cause: record.cause,
                    code: record.code.map(str::to_string),
                    stages: record
                        .stages
                        .iter()
                        .map(|stage| ProxyStageRecord {
                            stage: stage.stage.as_str().to_string(),
                            reached_ms: stage.reached_ms,
                            completed_ms: stage.completed_ms,
                        })
                        .collect(),
                    last_reached: record.last_reached.as_str().to_string(),
                    last_completed: record.last_completed.as_str().to_string(),
                    boundary: record.boundary.map(str::to_string),
                    tls_alert: record.tls_alert.clone(),
                    transport_category: record.transport_category.map(str::to_string),
                    os_error_code: record.os_error_code,
                    header_timeout_phase: record.header_timeout_phase.map(str::to_string),
                    completed_exchanges: record.completed_exchanges,
                })
                .collect(),
            connection_details_lost: value.resources.connection_details_dropped_oldest,
            connection_details_unavailable: value.resources.connection_details_unavailable,
            failure_details_lost: value.resources.failure_details_dropped_oldest,
            observations_lost: value.protocol.observations_dropped_oldest,
        }
    }

    /// Every admitted connection is terminal, live, or explicitly unaccounted.
    pub fn connections_reconcile(&self) -> bool {
        self.accepted_connections
            == self
                .completed_connections
                .saturating_add(self.failed_connections)
                .saturating_add(self.forced_connections)
                .saturating_add(self.live_connections)
                .saturating_add(self.incomplete_connections)
    }

    /// Retained closed-connection identities plus named loss/unavailability conserve terminals.
    pub fn terminal_details_reconcile(&self) -> bool {
        (self.connections.len() as u64)
            .saturating_add(self.connection_details_lost)
            .saturating_add(self.connection_details_unavailable)
            == self
                .completed_connections
                .saturating_add(self.failed_connections)
                .saturating_add(self.forced_connections)
    }

    /// A terminal failure/refusal/forced connection contributes to exactly one category.
    pub fn causes_reconcile(&self) -> bool {
        self.causes
            .authentication
            .saturating_add(self.causes.protocol)
            .saturating_add(self.causes.transport)
            .saturating_add(self.causes.upstream)
            .saturating_add(self.causes.timeout)
            .saturating_add(self.causes.cancelled)
            .saturating_add(self.causes.unavailable)
            == self
                .failed_connections
                .saturating_add(self.authentication_refused)
                .saturating_add(self.forced_connections)
    }
}
