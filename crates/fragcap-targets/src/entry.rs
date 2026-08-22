// SPDX-License-Identifier: Apache-2.0

//! The target entry model (slice S051), specification section 15.8.
//!
//! A capture target is a row in `local.db`, not a profile file. [`TargetEntry`]
//! is that row: identity (a stable identifier, a handle, a name), a
//! classification and the source that assigned it, a fidelity (reusing the
//! resolver's [`FidelityTier`] so there is one fidelity vocabulary, P-10), and
//! the launch entries, install root, provenance, and evidence carried whole as
//! JSON.
//!
//! [`TargetClassification`] is a distinct type from the catalog's coarse
//! [`crate::catalog::Classification`] (`Game`/`Other`): the entry model needs the
//! full set, and `Unknown` is a first-class, frequent state rather than a missing
//! value, because forcing a binary guess is the fabricated certainty P-9 forbids.

use fragcap_profile::FidelityTier;

use crate::TargetsError;

/// What a target is. `Unknown` is deliberate: a target the tool cannot classify
/// is stored as `Unknown`, never guessed into a wrong bucket (P-9).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TargetClassification {
    /// A game client: the process that holds the gameplay socket.
    Game,
    /// A platform or publisher launcher.
    Launcher,
    /// A supporting tool (an overlay, a companion utility).
    Tool,
    /// A modification of another title.
    Mod,
    /// An emulator hosting another platform's title.
    Emulator,
    /// Not yet classified. A real, frequent state.
    Unknown,
}

impl TargetClassification {
    /// The stored string, matching the schema CHECK set.
    pub fn as_str(self) -> &'static str {
        match self {
            TargetClassification::Game => "game",
            TargetClassification::Launcher => "launcher",
            TargetClassification::Tool => "tool",
            TargetClassification::Mod => "mod",
            TargetClassification::Emulator => "emulator",
            TargetClassification::Unknown => "unknown",
        }
    }

    /// Parse a stored string, rejecting an out-of-set value before it can be
    /// trusted as a classification.
    pub fn parse(s: &str) -> Result<TargetClassification, TargetsError> {
        match s {
            "game" => Ok(TargetClassification::Game),
            "launcher" => Ok(TargetClassification::Launcher),
            "tool" => Ok(TargetClassification::Tool),
            "mod" => Ok(TargetClassification::Mod),
            "emulator" => Ok(TargetClassification::Emulator),
            "unknown" => Ok(TargetClassification::Unknown),
            other => Err(TargetsError::Model(format!(
                "unknown classification {other:?}"
            ))),
        }
    }
}

/// What assigned a target's classification, so a higher-authority source can
/// overwrite a lower one without either guessing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClassificationSource {
    /// The shipped catalog.
    Catalog,
    /// A data-driven engine signature (S053).
    EngineSignature,
    /// A platform walker (Steam, and later others).
    Platform,
    /// A human authored or reviewed it.
    User,
    /// No source has classified it yet.
    Unset,
}

impl ClassificationSource {
    /// The stored string, matching the schema CHECK set.
    pub fn as_str(self) -> &'static str {
        match self {
            ClassificationSource::Catalog => "catalog",
            ClassificationSource::EngineSignature => "engine-signature",
            ClassificationSource::Platform => "platform",
            ClassificationSource::User => "user",
            ClassificationSource::Unset => "unset",
        }
    }

    /// Parse a stored string, rejecting an out-of-set value.
    pub fn parse(s: &str) -> Result<ClassificationSource, TargetsError> {
        match s {
            "catalog" => Ok(ClassificationSource::Catalog),
            "engine-signature" => Ok(ClassificationSource::EngineSignature),
            "platform" => Ok(ClassificationSource::Platform),
            "user" => Ok(ClassificationSource::User),
            "unset" => Ok(ClassificationSource::Unset),
            other => Err(TargetsError::Model(format!(
                "unknown classification source {other:?}"
            ))),
        }
    }
}

/// Whether a target's install directory was scanned for technologies, and whether
/// that scan covered everything it set out to (slice S065).
///
/// This is deliberately a distinct fact from the finding set. A complete scan that
/// matched nothing is a real answer; an incomplete scan that matched nothing is not,
/// and a directory nobody ever scanned is a third thing again. Collapsing the three
/// into one blank column is the silent loss P-4 forbids.
///
/// There is no `NotScanned` variant. Absence is modeled by `Option::None`, stored as
/// SQL `NULL`, because a variant would let a row assert that no scan happened, which
/// is a claim, where absence is simply the lack of one. Every row written before
/// this slice reads as `None`, which is correct rather than a migration gap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DetectionScan {
    /// The directory was scanned and the scan read everything it set out to read.
    Complete,
    /// The directory was scanned and coverage was reduced: something could not be
    /// read, or a scan bound truncated the candidate set.
    Incomplete,
}

