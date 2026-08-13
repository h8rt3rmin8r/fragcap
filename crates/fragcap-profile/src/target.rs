// SPDX-License-Identifier: Apache-2.0

//! The resolved answer of the target resolution cascade, specification section
//! 15.7.
//!
//! A [`Target`] is what the resolver hands onward: an identity to capture, the
//! [`FidelityTier`] of the source that produced it, and a [`Provenance`] naming
//! that source. It is deliberately distinct from a [`Profile`]. A profile is one
//! way to back a target, the authored or verified way; the runtime-observation
//! provider produces a target with no profile behind it at all. Keeping the
//! answer separate from the profile is what lets a later provider (an engine
//! rule, a platform walker, the hint database) answer without inventing a
//! profile first.
//!
//! Every target carries exactly one fidelity tier, stamped by its source and
//! never inferred (P-9). An observation answer is [`FidelityTier::Observed`], a
//! profile answer is whatever the profile declared.

use crate::engine_rule::Engine;
use crate::schema::{FidelityTier, MatchPredicates, Profile, Provenance};

/// A live process the runtime-observation provider matched, recorded from what
/// the process snapshot already holds.
///
/// It carries the image name and full path of the matched process, the process
/// identifier, and the identity that selected it, all from a toolhelp
/// enumeration or the process tree. No process handle is opened and no process
/// memory is read (constitution P-1): naming a process is not the same act as
/// reaching into it.
///
/// The identity is retained, not just the current match, so the target carries
/// its match rules (specification section 15.7): a later capture that re-arms
/// after a restart, or one that needs the path anchors again, has them without
/// re-deriving the identity from the process that happened to be live now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedTarget {
    pid: u32,
    image_name: String,
    image_path: String,
    identity: MatchPredicates,
}

impl ObservedTarget {
    /// Build an observed target from a matched process's snapshot fields and the
    /// identity that selected it.
    pub fn new(
        pid: u32,
        image_name: String,
        image_path: String,
        identity: MatchPredicates,
    ) -> ObservedTarget {
        ObservedTarget {
            pid,
            image_name,
            image_path,
            identity,
        }
    }

    /// The operating-system process identifier of the matched process.
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// The matched process's image file name.
    pub fn image_name(&self) -> &str {
        &self.image_name
    }

    /// The matched process's full image path.
    pub fn image_path(&self) -> &str {
        &self.image_path
    }

    /// The identity that selected this process: the match rules the target
    /// carries, reusable for a later re-match.
    pub fn identity(&self) -> &MatchPredicates {
        &self.identity
    }
}

/// A client executable an engine rule resolved from an install directory's
/// documented layout, recorded from the filesystem alone.
///
/// It carries which engine's rule matched, the image file name and full path of
/// the resolved client (the socket holder a launch stub relaunches), and the
/// identity the pipeline binds it by once the process appears. No process is
/// running yet and none is inspected: the resolution is a function of the install
/// directory's shape, so no process handle is opened and no memory is read
/// (constitution P-1).
///
/// The identity is carried, not just the resolved path, so watch mode (S028) can
/// bind the process by its match rules (the executable name plus the path anchor
/// the rule keyed on) once it starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineRuleTarget {
    engine: Engine,
    image_name: String,
    image_path: String,
    identity: MatchPredicates,
}

impl EngineRuleTarget {
    /// Build an engine-rule target from the matched engine, the resolved client's
    /// name and path, and the identity that binds it.
    pub fn new(
        engine: Engine,
        image_name: String,
        image_path: String,
        identity: MatchPredicates,
    ) -> EngineRuleTarget {
        EngineRuleTarget {
            engine,
            image_name,
            image_path,
            identity,
        }
    }

    /// Which engine's rule resolved this target.
    pub fn engine(&self) -> Engine {
        self.engine
    }

    /// The resolved client's image file name.
    pub fn image_name(&self) -> &str {
        &self.image_name
    }

    /// The resolved client's full path on disk.
    pub fn image_path(&self) -> &str {
        &self.image_path
    }

    /// The identity the pipeline binds the client by once it appears.
    pub fn identity(&self) -> &MatchPredicates {
        &self.identity
    }
}

/// A client executable a platform walker resolved from a storefront's installed
/// library, recorded from the filesystem alone.
///
/// It carries the storefront (`steam` in the first walker), the image file name
/// and full path of the resolved client, and the identity the pipeline binds it
/// by. Like an engine-rule target it names a file on disk, not a running process:
/// no process handle is opened and no memory is read (constitution P-1). The
/// walker resolves it by classifying the install directory's executables, so its
/// provenance names that method and not a source it did not read (P-9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalkerTarget {
    platform: String,
    image_name: String,
    image_path: String,
    identity: MatchPredicates,
}

impl WalkerTarget {
    /// Build a walker target from the storefront, the resolved client's name and
    /// path, and the identity that binds it.
    pub fn new(
        platform: String,
        image_name: String,
        image_path: String,
        identity: MatchPredicates,
    ) -> WalkerTarget {
        WalkerTarget {
            platform,
            image_name,
            image_path,
            identity,
        }
    }

    /// The storefront the walker read (for example `steam`).
    pub fn platform(&self) -> &str {
        &self.platform
    }

    /// The resolved client's image file name.
    pub fn image_name(&self) -> &str {
        &self.image_name
    }

    /// The resolved client's full path on disk.
    pub fn image_path(&self) -> &str {
        &self.image_path
    }

