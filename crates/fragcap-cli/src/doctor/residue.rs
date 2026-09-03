// SPDX-License-Identifier: Apache-2.0

//! Bounded native Deep Capture ownership and residue inventory.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use fragcap::deep_capture::{JournalStatus, ResourceState, RESOURCE_JOURNAL};

pub(crate) const SESSION_OWNER_REGISTRY: &str = "session-owners";
const OWNER_VERSION: u64 = 2;
const MAX_OWNER_BYTES: usize = 4096;
const MAX_OWNER_RECORDS: usize = 4096;
const MAX_SCAN_ENTRIES: usize = 200;
const MAX_SCAN_DEPTH: usize = 3;
const MAX_FINDINGS: usize = 200;
static LEASE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Doctor's stable health projection over native lifecycle evidence.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ResidueHealth {
    /// Completed terminal history.
    Healthy,
    /// A matching generation lease proves the session is live.
    Active,
    /// The owning generation ended with a nonterminal obligation.
    Stale,
    /// Cleanup failed or timed out.
    CleanupFailed,
    /// Evidence could not be classified safely.
    Unknown,
    /// Native Deep Capture is unavailable on this platform.
    Unsupported,
}

impl ResidueHealth {
    /// The stable machine-facing health word.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Active => "active",
            Self::Stale => "stale",
            Self::CleanupFailed => "cleanup-failed",
            Self::Unknown => "unknown",
            Self::Unsupported => "unsupported",
        }
    }
}

/// One latest native resource state and its exact recovery eligibility.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceFinding {
    /// Session identifier from the journal.
    pub session_id: String,
    /// Canonical bundle whose authority supplied the finding.
    pub bundle: PathBuf,
    /// Stable journal resource identifier.
    pub resource_id: String,
    /// Journal resource kind.
    pub kind: String,
    /// Latest journal lifecycle state.
    pub state: String,
    /// Derived diagnostic health.
    pub health: ResidueHealth,
    /// Whether the shared journal plan contains an exact action.
    pub recoverable: bool,
    /// Stable authority that supports this classification.
    pub ownership_authority: String,
    /// Bounded non-secret explanation.
    pub detail: String,
}

/// One bounded, read-only native session inventory.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NativeResidueInventory {
    /// Resource findings in stable session and resource order.
    pub findings: Vec<ResourceFinding>,
    /// Explicit scan, parse, version, and ownership limitations.
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SessionOwner {
    pub(crate) bundle: PathBuf,
    pub(crate) owner_pid: u32,
    pub(crate) lease_id: Option<String>,
    pub(crate) registry_path: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OwnerActivity {
    Active,
    Inactive,
    LegacyUnproven,
}

/// A generation-specific owner proof retained for the exact session lifetime.
#[derive(Debug)]
pub(crate) struct SessionOwnerLease {
    #[cfg(not(windows))]
    lease_id: String,
    #[cfg(windows)]
    handle: isize,
}

impl Drop for SessionOwnerLease {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.handle);
        }
        #[cfg(not(windows))]
        test_leases()
            .lock()
            .expect("lease registry lock")
            .remove(&self.lease_id);
    }
}

pub(crate) fn register_session_owner(root: &Path, bundle: &Path) -> io::Result<SessionOwnerLease> {
    let registry = root.join(SESSION_OWNER_REGISTRY);
    std::fs::create_dir_all(&registry)?;
    let bundle = bundle.canonicalize()?;
    let owner_pid = std::process::id();
    loop {
        let sequence = LEASE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let lease_id = format!("{owner_pid}-{epoch:x}-{sequence:x}");
        let lease = match create_lease(&lease_id)? {
            Some(lease) => lease,
            None => continue,
        };
        let path = registry.join(format!("{lease_id}.json"));
        let mut file = match std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        serde_json::to_writer(
            &mut file,
            &serde_json::json!({
                "version": OWNER_VERSION,
                "bundle": bundle,
                "owner_pid": owner_pid,
                "lease_id": lease_id,
            }),
        )
        .map_err(io::Error::other)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        return Ok(lease);
    }
}

