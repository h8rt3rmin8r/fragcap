// SPDX-License-Identifier: Apache-2.0

//! Local Steam launch-data accumulation (issue #78, slice S038).
//!
//! The orchestrator that learns a user's own game launch executables from their
//! local Steam appinfo cache and records them in their private local hint store,
//! accumulating over runs and refreshing only what Steam changed. It owns the
//! honest account of one walk (P-4, P-9), and it is the only place that can: the
//! considered set and the parse outcomes come from `fragcap-steam`, the stored
//! change-numbers and the merges from `fragcap-targets`, and those two are
//! siblings that may not depend on each other. This crate is the one that depends
//! on both, so the composition lives here (the S07/S08 facade-test precedent).
//!
//! It is passive: it reads files Steam already wrote and writes only launch data
//! to a local store. No network, no process handle (P-1). It ships nothing about
//! anyone's library; a future opt-in community pool is deferred (issue #94).

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::Path;

use fragcap_steam::{AppInfoReader, SectionInfo, SteamError, SteamInstallation, SteamLaunchEntry};
use fragcap_targets::{LaunchEntry, Store, TargetsError};

/// The reconciled account of one accumulation walk.
///
/// The four per-application outcomes are mutually exclusive and reconcile to the
/// number of applications considered (P-4): a partial or interrupted walk can
/// never read as a complete one. `file_faults` is a separate axis, the count of
/// file-level parse faults (an unreadable header, a truncated tail) that belong to
/// the file rather than to any considered application; it is surfaced, not folded
/// into the per-application identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LaunchAccumulationSummary {
    /// Installed applications examined.
    pub considered: u64,
    /// Applications whose launch data was written because it was missing or stale.
    pub written: u64,
    /// Applications whose stored change-number already matched the cache.
    pub skipped: u64,
    /// Applications whose appinfo section could not be parsed.
    pub failed: u64,
    /// Applications considered that yielded no storable launch entry (absent from
    /// the cache, or present with no executable). Not a failure (FR-009).
    pub empty: u64,
    /// File-level appinfo parse faults, not attributable to a considered
    /// application (for example an unrecognized magic or a truncated tail).
    pub file_faults: u64,
}

impl LaunchAccumulationSummary {
    /// The conservation identity: every considered application is written,
    /// skipped, failed, or empty.
    pub fn is_conserved(&self) -> bool {
        self.considered == self.written + self.skipped + self.failed + self.empty
    }
}

/// Progress through a walk, reported per considered application so a caller can
/// print a bounded line and a slow first run reads as working rather than hung
/// (FR-010).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccumulationProgress {
    /// Applications considered so far, one-based.
    pub done: usize,
    /// Total applications to consider.
    pub total: usize,
}

/// A fault that aborts the whole walk: a Steam metadata read error, or a store
/// error after the store was opened. A per-application parse failure is not this;
/// it is counted in the summary and the walk continues (FR-008).
#[derive(Debug)]
pub enum AccumulationError {
    /// Enumerating the library or reading the appinfo cache failed.
    Steam(SteamError),
    /// Reading or writing the local store failed.
    Store(TargetsError),
}

impl fmt::Display for AccumulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccumulationError::Steam(e) => write!(f, "reading Steam metadata: {e}"),
            AccumulationError::Store(e) => write!(f, "writing the local hint store: {e}"),
        }
    }
}

impl std::error::Error for AccumulationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AccumulationError::Steam(e) => Some(e),
            AccumulationError::Store(e) => Some(e),
        }
    }
}

/// Accumulate launch data from the local Steam installation, discovering the
/// Steam root through the platform.
///
/// When Steam is not installed (or not on this platform) there is nothing to walk:
/// this returns a zero-considered summary and does not fail, so a capture proceeds
/// without it. Any other Steam error, and any store error, aborts and is returned.
pub fn accumulate_from_local_steam(
    store: &mut Store,
    report: &mut dyn FnMut(AccumulationProgress),
) -> Result<LaunchAccumulationSummary, AccumulationError> {
    let installation = match fragcap_steam::discover() {
        Ok(installation) => installation,
        // No Steam, or not this platform: nothing to accumulate, not an error.
        Err(SteamError::NotInstalled) | Err(SteamError::UnsupportedPlatform) => {
            return Ok(LaunchAccumulationSummary::default());
        }
        Err(e) => return Err(AccumulationError::Steam(e)),
    };
    accumulate_launch_data(&installation.root, store, report)
}

