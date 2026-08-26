// SPDX-License-Identifier: Apache-2.0

//! The `TargetSource` discovery seam (slice S052), specification section 7.
//!
//! Every origin of a capture target, a platform walk, a known-roots scan, a
//! directory the user points at, implements one trait, [`TargetSource`], and
//! produces one kind of value, a [`CandidateTarget`], plus a truthful
//! [`DiscoveryAccount`]. Single-target authoring and bulk platform walking are the
//! same operation at different batch sizes, which is what principle P-10 asks for:
//! adding Epic, GOG, Xbox, Battle.net, or an emulator ROM directory later is a new
//! implementor of this seam with no downstream change.
//!
//! A source produces candidates for a live listing and writes nothing durable; a
//! candidate becomes a stored [`crate::entry::TargetEntry`] only when the user acts
//! on it (the persist-on-first-use rule, S052 spec FR-020/FR-021). The one durable
//! write this slice makes is the volume eligibility table (see [`crate::volume`]).
//!
//! # The account cannot lie
//!
//! [`DiscoveryAccount`] mirrors [`crate::seed::SeedSummary`]'s conservation: every
//! item a source considered lands in exactly one named outcome, and the outcomes
//! reconcile to the number considered. Container directories distinguish successful
//! bounded descent from depth-limited coverage. A discard path added later with no
//! counter fails [`DiscoveryAccount::is_conserved`] rather than dropping a candidate
//! silently (P-4).

use fragcap_profile::{DetectionFinding, FidelityTier};

use crate::entry::{DetectionScan, TargetClassification};
use crate::TargetsError;

/// What a source found, enough to identify it and later author it into an entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CandidateIdentity {
    /// A filesystem path (tiers 2 and 3: a game directory or an executable).
    Path(String),
    /// A Steam application id (tier 1); joins to the shipped catalog.
    SteamAppId(u32),
}

/// One thing a source found.
///
/// Not yet a stored entry: turning a candidate into a durable
/// [`crate::entry::TargetEntry`] is the S051 entry model's job, triggered when the
/// user acts on the candidate (FR-021). `classification` is `Unknown` when nothing
/// classified it, never guessed (P-9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateTarget {
    /// What was found (a path or a platform identity).
    pub identity: CandidateIdentity,
    /// The best available human name: platform metadata, a directory name, or an
    /// executable stem. Never invented beyond these observations (P-9).
    pub display_name: String,
    /// The fidelity the producing source stamped.
    pub fidelity: FidelityTier,
    /// The classification joined from the catalog or asserted by the source;
    /// `Unknown` when none applies.
    pub classification: TargetClassification,
    /// The technologies detected in the candidate's directory (slice S053), carried
    /// as neutral evidence: the detected engine plus any anti-cheat or DRM. Empty
    /// when the producing source ran no signature detection. A fact set only; no
    /// field here characterizes a title as off limits (specification section 3.6).
    pub evidence: Vec<DetectionFinding>,
    /// Whether the directory of the candidate was scanned for technologies and whether
    /// that scan was complete (slice S065). `None` when the producing source ran no
    /// detection at all, which is a different fact from a scan that found nothing:
    /// an empty `evidence` with `Some(Complete)` is an answer, an empty `evidence`
    /// with `None` is the absence of one (P-4, P-9).
    pub detection_scan: Option<DetectionScan>,
    /// The name of the source that produced this candidate.
    pub source_name: String,
    /// The resolved, absolute install directory, when the source found one.
    /// Carried explicitly rather than derived from `identity`, because a Steam
    /// candidate's identity is its app id, not a path: without this field a
    /// Steam-sourced registration stored no `install_root` at all, so the
    /// missing-install-root detection (issue #167) could never fire for it (the
    /// dominant real-world case). A path-based source's value here is the same
    /// path its `identity` already carries.
    pub install_root: Option<String>,
    /// The raw platform installdir or folder-identifying value, verbatim (slice
    /// S066, issue #173). `Some` for a Steam candidate; `None` for a known-roots or
    /// directory-scan candidate, which has no separate installdir concept distinct
    /// from its path-derived `display_name`.
    pub folder_name: Option<String>,
    /// The raw observed launch executable name, verbatim, or `None` when the
    /// producing source observed none. A findability hint only, never a claim
    /// about a resolved capture chain (P-9).
    pub executable_hint: Option<String>,
}