pub(crate) fn registered_session_owners(root: &Path) -> io::Result<Vec<SessionOwner>> {
    let canonical_root = match root.canonicalize() {
        Ok(path) => path,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let path = canonical_root.join(SESSION_OWNER_REGISTRY);
    let entries = match std::fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut owners = Vec::new();
    let mut observed = 0usize;
    for entry in entries {
        observed += 1;
        if observed > MAX_OWNER_RECORDS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "session owner registry exceeds its record limit",
            ));
        }
        let registry_path = entry?.path();
        let metadata = registry_path.symlink_metadata()?;
        if !metadata.file_type().is_file() || metadata.len() as usize > MAX_OWNER_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "session owner registry entry is not a bounded regular file",
            ));
        }
        let value: serde_json::Value = serde_json::from_slice(&std::fs::read(&registry_path)?)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let version = value
            .get("version")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(1);
        if !matches!(version, 1 | OWNER_VERSION) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "session owner registry entry has an unsupported version",
            ));
        }
        let bundle = value
            .get("bundle")
            .and_then(serde_json::Value::as_str)
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "owner lacks bundle"))?;
        if !bundle.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "session owner bundle path is not absolute",
            ));
        }
        let canonical_bundle = if bundle.exists() {
            bundle.canonicalize()?
        } else {
            bundle
        };
        let owner_pid = value
            .get("owner_pid")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "owner lacks valid pid"))?;
        let lease_id = if version == OWNER_VERSION {
            Some(
                value
                    .get("lease_id")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| valid_lease_id(value))
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "owner lacks valid lease")
                    })?
                    .to_string(),
            )
        } else {
            None
        };
        owners.push(SessionOwner {
            bundle: canonical_bundle,
            owner_pid,
            lease_id,
            registry_path,
        });
    }
    owners.sort_by(|left, right| left.registry_path.cmp(&right.registry_path));
    let mut seen = BTreeSet::new();
    if owners
        .iter()
        .filter_map(|owner| owner.lease_id.clone())
        .any(|lease_id| !seen.insert(lease_id))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "duplicate session owner lease identity",
        ));
    }
    Ok(owners)
}

pub(crate) fn owner_is_active(owner: &SessionOwner) -> io::Result<bool> {
    match owner.lease_id.as_deref() {
        Some(lease_id) => lease_is_active(lease_id),
        None if !owner.bundle.exists() => Ok(false),
        None => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "legacy session owner has no generation lease",
        )),
    }
}

