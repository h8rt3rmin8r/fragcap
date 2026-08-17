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
//! reconcile to the number considered. A discard path added later with no counter
//! fails [`DiscoveryAccount::is_conserved`] rather than dropping a candidate
//! silently (P-4).

use fragcap_profile::FidelityTier;

use crate::entry::TargetClassification;
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
    /// The name of the source that produced this candidate.
    pub source_name: String,
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
    /// Directories that matched no signature and no known-root rule.
    pub considered_not_a_game: u64,
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

/// What a discovery run returned: the produced candidates and the account.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Discovery {
    /// What the source produced this run.
    pub candidates: Vec<CandidateTarget>,
    /// The conserved tally of every item considered.
    pub account: DiscoveryAccount,
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
    let mut candidates = Vec::new();
    let mut account = DiscoveryAccount::default();
    for source in sources {
        let d = source.discover()?;
        candidates.extend(d.candidates);
        account.considered += d.account.considered;
        account.produced += d.account.produced;
        account.parse_failed += d.account.parse_failed;
        account.declined_by_user += d.account.declined_by_user;
        account.considered_not_a_game += d.account.considered_not_a_game;
        account.volume_skipped += d.account.volume_skipped;
        account.access_error += d.account.access_error;
    }
    Ok(Discovery {
        candidates,
        account,
    })
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
        CandidateTarget {
            identity: CandidateIdentity::Path(format!("C:/games/{name}")),
            display_name: name.to_string(),
            fidelity: FidelityTier::HeuristicUnverified,
            classification: TargetClassification::Unknown,
            source_name: source.to_string(),
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
            },
        );

        let merged = discover_all(&[&a, &b]).unwrap();
        assert_eq!(merged.candidates.len(), 2);
        assert_eq!(merged.account.considered, 3);
        assert_eq!(merged.account.produced, 2);
        assert_eq!(merged.account.considered_not_a_game, 1);
        assert!(merged.account.is_conserved());
    }
}
