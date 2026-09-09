// SPDX-License-Identifier: Apache-2.0

//! Conservative planning for historical discovery residue (slice S133).

use std::collections::{HashMap, HashSet};

use fragcap_profile::{FidelityTier, SignatureCategory};

use crate::{ClassificationSource, TargetEntry};

/// One authoritative platform title currently installed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthoritativePlatformInstall {
    /// Canonical platform anchor, for example `steam:620`.
    pub anchor: String,
    /// Exact installation root reported by the platform.
    pub install_root: String,
}

/// Injected platform facts used by the platform-neutral reconciliation planner.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlatformInventory {
    /// Exact platform-client roots.
    pub client_roots: Vec<String>,
    /// Exact installed-title identities and roots.
    pub authoritative_installs: Vec<AuthoritativePlatformInstall>,
    /// Platform records omitted because their metadata was malformed or bounded.
    pub truncated: u64,
}

/// Exact reason a historical row is safe to remove.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconciliationRemovalReason {
    /// The row is the platform client root itself.
    PlatformClientRoot,
    /// The row is platform infrastructure beneath the client root.
    PlatformInfrastructure,
    /// An anchored authoritative row owns the same exact installation root.
    DuplicateAuthoritativeInstall,
    /// The row aggregates evidence from more than one engine product.
    MultiTitleAggregate,
}

impl ReconciliationRemovalReason {
    /// Stable CLI spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PlatformClientRoot => "platform-client-root",
            Self::PlatformInfrastructure => "platform-infrastructure",
            Self::DuplicateAuthoritativeInstall => "duplicate-authoritative-install",
            Self::MultiTitleAggregate => "multi-title-aggregate",
        }
    }
}

/// Exact reason a row is preserved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconciliationPreservationReason {
    /// Human-authored authority cannot be removed by discovery cleanup.
    UserAuthored,
    /// Anchored rows represent durable platform identity.
    AnchoredAuthoritative,
    /// The platform inventory is incomplete, so removal cannot be proven safe.
    InventoryIncomplete,
    /// Provenance is absent, conflicting, or not the exact legacy source.
    OwnershipUnproven,
    /// No installation root exists to compare.
    InstallRootAbsent,
    /// A legacy row outside exact platform ownership has only location authority.
    LocationOnlyAmbiguous,
}

impl ReconciliationPreservationReason {
    /// Stable CLI spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UserAuthored => "user-authored",
            Self::AnchoredAuthoritative => "anchored-authoritative",
            Self::InventoryIncomplete => "inventory-incomplete",
            Self::OwnershipUnproven => "ownership-unproven",
            Self::InstallRootAbsent => "install-root-absent",
            Self::LocationOnlyAmbiguous => "location-only-ambiguous",
        }
    }
}

/// One row and its proven removal reason.
#[derive(Clone, Debug, PartialEq)]
pub struct ReconciliationRemoval {
    /// Complete apply-time row fingerprint.
    pub entry: TargetEntry,
    /// Proven reason for removal.
    pub reason: ReconciliationRemovalReason,
}

/// One preserved row and its reason.
#[derive(Clone, Debug, PartialEq)]
pub struct ReconciliationPreservation {
    /// Complete row.
    pub entry: TargetEntry,
    /// Reason it cannot be removed.
    pub reason: ReconciliationPreservationReason,
}

/// Conserved preview over every stored target.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReconciliationPlan {
    /// Exact rows safe to remove.
    pub removable: Vec<ReconciliationRemoval>,
    /// Every other row, with an explicit reason.
    pub preserved: Vec<ReconciliationPreservation>,
    /// Incomplete platform records that blocked mutation authority.
    pub inventory_truncated: u64,
}

impl ReconciliationPlan {
    /// Whether every input row reached exactly one disposition.
    pub fn is_conserved(&self, input_rows: usize) -> bool {
        input_rows == self.removable.len() + self.preserved.len()
    }
}