pub(crate) fn inventory(root: Option<&Path>) -> NativeResidueInventory {
    let Some(root) = root.filter(|path| path.is_dir()) else {
        return NativeResidueInventory::default();
    };
    let mut inventory = NativeResidueInventory::default();
    let owners = match registered_session_owners(root) {
        Ok(owners) => owners,
        Err(error) => {
            inventory
                .limitations
                .push(format!("owner-registry-invalid: {error}"));
            Vec::new()
        }
    };
    let mut owner_states = BTreeMap::new();
    for owner in &owners {
        match owner_is_active(owner) {
            Ok(active) => {
                owner_states.insert(
                    owner.bundle.clone(),
                    if active {
                        OwnerActivity::Active
                    } else {
                        OwnerActivity::Inactive
                    },
                );
            }
            Err(error)
                if error.kind() == io::ErrorKind::Unsupported && owner.lease_id.is_none() =>
            {
                owner_states.insert(owner.bundle.clone(), OwnerActivity::LegacyUnproven);
            }
            Err(error) => inventory.limitations.push(format!(
                "owner-lease-undetermined for {}: {error}",
                owner.bundle.display()
            )),
        }
    }

    let mut journals = Vec::new();
    let mut observed_files = Vec::new();
    let mut visited = 0usize;
    let mut scan_roots = vec![root.to_path_buf()];
    scan_roots.extend(owners.iter().map(|owner| owner.bundle.clone()));
    scan_roots.sort();
    scan_roots.dedup();
    for scan_root in scan_roots {
        walk_journals(
            &scan_root,
            0,
            &mut visited,
            &mut journals,
            &mut observed_files,
            &mut inventory.limitations,
        );
    }
    journals.sort();
    journals.dedup();
    observed_files.sort();
    observed_files.dedup();
    let mut journal_bundles = BTreeSet::new();
    for journal in journals {
        let bundle = journal.parent().unwrap_or(root).to_path_buf();
        journal_bundles.insert(bundle.clone());
        let owner_activity = owner_states
            .get(&bundle)
            .copied()
            .unwrap_or(OwnerActivity::Inactive);
        match fragcap::deep_capture::read_resource_journal(&journal) {
            Ok(prefix) if prefix.status == JournalStatus::UnknownVersion => {
                push_finding(
                    &mut inventory,
                    ResourceFinding {
                        session_id: prefix.session_id,
                        bundle: bundle.clone(),
                        resource_id: "journal".to_string(),
                        kind: "journal".to_string(),
                        state: "unknown-version".to_string(),
                        health: ResidueHealth::Unknown,
                        recoverable: false,
                        ownership_authority: "resource-journal".to_string(),
                        detail: "resource journal version is unsupported".to_string(),
                    },
                );
            }
            Ok(prefix) => {
                let plan = prefix.recovery_plan();
                for transition in prefix.latest().into_values() {
                    let recoverable = plan
                        .actions
                        .iter()
                        .any(|action| action.resource_id == transition.resource_id);
                    let health = if transition.state.terminal() {
                        ResidueHealth::Healthy
                    } else if owner_activity == OwnerActivity::Active {
                        ResidueHealth::Active
                    } else if owner_activity == OwnerActivity::LegacyUnproven {
                        ResidueHealth::Unknown
                    } else if matches!(
                        transition.state,
                        ResourceState::Failed | ResourceState::TimedOut
                    ) {
                        ResidueHealth::CleanupFailed
                    } else {
                        ResidueHealth::Stale
                    };
                    push_finding(
                        &mut inventory,
                        ResourceFinding {
                            session_id: prefix.session_id.clone(),
                            bundle: bundle.clone(),
                            resource_id: transition.resource_id.clone(),
                            kind: transition.kind.as_str().to_string(),
                            state: transition.state.as_str().to_string(),
                            health,
                            recoverable,
                            ownership_authority: "resource-journal".to_string(),
                            detail: if recoverable
                                && owner_activity == OwnerActivity::LegacyUnproven
                            {
                                "exact journal recovery requires explicit operator confirmation because legacy ownership has no generation lease"
                                    .to_string()
                            } else if recoverable {
                                "exact journal recovery action is available".to_string()
                            } else if transition.state.terminal() {
                                "resource reached a terminal journal state".to_string()
                            } else {
                                "journal recovery refused insufficient ownership or adapter authority"
                                    .to_string()
                            },
                        },
                    );
                }
                if prefix.status == JournalStatus::CrashPrefix && prefix.transitions.is_empty() {
                    push_finding(
                        &mut inventory,
                        ResourceFinding {
                            session_id: prefix.session_id,
                            bundle: bundle.clone(),
                            resource_id: "journal".to_string(),
                            kind: "journal".to_string(),
                            state: "crash-prefix".to_string(),
                            health: if owner_activity == OwnerActivity::Active {
                                ResidueHealth::Active
                            } else if owner_activity == OwnerActivity::LegacyUnproven {
                                ResidueHealth::Unknown
                            } else {
                                ResidueHealth::Stale
                            },
                            recoverable: owner_activity == OwnerActivity::LegacyUnproven,
                            ownership_authority: "resource-journal".to_string(),
                            detail: "journal has no resource transition to recover".to_string(),
                        },
                    );
                } else if prefix.status == JournalStatus::Complete && prefix.transitions.is_empty()
                {
                    push_finding(
                        &mut inventory,
                        ResourceFinding {
                            session_id: prefix.session_id,
                            bundle: bundle.clone(),
                            resource_id: "journal".to_string(),
                            kind: "journal".to_string(),
                            state: "complete".to_string(),
                            health: ResidueHealth::Healthy,
                            recoverable: false,
                            ownership_authority: "resource-journal".to_string(),
                            detail: "empty resource journal has a valid terminal trailer"
                                .to_string(),
                        },
                    );
                }
            }
            Err(error) => {
                inventory
                    .limitations
                    .push(format!("journal-invalid at {}: {error}", journal.display()));
            }
        }
    }
    for path in observed_files {
        let bundle = journal_bundles
            .iter()
            .filter(|bundle| path.starts_with(bundle))
            .max_by_key(|bundle| bundle.components().count())
            .cloned()
            .unwrap_or_else(|| path.parent().unwrap_or(root).to_path_buf());
        let owner_activity = owner_states
            .get(&bundle)
            .copied()
            .unwrap_or(OwnerActivity::Inactive);
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        let has_journal = journal_bundles.contains(&bundle);
        let (health, state, detail) = if owner_activity == OwnerActivity::Active {
            (
                ResidueHealth::Active,
                "present",
                "artifact belongs to a generation-proven active session",
            )
        } else if name == "manifest.json" && !super::probe::manifest_cleanup_unfinished(&path) {
            (
                ResidueHealth::Healthy,
                "complete",
                "completed manifest is retained historical evidence",
            )
        } else if owner_activity == OwnerActivity::LegacyUnproven {
            (
                ResidueHealth::Unknown,
                "legacy-owner-unproven",
                "artifact belongs to a legacy owner with no generation lease",
            )
        } else if has_journal {
            (
                ResidueHealth::Healthy,
                "retained",
                "artifact presence is governed by the session resource journal",
            )
        } else if name == "manifest.json" {
            (
                ResidueHealth::CleanupFailed,
                "cleanup-incomplete",
                "manifest reports incomplete cleanup without a readable resource journal",
            )
        } else {
            (
                ResidueHealth::Unknown,
                "unowned",
                "recognized sensitive artifact has no readable journal or complete manifest",
            )
        };
        push_finding(
            &mut inventory,
            ResourceFinding {
                session_id: bundle
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                bundle: bundle.clone(),
                resource_id: format!("artifact:{name}"),
                kind: "artifact".to_string(),
                state: state.to_string(),
                health,
                recoverable: false,
                ownership_authority: if has_journal {
                    "resource-journal"
                } else {
                    "manifest-or-path"
                }
                .to_string(),
                detail: detail.to_string(),
            },
        );
    }
    for owner in owners {
        if !journal_bundles.contains(&owner.bundle) {
            let owner_activity = owner_states
                .get(&owner.bundle)
                .copied()
                .unwrap_or(OwnerActivity::Inactive);
            push_finding(
                &mut inventory,
                ResourceFinding {
                    session_id: owner
                        .bundle
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    bundle: owner.bundle.clone(),
                    resource_id: "session-owner".to_string(),
                    kind: "owner".to_string(),
                    state: match owner_activity {
                        OwnerActivity::Active => "held",
                        OwnerActivity::Inactive => "abandoned",
                        OwnerActivity::LegacyUnproven => "legacy-unproven",
                    }
                    .to_string(),
                    health: match owner_activity {
                        OwnerActivity::Active => ResidueHealth::Active,
                        OwnerActivity::Inactive => ResidueHealth::Stale,
                        OwnerActivity::LegacyUnproven => ResidueHealth::Unknown,
                    },
                    recoverable: owner_activity != OwnerActivity::Active,
                    ownership_authority: "session-owner-record".to_string(),
                    detail: match owner_activity {
                        OwnerActivity::Active => {
                            "active session owner has not created its resource journal yet"
                        }
                        OwnerActivity::Inactive => {
                            "abandoned owner registration can be retired exactly"
                        }
                        OwnerActivity::LegacyUnproven => {
                            "legacy owner registration requires explicit operator confirmation"
                        }
                    }
                    .to_string(),
                },
            );
        }
    }
    inventory.findings.sort_by(|left, right| {
        (&left.session_id, &left.resource_id).cmp(&(&right.session_id, &right.resource_id))
    });
    inventory.limitations.sort();
    inventory.limitations.dedup();
    inventory
}

