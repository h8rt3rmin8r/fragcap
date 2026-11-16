// SPDX-License-Identifier: Apache-2.0

//! Bounded, advisory outcomes from individual Deep Capture attempts.
//!
//! These records never authorize a route or gate a later attempt. The native
//! bundle remains the detailed evidence authority; this table retains only
//! private, scrubbed dimensions needed to compare genuinely similar runs.

use serde_json::{json, Value};

use crate::compatibility::{
    CompatibilityInspectionScope, CompatibilityLaunchCase, CompatibilityRequestedInspectionScope,
    CompatibilityRouteIdentity,
};
use crate::TargetsError;

pub const HISTORY_LIMIT: usize = 100;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureHistory {
    pub id: Option<i64>,
    pub target_id: i64,
    pub recorded_at: i64,
    pub session_id: Option<String>,
    pub fragcap_version: String,
    pub target_build_clue: Option<String>,
    pub selected_launch: String,
    pub observed_launch: String,
    pub selected_route: String,
    pub observed_route: String,
    pub requested_scope: String,
    pub effective_scope: String,
    pub process_proof: String,
    pub transfer: String,
    pub inspection: String,
    pub failure: Option<String>,
    pub loss: String,
    pub cleanup: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HistoryAppend {
    pub id: i64,
    pub pruned: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HistoryComparison {
    Incomparable(&'static str),
    Comparable {
        build_unverified: bool,
        changes: Vec<&'static str>,
    },
}

impl CaptureHistory {
    pub fn new(
        target_id: i64,
        recorded_at: i64,
        fragcap_version: &str,
        selected_launch: &str,
        selected_route: &str,
        requested_scope: &str,
        effective_scope: &str,
    ) -> Self {
        Self {
            id: None,
            target_id,
            recorded_at,
            session_id: None,
            fragcap_version: fragcap_version.into(),
            target_build_clue: None,
            selected_launch: selected_launch.into(),
            observed_launch: "unknown".into(),
            selected_route: selected_route.into(),
            observed_route: "unknown".into(),
            requested_scope: requested_scope.into(),
            effective_scope: effective_scope.into(),
            process_proof: "unknown".into(),
            transfer: "unknown".into(),
            inspection: "unknown".into(),
            failure: None,
            loss: "unknown".into(),
            cleanup: "unknown".into(),
        }
    }

    pub fn validate(&self) -> Result<(), TargetsError> {
        if self.target_id <= 0 || self.recorded_at < 0 {
            return Err(model(
                "history target and time must be nonnegative and present",
            ));
        }
        limited_token(&self.fragcap_version, 40, "fragcap version")?;
        if let Some(value) = &self.session_id {
            limited_token(value, 128, "session id")?;
        }
        if let Some(value) = &self.target_build_clue {
            if !value.starts_with("blake3:")
                || value.len() != 71
                || !value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(model("history target build clue must be a BLAKE3 digest"));
            }
        }
        CompatibilityLaunchCase::parse(&self.selected_launch)?;
        if !matches!(
            self.observed_launch.as_str(),
            "unknown" | "not-attempted" | "failed"
        ) {
            CompatibilityLaunchCase::parse(&self.observed_launch)?;
        }
        CompatibilityRouteIdentity::parse(&self.selected_route)?;
        if !matches!(self.observed_route.as_str(), "unknown" | "unavailable") {
            CompatibilityRouteIdentity::parse(&self.observed_route)?;
        }
        CompatibilityRequestedInspectionScope::parse(&self.requested_scope)?;
        CompatibilityInspectionScope::parse(&self.effective_scope)?;
        closed(
            &self.process_proof,
            &[
                "unknown",
                "unavailable",
                "exact",
                "ambiguous",
                "escaped",
                "partial",
            ],
            "process proof",
        )?;
        closed(
            &self.transfer,
            &["unknown", "none", "one-way", "bidirectional", "partial"],
            "transfer",
        )?;
        closed(
            &self.inspection,
            &[
                "unknown",
                "none",
                "opaque",
                "metadata-only",
                "full",
                "decrypted-unknown",
                "partial",
                "incomplete",
                "failed",
            ],
            "inspection",
        )?;
        closed(
            &self.loss,
            &["unknown", "none", "partial", "observed"],
            "loss",
        )?;
        closed(
            &self.cleanup,
            &["unknown", "not-needed", "succeeded", "partial", "failed"],
            "cleanup",
        )?;
        if let Some(value) = &self.failure {
            limited_token(value, 80, "failure code")?;
        }
        Ok(())
    }

    pub fn compare(&self, newer: &Self) -> HistoryComparison {
        if self.target_id != newer.target_id {
            return HistoryComparison::Incomparable("target-changed");
        }
        if self.fragcap_version != newer.fragcap_version {
            return HistoryComparison::Incomparable("fragcap-version-changed");
        }
        if self.requested_scope != newer.requested_scope
            || self.effective_scope != newer.effective_scope
        {
            return HistoryComparison::Incomparable("scope-changed");
        }
        if self.selected_route != newer.selected_route {
            return HistoryComparison::Incomparable("route-changed");
        }
        if self.selected_launch != newer.selected_launch {
            return HistoryComparison::Incomparable("launch-choice-changed");
        }
        if self.target_build_clue.is_some()
            && newer.target_build_clue.is_some()
            && self.target_build_clue != newer.target_build_clue
        {
            return HistoryComparison::Incomparable("target-clue-changed");
        }
        let mut changes = Vec::new();
        for (name, old, new) in [
            (
                "observed-launch",
                &self.observed_launch,
                &newer.observed_launch,
            ),
            (
                "observed-route",
                &self.observed_route,
                &newer.observed_route,
            ),
            ("process-proof", &self.process_proof, &newer.process_proof),
            ("transfer", &self.transfer, &newer.transfer),
            ("inspection", &self.inspection, &newer.inspection),
            ("loss", &self.loss, &newer.loss),
            ("cleanup", &self.cleanup, &newer.cleanup),
        ] {
            if old != new {
                changes.push(name);
            }
        }
        if self.failure != newer.failure {
            changes.push("failure");
        }
        HistoryComparison::Comparable {
            build_unverified: self.target_build_clue.is_none() || newer.target_build_clue.is_none(),
            changes,
        }
    }

    pub(crate) fn payload(&self) -> Value {
        json!({
            "session_id": self.session_id,
            "fragcap_version": self.fragcap_version,
            "target_build_clue": self.target_build_clue,
            "selected_launch": self.selected_launch,
            "observed_launch": self.observed_launch,
            "selected_route": self.selected_route,
            "observed_route": self.observed_route,
            "requested_scope": self.requested_scope,
            "effective_scope": self.effective_scope,
            "process_proof": self.process_proof,
            "transfer": self.transfer,
            "inspection": self.inspection,
            "failure": self.failure,
            "loss": self.loss,
            "cleanup": self.cleanup,
        })
    }

    pub(crate) fn from_payload(
        id: i64,
        target_id: i64,
        recorded_at: i64,
        payload: &str,
    ) -> Result<Self, TargetsError> {
        let value: Value = serde_json::from_str(payload)
            .map_err(|error| model(format!("invalid history payload: {error}")))?;
        let object = value
            .as_object()
            .ok_or_else(|| model("history payload is not an object"))?;
        const FIELDS: [&str; 15] = [
            "session_id",
            "fragcap_version",
            "target_build_clue",
            "selected_launch",
            "observed_launch",
            "selected_route",
            "observed_route",
            "requested_scope",
            "effective_scope",
            "process_proof",
            "transfer",
            "inspection",
            "failure",
            "loss",
            "cleanup",
        ];
        if object.len() != FIELDS.len() || !FIELDS.iter().all(|key| object.contains_key(*key)) {
            return Err(model("history payload field set is invalid"));
        }
        let required = |key: &str| -> Result<String, TargetsError> {
            object
                .get(key)
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| model(format!("history field {key} must be a string")))
        };
        let optional = |key: &str| -> Result<Option<String>, TargetsError> {
            match object.get(key) {
                Some(Value::Null) => Ok(None),
                Some(Value::String(value)) => Ok(Some(value.clone())),
                _ => Err(model(format!(
                    "history field {key} must be a string or null"
                ))),
            }
        };
        let row = Self {
            id: Some(id),
            target_id,
            recorded_at,
            session_id: optional("session_id")?,
            fragcap_version: required("fragcap_version")?,
            target_build_clue: optional("target_build_clue")?,
            selected_launch: required("selected_launch")?,
            observed_launch: required("observed_launch")?,
            selected_route: required("selected_route")?,
            observed_route: required("observed_route")?,
            requested_scope: required("requested_scope")?,
            effective_scope: required("effective_scope")?,
            process_proof: required("process_proof")?,
            transfer: required("transfer")?,
            inspection: required("inspection")?,
            failure: optional("failure")?,
            loss: required("loss")?,
            cleanup: required("cleanup")?,
        };
        row.validate()?;
        Ok(row)
    }
}

fn model(message: impl Into<String>) -> TargetsError {
    TargetsError::Model(message.into())
}

fn limited_token(value: &str, limit: usize, field: &str) -> Result<(), TargetsError> {
    if value.is_empty()
        || value.len() > limit
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        return Err(model(format!("history {field} is not a bounded token")));
    }
    Ok(())
}

