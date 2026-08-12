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

use crate::schema::{FidelityTier, Profile, Provenance};

/// A live process the runtime-observation provider matched, recorded from what
/// the process snapshot already holds.
///
/// It carries the image name and full path (the identity anchors) and the
/// process identifier, all of which come from a toolhelp enumeration or the
/// process tree. No process handle is opened and no process memory is read
/// (constitution P-1): naming a process is not the same act as reaching into it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedTarget {
    pid: u32,
    image_name: String,
    image_path: String,
}

impl ObservedTarget {
    /// Build an observed target from a matched process's snapshot fields.
    pub fn new(pid: u32, image_name: String, image_path: String) -> ObservedTarget {
        ObservedTarget {
            pid,
            image_name,
            image_path,
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
}

/// How a resolved target is backed.
///
/// Either a validated profile (from the profile provider) or a live process the
/// observation provider matched. A later slice may add further origins, but every
/// origin still resolves to one identity to capture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetOrigin {
    /// Backed by a validated profile. The profile's stages are the match rules.
    Profile(Profile),
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
            TargetOrigin::Observed(_) => None,
        }
    }

    /// Consume the target and return its backing profile, when it came from one.
    ///
    /// The capture pipeline takes an owned [`Profile`], so a profile-backed
    /// target hands its profile onward this way.
    pub fn into_profile(self) -> Option<Profile> {
        match self.origin {
            TargetOrigin::Profile(p) => Some(p),
            TargetOrigin::Observed(_) => None,
        }
    }
}