fn push_finding(inventory: &mut NativeResidueInventory, finding: ResourceFinding) {
    if inventory.findings.len() < MAX_FINDINGS {
        inventory.findings.push(finding);
    } else if !inventory
        .limitations
        .iter()
        .any(|item| item.starts_with("finding-limit"))
    {
        inventory
            .limitations
            .push("finding-limit: additional resource findings omitted".to_string());
    }
}

fn walk_journals(
    directory: &Path,
    depth: usize,
    visited: &mut usize,
    journals: &mut Vec<PathBuf>,
    observed_files: &mut Vec<PathBuf>,
    limitations: &mut Vec<String>,
) {
    if depth > MAX_SCAN_DEPTH {
        limitations.push(format!(
            "scan-depth-limit at {}: descendants omitted",
            directory.display()
        ));
        return;
    }
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return,
        Err(error) => {
            limitations.push(format!(
                "scan-read-failed at {}: {error}",
                directory.display()
            ));
            return;
        }
    };
    for entry in entries {
        if *visited >= MAX_SCAN_ENTRIES {
            limitations.push("scan-entry-limit: additional entries omitted".to_string());
            return;
        }
        *visited += 1;
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                limitations.push(format!("scan-entry-failed: {error}"));
                continue;
            }
        };
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error) => {
                limitations.push(format!("scan-type-failed at {}: {error}", path.display()));
                continue;
            }
        };
        if file_type.is_dir() {
            walk_journals(
                &path,
                depth + 1,
                visited,
                journals,
                observed_files,
                limitations,
            );
        } else if file_type.is_file()
            && path
                .file_name()
                .is_some_and(|name| name == RESOURCE_JOURNAL)
        {
            journals.push(path);
        } else if file_type.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name == "manifest.json"
                        || name == fragcap::deep_capture::MANIFEST_PREFIX
                        || matches!(
                            name,
                            "application.jsonl"
                                | "http.har"
                                | "tls-keylog.log"
                                | "sslkeylog.log"
                                | "proxy.jsonl"
                                | "cleanup.jsonl"
                                | "process-trace.jsonl"
                        )
                })
        {
            observed_files.push(path);
        }
    }
}

