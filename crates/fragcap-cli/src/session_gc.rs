// SPDX-License-Identifier: Apache-2.0

//! Declared session retention and exact collection command orchestration.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use fragcap::deep_capture::{
    collect_session_contents, inspect_session_collection, purge_session_container,
    CollectionInspection,
};
use serde_json::{json, Value};

use crate::doctor::residue::{
    acquire_maintenance_lease, is_reparse, owner_is_active, owner_is_active_in_inventory,
    registered_session_owners, SessionOwner,
};

pub(crate) const RETIREMENTS: &str = ".session-retirements";
pub(crate) const RETENTION: &str = ".session-retention.json";
const MAX_AGE_SECONDS: u64 = 30 * 24 * 60 * 60;
const MAX_SESSIONS: usize = 20;
const MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_CONTAINERS: usize = 100_000;

fn bounded_owner_read(path: &Path) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?.take(4097).read_to_end(&mut bytes)?;
    if bytes.len() > 4096 {
        return Err(invalid("owner record exceeded its 4096-byte bound"));
    }
    Ok(bytes)
}

pub(crate) fn declare_retention(bundle: &Path, managed: bool) -> io::Result<()> {
    let path = bundle.join(RETENTION);
    let mut file = fragcap::deep_capture::open_sensitive_file(&path)?;
    serde_json::to_writer(
        &mut file,
        &json!({
            "schema_version":1,
            "storage_class": if managed { "managed-history" } else { "explicitly-retained" },
            "created_at_unix_ms": now_ms(),
            "max_age_seconds": if managed { Some(MAX_AGE_SECONDS) } else { None },
            "max_sessions": if managed { Some(MAX_SESSIONS) } else { None },
            "max_bytes": if managed { Some(MAX_BYTES) } else { None },
            "preserve_empty_containers":true
        }),
    )
    .map_err(io::Error::other)?;
    file.write_all(b"\n")?;
    file.sync_all()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn invalid(message: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

fn root_path(root: &Path) -> io::Result<PathBuf> {
    crate::doctor::residue::reject_reparse_ancestors(root)?;
    match root.symlink_metadata() {
        Ok(metadata) if is_reparse(&metadata) || !metadata.is_dir() => Err(invalid(
            "session root must be an exact directory without reparse indirection",
        )),
        Ok(_) => root.canonicalize(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => std::path::absolute(root),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
fn retention(bundle: &Path) -> io::Result<(bool, Value)> {
    let path = bundle.join(RETENTION);
    let metadata = match path.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok((
                false,
                json!({"storage_class":"legacy-retained","policy":"retain-until-explicit-cleanup"}),
            ))
        }
        Err(error) => return Err(error),
    };
    if is_reparse(&metadata) || !metadata.is_file() || metadata.len() > 4096 {
        return Err(invalid("retention metadata must be a bounded regular file"));
    }
    let value: Value = serde_json::from_slice(&fs::read(path)?).map_err(invalid)?;
    retention_value(value)
}

fn retention_value(value: Value) -> io::Result<(bool, Value)> {
    if value["schema_version"] != 1
        || value["preserve_empty_containers"] != true
        || value["created_at_unix_ms"].as_u64().is_none()
    {
        return Err(invalid("retention metadata is unsupported or malformed"));
    }
    match value["storage_class"].as_str() {
        Some("managed-history")
            if value["max_age_seconds"] == MAX_AGE_SECONDS
                && value["max_sessions"] == MAX_SESSIONS
                && value["max_bytes"] == MAX_BYTES =>
        {
            Ok((true, value))
        }
        Some("explicitly-retained")
            if value["max_age_seconds"].is_null()
                && value["max_sessions"].is_null()
                && value["max_bytes"].is_null() =>
        {
            Ok((false, value))
        }
        _ => Err(invalid(
            "retention class or finite policy differs from the supported contract",
        )),
    }
}

struct Candidate {
    inspection: CollectionInspection,
    owners: Vec<SessionOwner>,
    managed: bool,
    row: Value,
    selected: bool,
}

struct Scan {
    root: PathBuf,
    candidates: Vec<Candidate>,
    rows: Vec<Value>,
    limitations: Vec<String>,
    owner_digest: String,
    orphan_owners: Vec<SessionOwner>,
}

fn scan(
    root: &Path,
    bundle: Option<&Path>,
    include_retained: bool,
    purge_empty: bool,
) -> io::Result<Scan> {
    let deadline = Instant::now() + Duration::from_secs(30);
    let root = root_path(root)?;
    let owners = registered_session_owners(&root);
    let mut limitations = Vec::new();
    let owners = match owners {
        Ok(owners) => owners,
        Err(error) => {
            limitations.push(format!("owner-registry-unresolved: {error}"));
            Vec::new()
        }
    };
    let mut owner_hasher = blake3::Hasher::new();
    if root.exists() {
        #[cfg(windows)]
        {
            owner_hasher.update(
                crate::doctor::residue::pin_maintenance_root(&root)?
                    .0
                    .as_bytes(),
            );
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = root.metadata()?;
            owner_hasher.update(&metadata.dev().to_le_bytes());
            owner_hasher.update(&metadata.ino().to_le_bytes());
        }
    } else {
        owner_hasher.update(b"root-absent");
    }
    for owner in &owners {
        owner_hasher.update(owner.registry_path.to_string_lossy().as_bytes());
        owner_hasher.update(&bounded_owner_read(&owner.registry_path)?);
    }
    let owner_digest = owner_hasher.finalize().to_hex().to_string();
    #[cfg(windows)]
    let legacy_processes = owners
        .iter()
        .any(|owner| {
            owner
                .lease_id
                .as_ref()
                .is_some_and(|id| !id.starts_with("g3-"))
        })
        .then(crate::doctor::residue::legacy_process_snapshot);
    #[cfg(not(windows))]
    let legacy_processes = None;
    let mut paths = Vec::new();
    let exact = bundle.is_some();
    if let Some(bundle) = bundle {
        crate::doctor::residue::reject_reparse_ancestors(bundle)?;
        paths.push(std::path::absolute(bundle)?);
    } else {
        match fs::read_dir(&root) {
            Ok(entries) => {
                for (index, entry) in entries.enumerate() {
                    if index >= MAX_CONTAINERS || Instant::now() >= deadline {
                        limitations.push(
                            "session-scan-limit: additional containers are unresolved".into(),
                        );
                        break;
                    }
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(error) => {
                            limitations.push(format!("session-entry-unreadable: {error}"));
                            continue;
                        }
                    };
                    let path = entry.path();
                    if entry.file_name() == RETIREMENTS
                        || entry.file_name() == crate::doctor::residue::SESSION_OWNER_REGISTRY
                    {
                        continue;
                    }
                    match path.symlink_metadata() {
                        Ok(metadata) if is_reparse(&metadata) => {
                            limitations
                                .push(format!("session-reparse-refused at {}", path.display()));
                        }
                        Ok(metadata) if metadata.is_dir() => paths.push(path),
                        Ok(_) => {}
                        Err(error) => limitations.push(format!(
                            "session-entry-unreadable at {}: {error}",
                            path.display()
                        )),
                    }
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => limitations.push(format!("session-root-unreadable: {error}")),
        }
    }
    paths.sort();
    paths.dedup();
    let mut rows = Vec::new();
    let mut candidates = Vec::new();
    let mut orphan_owners = Vec::new();
    for owner in &owners {
        if exact
            && bundle.is_some_and(|selected| {
                std::path::absolute(selected).ok().as_ref() != Some(&owner.bundle)
            })
        {
            continue;
        }
        if owner
            .bundle
            .symlink_metadata()
            .is_err_and(|error| error.kind() == io::ErrorKind::NotFound)
        {
            match owner_is_active_in_inventory(owner,legacy_processes.as_ref()) {
                Ok(false)=>{
                    rows.push(json!({"bundle":owner.bundle,"registry_path":owner.registry_path,"status":"eligible","reason":"obsolete-owner-record","recoverable_logical_bytes":0}));
                    orphan_owners.push(owner.clone());
                }
                Ok(true)=>rows.push(json!({"bundle":owner.bundle,"registry_path":owner.registry_path,"status":"retained","reason":"active-generation"})),
                Err(error)=>rows.push(json!({"bundle":owner.bundle,"registry_path":owner.registry_path,"status":"unresolved","reason":"owner-generation-unproven","detail":error.to_string()})),
            }
        }
    }
    for path in paths {
        if Instant::now() >= deadline {
            limitations.push("session-scan-deadline: additional provenance is unresolved".into());
            break;
        }
        if path
            .symlink_metadata()
            .is_ok_and(|metadata| is_reparse(&metadata))
        {
            rows.push(
                json!({"bundle":path,"status":"unresolved","reason":"session-reparse-refused"}),
            );
            continue;
        }
        let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
        let matching: Vec<_> = owners
            .iter()
            .filter(|owner| owner.bundle == canonical)
            .cloned()
            .collect();
        let activity = matching.iter().try_fold(false, |active, owner| {
            owner_is_active_in_inventory(owner, legacy_processes.as_ref())
                .map(|observed| active || observed)
        });
        match activity {
            Ok(true) => {
                rows.push(
                    json!({"bundle":canonical,"status":"retained","reason":"active-generation"}),
                );
                continue;
            }
            Err(error) => {
                rows.push(json!({"bundle":canonical,"status":"unresolved","reason":"owner-generation-unproven","detail":error.to_string()}));
                continue;
            }
            Ok(false) => {}
        }
        if !limitations.is_empty() && owners.is_empty() {
            rows.push(json!({"bundle":canonical,"status":"unresolved","reason":"owner-registry-unresolved"}));
            continue;
        }
        let inspection = match inspect_session_collection(&path, &root.join(RETIREMENTS)) {
            Ok(inspection) => inspection,
            Err(error) => {
                rows.push(json!({"bundle":canonical,"status":"unresolved","reason":"whole-session-authority-unresolved","detail":error.to_string()}));
                continue;
            }
        };
        let (managed, policy) = match inspection.retention.clone().map_or_else(
            || Ok((false,json!({"storage_class":"legacy-retained","policy":"retain-until-explicit-cleanup"}))),
            retention_value,
        ) {
            Ok(value) => value,
            Err(error) => { rows.push(json!({"bundle":canonical,"status":"unresolved","reason":"retention-unresolved","detail":error.to_string()})); continue; }
        };
        let selected = inspection.owned
            && if inspection.empty {
                purge_empty || !matching.is_empty()
            } else {
                managed || include_retained
            };
        let status = if !inspection.owned {
            "retained"
        } else if selected {
            "eligible"
        } else {
            "retained"
        };
        let reason = if !inspection.owned {
            "empty-container-ownership-unproven"
        } else if inspection.empty && !purge_empty && matching.is_empty() {
            "empty-container-preserved"
        } else if inspection.empty && !purge_empty {
            "collected-container-owner-retirement"
        } else if selected {
            "whole-session-reconciled"
        } else {
            "explicit-evidence-retention"
        };
        let mut row = inspection.json();
        row["status"] = json!(status);
        row["reason"] = json!(reason);
        row["retention"] = policy;
        candidates.push(Candidate {
            inspection,
            owners: matching,
            managed,
            row,
            selected,
        });
    }
    if !exact {
        for owner in owners
            .iter()
            .filter(|owner| !owner.bundle.starts_with(&root) && owner.bundle.exists())
        {
            rows.push(json!({"bundle":owner.bundle,"status":"retained","reason":"custom-output-requires-exact-bundle-selection"}));
        }
    }
    Ok(Scan {
        root,
        candidates,
        rows,
        limitations,
        owner_digest,
        orphan_owners,
    })
}

fn proposal(scan: &Scan, include_retained: bool, purge_empty: bool) -> String {
    let mut hash = blake3::Hasher::new();
    hash.update(b"fragcap.session-collection.v1\0");
    hash.update(scan.root.to_string_lossy().as_bytes());
    hash.update(scan.owner_digest.as_bytes());
    hash.update(&[u8::from(include_retained), u8::from(purge_empty)]);
    for candidate in &scan.candidates {
        hash.update(candidate.inspection.id.as_bytes());
        hash.update(&serde_json::to_vec(&candidate.row).expect("JSON value serializes"));
    }
    hash.update(&serde_json::to_vec(&scan.rows).expect("JSON value serializes"));
    hash.update(&serde_json::to_vec(&scan.limitations).expect("JSON value serializes"));
    format!("fcap-collect-{}", hash.finalize().to_hex())
}

fn render(scan: &Scan, include_retained: bool, purge_empty: bool, mode: &str) -> Value {
    let mut rows = scan.rows.clone();
    rows.extend(
        scan.candidates
            .iter()
            .map(|candidate| candidate.row.clone()),
    );
    rows.sort_by_key(|row| row["bundle"].as_str().unwrap_or_default().to_owned());
    let eligible = scan.orphan_owners.len()
        + scan
            .candidates
            .iter()
            .filter(|candidate| candidate.selected)
            .count();
    let retained = rows
        .iter()
        .filter(|row| row["status"] == "retained")
        .count();
    let unresolved = rows
        .iter()
        .filter(|row| row["status"] == "unresolved")
        .count();
    let recoverable_bytes: u64 = scan
        .candidates
        .iter()
        .filter(|candidate| candidate.selected)
        .map(|candidate| candidate.inspection.bytes)
        .sum();
    json!({"schema_version":1,"mode":mode,"proposal_id":proposal(scan,include_retained,purge_empty),
        "root":scan.root,"include_retained":include_retained,"purge_empty":purge_empty,
        "policy":{"max_age_seconds":MAX_AGE_SECONDS,"max_sessions":MAX_SESSIONS,"max_bytes":MAX_BYTES,"preserve_empty_containers":true},
        "eligible_sessions":eligible,"retained_sessions":retained,"unresolved_sessions":unresolved,
        "empty_containers":scan.candidates.iter().filter(|candidate|candidate.inspection.empty).count(),
        "recoverable_bytes":recoverable_bytes,"removed_bytes":0,"removed_files":0,
        "complete":unresolved == 0 && scan.limitations.is_empty(),"results":rows,"limitations":scan.limitations})
}

pub(crate) fn preview(
    root: &Path,
    bundle: Option<&Path>,
    include_retained: bool,
    purge_empty: bool,
) -> io::Result<Value> {
    let scan = scan(root, bundle, include_retained, purge_empty)?;
    Ok(render(&scan, include_retained, purge_empty, "preview"))
}

pub(crate) fn apply(
    root: &Path,
    bundle: Option<&Path>,
    include_retained: bool,
    purge_empty: bool,
    authorize: &str,
) -> io::Result<Value> {
    let root = root_path(root)?;
    let _guard = acquire_maintenance_lease(&root)?;
    let scan = scan(&root, bundle, include_retained, purge_empty)?;
    if authorize != proposal(&scan, include_retained, purge_empty) {
        return Err(invalid(
            "collection proposal changed; obtain and authorize a fresh preview",
        ));
    }
    execute(scan, include_retained, purge_empty, "apply")
}

fn execute(
    mut scan: Scan,
    include_retained: bool,
    purge_empty: bool,
    mode: &str,
) -> io::Result<Value> {
    let mut result = render(&scan, include_retained, purge_empty, mode);
    let store = scan.root.join(RETIREMENTS);
    let mut removed_bytes = 0u64;
    let mut removed_files = 0u64;
    for candidate in &mut scan.candidates {
        if !candidate.selected {
            continue;
        }
        // Recheck every matching lease immediately before the facade mutation.
        let active = candidate.owners.iter().try_fold(false, |active, owner| {
            owner_is_active(owner).map(|observed| active || observed)
        });
        match active {
            Ok(false) => {}
            value => {
                candidate.row["status"] = json!("unresolved");
                candidate.row["reason"] = json!(match value {
                    Ok(true) => "active-generation".into(),
                    Err(error) => error.to_string(),
                    _ => unreachable!(),
                });
                continue;
            }
        }
        let report = if candidate.inspection.empty && purge_empty {
            purge_session_container(
                &candidate.inspection.bundle,
                &store,
                &candidate.inspection.id,
            )
        } else {
            collect_session_contents(
                &candidate.inspection.bundle,
                &store,
                &candidate.inspection.id,
            )
        };
        match report {
            Ok(report) => {
                removed_bytes = removed_bytes.saturating_add(report.removed_bytes);
                removed_files = removed_files.saturating_add(report.removed_files as u64);
                let mut complete = report.complete;
                let report = report.json();
                if complete {
                    for owner in &candidate.owners {
                        if let Err(error) = retire_owner(owner) {
                            scan.limitations.push(format!(
                                "owner-retirement-failed at {}: {error}",
                                owner.registry_path.display()
                            ));
                            complete = false;
                        }
                    }
                }
                candidate.row["status"] = json!(if complete { "collected" } else { "unresolved" });
                candidate.row["collection"] = report;
            }
            Err(error) => {
                candidate.row["status"] = json!("unresolved");
                candidate.row["reason"] = json!(error.to_string());
            }
        }
    }
    for owner in &scan.orphan_owners {
        let status = if owner
            .bundle
            .symlink_metadata()
            .is_err_and(|error| error.kind() == io::ErrorKind::NotFound)
        {
            retire_owner(owner)
        } else {
            Err(invalid(
                "previously absent bundle reappeared; owner retirement requires a fresh preview",
            ))
        };
        for row in scan.rows.iter_mut().filter(|row| {
            row["registry_path"].as_str().map(PathBuf::from).as_ref() == Some(&owner.registry_path)
        }) {
            match &status {
                Ok(()) => row["status"] = json!("owner-retired"),
                Err(error) => {
                    row["status"] = json!("unresolved");
                    row["reason"] = json!(error.to_string());
                }
            }
        }
    }
    let refreshed = render(&scan, include_retained, purge_empty, mode);
    result["results"] = refreshed["results"].clone();
    result["unresolved_sessions"] = refreshed["unresolved_sessions"].clone();
    result["complete"] = refreshed["complete"].clone();
    result["limitations"] = refreshed["limitations"].clone();
    result["removed_bytes"] = json!(removed_bytes);
    result["removed_files"] = json!(removed_files);
    Ok(result)
}

pub(crate) fn maintenance(root: &Path, exclude: Option<&Path>) -> io::Result<Value> {
    let root = root_path(root)?;
    let _guard = acquire_maintenance_lease(&root)?;
    let mut scan = scan(&root, None, false, false)?;
    let exclude = exclude.map(|path| path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
    plan_maintenance(&mut scan, exclude.as_deref(), now_ms());
    execute(scan, false, false, "automatic-maintenance")
}

fn plan_maintenance(scan: &mut Scan, exclude: Option<&Path>, now: u64) {
    let mut ordered: Vec<_> = scan
        .candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| {
            candidate.managed && candidate.inspection.owned && !candidate.inspection.empty
        })
        .map(|(index, candidate)| {
            (
                candidate
                    .inspection
                    .completed_at_unix_ms
                    .unwrap_or(u64::MAX),
                candidate.inspection.bundle.clone(),
                index,
            )
        })
        .collect();
    ordered.sort();
    let mut count = ordered.len();
    let mut bytes = ordered.iter().fold(0u64, |sum, (_, _, index)| {
        sum.saturating_add(scan.candidates[*index].inspection.bytes)
    });
    for candidate in &mut scan.candidates {
        candidate.selected = candidate.inspection.owned
            && candidate.inspection.empty
            && !candidate.owners.is_empty();
    }
    for (completed, path, index) in ordered {
        let candidate = &mut scan.candidates[index];
        if exclude == Some(path.as_path()) {
            candidate.row["status"] = json!("retained");
            candidate.row["reason"] = json!("current-returned-output-excluded");
            continue;
        }
        let expired =
            completed != u64::MAX && now.saturating_sub(completed) >= MAX_AGE_SECONDS * 1000;
        candidate.selected = expired || count > MAX_SESSIONS || bytes > MAX_BYTES;
        candidate.row["status"] = json!(if candidate.selected {
            "eligible"
        } else {
            "retained"
        });
        candidate.row["reason"] = json!(if candidate.selected {
            "finite-managed-retention-exceeded"
        } else {
            "within-managed-retention"
        });
        if candidate.selected {
            count = count.saturating_sub(1);
            bytes = bytes.saturating_sub(candidate.inspection.bytes);
        }
    }
    if count > MAX_SESSIONS || bytes > MAX_BYTES {
        scan.limitations.push(format!("managed-retention-budget-unmet: protected current output leaves {count} completed sessions and {bytes} logical bytes; limit {MAX_SESSIONS} sessions and {MAX_BYTES} bytes"));
    }
}

fn retire_owner(owner: &SessionOwner) -> io::Result<()> {
    if owner_is_active(owner)? {
        return Err(invalid("owner became active before retirement"));
    }
    if owner
        .registry_path
        .symlink_metadata()
        .is_err_and(|error| error.kind() == io::ErrorKind::NotFound)
    {
        return Ok(());
    }
    #[cfg(windows)]
    let _ancestors = pin_registry_ancestors(&owner.registry_path)?;
    let expected = match bounded_owner_read(&owner.registry_path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    let value: Value = serde_json::from_slice(&expected).map_err(invalid)?;
    if value["lease_id"].as_str() != owner.lease_id.as_deref()
        || value["bundle"].as_str().map(PathBuf::from).as_ref() != Some(&owner.bundle)
        || value["owner_pid"].as_u64() != Some(u64::from(owner.owner_pid))
    {
        return Err(invalid("owner record changed before retirement"));
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Storage::FileSystem::{
            FileDispositionInfo, GetFileInformationByHandle, SetFileInformationByHandle,
            BY_HANDLE_FILE_INFORMATION, FILE_DISPOSITION_INFO, FILE_FLAG_OPEN_REPARSE_POINT,
            FILE_SHARE_READ,
        };
        let mut file = fs::OpenOptions::new()
            .read(true)
            .access_mode(0x8000_0000 | 0x0001_0000)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&owner.registry_path)?;
        if is_reparse(&file.metadata()?) {
            return Err(invalid("owner retirement refuses reparse record"));
        }
        let mut identity: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe { GetFileInformationByHandle(file.as_raw_handle() as isize, &mut identity) } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if identity.nNumberOfLinks != 1 {
            return Err(invalid("owner retirement refuses hard-link aliases"));
        }
        let mut observed = Vec::new();
        Read::by_ref(&mut file)
            .take(4097)
            .read_to_end(&mut observed)?;
        if observed != expected {
            return Err(invalid("owner record changed before pinned retirement"));
        }
        let disposition = FILE_DISPOSITION_INFO { DeleteFileA: 1 };
        if unsafe {
            SetFileInformationByHandle(
                file.as_raw_handle() as isize,
                FileDispositionInfo,
                &disposition as *const _ as *const _,
                std::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        drop(file);
        match owner.registry_path.symlink_metadata() {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
            Ok(_) => Err(invalid(
                "owner retirement did not remove the exact registry path",
            )),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = expected;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "exact owner deletion requires the Windows filesystem adapter",
        ))
    }
}

#[cfg(windows)]
fn pin_registry_ancestors(path: &Path) -> io::Result<Vec<fs::File>> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ, FILE_SHARE_WRITE,
    };
    let absolute = std::path::absolute(path)?;
    let mut parents: Vec<_> = absolute
        .parent()
        .ok_or_else(|| invalid("owner record has no exact parent"))?
        .ancestors()
        .collect();
    parents.reverse();
    let mut guards = Vec::new();
    for parent in parents {
        let file = fs::OpenOptions::new()
            .access_mode(0x8000_0000)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(parent)?;
        let metadata = file.metadata()?;
        if !metadata.is_dir() || is_reparse(&metadata) {
            return Err(invalid(
                "owner retirement refuses reparse or non-directory ancestors",
            ));
        }
        guards.push(file);
    }
    Ok(guards)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_policy_preserves_custom_and_saved_evidence() {
        let root = tempfile::tempdir().unwrap();
        fragcap::deep_capture::prepare_bundle(root.path()).unwrap();
        declare_retention(root.path(), false).unwrap();
        let (managed, value) = retention(root.path()).unwrap();
        assert!(!managed);
        assert_eq!(value["storage_class"], "explicitly-retained");
        assert_eq!(value["preserve_empty_containers"], true);
        assert!(
            declare_retention(root.path(), true).is_err(),
            "declaration cannot overwrite a prior promise"
        );
    }

    #[test]
    fn declared_managed_policy_has_all_three_finite_limits() {
        let root = tempfile::tempdir().unwrap();
        fragcap::deep_capture::prepare_bundle(root.path()).unwrap();
        declare_retention(root.path(), true).unwrap();
        let (managed, value) = retention(root.path()).unwrap();
        assert!(managed);
        assert_eq!(value["max_age_seconds"], 2_592_000);
        assert_eq!(value["max_sessions"], 20);
        assert_eq!(value["max_bytes"], 2_147_483_648u64);
    }

    #[test]
    fn legacy_policy_remains_explicit_retention() {
        let root = tempfile::tempdir().unwrap();
        assert!(!retention(root.path()).unwrap().0);
        fs::write(root.path().join(RETENTION), "{}\n").unwrap();
        assert!(retention(root.path()).is_err());
    }

    #[test]
    fn active_owner_blocks_even_explicit_collection() {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("active");
        fs::create_dir(&bundle).unwrap();
        let _lease = crate::doctor::residue::register_session_owner(root.path(), &bundle).unwrap();
        let value = preview(root.path(), Some(&bundle), true, true).unwrap();
        assert_eq!(value["eligible_sessions"], 0);
        assert_eq!(value["results"][0]["reason"], "active-generation");
        assert_eq!(fs::read_dir(&bundle).unwrap().count(), 0);
    }

    #[test]
    fn proposal_scope_and_owner_change_require_a_fresh_preview() {
        let root = tempfile::tempdir().unwrap();
        let value = preview(root.path(), None, false, false).unwrap();
        let proposal = value["proposal_id"].as_str().unwrap();
        assert!(apply(root.path(), None, true, false, proposal).is_err());
        let bundle = root.path().join("active");
        fs::create_dir(&bundle).unwrap();
        let _lease = crate::doctor::residue::register_session_owner(root.path(), &bundle).unwrap();
        assert!(apply(root.path(), None, false, false, proposal).is_err());
    }

    fn policy_scan(sizes: &[u64], completed: u64) -> Scan {
        Scan {
            root: PathBuf::from("root"),
            rows: Vec::new(),
            limitations: Vec::new(),
            owner_digest: String::new(),
            orphan_owners: Vec::new(),
            candidates: sizes
                .iter()
                .enumerate()
                .map(|(index, bytes)| {
                    let inspection = CollectionInspection {
                        id: format!("proposal-{index}"),
                        bundle: PathBuf::from(format!("root/{index:04}")),
                        session_id: format!("session-{index}"),
                        bytes: *bytes,
                        files: 1,
                        empty: false,
                        resumed: false,
                        owned: true,
                        completed_at_unix_ms: Some(completed + index as u64),
                        retention: None,
                    };
                    Candidate {
                        row: inspection.json(),
                        inspection,
                        owners: Vec::new(),
                        managed: true,
                        selected: false,
                    }
                })
                .collect(),
        }
    }

    #[test]
    fn newest_returned_bundle_counts_toward_the_twenty_session_budget() {
        let mut scan = policy_scan(&[100; 21], 1_000);
        let exclude = scan.candidates[20].inspection.bundle.clone();
        plan_maintenance(&mut scan, Some(&exclude), 2_000);
        assert!(scan.candidates[0].selected);
        assert_eq!(
            scan.candidates
                .iter()
                .filter(|candidate| candidate.selected)
                .count(),
            1
        );
        assert!(!scan.candidates[20].selected);
        assert!(scan.limitations.is_empty());
    }

    #[test]
    fn age_and_byte_limits_select_oldest_whole_sessions() {
        let mut old = policy_scan(&[1], 1_000);
        plan_maintenance(&mut old, None, 1_000 + MAX_AGE_SECONDS * 1_000);
        assert!(old.candidates[0].selected);
        let mut large = policy_scan(&[MAX_BYTES, 1], 1_000);
        plan_maintenance(&mut large, None, 2_000);
        assert!(large.candidates[0].selected);
        assert!(!large.candidates[1].selected);
    }

    #[test]
    fn oversized_current_output_is_preserved_and_unmet_budget_is_visible() {
        let mut scan = policy_scan(&[MAX_BYTES + 1], 1_000);
        let exclude = scan.candidates[0].inspection.bundle.clone();
        plan_maintenance(&mut scan, Some(&exclude), 2_000);
        assert!(!scan.candidates[0].selected);
        assert!(scan.limitations[0].starts_with("managed-retention-budget-unmet"));
    }

    #[cfg(windows)]
    fn completed_fixture(root: &Path, name: &str, managed: bool) -> PathBuf {
        let bundle = root.join(name);
        fragcap::deep_capture::prepare_bundle(&bundle).unwrap();
        declare_retention(&bundle, managed).unwrap();
        let mut journal =
            fragcap::deep_capture::ResourceJournal::create(&bundle, name, "plan").unwrap();
        journal.finish().unwrap();
        drop(journal);
        fs::write(
            bundle.join("capture.fcapng"),
            b"fixture retained packet bytes\n",
        )
        .unwrap();
        fs::write(
            bundle.join("manifest.json"),
            serde_json::to_vec(&json!({
                "manifest_version":1,"session_id":name,"state":"complete",
                "artifacts":[{"path":"capture.fcapng","sensitivity":"ordinary"}]
            }))
            .unwrap(),
        )
        .unwrap();
        let lease = crate::doctor::residue::register_session_owner(root, &bundle).unwrap();
        drop(lease);
        bundle
    }

    #[cfg(windows)]
    #[test]
    fn collection_retires_owner_preserves_empty_container_and_purge_is_separate() {
        let root = tempfile::tempdir().unwrap();
        let bundle = completed_fixture(root.path(), "managed", true);
        let value = preview(root.path(), None, false, false).unwrap();
        assert_eq!(value["eligible_sessions"], 1);
        assert!(!root.path().join(RETIREMENTS).exists());
        let collected = apply(
            root.path(),
            None,
            false,
            false,
            value["proposal_id"].as_str().unwrap(),
        )
        .unwrap();
        assert_eq!(collected["complete"], true, "{collected}");
        assert!(collected["removed_bytes"].as_u64().unwrap() > 0);
        assert_eq!(fs::read_dir(&bundle).unwrap().count(), 0);
        assert!(registered_session_owners(root.path()).unwrap().is_empty());
        assert!(crate::doctor::residue::inventory(Some(root.path()))
            .findings
            .is_empty());
        let preserved = preview(root.path(), None, false, false).unwrap();
        assert_eq!(preserved["eligible_sessions"], 0);
        let purge = preview(root.path(), None, false, true).unwrap();
        assert_eq!(purge["eligible_sessions"], 1);
        let purged = apply(
            root.path(),
            None,
            false,
            true,
            purge["proposal_id"].as_str().unwrap(),
        )
        .unwrap();
        assert_eq!(purged["complete"], true, "{purged}");
        assert_eq!(purged["removed_bytes"], 0);
        assert!(!bundle.exists());
    }

    #[cfg(windows)]
    #[test]
    fn saved_and_custom_bundles_require_explicit_matching_scope() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let saved = completed_fixture(root.path(), "saved", false);
        let custom = completed_fixture(outside.path(), "custom", false);
        let lease = crate::doctor::residue::register_session_owner(root.path(), &custom).unwrap();
        drop(lease);
        let ordinary = preview(root.path(), None, false, false).unwrap();
        assert_eq!(ordinary["eligible_sessions"], 0);
        let historical = preview(root.path(), None, true, false).unwrap();
        assert_eq!(historical["eligible_sessions"], 1);
        apply(
            root.path(),
            None,
            true,
            false,
            historical["proposal_id"].as_str().unwrap(),
        )
        .unwrap();
        assert_eq!(fs::read_dir(&saved).unwrap().count(), 0);
        assert!(custom.join("capture.fcapng").exists());
        let exact = preview(root.path(), Some(&custom), true, false).unwrap();
        assert_eq!(exact["eligible_sessions"], 1);
        let report = apply(
            root.path(),
            Some(&custom),
            true,
            false,
            exact["proposal_id"].as_str().unwrap(),
        )
        .unwrap();
        assert_eq!(report["complete"], true, "{report}");
        assert_eq!(fs::read_dir(&custom).unwrap().count(), 0);
    }

    #[cfg(windows)]
    #[test]
    fn already_retired_owner_is_an_idempotent_success() {
        let root = tempfile::tempdir().unwrap();
        let bundle = completed_fixture(root.path(), "managed", true);
        let owner = registered_session_owners(root.path()).unwrap().remove(0);
        retire_owner(&owner).unwrap();
        retire_owner(&owner).unwrap();
        assert!(bundle.join("manifest.json").exists());
    }

    #[cfg(windows)]
    #[test]
    fn owner_retirement_refuses_linked_record_and_ancestor_replacement() {
        let root = tempfile::tempdir().unwrap();
        completed_fixture(root.path(), "managed", true);
        let owner = registered_session_owners(root.path()).unwrap().remove(0);
        let outside = root.path().join("outside-owner-copy.json");
        fs::hard_link(&owner.registry_path, &outside).unwrap();
        assert!(retire_owner(&owner)
            .unwrap_err()
            .to_string()
            .contains("hard-link"));
        assert!(owner.registry_path.exists() && outside.exists());
        fs::remove_file(&outside).unwrap();
        let guards = pin_registry_ancestors(&owner.registry_path).unwrap();
        let registry = owner.registry_path.parent().unwrap();
        assert!(fs::rename(registry, root.path().join("replacement-registry")).is_err());
        drop(guards);
        retire_owner(&owner).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn orphan_owner_registry_retirement_does_not_require_container_purge() {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("vanished");
        fs::create_dir(&bundle).unwrap();
        let lease = crate::doctor::residue::register_session_owner(root.path(), &bundle).unwrap();
        drop(lease);
        fs::remove_dir(&bundle).unwrap();
        let value = preview(root.path(), None, false, false).unwrap();
        assert_eq!(value["eligible_sessions"], 1);
        assert_eq!(value["results"][0]["reason"], "obsolete-owner-record");
        let result = apply(
            root.path(),
            None,
            false,
            false,
            value["proposal_id"].as_str().unwrap(),
        )
        .unwrap();
        assert_eq!(result["complete"], true, "{result}");
        assert_eq!(result["removed_bytes"], 0);
        assert!(registered_session_owners(root.path()).unwrap().is_empty());
        assert!(!bundle.exists());
    }

    #[cfg(windows)]
    #[test]
    fn empty_retirement_retries_leftover_owner_record_without_purge() {
        let root = tempfile::tempdir().unwrap();
        let bundle = completed_fixture(root.path(), "managed", true);
        let inspection =
            inspect_session_collection(&bundle, &root.path().join(RETIREMENTS)).unwrap();
        collect_session_contents(&bundle, &root.path().join(RETIREMENTS), &inspection.id).unwrap();
        assert_eq!(registered_session_owners(root.path()).unwrap().len(), 1);
        let report = maintenance(root.path(), None).unwrap();
        assert_eq!(report["complete"], true, "{report}");
        assert_eq!(report["removed_bytes"], 0);
        assert!(registered_session_owners(root.path()).unwrap().is_empty());
        assert!(bundle.exists());
        assert_eq!(fs::read_dir(bundle).unwrap().count(), 0);
    }
}
