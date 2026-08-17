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