fn valid_lease_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

#[cfg(windows)]
fn lease_name(lease_id: &str) -> Vec<u16> {
    format!("Local\\fragcap-deep-capture-{lease_id}\0")
        .encode_utf16()
        .collect()
}

#[cfg(windows)]
fn create_lease(lease_id: &str) -> io::Result<Option<SessionOwnerLease>> {
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;

    let name = lease_name(lease_id);
    let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
    if handle == 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
        return Ok(None);
    }
    Ok(Some(SessionOwnerLease { handle }))
}

#[cfg(windows)]
fn lease_is_active(lease_id: &str) -> io::Result<bool> {
    use windows_sys::Win32::System::Threading::OpenMutexW;

    const SYNCHRONIZE: u32 = 0x0010_0000;

    let name = lease_name(lease_id);
    let handle = unsafe { OpenMutexW(SYNCHRONIZE, 0, name.as_ptr()) };
    if handle == 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(windows_sys::Win32::Foundation::ERROR_FILE_NOT_FOUND as i32)
        {
            return Ok(false);
        }
        return Err(error);
    }
    unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
    Ok(true)
}

#[cfg(not(windows))]
fn test_leases() -> &'static std::sync::Mutex<BTreeSet<String>> {
    static LEASES: std::sync::OnceLock<std::sync::Mutex<BTreeSet<String>>> =
        std::sync::OnceLock::new();
    LEASES.get_or_init(|| std::sync::Mutex::new(BTreeSet::new()))
}

