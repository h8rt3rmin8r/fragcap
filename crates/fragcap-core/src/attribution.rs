// SPDX-License-Identifier: Apache-2.0

//! What a flow belongs to: a process, its role, and the launcher chain stage it
//! was resolved at.
//!
//! Nothing in this module reaches into a process. Attribution is reconstructed
//! from outside, from the socket table and from creation-time ancestry, which
//! is what constitution P-1 requires and what slice S10 implements.

use std::sync::Arc;

/// Identifies a stage in a launcher chain.
///
/// A profile in slice S05 names the stages of a title's launch, and slice S12
/// matches an observed process tree against them. Modelled as a shared
/// immutable string rather than an index, because an index is only meaningful
/// relative to the profile that produced it, and an attribution travels into an
/// output file without carrying its profile.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StageId(Arc<str>);

impl StageId {
    pub fn new(name: impl AsRef<str>) -> Self {
        StageId(Arc::from(name.as_ref()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for StageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The process a flow belongs to.
///
/// `process` and `role` are reference-counted shared strings because both are
/// drawn from a small set and repeat on every packet of a flow. Copying either
/// per packet would be an allocation on the hot path for no new information.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Attribution {
    /// Operating system process identifier.
    pub pid: u32,
    /// Process image name.
    pub process: Arc<str>,
    /// Role from the matching profile, when one matched. Slice S05 and S12.
    pub role: Option<Arc<str>>,
    /// Launcher chain stage, when one was resolved. Slice S12.
    pub stage: Option<StageId>,
}

impl Attribution {
    /// The minimum an attribution can carry: a process, named.
    pub fn new(pid: u32, process: impl AsRef<str>) -> Self {
        Attribution {
            pid,
            process: Arc::from(process.as_ref()),
            role: None,
            stage: None,
        }
    }

    pub fn with_role(mut self, role: impl AsRef<str>) -> Self {
        self.role = Some(Arc::from(role.as_ref()));
        self
    }

    pub fn with_stage(mut self, stage: StageId) -> Self {
        self.stage = Some(stage);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // V-4. The reason these fields are shared rather than owned.
    #[test]
    fn cloning_shares_the_process_name_rather_than_allocating() {
        let a = Attribution::new(4242, "eso64.exe");
        let b = a.clone();
        assert!(
            Arc::ptr_eq(&a.process, &b.process),
            "cloning must share the name, not copy it"
        );
    }

    #[test]
    fn cloning_shares_the_role_too() {
        let a = Attribution::new(4242, "eso64.exe").with_role("game-client");
        let b = a.clone();
        let (ra, rb) = (a.role.as_ref().unwrap(), b.role.as_ref().unwrap());
        assert!(Arc::ptr_eq(ra, rb));
    }

    #[test]
    fn a_minimal_attribution_has_no_role_or_stage() {
        let a = Attribution::new(1, "x.exe");
        assert!(a.role.is_none());
        assert!(a.stage.is_none());
    }

    #[test]
    fn role_and_stage_are_attachable() {
        let a = Attribution::new(1, "x.exe")
            .with_role("game-client")
            .with_stage(StageId::new("launcher"));
        assert_eq!(a.role.as_deref(), Some("game-client"));
        assert_eq!(a.stage.as_ref().map(StageId::as_str), Some("launcher"));
    }

    #[test]
    fn stage_ids_compare_by_name() {
        assert_eq!(StageId::new("launcher"), StageId::new("launcher"));
        assert_ne!(StageId::new("launcher"), StageId::new("client"));
        assert_eq!(StageId::new("launcher").to_string(), "launcher");
    }
}