/// The truthful per-run account. Every considered item lands in exactly one named
/// outcome; [`DiscoveryAccount::is_conserved`] is the P-4 guard.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiscoveryAccount {
    /// Total items the source examined.
    pub considered: u64,
    /// Items emitted as candidates.
    pub produced: u64,
    /// Items whose metadata was present but could not be parsed.
    pub parse_failed: u64,
    /// Candidates a human rejected at the interactive step.
    pub declined_by_user: u64,
    /// Considered and determined not to be a capturable game: a known-root
    /// directory matching no signature and no known-root rule, or a Steam title
    /// whose appinfo type is `Music` (a soundtrack, which has no network
    /// behavior; slice S066, issue #166).
    pub considered_not_a_game: u64,
    /// Container directories that were not emitted and whose immediate children
    /// were enumerated within the known-roots depth bound.
    pub container_descended: u64,
    /// Container directories that were not emitted and whose children could not
    /// be enumerated because the known-roots depth bound was reached.
    pub container_descent_truncated: u64,
    /// Items not examined because their volume was ineligible.
    pub volume_skipped: u64,
    /// Items not examined because of a permission or I/O error.
    pub access_error: u64,
}

impl DiscoveryAccount {
    /// The conservation invariant: the named outcomes reconcile to the number of
    /// items considered. Asserted in every source test; a new discard path without
    /// a counter fails here rather than shipping silently (P-4).
    pub fn is_conserved(&self) -> bool {
        self.produced
            + self.parse_failed
            + self.declined_by_user
            + self.considered_not_a_game
            + self.container_descended
            + self.container_descent_truncated
            + self.volume_skipped
            + self.access_error
            == self.considered
    }

    /// Count one produced candidate (advances `considered` and `produced`).
    pub fn produce(&mut self) {
        self.considered += 1;
        self.produced += 1;
    }
}

/// What a discovery run returned: the produced candidates, the conserved account,
/// and any non-fatal diagnostics.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Discovery {
    /// What the source produced this run.
    pub candidates: Vec<CandidateTarget>,
    /// The conserved tally of every item considered.
    pub account: DiscoveryAccount,
    /// Non-fatal, named diagnostics a source surfaces so a loss is visible rather
    /// than silent: the path of a root it could not read, a manifest it could not
    /// parse, a duplicate it merged. These name what the scalar account counts, so
    /// "some access error occurred" is recoverable to "which root failed" (P-4).
    /// A warning is a report, not an outcome bucket: it does not affect
    /// [`DiscoveryAccount::is_conserved`].
    pub warnings: Vec<String>,
}

/// The discovery seam. Every origin of a capture target implements it.
///
/// An `Err` is a whole-run failure (an unreadable metadata store); a single item
/// that cannot be parsed, is declined, or is not a game is counted in the account,
/// never returned as an `Err` and never dropped silently (P-4, P-9).
pub trait TargetSource {
    /// A stable, human-readable name, used in the account and in listings.
    fn name(&self) -> &str;

    /// Produce the candidates plus the truthful account.
    fn discover(&self) -> Result<Discovery, TargetsError>;

    /// The fidelity this source stamps on the candidates it produces.
    fn default_fidelity(&self) -> FidelityTier;
}