#[cfg(not(windows))]
fn create_lease(lease_id: &str) -> io::Result<Option<SessionOwnerLease>> {
    let mut leases = test_leases().lock().expect("lease registry lock");
    if !leases.insert(lease_id.to_string()) {
        return Ok(None);
    }
    Ok(Some(SessionOwnerLease {
        lease_id: lease_id.to_string(),
    }))
}

#[cfg(not(windows))]
fn lease_is_active(lease_id: &str) -> io::Result<bool> {
    Ok(test_leases()
        .lock()
        .expect("lease registry lock")
        .contains(lease_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap::deep_capture::{ResourceJournal, ResourceKind, ResourceTransition};

    #[test]
    fn owner_liveness_follows_lease_not_pid() {
        let root = tempfile::tempdir().expect("root");
        let bundle = root.path().join("bundle");
        std::fs::create_dir(&bundle).expect("bundle");
        let lease = register_session_owner(root.path(), &bundle).expect("owner");
        let owner = registered_session_owners(root.path())
            .expect("owners")
            .remove(0);
        assert_eq!(owner.owner_pid, std::process::id());
        assert!(owner_is_active(&owner).expect("active"));
        drop(lease);
        assert!(!owner_is_active(&owner).expect("inactive"));
    }

    #[test]
    fn exact_registered_custom_bundle_is_supported() {
        let root = tempfile::tempdir().expect("root");
        let outside = tempfile::tempdir().expect("outside");
        let lease = register_session_owner(root.path(), outside.path()).expect("custom owner");
        let owner = registered_session_owners(root.path())
            .expect("owners")
            .remove(0);
        assert_eq!(
            owner.bundle,
            outside.path().canonicalize().expect("canonical")
        );
        assert!(owner_is_active(&owner).expect("active"));
        drop(lease);
    }

    #[test]
    fn legacy_owner_is_retirable_only_after_its_bundle_is_gone() {
        let root = tempfile::tempdir().expect("root");
        let registry = root.path().join(SESSION_OWNER_REGISTRY);
        let bundle = root.path().join("legacy");
        std::fs::create_dir(&registry).expect("registry");
        std::fs::create_dir(&bundle).expect("bundle");
        let record = registry.join("legacy.json");
        std::fs::write(
            &record,
            serde_json::to_vec(&serde_json::json!({
                "bundle": bundle,
                "owner_pid": std::process::id(),
            }))
            .expect("json"),
        )
        .expect("record");
        let owner = registered_session_owners(root.path())
            .expect("owners")
            .remove(0);
        assert_eq!(
            owner_is_active(&owner)
                .expect_err("legacy live state is unknown")
                .kind(),
            io::ErrorKind::Unsupported
        );
        std::fs::remove_dir(&bundle).expect("remove bundle");
        assert!(!owner_is_active(&owner).expect("missing legacy bundle is dead"));
    }

    #[test]
    fn vanished_registered_bundle_is_absence_not_an_incomplete_scan() {
        let root = tempfile::tempdir().expect("root");
        let bundle = root.path().join("vanished");
        std::fs::create_dir(&bundle).expect("bundle");
        let lease = register_session_owner(root.path(), &bundle).expect("owner");
        drop(lease);
        std::fs::remove_dir(&bundle).expect("remove bundle");

        let inventory = inventory(Some(root.path()));

        assert!(inventory.limitations.is_empty());
        assert_eq!(inventory.findings.len(), 1);
        assert_eq!(inventory.findings[0].resource_id, "session-owner");
        assert_eq!(inventory.findings[0].health, ResidueHealth::Stale);
        assert!(inventory.findings[0].recoverable);
    }

    #[test]
    fn inventory_distinguishes_active_stale_and_terminal_resources() {
        let root = tempfile::tempdir().expect("root");
        let bundle = root.path().join("bundle");
        std::fs::create_dir(&bundle).expect("bundle");
        let lease = register_session_owner(root.path(), &bundle).expect("owner");
        let mut journal = ResourceJournal::create(&bundle, "session", "plan").expect("journal");
        journal
            .append(ResourceTransition::new(
                "proxy",
                ResourceKind::Proxy,
                "127.0.0.1:1234",
                "session:session",
                "release",
                ResourceState::Applied,
                "active",
            ))
            .expect("append");
        assert_eq!(
            inventory(Some(root.path())).findings[0].health,
            ResidueHealth::Active
        );
        drop(lease);
        assert_eq!(
            inventory(Some(root.path())).findings[0].health,
            ResidueHealth::Stale
        );
        journal
            .append(ResourceTransition::new(
                "proxy",
                ResourceKind::Proxy,
                "127.0.0.1:1234",
                "session:session",
                "release",
                ResourceState::CleanupPending,
                "cleanup",
            ))
            .expect("append");
        journal
            .append(ResourceTransition::new(
                "proxy",
                ResourceKind::Proxy,
                "127.0.0.1:1234",
                "session:session",
                "release",
                ResourceState::Released,
                "released",
            ))
            .expect("release");
        journal.finish().expect("finish");
        assert_eq!(
            inventory(Some(root.path())).findings[0].health,
            ResidueHealth::Healthy
        );
    }

    #[test]
    fn malformed_journal_and_scan_limit_are_explicit() {
        let root = tempfile::tempdir().expect("root");
        let bundle = root.path().join("bundle");
        std::fs::create_dir(&bundle).expect("bundle");
        std::fs::write(bundle.join(RESOURCE_JOURNAL), "not json\n").expect("journal");
        for index in 0..MAX_SCAN_ENTRIES {
            std::fs::write(root.path().join(format!("entry-{index}")), "x").expect("entry");
        }
        let observed = inventory(Some(root.path()));
        assert!(observed
            .limitations
            .iter()
            .any(|item| item.starts_with("journal-invalid")));
        assert!(observed
            .limitations
            .iter()
            .any(|item| item.starts_with("scan-entry-limit")));
        assert!(!observed.limitations.is_empty());
    }

    #[test]
    fn retained_history_and_unowned_sensitive_artifacts_are_distinct() {
        let root = tempfile::tempdir().expect("root");
        let complete = root.path().join("complete");
        let unknown = root.path().join("unknown");
        std::fs::create_dir(&complete).expect("complete");
        std::fs::create_dir(&unknown).expect("unknown");
        std::fs::write(
            complete.join("manifest.json"),
            r#"{"cleanup":{"status":"succeeded"}}"#,
        )
        .expect("manifest");
        std::fs::write(unknown.join("tls-keylog.log"), "key material").expect("keylog");

        let observed = inventory(Some(root.path()));
        assert!(observed.findings.iter().any(|finding| {
            finding.resource_id == "artifact:manifest.json"
                && finding.health == ResidueHealth::Healthy
        }));
        assert!(observed.findings.iter().any(|finding| {
            finding.resource_id == "artifact:tls-keylog.log"
                && finding.health == ResidueHealth::Unknown
        }));
    }

    #[test]
    fn unrelated_listener_is_not_invented_as_fragcap_residue() {
        let root = tempfile::tempdir().expect("root");
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("listener");
        let observed = inventory(Some(root.path()));
        assert!(observed.findings.is_empty());
        assert!(observed.limitations.is_empty());
        drop(listener);
    }
}
