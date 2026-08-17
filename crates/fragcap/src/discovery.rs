// SPDX-License-Identifier: Apache-2.0

//! Platform discovery source adapters (slice S052), behind the `targets` feature.
//!
//! The pure discovery model, the seam, the account, the known-roots and
//! user-pointed sources, and the volume eligibility store, lives in
//! `fragcap-targets`. The two adapters that touch a platform live here in the
//! facade, the one crate that legitimately depends on both `fragcap-steam` and
//! `fragcap-targets` (the S07/S08/S038 composition precedent): [`SteamSource`],
//! which expresses the existing Steam library walk as a
//! [`fragcap_targets::TargetSource`], and [`WindowsVolumeInventory`], the real
//! fixed-volume enumeration the known-roots walk consumes.
//!
//! `SteamSource` is a thin wrapper: it calls `fragcap-steam`'s library walk
//! unchanged, so the observable set of Steam candidates does not change (FR-006),
//! maps each installed title to a candidate at heuristic-unverified fidelity, and
//! joins the title's appid against the shipped catalog for a classification. A
//! title whose appid is not a number is counted `parse_failed` and the rest
//! survive (P-4); an appid absent from the catalog is classified `Unknown`, never
//! dropped (P-9). A manifest that was present but would not parse is a title the
//! walk omitted: `fragcap-steam` counts them, and this adapter folds that count
//! into the discovery account's `parse_failed` and surfaces every Steam warning on
//! the discovery result, so a damaged install cannot omit games while the account
//! reports a clean run (P-4).

use std::path::{Path, PathBuf};

use fragcap_profile::signature::SignatureSet;
use fragcap_profile::{DetectionFinding, FidelityTier};
use fragcap_targets::{
    CandidateIdentity, CandidateTarget, Discovery, DiscoveryAccount, Store, TargetClassification,
    TargetSource, TargetsError,
};

/// Detect the technologies in an install directory (slice S053), returning the
/// fidelity the local evidence earns and the findings as neutral evidence. A
/// detected engine raises the fidelity to what its signature earns (a definitive
/// marker is verified, which outranks a remote catalog attribution, P-9); a subtree
/// or root that could not be read is surfaced into `warnings` rather than dropped
/// (P-4).
fn detect_evidence(
    signatures: &SignatureSet,
    install_dir: &Path,
    warnings: &mut Vec<String>,
) -> (FidelityTier, Vec<DetectionFinding>) {
    match signatures.detect(install_dir) {
        Ok(outcome) => {
            for path in &outcome.unreadable {
                warnings.push(format!(
                    "could not read subtree during detection: {}",
                    path.display()
                ));
            }
            let fidelity = outcome
                .detected_engine()
                .map(|e| e.fidelity)
                .unwrap_or(FidelityTier::HeuristicUnverified);
            (fidelity, outcome.findings)
        }
        Err(e) => {
            warnings.push(format!(
                "could not read install directory during detection: {}",
                e.path.display()
            ));
            (FidelityTier::HeuristicUnverified, Vec::new())
        }
    }
}

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
}

impl TargetSource for SteamSource<'_> {
    fn name(&self) -> &str {
        "steam"
    }

    fn discover(&self) -> Result<Discovery, TargetsError> {
        let installation = fragcap_steam::discover_in(&self.steam_root)
            .map_err(|e| TargetsError::Discovery(format!("steam discovery failed: {e}")))?;

        // Detection is signature-driven and runs in every source's scan phase
        // (FR-006): load the catalog's signatures once and classify each installed
        // title's install directory below.
        let signature_set = SignatureSet::compile(&self.catalog.load_signatures()?);
        let mut warnings = installation.warnings;

        let mut account = DiscoveryAccount::default();
        // A manifest that was present but would not parse is one title omitted from
        // the walk. Count each as considered-and-parse-failed so the account
        // reflects the loss rather than silently reporting a clean run (P-4); the
        // per-manifest reason is surfaced through the warnings below.
        account.considered += installation.malformed_manifests;
        account.parse_failed += installation.malformed_manifests;
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
            // Scan the install directory for technologies (FR-006): a detected engine
            // rides as evidence and raises the fidelity to `verified`, outranking the
            // remote catalog attribution (P-9); any anti-cheat or DRM rides as neutral
            // evidence. A title with no local engine keeps heuristic-unverified.
            let (fidelity, evidence) =
                detect_evidence(&signature_set, &title.install_dir, &mut warnings);
            candidates.push(CandidateTarget {
                identity: CandidateIdentity::SteamAppId(appid),
                display_name: title.name.clone(),
                fidelity,
                classification,
                evidence,
                source_name: self.name().to_string(),
            });
        }
        Ok(Discovery {
            candidates,
            account,
            // Surface every non-fatal Steam diagnostic (a malformed manifest, a
            // duplicate appid, an unreadable library) plus any detection coverage gap,
            // so an omission is visible rather than left on an unused side channel.
            warnings,
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
            if ok == 0 {
                // No stable volume GUID: this is exactly the userspace or
                // misreporting mount the eligibility allowlist exists to keep out.
                // Falling back to the reassignable drive letter as the identity
                // would let a later, different volume reusing that letter inherit
                // this one's eligibility, so the volume is instead omitted from the
                // inventory entirely and is never walked (research.md D3).
                continue;
            }
            let len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
            let identity = String::from_utf16_lossy(&name[..len]);

            out.push(Volume {
                identity,
                mount_point: format!("{letter}:"),
                drive_type: DriveType::Fixed,
            });
        }
        out
    }
}
