// SPDX-License-Identifier: Apache-2.0

//! Volume identity and the volume eligibility allowlist (slice S052), spec 7.4.
//!
//! The cross-volume known-roots walk enumerates known roots on every eligible
//! fixed volume, so it needs a persistent, user-editable record of which volumes
//! it may touch. That record is an allowlist, not a denylist: a static denylist
//! cannot recognize a userspace or FUSE mount that presents itself as an ordinary
//! fixed drive (the operator's box presents two RustFS buckets reporting fixed),
//! so eligibility is decided by what the user has affirmed. On first run the
//! allowlist is seeded with the fixed volumes then present; from then on a volume
//! not recorded eligible is not walked until an explicit opt-in.
//!
//! A volume is keyed on a stable identity that survives drive-letter reassignment
//! (the volume GUID path; the serial is an acceptable fallback), because a
//! reassigned letter must not inherit a prior volume's eligibility (research.md
//! D3). The [`VolumeInventory`] seam supplies the live volume list, injected so the
//! walk is testable with no platform dependency (FR-019); the real Windows adapter
//! lives in the facade.

/// The type of a volume as the operating system reports it. Only [`DriveType::Fixed`]
/// volumes are candidates for the walk; the others are recorded for context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriveType {
    /// A fixed disk (the only type the walk considers).
    Fixed,
    /// Removable media.
    Removable,
    /// A network drive.
    Remote,
    /// Optical media.
    CdRom,
    /// A RAM disk.
    RamDisk,
    /// The OS could not classify it, or the type is not one of the above.
    Unknown,
}

impl DriveType {
    /// The stored/display string.
    pub fn as_str(self) -> &'static str {
        match self {
            DriveType::Fixed => "fixed",
            DriveType::Removable => "removable",
            DriveType::Remote => "remote",
            DriveType::CdRom => "cdrom",
            DriveType::RamDisk => "ramdisk",
            DriveType::Unknown => "unknown",
        }
    }

    /// Parse a stored string, mapping any unrecognized value to [`DriveType::Unknown`].
    pub fn parse(s: &str) -> DriveType {
        match s {
            "fixed" => DriveType::Fixed,
            "removable" => DriveType::Removable,
            "remote" => DriveType::Remote,
            "cdrom" => DriveType::CdRom,
            "ramdisk" => DriveType::RamDisk,
            _ => DriveType::Unknown,
        }
    }
}

/// A volume the inventory reports.
///
/// `identity` is the stable key (the volume GUID path); `mount_point` is the
/// current drive letter or mount path, a mutable display attribute only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Volume {
    /// The stable identity that survives drive-letter reassignment.
    pub identity: String,
    /// The current drive letter or mount path (display; mutable).
    pub mount_point: String,
    /// The drive type observed.
    pub drive_type: DriveType,
}

/// Why a volume is or is not eligible, recorded so each decision is statable
/// (FR-017).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EligibilityReason {
    /// Recorded eligible by the permissive first-run seeding (FR-016a).
    SeededFirstRun,
    /// Recorded eligible by an explicit user opt-in.
    UserAdded,
    /// Recorded ineligible by an explicit user exclusion.
    UserExcluded,
}

impl EligibilityReason {
    /// The stored string, matching the schema CHECK set.
    pub fn as_str(self) -> &'static str {
        match self {
            EligibilityReason::SeededFirstRun => "seeded-first-run",
            EligibilityReason::UserAdded => "user-added",
            EligibilityReason::UserExcluded => "user-excluded",
        }
    }

    /// Parse a stored string, rejecting an out-of-set value.
    pub fn parse(s: &str) -> Option<EligibilityReason> {
        match s {
            "seeded-first-run" => Some(EligibilityReason::SeededFirstRun),
            "user-added" => Some(EligibilityReason::UserAdded),
            "user-excluded" => Some(EligibilityReason::UserExcluded),
            _ => None,
        }
    }
}

/// One recorded eligibility decision, read back from the store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VolumeEligibility {
    /// The stable volume identity.
    pub volume_id: String,
    /// The mount point last recorded (display).
    pub mount_point: Option<String>,
    /// The drive type last recorded.
    pub drive_type: Option<String>,
    /// Whether the volume is eligible for the walk.
    pub eligible: bool,
    /// Why (recorded so the decision is statable).
    pub reason: EligibilityReason,
    /// When the volume was first recorded, or `None`.
    pub first_seen: Option<String>,
}

/// A source of the machine's fixed volumes. Injected so the cross-volume walk is a
/// pure decision over a value in tests (FR-019); the real Windows adapter over
/// `GetLogicalDrives`/`GetDriveTypeW` lives in the facade.
pub trait VolumeInventory {
    /// The fixed volumes the machine currently presents.
    fn fixed_volumes(&self) -> Vec<Volume>;
}

/// A fixture inventory returning a canned volume list. Keeps the walk testable
/// offline (the same posture as [`crate::FixtureCatalog`]).
pub struct FixtureInventory {
    volumes: Vec<Volume>,
}

impl FixtureInventory {
    /// Build a fixture inventory from a canned volume list.
    pub fn new(volumes: Vec<Volume>) -> Self {
        FixtureInventory { volumes }
    }
}

impl VolumeInventory for FixtureInventory {
    fn fixed_volumes(&self) -> Vec<Volume> {
        self.volumes.clone()
    }
}