    /// The identity the pipeline binds the client by once it appears.
    pub fn identity(&self) -> &MatchPredicates {
        &self.identity
    }
}

/// How a resolved target is backed.
///
/// A validated profile (from the profile provider), a client resolved from an
/// engine's install layout (from the engine-rule provider), a client resolved
/// from a storefront's installed library (from the platform walker), or a live
/// process the observation provider matched. Every origin resolves to one
/// identity to capture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetOrigin {
    /// Backed by a validated profile. The profile's stages are the match rules.
    Profile(Profile),
    /// Resolved from an engine's documented install layout, with no profile.
    EngineRule(EngineRuleTarget),
    /// Resolved from a storefront's installed library, with no profile.
    PlatformWalker(WalkerTarget),
    /// Derived from a live process matched by an identity, with no profile.
    Observed(ObservedTarget),
}

/// The resolved answer the resolver returns and the pipeline acts on.
///
/// Carries the fidelity tier and provenance of the source that produced it, and
/// the origin that says how to act on it. A profile-backed target still yields
/// its [`Profile`] through [`Target::profile`], so the existing capture path can
/// consume it unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    fidelity: FidelityTier,
    provenance: Provenance,
    origin: TargetOrigin,
}

impl Target {
    /// Build a target from its stamped fidelity, provenance, and origin.
    pub fn new(fidelity: FidelityTier, provenance: Provenance, origin: TargetOrigin) -> Target {
        Target {
            fidelity,
            provenance,
            origin,
        }
    }

    /// The trust tier of the source that produced this answer.
    pub fn fidelity(&self) -> FidelityTier {
        self.fidelity
    }

    /// Where this answer came from.
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// How this target is backed.
    pub fn origin(&self) -> &TargetOrigin {
        &self.origin
    }

    /// The backing profile, when this target came from a profile. The capture
    /// pipeline consumes this for a profile-backed target.
    pub fn profile(&self) -> Option<&Profile> {
        match &self.origin {
            TargetOrigin::Profile(p) => Some(p),
            TargetOrigin::EngineRule(_)
            | TargetOrigin::PlatformWalker(_)
            | TargetOrigin::Observed(_) => None,
        }
    }

    /// Consume the target and return its backing profile, when it came from one.
    ///
    /// The capture pipeline takes an owned [`Profile`], so a profile-backed
    /// target hands its profile onward this way.
    pub fn into_profile(self) -> Option<Profile> {
        match self.origin {
            TargetOrigin::Profile(p) => Some(p),
            TargetOrigin::EngineRule(_)
            | TargetOrigin::PlatformWalker(_)
            | TargetOrigin::Observed(_) => None,
        }
    }

    /// The recognition identity of a non-profile target.
    ///
    /// A target resolved by the engine rule, the platform walker, or runtime
    /// observation carries a [`MatchPredicates`] identity (an image name plus
    /// optional path anchors) rather than a backing profile. The non-profile
    /// capture path reads it here to synthesize a one-stage capture identity,
    /// exactly as `watch` builds one from a typed identity. A profile-backed
    /// target returns `None`: its identity lives in its stages, and it is
    /// captured through [`Target::into_profile`] instead.
    pub fn identity(&self) -> Option<&MatchPredicates> {
        match &self.origin {
            TargetOrigin::Profile(_) => None,
            TargetOrigin::EngineRule(t) => Some(t.identity()),
            TargetOrigin::PlatformWalker(t) => Some(t.identity()),
            TargetOrigin::Observed(t) => Some(t.identity()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_rule::Engine;

    fn provenance() -> Provenance {
        Provenance::new("engine-rule".to_string(), None)
    }

    fn identity() -> MatchPredicates {
        MatchPredicates::with_exe("game.exe").expect("a valid exe glob")
    }

    #[test]
    fn identity_is_some_for_a_non_profile_origin() {
        let engine_rule = Target::new(
            FidelityTier::HeuristicUnverified,
            provenance(),
            TargetOrigin::EngineRule(EngineRuleTarget::new(
                Engine::Unreal,
                "game.exe".to_string(),
                "C:/game/game.exe".to_string(),
                identity(),
            )),
        );
        assert!(engine_rule.identity().is_some());

        let walker = Target::new(
            FidelityTier::HeuristicUnverified,
            Provenance::new("steam-library".to_string(), None),
            TargetOrigin::PlatformWalker(WalkerTarget::new(
                "steam".to_string(),
                "game.exe".to_string(),
                "C:/game/game.exe".to_string(),
                identity(),
            )),
        );
        assert!(walker.identity().is_some());

        let observed = Target::new(
            FidelityTier::Observed,
            Provenance::new("observation".to_string(), None),
            TargetOrigin::Observed(ObservedTarget::new(
                1234,
                "game.exe".to_string(),
                "C:/game/game.exe".to_string(),
                identity(),
            )),
        );
        assert!(observed.identity().is_some());
    }

    #[test]
    fn identity_is_none_for_a_profile_origin() {
        let profile = Profile::parse(
            &serde_json::json!({
                "schema": 1,
                "kind": "profile",
                "fidelity": "authored",
                "game": { "id": "g", "name": "Game" },
                "stage": [
                    { "role": "client", "lifecycle": "session", "terminal": true,
                      "match": { "exe": "game.exe" } }
                ]
            })
            .to_string(),
        )
        .expect("a valid profile");
        let target = Target::new(
            FidelityTier::Authored,
            Provenance::new("user".to_string(), None),
            TargetOrigin::Profile(profile),
        );
        assert!(target.identity().is_none());
    }
}
