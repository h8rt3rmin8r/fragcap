// SPDX-License-Identifier: Apache-2.0

//! Platform discovery source adapters (slice S052), behind the `targets` feature.
//!
//! The pure discovery model, the seam, the account, the known-roots and
//! user-pointed sources, and the volume eligibility store, lives in
//! `fragcap-targets`. The two adapters that touch a platform live here in the
//! facade, the one crate that legitimately depends on both `fragcap-steam` and
//! `fragcap-targets` (the S07/S08/S038 composition precedent). This module carries
//! [`SteamSource`], which expresses the existing Steam library walk as a
//! [`fragcap_targets::TargetSource`]; the real Windows volume inventory adapter
//! joins it here in a later task.
//!
//! `SteamSource` is a thin wrapper: it calls `fragcap-steam`'s library walk
//! unchanged, so the observable set of Steam candidates does not change (FR-006),
//! maps each installed title to a candidate at heuristic-unverified fidelity, and
//! joins the title's appid against the shipped catalog for a classification. A
//! title whose appid is not a number is counted `parse_failed` and the rest
//! survive (P-4); an appid absent from the catalog is classified `Unknown`, never
//! dropped (P-9). Manifest-level faults (a malformed `appmanifest`) are
//! `fragcap-steam`'s own accounting: it drops them from the title set and reports
//! them as enumeration warnings, which [`SteamSource::warnings`] surfaces.

use std::path::{Path, PathBuf};

use fragcap_profile::FidelityTier;
use fragcap_targets::{
    CandidateIdentity, CandidateTarget, Discovery, DiscoveryAccount, Store, TargetClassification,
    TargetSource, TargetsError,
};

/// Tier 1 discovery: Steam. Wraps `fragcap-steam`'s library walk and joins each
/// installed title's appid to the shipped catalog.
pub struct SteamSource<'a> {
    steam_root: PathBuf,
    catalog: &'a Store,
}

impl<'a> SteamSource<'a> {
    /// Build a Steam source over a Steam installation root and the catalog store
    /// the appids join against.
    pub fn new(steam_root: impl AsRef<Path>, catalog: &'a Store) -> Self {
        SteamSource {
            steam_root: steam_root.as_ref().to_path_buf(),
            catalog,
        }
    }

    /// The enumeration warnings from the most recent walk are not collected here;
    /// callers that want the manifest-level diagnostics call
    /// [`fragcap_steam::discover_in`] directly. This method exists so the
    /// composition point is documented: `fragcap-steam` owns manifest-fault
    /// accounting, `SteamSource` owns title-to-candidate accounting.
    pub fn warnings(&self) -> Result<Vec<String>, TargetsError> {
        let installation = fragcap_steam::discover_in(&self.steam_root)
            .map_err(|e| TargetsError::Discovery(format!("steam discovery failed: {e}")))?;
        Ok(installation.warnings)
    }
}

impl TargetSource for SteamSource<'_> {
    fn name(&self) -> &str {
        "steam"
    }

    fn discover(&self) -> Result<Discovery, TargetsError> {
        let installation = fragcap_steam::discover_in(&self.steam_root)
            .map_err(|e| TargetsError::Discovery(format!("steam discovery failed: {e}")))?;

        let mut account = DiscoveryAccount::default();
        let mut candidates = Vec::new();
        for title in &installation.titles {
            account.considered += 1;
            let appid = match title.app_id.parse::<u32>() {
                Ok(appid) => appid,
                // A title whose appid is not a number cannot be used as a platform
                // identity or joined to the catalog: counted, never dropped (P-4).
                Err(_) => {
                    account.parse_failed += 1;
                    continue;
                }
            };
            // A catalog hit classifies the title a game; an absent appid stays
            // Unknown, never guessed (P-9). A store error aborts the run.
            let classification = match self.catalog.game(appid)? {
                Some(_) => TargetClassification::Game,
                None => TargetClassification::Unknown,
            };
            account.produced += 1;
            candidates.push(CandidateTarget {
                identity: CandidateIdentity::SteamAppId(appid),
                display_name: title.name.clone(),
                fidelity: self.default_fidelity(),
                classification,
                source_name: self.name().to_string(),
            });
        }
        Ok(Discovery {
            candidates,
            account,
        })
    }

    fn default_fidelity(&self) -> FidelityTier {
        FidelityTier::HeuristicUnverified
    }
}