/// Plan historical cleanup without mutating any row.
pub fn plan_reconciliation(
    entries: &[TargetEntry],
    inventory: &PlatformInventory,
) -> ReconciliationPlan {
    let roots: Vec<String> = inventory
        .client_roots
        .iter()
        .map(|path| normalized_dir_key(path))
        .collect();
    let installs: HashMap<String, String> = inventory
        .authoritative_installs
        .iter()
        .map(|install| {
            (
                normalized_dir_key(&install.install_root),
                crate::identifier::canonicalize_anchor(&install.anchor),
            )
        })
        .collect();
    let stored_authoritative: HashSet<(String, String)> = entries
        .iter()
        .filter_map(|entry| {
            Some((
                crate::identifier::canonicalize_anchor(entry.anchor.as_deref()?),
                normalized_dir_key(entry.install_root.as_deref()?),
            ))
        })
        .collect();
    let mut plan = ReconciliationPlan {
        inventory_truncated: inventory.truncated,
        ..ReconciliationPlan::default()
    };

    for entry in entries {
        let preserved = |reason| ReconciliationPreservation {
            entry: entry.clone(),
            reason,
        };
        if entry.classification_source == ClassificationSource::User
            || entry.fidelity == FidelityTier::Authored
        {
            plan.preserved
                .push(preserved(ReconciliationPreservationReason::UserAuthored));
            continue;
        }
        if entry.anchor.is_some() {
            plan.preserved.push(preserved(
                ReconciliationPreservationReason::AnchoredAuthoritative,
            ));
            continue;
        }
        if inventory.truncated > 0 {
            plan.preserved.push(preserved(
                ReconciliationPreservationReason::InventoryIncomplete,
            ));
            continue;
        }
        if entry.classification_source != ClassificationSource::Platform
            || !has_exact_known_roots_provenance(entry)
        {
            plan.preserved.push(preserved(
                ReconciliationPreservationReason::OwnershipUnproven,
            ));
            continue;
        }
        let Some(install_root) = entry.install_root.as_deref() else {
            plan.preserved.push(preserved(
                ReconciliationPreservationReason::InstallRootAbsent,
            ));
            continue;
        };
        let root = normalized_dir_key(install_root);
        let reason = match installs.get(&root) {
            Some(anchor) if stored_authoritative.contains(&(anchor.clone(), root.clone())) => {
                Some(ReconciliationRemovalReason::DuplicateAuthoritativeInstall)
            }
            // A platform manifest proves this is a title path, not infrastructure,
            // but it is not a duplicate until a matching anchored row is stored.
            Some(_) => None,
            None if roots.contains(&root) => Some(ReconciliationRemovalReason::PlatformClientRoot),
            None if roots.iter().any(|client| is_descendant(&root, client)) => {
                Some(ReconciliationRemovalReason::PlatformInfrastructure)
            }
            None if engine_product_count(entry) > 1 => {
                Some(ReconciliationRemovalReason::MultiTitleAggregate)
            }
            None => None,
        };
        match reason {
            Some(reason) => plan.removable.push(ReconciliationRemoval {
                entry: entry.clone(),
                reason,
            }),
            None => plan.preserved.push(preserved(
                ReconciliationPreservationReason::LocationOnlyAmbiguous,
            )),
        }
    }
    plan
}

fn has_exact_known_roots_provenance(entry: &TargetEntry) -> bool {
    let Some(object) = entry
        .provenance
        .as_ref()
        .and_then(|value| value.as_object())
    else {
        return false;
    };
    object.len() == 1
        && object.get("source").and_then(|value| value.as_str()) == Some("known-roots")
}

fn engine_product_count(entry: &TargetEntry) -> usize {
    entry
        .evidence
        .as_ref()
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|finding| {
            let object = finding.as_object()?;
            (object.get("category")?.as_str()? == SignatureCategory::Engine.as_str())
                .then(|| object.get("product")?.as_str().map(str::to_owned))
                .flatten()
        })
        .collect::<HashSet<_>>()
        .len()
}

fn normalized_dir_key(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let normalized = if let Some(rest) = normalized.strip_prefix("//?/UNC/") {
        format!("//{rest}")
    } else if let Some(rest) = normalized.strip_prefix("//?/") {
        rest.to_string()
    } else {
        normalized
    };
    normalized.trim_end_matches('/').to_ascii_lowercase()
}