impl DetectionScan {
    /// The stored string, matching the schema CHECK set.
    pub fn as_str(self) -> &'static str {
        match self {
            DetectionScan::Complete => "complete",
            DetectionScan::Incomplete => "incomplete",
        }
    }

    /// Parse a stored string, rejecting an out-of-set value before it can be trusted
    /// as a coverage claim. An unrecognized value is an error rather than a
    /// permissive fallback: reading it as "no scan recorded" would silently discard a
    /// claim the store made, and reading it as "complete" would invent one (P-9).
    pub fn parse(s: &str) -> Result<DetectionScan, TargetsError> {
        match s {
            "complete" => Ok(DetectionScan::Complete),
            "incomplete" => Ok(DetectionScan::Incomplete),
            other => Err(TargetsError::Model(format!(
                "unknown detection scan state {other:?}"
            ))),
        }
    }

    /// The coverage state a scan outcome earns: `Complete` only when nothing was
    /// unreadable and no bound truncated the candidate set.
    pub fn from_outcome(outcome: &fragcap_profile::signature::ScanOutcome) -> DetectionScan {
        if outcome.is_complete() {
            DetectionScan::Complete
        } else {
            DetectionScan::Incomplete
        }
    }
}

/// One capture target, a row in the `targets` table of `local.db`.
///
/// `id` is `None` before insertion (the store assigns the autoincrement primary
/// key) and `Some` once read back. `stable_id` is the durable 63-bit identifier
/// (see [`crate::identifier`]); `handle` is the unique human selector (see
/// [`crate::handle`]). `provenance`, `launch_entries`, and `evidence` are carried
/// whole as JSON so this slice neither interprets nor reshapes them.
#[derive(Clone, Debug, PartialEq)]
pub struct TargetEntry {
    /// The autoincrement row key, `None` until inserted.
    pub id: Option<i64>,
    /// The durable 63-bit stable identifier.
    pub stable_id: i64,
    /// The unique, normalized human handle.
    pub handle: String,
    /// The display name the handle was derived from.
    pub name: String,
    /// What the target is.
    pub classification: TargetClassification,
    /// What assigned the classification.
    pub classification_source: ClassificationSource,
    /// The confidence stamp, ordered `Authored > Verified > HeuristicUnverified > Observed`.
    pub fidelity: FidelityTier,
    /// How this entry was produced, carried whole as JSON.
    pub provenance: Option<serde_json::Value>,
    /// The canonical anchor string, or `None` for an unanchored target.
    pub anchor: Option<String>,
    /// The launch entries, carried whole as JSON.
    pub launch_entries: Option<serde_json::Value>,
    /// The filesystem install root, or `None`.
    pub install_root: Option<String>,
    /// Supporting evidence, carried whole as JSON.
    pub evidence: Option<serde_json::Value>,
    /// Whether this target's install directory was scanned for technologies and
    /// whether that scan was complete. `None` means no scan is recorded, which is
    /// what a row produced by a source that ran no detection carries.
    pub detection_scan: Option<DetectionScan>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_round_trips_every_variant() {
        for c in [
            TargetClassification::Game,
            TargetClassification::Launcher,
            TargetClassification::Tool,
            TargetClassification::Mod,
            TargetClassification::Emulator,
            TargetClassification::Unknown,
        ] {
            assert_eq!(TargetClassification::parse(c.as_str()).unwrap(), c);
        }
        assert!(TargetClassification::parse("nonsense").is_err());
    }

    #[test]
    fn detection_scan_round_trips_every_variant_and_rejects_the_rest() {
        for d in [DetectionScan::Complete, DetectionScan::Incomplete] {
            assert_eq!(DetectionScan::parse(d.as_str()).unwrap(), d);
        }
        assert!(DetectionScan::parse("nonsense").is_err());
        // "not-scanned" is deliberately not a value: absence is None, never a
        // stored claim that no scan happened.
        assert!(DetectionScan::parse("not-scanned").is_err());
        assert!(DetectionScan::parse("").is_err());
    }

    #[test]
    fn classification_source_round_trips_every_variant() {
        for s in [
            ClassificationSource::Catalog,
            ClassificationSource::EngineSignature,
            ClassificationSource::Platform,
            ClassificationSource::User,
            ClassificationSource::Unset,
        ] {
            assert_eq!(ClassificationSource::parse(s.as_str()).unwrap(), s);
        }
        assert!(ClassificationSource::parse("nonsense").is_err());
    }
}