/// Run a set of sources and merge their candidates and accounts into one
/// [`Discovery`]. This is the discovery driver the seam is built around: adding a
/// new source is the whole cost of extending discovery, the driver does not change
/// (S052 spec SC-006). A source that fails returns its `Err` unchanged.
pub fn discover_all(sources: &[&dyn TargetSource]) -> Result<Discovery, TargetsError> {
    let mut merged = Discovery::default();
    for source in sources {
        let d = source.discover()?;
        merged.candidates.extend(d.candidates);
        merged.warnings.extend(d.warnings);
        merged.account.considered += d.account.considered;
        merged.account.produced += d.account.produced;
        merged.account.parse_failed += d.account.parse_failed;
        merged.account.declined_by_user += d.account.declined_by_user;
        merged.account.considered_not_a_game += d.account.considered_not_a_game;
        merged.account.container_descended += d.account.container_descended;
        merged.account.container_descent_truncated += d.account.container_descent_truncated;
        merged.account.volume_skipped += d.account.volume_skipped;
        merged.account.access_error += d.account.access_error;
    }
    Ok(merged)
}

/// A source backed by a canned candidate list and account.
///
/// It keeps the seam testable offline (the same posture as [`crate::FixtureCatalog`]):
/// a test adds a `FixtureSource` to a discovery run and proves the driver needs no
/// change to gain a new source (SC-006).
pub struct FixtureSource {
    name: String,
    fidelity: FidelityTier,
    discovery: Discovery,
}

impl FixtureSource {
    /// Build a fixture source from a name, the fidelity it reports, and the
    /// discovery it will return.
    pub fn new(name: impl Into<String>, fidelity: FidelityTier, discovery: Discovery) -> Self {
        FixtureSource {
            name: name.into(),
            fidelity,
            discovery,
        }
    }
}

impl TargetSource for FixtureSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn discover(&self) -> Result<Discovery, TargetsError> {
        Ok(self.discovery.clone())
    }

    fn default_fidelity(&self) -> FidelityTier {
        self.fidelity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(name: &str, source: &str) -> CandidateTarget {
        let path = format!("C:/games/{name}");
        CandidateTarget {
            identity: CandidateIdentity::Path(path.clone()),
            display_name: name.to_string(),
            fidelity: FidelityTier::HeuristicUnverified,
            classification: TargetClassification::Unknown,
            evidence: Vec::new(),
            detection_scan: None,
            source_name: source.to_string(),
            install_root: Some(path),
            folder_name: None,
            executable_hint: None,
        }
    }

    #[test]
    fn account_conserves_only_when_outcomes_sum() {
        let mut a = DiscoveryAccount::default();
        a.produce();
        a.produce();
        a.considered += 1;
        a.parse_failed += 1;
        assert!(a.is_conserved());
        a.considered += 1; // an unaccounted item breaks conservation
        assert!(!a.is_conserved());
    }

    #[test]
    fn discover_all_merges_candidates_and_accounts() {
        let mut acc_a = DiscoveryAccount::default();
        acc_a.produce();
        let a = FixtureSource::new(
            "a",
            FidelityTier::HeuristicUnverified,
            Discovery {
                candidates: vec![candidate("alpha", "a")],
                account: acc_a,
                ..Discovery::default()
            },
        );
        let mut acc_b = DiscoveryAccount::default();
        acc_b.produce();
        acc_b.considered += 1;
        acc_b.considered_not_a_game += 1;
        let b = FixtureSource::new(
            "b",
            FidelityTier::Authored,
            Discovery {
                candidates: vec![candidate("beta", "b")],
                account: acc_b,
                ..Discovery::default()
            },
        );

        let merged = discover_all(&[&a, &b]).unwrap();
        assert_eq!(merged.candidates.len(), 2);
        assert_eq!(merged.account.considered, 3);
        assert_eq!(merged.account.produced, 2);
        assert_eq!(merged.account.considered_not_a_game, 1);
        assert_eq!(merged.account.container_descended, 0);
        assert_eq!(merged.account.container_descent_truncated, 0);
        assert!(merged.account.is_conserved());
    }

    #[test]
    fn account_conserves_container_outcomes() {
        let account = DiscoveryAccount {
            considered: 2,
            container_descended: 1,
            container_descent_truncated: 1,
            ..DiscoveryAccount::default()
        };

        assert!(account.is_conserved());
    }
}