fn is_descendant(path: &str, root: &str) -> bool {
    path.strip_prefix(root)
        .is_some_and(|suffix| suffix.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TargetClassification, TargetEntry};
    use serde_json::json;

    fn entry(id: i64, root: Option<&str>) -> TargetEntry {
        TargetEntry {
            id: Some(id),
            stable_id: id,
            handle: format!("target_{id}"),
            name: format!("Target {id}"),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::Platform,
            fidelity: FidelityTier::HeuristicUnverified,
            provenance: Some(json!({"source": "known-roots"})),
            anchor: None,
            launch_entries: None,
            install_root: root.map(str::to_owned),
            evidence: None,
            detection_scan: None,
            folder_name: None,
            executable_hint: None,
        }
    }

    fn inventory() -> PlatformInventory {
        PlatformInventory {
            client_roots: vec!["C:/Games/Steam".to_string()],
            authoritative_installs: vec![AuthoritativePlatformInstall {
                anchor: "steam:620".to_string(),
                install_root: "C:/Games/Steam/steamapps/common/Portal 2".to_string(),
            }],
            truncated: 0,
        }
    }

    #[test]
    fn plans_only_exact_tool_owned_platform_residue() {
        let mut aggregate = entry(3, Some("D:/Games/Aggregate"));
        aggregate.evidence = Some(json!([
            {"category": "engine", "product": "Unity"},
            {"category": "engine", "product": "Unreal"}
        ]));
        let mut authoritative = entry(5, Some("C:/Games/Steam/steamapps/common/Portal 2"));
        authoritative.anchor = Some("steam:620".to_string());
        let rows = vec![
            entry(1, Some("C:/Games/Steam")),
            entry(2, Some("C:/Games/Steam/steamui")),
            aggregate,
            entry(4, Some("C:/Games/Steam/steamapps/common/Portal 2")),
            authoritative,
        ];
        let plan = plan_reconciliation(&rows, &inventory());
        assert!(plan.is_conserved(rows.len()));
        assert_eq!(plan.removable.len(), 4);
        assert_eq!(
            plan.removable
                .iter()
                .map(|item| item.reason)
                .collect::<Vec<_>>(),
            vec![
                ReconciliationRemovalReason::PlatformClientRoot,
                ReconciliationRemovalReason::PlatformInfrastructure,
                ReconciliationRemovalReason::MultiTitleAggregate,
                ReconciliationRemovalReason::DuplicateAuthoritativeInstall,
            ]
        );
    }

    #[test]
    fn manifest_inventory_alone_does_not_make_the_only_stored_target_a_duplicate() {
        let row = entry(1, Some("C:/Games/Steam/steamapps/common/Portal 2"));
        let plan = plan_reconciliation(&[row], &inventory());
        assert!(plan.removable.is_empty());
        assert_eq!(
            plan.preserved[0].reason,
            ReconciliationPreservationReason::LocationOnlyAmbiguous
        );
    }

    #[test]
    fn preserves_authored_anchored_ambiguous_and_incomplete_inventory_rows() {
        let mut authored = entry(1, Some("C:/Games/Steam"));
        authored.classification_source = ClassificationSource::User;
        let mut anchored = entry(2, Some("C:/Games/Steam"));
        anchored.anchor = Some("steam:2".to_string());
        let mut ambiguous = entry(3, Some("C:/Games/Steam"));
        ambiguous.provenance = Some(json!({"source": "known-roots", "reviewed": true}));
        let missing = entry(4, None);
        let location = entry(5, Some("D:/Games/Maybe"));
        let rows = vec![authored, anchored, ambiguous, missing, location];
        let plan = plan_reconciliation(&rows, &inventory());
        assert!(plan.removable.is_empty());
        assert_eq!(plan.preserved.len(), rows.len());

        let mut truncated = inventory();
        truncated.truncated = 2;
        let blocked = plan_reconciliation(&[entry(6, Some("C:/Games/Steam"))], &truncated);
        assert!(blocked.removable.is_empty());
        assert_eq!(blocked.inventory_truncated, 2);
        assert_eq!(
            blocked.preserved[0].reason,
            ReconciliationPreservationReason::InventoryIncomplete
        );
    }

    #[test]
    fn extended_path_spelling_matches_the_exact_platform_root() {
        let row = entry(1, Some("\\\\?\\C:\\Games\\Steam\\"));
        let plan = plan_reconciliation(&[row], &inventory());
        assert_eq!(plan.removable.len(), 1);
        assert_eq!(
            plan.removable[0].reason,
            ReconciliationRemovalReason::PlatformClientRoot
        );
    }
}