fn closed(value: &str, allowed: &[&str], field: &str) -> Result<(), TargetsError> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(model(format!("unknown history {field} {value:?}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row() -> CaptureHistory {
        CaptureHistory::new(
            1,
            10,
            "0.11.1",
            "steam-protocol-warm",
            "direct-socket-v1",
            "exact-game",
            "exact-game",
        )
    }

    #[test]
    fn comparable_history_names_changes_without_becoming_authority() {
        let first = row();
        let mut later = first.clone();
        later.recorded_at = 11;
        later.transfer = "bidirectional".into();
        assert_eq!(
            first.compare(&later),
            HistoryComparison::Comparable {
                build_unverified: true,
                changes: vec!["transfer"],
            }
        );
        later.selected_route = "child-environment+direct-socket-v1".into();
        assert_eq!(
            first.compare(&later),
            HistoryComparison::Incomparable("route-changed")
        );
    }

    #[test]
    fn updates_scope_changes_and_rollback_do_not_invent_like_for_like_drift() {
        let mut first = row();
        first.target_build_clue = Some(format!("blake3:{}", "a".repeat(64)));
        first.observed_route = "direct-socket-v1".into();
        let mut failed = first.clone();
        failed.recorded_at = 11;
        failed.observed_route = "unavailable".into();
        failed.failure = Some("route-unavailable".into());
        assert_eq!(
            first.compare(&failed),
            HistoryComparison::Comparable {
                build_unverified: false,
                changes: vec!["observed-route", "failure"],
            }
        );
        let mut updated = failed.clone();
        updated.target_build_clue = Some(format!("blake3:{}", "b".repeat(64)));
        assert_eq!(
            failed.compare(&updated),
            HistoryComparison::Incomparable("target-clue-changed")
        );
        let mut changed_scope = failed.clone();
        changed_scope.effective_scope = "launch-chain".into();
        assert_eq!(
            failed.compare(&changed_scope),
            HistoryComparison::Incomparable("scope-changed")
        );
        let mut rollback = failed.clone();
        rollback.recorded_at = 12;
        rollback.observed_route = first.observed_route.clone();
        rollback.failure = None;
        assert_eq!(
            failed.compare(&rollback),
            HistoryComparison::Comparable {
                build_unverified: false,
                changes: vec!["observed-route", "failure"],
            }
        );
    }

    #[test]
    fn rejects_raw_paths_and_unrecognized_states() {
        let mut value = row();
        value.failure = Some("C:\\private\\game.exe".into());
        assert!(value.validate().is_err());
        value.failure = None;
        value.inspection = "guessed-decrypted".into();
        assert!(value.validate().is_err());
    }
}