/// The real Windows fixed-volume inventory (slice S052), spec 7.4.
///
/// Enumerates the machine's drive letters through `GetLogicalDrives`, keeps only
/// those `GetDriveTypeW` reports as fixed, and reads each one's stable volume GUID
/// path through `GetVolumeNameForVolumeMountPointW` as its identity (the drive
/// letter is the mutable mount point, never the key: a reassigned letter must not
/// inherit a prior volume's eligibility, research.md D3). It feeds
/// [`fragcap_targets::KnownRootsSource`]; the tests drive that source with a
/// fixture inventory instead, so this adapter is exercised only on a real machine.
#[cfg(windows)]
pub struct WindowsVolumeInventory;

#[cfg(windows)]
impl WindowsVolumeInventory {
    /// Build the Windows volume inventory adapter.
    pub fn new() -> Self {
        WindowsVolumeInventory
    }
}

#[cfg(windows)]
impl Default for WindowsVolumeInventory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(windows)]
impl fragcap_targets::VolumeInventory for WindowsVolumeInventory {
    fn fixed_volumes(&self) -> Vec<fragcap_targets::Volume> {
        use fragcap_targets::{DriveType, Volume};
        use windows_sys::Win32::Storage::FileSystem::{
            GetDriveTypeW, GetLogicalDrives, GetVolumeNameForVolumeMountPointW,
        };

        // The Win32 drive-type return codes (winbase.h). Hard-coded rather than
        // pulled from a version-specific constant so the adapter does not depend on
        // which windows-sys line exposes them.
        const DRIVE_FIXED: u32 = 3;

        fn wide(s: &str) -> Vec<u16> {
            s.encode_utf16().chain(std::iter::once(0)).collect()
        }

        // SAFETY: GetLogicalDrives takes no arguments and returns a bitmask.
        let mask = unsafe { GetLogicalDrives() };
        let mut out = Vec::new();
        for i in 0..26u32 {
            if mask & (1 << i) == 0 {
                continue;
            }
            let letter = (b'A' + i as u8) as char;
            // GetDriveTypeW and the volume-name lookup want the root with a
            // trailing separator ("C:\"); the recorded mount point drops it.
            let root = wide(&format!("{letter}:\\"));

            // SAFETY: `root` is a live, null-terminated UTF-16 string.
            let drive_type = unsafe { GetDriveTypeW(root.as_ptr()) };
            if drive_type != DRIVE_FIXED {
                continue;
            }

            // "\\?\Volume{GUID}\" is 49 UTF-16 code units; MAX_PATH is ample.
            let mut name = [0u16; 260];
            // SAFETY: `root` is a live null-terminated string; `name` is a live
            // buffer of `name.len()` code units the API fills and null-terminates.
            let ok = unsafe {
                GetVolumeNameForVolumeMountPointW(
                    root.as_ptr(),
                    name.as_mut_ptr(),
                    name.len() as u32,
                )
            };
            let identity = if ok != 0 {
                let len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
                String::from_utf16_lossy(&name[..len])
            } else {
                // No volume GUID available (some userspace mounts): fall back to the
                // drive letter. The allowlist still requires an explicit opt-in for
                // an unseen volume, so a misreporting mount is not silently walked.
                format!("{letter}:")
            };

            out.push(Volume {
                identity,
                mount_point: format!("{letter}:"),
                drive_type: DriveType::Fixed,
            });
        }
        out
    }
}