/// Accumulate launch data for the installed library under `root` into `store`,
/// reporting progress.
///
/// Portable (takes a root), so it is tested against a fixture Steam-root tree and
/// an in-memory store. The considered set is the installed library; the appinfo
/// cache is the launch-data source. Never prunes: an installed application absent
/// from the cache leaves any stored launch data in place.
pub fn accumulate_launch_data(
    root: &Path,
    store: &mut Store,
    report: &mut dyn FnMut(AccumulationProgress),
) -> Result<LaunchAccumulationSummary, AccumulationError> {
    let installation = fragcap_steam::discover_in(root).map_err(AccumulationError::Steam)?;

    // No appinfo cache is not an error: every installed title is considered and
    // yields nothing to learn.
    let Some(bytes) = fragcap_steam::read_appinfo_bytes(root).map_err(AccumulationError::Steam)?
    else {
        return Ok(empty_walk(&installation, report));
    };

    // A file-level header fault (bad magic, a malformed string table) means nothing
    // in the file can be trusted: record nothing and surface the fault (spec
    // 15.6.2, P-4). No per-title outcome is produced, so the conservation identity
    // holds trivially (considered stays zero).
    let mut reader = match AppInfoReader::open(&bytes) {
        Ok(reader) => reader,
        Err(_) => {
            return Ok(LaunchAccumulationSummary {
                file_faults: 1,
                ..Default::default()
            })
        }
    };

    // Pass 1: stream section headers, collecting the installed applications'
    // sections and detecting any file-level framing fault, without decoding any
    // key-values body. A truncated tail or a broken frame is a top-level malformed
    // file: record nothing rather than write a valid-looking prefix (the review's
    // P1, spec 15.6.2).
    let installed_ids: HashSet<u32> = installation
        .titles
        .iter()
        .filter_map(|t| t.app_id.parse::<u32>().ok())
        .collect();
    let mut sections: HashMap<u32, SectionInfo> = HashMap::new();
    loop {
        match reader.next_section() {
            None => break, // clean terminator
            Some(Err(_)) => {
                return Ok(LaunchAccumulationSummary {
                    file_faults: 1,
                    ..Default::default()
                })
            }
            Some(Ok(section)) => {
                if installed_ids.contains(&section.appid) {
                    sections.insert(section.appid, section);
                }
            }
        }
    }

    // Pass 2: classify each installed title, decoding a body only for one that is
    // present and stale. An unchanged application is skipped from its cheap section
    // header, never decoded, which is the near-zero repeat-run path (the review's
    // P2).
    let total = installation.titles.len();
    let mut summary = LaunchAccumulationSummary::default();
    for (i, title) in installation.titles.iter().enumerate() {
        summary.considered += 1;
        report(AccumulationProgress { done: i + 1, total });

        // A non-numeric appid cannot key the cache; it yields nothing, not a fault.
        let Ok(appid) = title.app_id.parse::<u32>() else {
            summary.empty += 1;
            continue;
        };
        let Some(section) = sections.get(&appid) else {
            // Installed but absent from the cache: nothing to learn, never pruned.
            summary.empty += 1;
            continue;
        };

        // Staleness by change-number, decided before decoding: skip when the store
        // already holds this version (FR-011a).
        let stored = store
            .stored_change_number(appid)
            .map_err(AccumulationError::Store)?;
        if let Some(stored) = stored {
            if section.change_number <= stored {
                summary.skipped += 1;
                continue;
            }
        }

        // Decode the body only now, for a present, stale application.
        match reader.launch_entries(section) {
            Ok(steam_entries) => {
                let entries: Vec<LaunchEntry> =
                    steam_entries.iter().filter_map(to_launch_entry).collect();
                if entries.is_empty() {
                    summary.empty += 1;
                } else {
                    store
                        .merge_launch(appid, section.change_number, &entries)
                        .map_err(AccumulationError::Store)?;
                    summary.written += 1;
                }
            }
            // A section whose framing was intact but whose key-values body is
            // malformed (an unknown type, a truncation inside an object): counted,
            // not fatal (FR-008).
            Err(_) => summary.failed += 1,
        }
    }

    debug_assert!(
        summary.is_conserved(),
        "outcomes must reconcile to considered"
    );
    Ok(summary)
}

/// Report every installed title as considered-but-empty, used when no appinfo
/// cache is present (a Steam that has fetched nothing).
fn empty_walk(
    installation: &SteamInstallation,
    report: &mut dyn FnMut(AccumulationProgress),
) -> LaunchAccumulationSummary {
    let total = installation.titles.len();
    let mut summary = LaunchAccumulationSummary::default();
    for (i, _title) in installation.titles.iter().enumerate() {
        summary.considered += 1;
        summary.empty += 1;
        report(AccumulationProgress { done: i + 1, total });
    }
    summary
}

/// Map a boundary-neutral Steam launch entry to the store's launch entry, verbatim.
/// `None` only if the executable is empty, which the parser already excludes.
fn to_launch_entry(s: &SteamLaunchEntry) -> Option<LaunchEntry> {
    let mut entry = LaunchEntry::new(&s.executable).ok()?;
    entry.os = s.os.clone();
    entry.osarch = s.osarch.clone();
    entry.launch_type = s.launch_type.clone();
    entry.beta_branch = s.beta_branch.clone();
    entry.arguments = s.arguments.clone();
    entry.description = s.description.clone();
    Some(entry)
}
