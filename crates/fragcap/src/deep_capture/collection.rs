// SPDX-License-Identifier: Apache-2.0

//! Exact whole-session collection, separate from sensitive artifact cleanup.
//! CLI callers additionally serialize maintenance and establish inactive owner leases.

mod filesystem;

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io;
#[cfg(windows)]
use std::io::Write;
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::sync::atomic::{AtomicU64, Ordering};

use super::{
    JournalStatus, ManifestDocument, ResourceKind, ResourceState, MANIFEST_PREFIX, RESOURCE_JOURNAL,
};
use filesystem::Object;
use serde_json::{json, Value};

const MAX_OBJECTS: usize = 256;
const MAX_DEPTH: usize = 8;
const MAX_PROVENANCE: u64 = 4 * 1024 * 1024;
const MAX_RETIREMENT: u64 = 256 * 1024;
#[cfg(windows)]
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub struct CollectionInspection {
    pub id: String,
    pub bundle: PathBuf,
    pub session_id: String,
    pub bytes: u64,
    pub files: usize,
    pub empty: bool,
    pub resumed: bool,
    pub owned: bool,
    pub completed_at_unix_ms: Option<u64>,
    pub retention: Option<Value>,
}

impl CollectionInspection {
    pub fn json(&self) -> Value {
        json!({"schema_version":1,"type":"session-collection.inspection","proposal_id":self.id,"bundle":self.bundle,"session_id":self.session_id,"recoverable_logical_bytes":self.bytes,"files":self.files,"empty":self.empty,"resumed":self.resumed,"owned":self.owned,"completed_at_unix_ms":self.completed_at_unix_ms,"retention":self.retention})
    }
}

#[derive(Clone, Debug)]
pub struct CollectionPathResult {
    pub path: PathBuf,
    pub status: String,
    pub bytes: u64,
    pub reason: String,
}

#[derive(Clone, Debug)]
pub struct CollectionReport {
    pub proposal_id: String,
    pub bundle: PathBuf,
    pub session_id: String,
    pub removed_files: usize,
    pub removed_bytes: u64,
    pub preserved_container: bool,
    pub purged: bool,
    pub complete: bool,
    pub paths: Vec<CollectionPathResult>,
    pub limitations: Vec<String>,
}

impl CollectionReport {
    pub fn json(&self) -> Value {
        json!({"schema_version":1,"type":"session-collection.result","proposal_id":self.proposal_id,"bundle":self.bundle,"session_id":self.session_id,"removed_files":self.removed_files,"removed_bytes":self.removed_bytes,"removed_logical_bytes":self.removed_bytes,"preserved_container":self.preserved_container,"purged":self.purged,"complete":self.complete,"paths":self.paths.iter().map(|p|json!({"path":p.path,"status":p.status,"bytes":p.bytes,"logical_bytes":p.bytes,"reason":p.reason})).collect::<Vec<_>>(),"limitations":self.limitations})
    }
}

// Portable inspection keeps the same snapshot shape; only Windows owns mutation.
#[cfg_attr(not(windows), allow(dead_code))]
struct Population {
    inspection: CollectionInspection,
    root: Object,
    objects: Vec<Object>,
    record: Option<Value>,
    store: PathBuf,
    record_path: PathBuf,
    _ancestors: Vec<File>,
}

/// Inspect one exact bounded session without changing files or retention state.
/// A proposal is filesystem/lifecycle authority only: callers must separately
/// establish inactive matching owner generations and serialize ownership changes.
pub fn inspect_session_collection(
    bundle: &Path,
    retirement_store: &Path,
) -> io::Result<CollectionInspection> {
    match population(bundle, retirement_store, false, false) {
        Ok(population) => Ok(population.inspection),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            missing_purge_inspection(bundle, retirement_store)?.ok_or(error)
        }
        Err(error) => Err(error),
    }
}

/// Recognize a genuinely empty container through exact completed retirement authority.
/// This read-only recognition does not establish absence of an active owner generation.
pub fn collected_session_container(bundle: &Path, retirement_store: &Path) -> io::Result<bool> {
    let population = population(bundle, retirement_store, false, false)?;
    population.root.verify()?;
    recheck_inventory(&population)?;
    Ok(population.inspection.owned
        && population.inspection.empty
        && population
            .record
            .as_ref()
            .is_some_and(|r| r["phase"] == "collected"))
}

/// Remove an unchanged selected population while preserving its session container.
/// Callers must establish inactive matching owner generations, authorize retention
/// selection, and serialize owner registration/recovery/maintenance through completion.
/// The facade validates filesystem and lifecycle authority, not the CLI owner registry.
pub fn collect_session_contents(
    bundle: &Path,
    retirement_store: &Path,
    authorization_id: &str,
) -> io::Result<CollectionReport> {
    apply(bundle, retirement_store, authorization_id, false)
}

/// Explicitly remove one already empty, exactly owned session container.
/// Callers must establish inactive matching owner generations and serialize all
/// ownership changes; an inspection identifier alone is not generation authority.
pub fn purge_session_container(
    bundle: &Path,
    retirement_store: &Path,
    authorization_id: &str,
) -> io::Result<CollectionReport> {
    apply(bundle, retirement_store, authorization_id, true)
}

fn apply(
    bundle: &Path,
    store: &Path,
    authorization_id: &str,
    purge: bool,
) -> io::Result<CollectionReport> {
    #[cfg(not(windows))]
    {
        let _ = (bundle, store, authorization_id, purge);
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "exact session collection requires the Windows object-handle adapter",
        ));
    }
    #[cfg(windows)]
    {
        if purge {
            if let Some(inspection) = missing_purge_inspection(bundle, store)? {
                if inspection.id != authorization_id {
                    return Err(invalid(
                        "purge proposal changed; preview again before retrying",
                    ));
                }
                let record_path = std::path::absolute(store)?
                    .join(format!("{}.json", path_key(&inspection.bundle)));
                let mut record = read_record(&record_path)?
                    .ok_or_else(|| invalid("purge retirement authority disappeared"))?;
                let _guards = prepare_store(&std::path::absolute(store)?)?;
                record["phase"] = json!("purged");
                write_record(&record_path, &record)?;
                return Ok(CollectionReport {
                    proposal_id: authorization_id.into(),
                    bundle: inspection.bundle,
                    session_id: inspection.session_id,
                    removed_files: 0,
                    removed_bytes: 0,
                    preserved_container: false,
                    purged: true,
                    complete: true,
                    paths: Vec::new(),
                    limitations: Vec::new(),
                });
            }
        }
        let mut population = population(bundle, store, true, purge)?;
        if authorization_id.is_empty() || population.inspection.id != authorization_id {
            return Err(invalid(
                "collection proposal changed; preview again before applying",
            ));
        }
        if !population.inspection.owned {
            return Err(invalid("empty container has no exact collection ownership"));
        }
        if purge && !population.inspection.empty {
            return Err(invalid(
                "container purge requires an already empty owned container",
            ));
        }
        // Check every handle and complete population before recording any effect.
        population.root.verify()?;
        for object in &population.objects {
            object.verify()?;
        }
        recheck_inventory(&population)?;
        let mut report = CollectionReport {
            proposal_id: authorization_id.into(),
            bundle: population.inspection.bundle.clone(),
            session_id: population.inspection.session_id.clone(),
            removed_files: 0,
            removed_bytes: 0,
            preserved_container: true,
            purged: false,
            complete: false,
            paths: Vec::new(),
            limitations: Vec::new(),
        };
        if population.inspection.empty && !purge {
            if let Some(mut record) = population.record {
                if record["phase"] == "collecting" {
                    let _guards = prepare_store(&population.store)?;
                    record["phase"] = json!("collected");
                    write_record(&population.record_path, &record)?;
                }
            }
            report.complete = true;
            return Ok(report);
        }
        let mut record=population.record.take().unwrap_or_else(||json!({
            "schema_version":1,"type":"session-collection.retirement","phase":"collecting",
            "bundle":population.inspection.bundle,"session_id":population.inspection.session_id,
            "root_identity":population.root.identity,"source_proposal_id":authorization_id,
            "completed_at_unix_ms":population.inspection.completed_at_unix_ms,
            "retention":population.inspection.retention,
            "entries":population.objects.iter().map(object_value).collect::<Vec<_>>(),"removed":[],
        }));
        // Establish protected, synchronized external authority before deleting content.
        let _store_guards = prepare_store(&population.store)?;
        write_record(&population.record_path, &record)?;
        checkpoint("record-prepared")?;
        if purge {
            record["phase"] = json!("purge-pending");
            write_record(&population.record_path, &record)?;
            let path = population.root.path.clone();
            match population.root.remove() {
                Ok(()) => {
                    report.preserved_container = false;
                    report.purged = true;
                    report.paths.push(CollectionPathResult {
                        path,
                        status: "purged".into(),
                        bytes: 0,
                        reason: "exact empty owned container removed".into(),
                    });
                    record["phase"] = json!("purged");
                    if let Err(error) = checkpoint("container-purged") {
                        report.limitations.push(error.to_string());
                        return Ok(report);
                    }
                    if let Err(error) = write_record(&population.record_path, &record) {
                        report.limitations.push(format!("container removed but retirement completion could not be synchronized: {error}"));
                    } else {
                        report.complete = true;
                    }
                }
                Err(error) => report
                    .limitations
                    .push(format!("empty container purge failed: {error}")),
            }
            return Ok(report);
        }
        population.objects.sort_by_key(|o| {
            let priority = if o.directory {
                3
            } else if o.path.file_name().is_some_and(|n| n == RESOURCE_JOURNAL) {
                2
            } else if o
                .path
                .file_name()
                .is_some_and(|n| n == "manifest.json" || n == MANIFEST_PREFIX)
            {
                1
            } else {
                0
            };
            (
                priority,
                std::cmp::Reverse(o.path.components().count()),
                o.path.clone(),
            )
        });
        let mut pending = population.objects.into_iter();
        for object in pending.by_ref() {
            let path = object.path.clone();
            let bytes = object.bytes;
            let directory = object.directory;
            let relative = relative_text(&population.inspection.bundle, &path)?;
            match object.remove() {
                Ok(()) => {
                    if !directory {
                        report.removed_files += 1;
                        report.removed_bytes = report.removed_bytes.saturating_add(bytes);
                    }
                    report.paths.push(CollectionPathResult {
                        path,
                        status: "removed".into(),
                        bytes,
                        reason: "original exact object is absent after handle closure".into(),
                    });
                    record["removed"]
                        .as_array_mut()
                        .expect("validated retirement removal list")
                        .push(json!(relative));
                    if let Err(error) = checkpoint(&format!(
                        "removed:{}",
                        report
                            .paths
                            .last()
                            .expect("removed result")
                            .path
                            .file_name()
                            .expect("selected name")
                            .to_string_lossy()
                    )) {
                        report.limitations.push(error.to_string());
                        break;
                    }
                    if let Err(error) = write_record(&population.record_path, &record) {
                        report.limitations.push(format!("removal completed but transaction progress could not be synchronized: {error}"));
                        break;
                    }
                }
                Err(error) => {
                    report.paths.push(CollectionPathResult {
                        path,
                        status: "failed".into(),
                        bytes: 0,
                        reason: error.to_string(),
                    });
                    report.limitations.push("collection stopped; protected retirement authority supports a fresh-preview retry".into());
                    break;
                }
            }
        }
        for object in pending {
            report.paths.push(CollectionPathResult {
                path: object.path,
                status: "not-attempted".into(),
                bytes: 0,
                reason: "earlier collection or transaction synchronization failed".into(),
            });
        }
        if report.limitations.is_empty() {
            if fs::read_dir(&population.inspection.bundle)?
                .next()
                .is_some()
            {
                report
                    .limitations
                    .push("container is not empty after selected removals".into());
            } else {
                record["phase"] = json!("collected");
                if let Err(error) = write_record(&population.record_path, &record) {
                    report.limitations.push(format!(
                        "empty container retirement could not be synchronized: {error}"
                    ));
                } else {
                    report.complete = true;
                }
            }
        }
        Ok(report)
    }
}

fn population(
    bundle: &Path,
    retirement_store: &Path,
    deleting: bool,
    purge: bool,
) -> io::Result<Population> {
    let mut ancestors = filesystem::guard_ancestors(bundle)?;
    let initial = Object::open(bundle, deleting && purge)?;
    if !initial.directory {
        return Err(invalid("session collection requires an exact directory"));
    }
    let root = bundle.canonicalize()?;
    // The opened root and every outer ancestor deny replacement until this population drops.
    validate_path_aliases(&root)?;
    let mut root_object = initial;
    root_object.path = root.clone();
    let store = std::path::absolute(retirement_store)?;
    if store.starts_with(&root) || root.starts_with(&store) {
        return Err(invalid(
            "retirement authority must be separate from the selected bundle",
        ));
    }
    validate_path_aliases(&store)?;
    match fs::symlink_metadata(&store) {
        Ok(_) => {
            ancestors.extend(filesystem::guard_ancestors(&store.join("authority.json"))?);
            #[cfg(windows)]
            filesystem::validate_private_store(&store)?;
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let key = path_key(&root);
    let record_path = store.join(format!("{key}.json"));
    let record = read_record(&record_path)?;
    let mut objects = Vec::new();
    inventory(&root, 0, deleting && !purge, &mut objects)?;
    let empty = objects.is_empty();
    let (session_id, completed_at, owned, resumed, authority, retention) =
        if let Some(record) = &record {
            validate_record(record, &root_object.path, &root_object.identity)?;
            validate_remaining(record, &root, &objects)?;
            let phase = required(record, "phase")?;
            if matches!(phase, "collected" | "purged" | "purge-pending") && !empty {
                return Err(invalid(
                "collected container received new contents; prior retirement cannot authorize them",
            ));
            }
            (
                required(record, "session_id")?.to_string(),
                record["completed_at_unix_ms"].as_u64(),
                true,
                phase == "collecting",
                record.clone(),
                record.get("retention").filter(|v| !v.is_null()).cloned(),
            )
        } else if empty {
            (String::new(), None, false, false, Value::Null, None)
        } else {
            let (session, time, proof, retention) = validate_provenance(&root, &mut objects)?;
            (session, time, true, false, proof, retention)
        };
    let mut bytes = 0u64;
    let mut files = 0usize;
    for object in &objects {
        if !object.directory {
            bytes = bytes
                .checked_add(object.bytes)
                .ok_or_else(|| invalid("collection byte total overflow"))?;
            files += 1;
        }
    }
    let snapshot = json!({"bundle":root,"root_identity":root_object.identity,"authority":authority,"objects":objects.iter().map(object_value).collect::<Vec<_>>()});
    let id = blake3::hash(&serde_json::to_vec(&snapshot).map_err(io::Error::other)?)
        .to_hex()
        .to_string();
    Ok(Population {
        inspection: CollectionInspection {
            id,
            bundle: root,
            session_id,
            bytes,
            files,
            empty,
            resumed,
            owned,
            completed_at_unix_ms: completed_at,
            retention,
        },
        root: root_object,
        objects,
        record,
        store,
        record_path,
        _ancestors: ancestors,
    })
}

fn inventory(
    directory: &Path,
    depth: usize,
    deleting: bool,
    objects: &mut Vec<Object>,
) -> io::Result<()> {
    if depth > MAX_DEPTH {
        return Err(invalid("session collection depth exceeds eight levels"));
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(directory)? {
        if entries.len() >= MAX_OBJECTS {
            return Err(invalid(
                "session collection directory population exceeds 256 objects",
            ));
        }
        entries.push(entry?.path());
    }
    entries.sort();
    for path in entries {
        if objects.len() >= MAX_OBJECTS {
            return Err(invalid("session collection population exceeds 256 objects"));
        }
        validate_path_aliases(&path)?;
        let object = Object::open(&path, deleting)?;
        let directory = object.directory;
        objects.push(object);
        if directory {
            inventory(&path, depth + 1, deleting, objects)?;
        }
    }
    Ok(())
}

fn validate_provenance(
    root: &Path,
    objects: &mut [Object],
) -> io::Result<(String, Option<u64>, Value, Option<Value>)> {
    let manifest_index = objects
        .iter()
        .position(|o| o.path == root.join("manifest.json"))
        .ok_or_else(|| invalid("whole-session collection requires a terminal manifest"))?;
    let manifest_bytes = objects[manifest_index].read(MAX_PROVENANCE)?;
    let manifest = ManifestDocument::parse(&manifest_bytes)?;
    let session = required(manifest.value(), "session_id")?;
    if !matches!(
        manifest.value()["state"].as_str(),
        Some("complete" | "partial" | "failed" | "interrupted")
    ) {
        return Err(invalid(
            "whole-session collection requires a terminal manifest",
        ));
    }
    let journal_index = objects
        .iter()
        .position(|o| o.path == root.join(RESOURCE_JOURNAL))
        .ok_or_else(|| invalid("whole-session collection requires a closed resource journal"))?;
    let journal_bytes = objects[journal_index].read(MAX_PROVENANCE)?;
    let journal = super::journal::read_resource_journal_bytes(&journal_bytes)?;
    if journal.status != JournalStatus::Complete || journal.session_id != session {
        return Err(invalid(
            "session collection manifest and closed journal identities disagree",
        ));
    }
    let text = std::str::from_utf8(&journal_bytes).map_err(io::Error::other)?;
    let trailer: Value = serde_json::from_str(
        text.lines()
            .last()
            .ok_or_else(|| invalid("resource journal trailer absent"))?,
    )
    .map_err(io::Error::other)?;
    if !journal_bytes.ends_with(b"\n")
        || trailer["type"] != "resource-journal.trailer"
        || trailer["schema_version"] != 1
        || trailer["session_id"] != session
        || trailer["records"].as_u64() != Some(journal.transitions.len() as u64)
    {
        return Err(invalid(
            "resource journal trailer identity, version or count is invalid",
        ));
    }
    for transition in journal.latest().into_values() {
        // A failed closed artifact writer is evidence production failure, not an
        // outstanding external effect. Exclusive deletion pins independently reject
        // a remaining writer; inactive generation authority belongs to the CLI.
        let settled = matches!(
            transition.state,
            ResourceState::Released | ResourceState::NotApplied
        ) || (transition.kind == ResourceKind::Artifact
            && matches!(
                transition.state,
                ResourceState::Retained | ResourceState::Failed | ResourceState::TimedOut
            ));
        if !settled {
            return Err(invalid(format!(
                "unresolved whole-session recovery obligation {} ({})",
                transition.resource_id,
                transition.state.as_str()
            )));
        }
    }
    let mut allowed = auxiliary_paths();
    let artifacts = manifest.value()["artifacts"]
        .as_array()
        .ok_or_else(|| invalid("manifest artifact population absent"))?;
    if artifacts.len() > MAX_OBJECTS {
        return Err(invalid("manifest artifact population exceeds bound"));
    }
    for artifact in artifacts {
        if let Some(path) = artifact["path"].as_str() {
            let path = super::validate_relative_path(path)?;
            validate_path_aliases(&path)?;
            allowed.insert(path);
        }
    }
    let mut directories = BTreeSet::new();
    for path in &allowed {
        for parent in path
            .ancestors()
            .skip(1)
            .filter(|p| !p.as_os_str().is_empty())
        {
            directories.insert(parent.to_path_buf());
        }
    }
    let mut retention = None;
    for object in objects.iter_mut() {
        let relative = object
            .path
            .strip_prefix(root)
            .map_err(|_| invalid("collection object escaped root"))?
            .to_path_buf();
        if if object.directory {
            !directories.contains(&relative)
        } else {
            !allowed.contains(&relative)
        } {
            return Err(invalid(format!(
                "unrecognized session object {}",
                relative.display()
            )));
        }
        if relative == Path::new(".sensitive-actions.jsonl") {
            validate_sensitive_journal(&object.read(MAX_PROVENANCE)?)?;
        }
        if relative == Path::new(".session-retention.json") {
            let value: Value =
                serde_json::from_slice(&object.read(4096)?).map_err(io::Error::other)?;
            validate_retention(&value)?;
            retention = Some(value);
        }
    }
    if let Some(sensitive) = manifest.value().get("sensitive_artifacts") {
        let declared = sensitive
            .get("retention")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("manifest sensitive artifact retention is malformed"))?;
        let managed = retention
            .as_ref()
            .is_some_and(|policy| policy["storage_class"] == "managed-history");
        match declared {
            "retain-until-explicit-cleanup" if !managed => {}
            "managed-history-30-days-20-sessions-2-gib" if managed => {
                let policy = retention.as_ref().expect("managed sidecar");
                if policy["max_age_seconds"] != 2592000
                    || policy["max_sessions"] != 20
                    || policy["max_bytes"] != 2147483648u64
                {
                    return Err(invalid(
                        "managed manifest and session retention limits disagree",
                    ));
                }
            }
            "retain-until-explicit-cleanup" | "managed-history-30-days-20-sessions-2-gib" => {
                return Err(invalid("manifest and session retention policy disagree"))
            }
            _ => {
                return Err(invalid(
                    "manifest sensitive artifact retention is unsupported",
                ))
            }
        }
    }
    #[cfg(windows)]
    let completed_at = objects[manifest_index]
        .modified
        .checked_sub(116_444_736_000_000_000)
        .map(|v| v / 10_000);
    #[cfg(not(windows))]
    let completed_at = Some(objects[manifest_index].modified / 1_000_000);
    Ok((
        session.into(),
        completed_at,
        json!({"manifest":blake3::hash(&manifest_bytes).to_hex().to_string(),"journal":blake3::hash(&journal_bytes).to_hex().to_string(),"retention":retention}),
        retention,
    ))
}

fn validate_sensitive_journal(bytes: &[u8]) -> io::Result<()> {
    let text = std::str::from_utf8(bytes).map_err(io::Error::other)?;
    if !bytes.ends_with(b"\n") {
        return Err(invalid(
            "sensitive action journal is interrupted or malformed",
        ));
    }
    let mut pending = BTreeSet::new();
    for (index, line) in text.lines().enumerate() {
        if index >= 4096 {
            return Err(invalid("sensitive action journal exceeds record bound"));
        }
        let value: Value = serde_json::from_str(line).map_err(io::Error::other)?;
        if value["version"] != 1 {
            return Err(invalid("sensitive action journal version unsupported"));
        }
        if index == 0 {
            if value["type"] != "header" {
                return Err(invalid("sensitive action journal header missing"));
            }
        } else {
            if value["op"] != "delete"
                || !matches!(value["phase"].as_str(), Some("intent" | "result"))
            {
                return Err(invalid("sensitive action journal operation unsupported"));
            }
            let path = super::validate_relative_path(required(&value, "path")?)?;
            validate_path_aliases(&path)?;
            if value["phase"] == "intent" {
                if value["status"] != "pending" {
                    return Err(invalid("sensitive action intent status invalid"));
                }
                pending.insert(path);
            } else {
                match value["status"].as_str() {
                    Some("removed" | "already-absent") => {
                        pending.remove(&path);
                    }
                    Some("failed") => {
                        pending.insert(path);
                    }
                    _ => return Err(invalid("sensitive action result status invalid")),
                }
            }
        }
    }
    if text.lines().next().is_none() {
        return Err(invalid("sensitive action journal header missing"));
    }
    if !pending.is_empty() {
        return Err(invalid(
            "sensitive action journal has unresolved cleanup obligations",
        ));
    }
    Ok(())
}

fn auxiliary_paths() -> BTreeSet<PathBuf> {
    [
        "manifest.json",
        MANIFEST_PREFIX,
        RESOURCE_JOURNAL,
        ".sensitive-actions.jsonl",
        ".session-retention.json",
        "capture.fcapng",
        "application.jsonl",
        "proxy.jsonl",
        "cleanup.jsonl",
        "process-trace.jsonl",
        "compatibility.json",
        "cleanup.json",
        "http.har",
        "tls-keylog.log",
        "manifest.json.tmp",
        "manifest.prefix.json.tmp",
        "resource-journal.jsonl.tmp",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect()
}

fn object_value(object: &Object) -> Value {
    json!({"path":object.path,"identity":object.identity,"directory":object.directory,"bytes":object.bytes,"modified":if object.directory{0}else{object.modified}})
}

fn read_record(path: &Path) -> io::Result<Option<Value>> {
    if !path.exists() {
        // exists() deliberately cannot convert access denial or a dangling link into absence.
        match fs::symlink_metadata(path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
            Ok(_) => {}
        }
    }
    let _guards = filesystem::guard_ancestors(path)?;
    #[cfg(windows)]
    filesystem::validate_private_store(
        path.parent()
            .ok_or_else(|| invalid("retirement record has no store"))?,
    )?;
    let mut object = Object::open(path, false)?;
    let (value, _) = parse_record_bytes(&object.read(MAX_RETIREMENT)?)?;
    Ok(Some(value))
}

fn parse_record_bytes(bytes: &[u8]) -> io::Result<(Value, usize)> {
    let prefix = bytes
        .iter()
        .rposition(|b| *b == b'\n')
        .map(|p| p + 1)
        .ok_or_else(|| invalid("retirement authority has no committed header"))?;
    let text = std::str::from_utf8(&bytes[..prefix]).map_err(io::Error::other)?;
    let mut lines = text.lines();
    let mut record: Value = serde_json::from_str(
        lines
            .next()
            .ok_or_else(|| invalid("retirement authority is empty"))?,
    )
    .map_err(io::Error::other)?;
    for (index, line) in lines.enumerate() {
        if index >= MAX_OBJECTS + 16 {
            return Err(invalid("retirement progress exceeds record bound"));
        }
        let delta: Value = serde_json::from_str(line).map_err(io::Error::other)?;
        if delta["schema_version"] != 1 || delta["type"] != "session-collection.progress" {
            return Err(invalid("retirement progress version/type unsupported"));
        }
        let new = delta["removed"]
            .as_array()
            .ok_or_else(|| invalid("retirement progress population invalid"))?;
        let removed = record["removed"]
            .as_array_mut()
            .ok_or_else(|| invalid("retirement authority population invalid"))?;
        removed.extend(new.iter().cloned());
        if removed.len() > MAX_OBJECTS {
            return Err(invalid("retirement progress population exceeds bound"));
        }
        record["phase"] = delta["phase"].clone();
    }
    Ok((record, prefix))
}

fn validate_record(record: &Value, root: &Path, identity: &Value) -> io::Result<()> {
    if record["schema_version"] != 1
        || record["type"] != "session-collection.retirement"
        || record["root_identity"] != *identity
        || record["bundle"].as_str() != root.to_str()
        || !matches!(
            record["phase"].as_str(),
            Some("collecting" | "collected" | "purge-pending" | "purged")
        )
    {
        return Err(invalid(
            "retirement authority does not match this exact session container",
        ));
    }
    if identity["volume"].as_u64().is_none()
        || identity["index"].as_u64().is_none()
        || identity["directory"] != true
    {
        return Err(invalid("retirement root identity invalid"));
    }
    required(record, "session_id")?;
    if let Some(retention) = record.get("retention").filter(|v| !v.is_null()) {
        validate_retention(retention)?;
    }
    let source = required(record, "source_proposal_id")?;
    if source.len() != 64 || !source.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid("retirement source proposal identity invalid"));
    }
    let entries = record["entries"]
        .as_array()
        .filter(|e| e.len() <= MAX_OBJECTS)
        .ok_or_else(|| invalid("retirement population invalid or exceeds bound"))?;
    let mut paths = BTreeSet::new();
    for entry in entries {
        let path = PathBuf::from(required(entry, "path")?);
        let relative = path
            .strip_prefix(root)
            .map_err(|_| invalid("retirement object escaped selected root"))?;
        if relative.as_os_str().is_empty() {
            return Err(invalid("retirement entry cannot name session root"));
        }
        validate_path_aliases(relative)?;
        if !paths.insert(path)
            || entry["bytes"].as_u64().is_none()
            || entry["modified"].as_u64().is_none()
            || entry["directory"].as_bool().is_none()
            || entry["identity"]["volume"].as_u64().is_none()
            || entry["identity"]["index"].as_u64().is_none()
        {
            return Err(invalid("retirement object metadata is invalid"));
        }
    }
    let removed = record["removed"]
        .as_array()
        .filter(|a| a.len() <= MAX_OBJECTS)
        .ok_or_else(|| invalid("retirement progress invalid or exceeds bound"))?;
    let mut seen = BTreeSet::new();
    for path in removed {
        let relative = super::validate_relative_path(
            path.as_str()
                .ok_or_else(|| invalid("retirement removal path invalid"))?,
        )?;
        validate_path_aliases(&relative)?;
        if !paths.contains(&root.join(&relative)) || !seen.insert(relative) {
            return Err(invalid(
                "retirement progress names an unselected or duplicate object",
            ));
        }
    }
    Ok(())
}

fn validate_retention(value: &Value) -> io::Result<()> {
    if value["schema_version"] != 1
        || value["created_at_unix_ms"].as_u64().is_none()
        || value["preserve_empty_containers"] != true
    {
        return Err(invalid("session retention policy unsupported or malformed"));
    }
    match value["storage_class"].as_str() {
        Some("managed-history")
            if value["max_age_seconds"] == 2592000
                && value["max_sessions"] == 20
                && value["max_bytes"] == 2147483648u64 =>
        {
            Ok(())
        }
        Some("explicitly-retained")
            if value["max_age_seconds"].is_null()
                && value["max_sessions"].is_null()
                && value["max_bytes"].is_null() =>
        {
            Ok(())
        }
        _ => Err(invalid(
            "session retention class or finite limits unsupported",
        )),
    }
}

fn missing_purge_inspection(
    bundle: &Path,
    store: &Path,
) -> io::Result<Option<CollectionInspection>> {
    match fs::symlink_metadata(bundle) {
        Ok(_) => return Ok(None),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    let _guards = filesystem::guard_ancestors(bundle)?;
    let absolute = std::path::absolute(bundle)?;
    validate_path_aliases(&absolute)?;
    let parent = absolute
        .parent()
        .ok_or_else(|| invalid("purged container has no parent"))?
        .canonicalize()?;
    let root = parent.join(
        absolute
            .file_name()
            .ok_or_else(|| invalid("purged container has no name"))?,
    );
    let record_path = std::path::absolute(store)?.join(format!("{}.json", path_key(&root)));
    let Some(record) = read_record(&record_path)? else {
        return Ok(None);
    };
    validate_record(&record, &root, &record["root_identity"])?;
    if !matches!(record["phase"].as_str(), Some("purge-pending" | "purged")) {
        return Ok(None);
    }
    let id = blake3::hash(
        &serde_json::to_vec(&json!({"missing_owned_container":root,"retirement":record}))
            .map_err(io::Error::other)?,
    )
    .to_hex()
    .to_string();
    Ok(Some(CollectionInspection {
        id,
        bundle: root,
        session_id: required(&record, "session_id")?.into(),
        bytes: 0,
        files: 0,
        empty: true,
        resumed: record["phase"] == "purge-pending",
        owned: true,
        completed_at_unix_ms: record["completed_at_unix_ms"].as_u64(),
        retention: record.get("retention").filter(|v| !v.is_null()).cloned(),
    }))
}

fn validate_remaining(record: &Value, root: &Path, objects: &[Object]) -> io::Result<()> {
    let expected = record["entries"]
        .as_array()
        .expect("validated retirement entries")
        .iter()
        .map(|e| {
            (
                PathBuf::from(e["path"].as_str().expect("validated entry path")),
                e,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let removed = record["removed"]
        .as_array()
        .expect("validated progress")
        .iter()
        .map(|p| root.join(p.as_str().expect("validated removal path")))
        .collect::<BTreeSet<_>>();
    for object in objects {
        if removed.contains(&object.path)
            || expected
                .get(&object.path)
                .is_none_or(|entry| **entry != object_value(object))
        {
            return Err(invalid("session population changed after retirement began; replacement objects are not collectible"));
        }
    }
    Ok(())
}

fn recheck_inventory(population: &Population) -> io::Result<()> {
    let mut actual = Vec::new();
    // Opening a second DELETE handle would conflict with held no-delete sharing.
    // A read-only lookup verifies names through the retained exact root.
    fn walk(path: &Path, depth: usize, actual: &mut Vec<PathBuf>) -> io::Result<()> {
        if depth > MAX_DEPTH {
            return Err(invalid("session collection depth changed"));
        }
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            actual.push(path.clone());
            if actual.len() > MAX_OBJECTS {
                return Err(invalid("session collection population changed"));
            }
            if entry.file_type()?.is_dir() {
                walk(&path, depth + 1, actual)?;
            }
        }
        Ok(())
    }
    walk(&population.root.path, 0, &mut actual)?;
    actual.sort();
    let mut expected = population
        .objects
        .iter()
        .map(|o| o.path.clone())
        .collect::<Vec<_>>();
    expected.sort();
    if actual != expected {
        return Err(invalid("session population changed after inspection"));
    }
    Ok(())
}

#[cfg(windows)]
fn prepare_store(store: &Path) -> io::Result<Vec<File>> {
    let guards = filesystem::guard_ancestors(store)?;
    filesystem::create_private_store(store)?;
    let object = Object::open(store, false)?;
    if !object.directory {
        return Err(invalid("retirement store is not an exact directory"));
    }
    let mut result = guards;
    // Hold the store itself against rename/reparse substitution until all writes finish.
    result.extend(filesystem::guard_ancestors(
        &store.join("transaction.json"),
    )?);
    Ok(result)
}

#[cfg(windows)]
fn write_record(path: &Path, record: &Value) -> io::Result<()> {
    let bytes = serde_json::to_vec(record).map_err(io::Error::other)?;
    if bytes.len() as u64 > MAX_RETIREMENT {
        return Err(invalid("retirement record exceeds byte bound"));
    }
    match Object::open_for_append(path) {
        Ok(mut object) => {
            let (old, prefix) = parse_record_bytes(&object.read(MAX_RETIREMENT)?)?;
            for key in [
                "schema_version",
                "type",
                "bundle",
                "session_id",
                "root_identity",
                "source_proposal_id",
                "completed_at_unix_ms",
                "entries",
                "retention",
            ] {
                if old[key] != record[key] {
                    return Err(invalid(
                        "retirement authority changed before synchronized progress",
                    ));
                }
            }
            let previous = old["removed"]
                .as_array()
                .ok_or_else(|| invalid("retirement progress invalid"))?;
            let next = record["removed"]
                .as_array()
                .ok_or_else(|| invalid("retirement progress invalid"))?;
            if !next.starts_with(previous) {
                return Err(invalid(
                    "retirement progress cannot erase committed removals",
                ));
            }
            if old["phase"] == record["phase"] && next.len() == previous.len() {
                return Ok(());
            }
            let delta = json!({"schema_version":1,"type":"session-collection.progress","removed":&next[previous.len()..],"phase":record["phase"]});
            let mut data = serde_json::to_vec(&delta).map_err(io::Error::other)?;
            data.push(b'\n');
            if prefix as u64 + data.len() as u64 > MAX_RETIREMENT {
                return Err(invalid("retirement progress exceeds byte bound"));
            }
            return object.append_at(prefix as u64, &data);
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let temp = path.with_extension(format!(
        "{}.{}.tmp",
        std::process::id(),
        TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = super::access::open_producer_private_file(&temp)?;
    let identity = filesystem::file_identity(&file)?;
    let write_result = (|| {
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
        file.sync_all()
    })();
    drop(file);
    if let Err(error) = write_result {
        return Err(cleanup_temporary_record(&temp, &identity, error));
    }
    if let Err(error) = checkpoint("initial-record-commit") {
        return Err(cleanup_temporary_record(&temp, &identity, error));
    }
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_WRITE_THROUGH};
    let from = temp
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let to = path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), MOVEFILE_WRITE_THROUGH) } == 0 {
        return Err(cleanup_temporary_record(
            &temp,
            &identity,
            io::Error::last_os_error(),
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn cleanup_temporary_record(path: &Path, identity: &Value, error: io::Error) -> io::Error {
    let cleanup = (|| {
        let object = Object::open(path, true)?;
        if object.identity != *identity {
            return Err(invalid(
                "temporary retirement identity changed; replacement preserved",
            ));
        }
        object.remove()
    })();
    match cleanup {
        Ok(()) => error,
        Err(cleanup) => io::Error::other(format!(
            "{error}; temporary retirement cleanup remains unresolved: {cleanup}"
        )),
    }
}

fn path_key(path: &Path) -> String {
    #[cfg(windows)]
    let text = path.to_string_lossy().to_lowercase();
    #[cfg(not(windows))]
    let text = path.to_string_lossy().into_owned();
    blake3::hash(text.as_bytes()).to_hex().to_string()
}

fn validate_path_aliases(path: &Path) -> io::Result<()> {
    for component in path.components() {
        match component {
            std::path::Component::ParentDir | std::path::Component::CurDir => {
                return Err(invalid("collection path contains normalized components"))
            }
            std::path::Component::Normal(name) => {
                let text = name
                    .to_str()
                    .ok_or_else(|| invalid("collection path is not valid Unicode"))?;
                if text.contains(':')
                    || text.ends_with('.')
                    || text.ends_with(' ')
                    || text.chars().any(char::is_control)
                {
                    return Err(invalid(
                        "collection refuses alternate-stream or normalized aliases",
                    ));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(windows)]
fn relative_text(root: &Path, path: &Path) -> io::Result<String> {
    Ok(path
        .strip_prefix(root)
        .map_err(|_| invalid("collection object escaped root"))?
        .to_str()
        .ok_or_else(|| invalid("collection path is not Unicode"))?
        .replace('\\', "/"))
}

fn required<'a>(value: &'a Value, key: &str) -> io::Result<&'a str> {
    value[key]
        .as_str()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| invalid(format!("missing exact {key}")))
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

#[cfg(all(test, windows))]
thread_local! {static FAILURE:std::cell::RefCell<Option<String>>=const {std::cell::RefCell::new(None)};}

#[cfg(windows)]
fn checkpoint(stage: &str) -> io::Result<()> {
    #[cfg(test)]
    if FAILURE.with(|failure| failure.borrow().as_deref() == Some(stage)) {
        return Err(io::Error::other(format!(
            "injected collection interruption at {stage}"
        )));
    }
    let _ = stage;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(parent: &Path) -> (PathBuf, PathBuf) {
        let bundle = parent.join("session");
        super::super::prepare_bundle(&bundle).unwrap();
        let mut journal =
            super::super::ResourceJournal::create(&bundle, "fault-fixture", "plan").unwrap();
        journal.finish().unwrap();
        drop(journal);
        fs::write(bundle.join("capture.fcapng"), b"synthetic-fault-fixture\n").unwrap();
        fs::write(bundle.join("manifest.json"),serde_json::to_vec(&json!({"manifest_version":1,"session_id":"fault-fixture","state":"complete","artifacts":[{"path":"capture.fcapng"}]})).unwrap()).unwrap();
        (bundle, parent.join(".session-retirements"))
    }

    #[cfg(windows)]
    #[test]
    fn every_content_interruption_checkpoint_resumes_without_losing_authority() {
        for checkpoint in [
            "record-prepared",
            "removed:capture.fcapng",
            "removed:manifest.json",
            "removed:resource-journal.jsonl",
        ] {
            let temp = tempfile::tempdir().unwrap();
            let (bundle, store) = fixture(temp.path());
            let preview = inspect_session_collection(&bundle, &store).unwrap();
            FAILURE.with(|failure| *failure.borrow_mut() = Some(checkpoint.into()));
            let result = collect_session_contents(&bundle, &store, &preview.id);
            FAILURE.with(|failure| *failure.borrow_mut() = None);
            assert!(
                result.is_err() || !result.as_ref().unwrap().complete,
                "checkpoint {checkpoint}"
            );
            let retry = inspect_session_collection(&bundle, &store).unwrap();
            assert!(retry.resumed, "checkpoint {checkpoint}");
            let result = collect_session_contents(&bundle, &store, &retry.id).unwrap();
            assert!(result.complete, "checkpoint {checkpoint}: {result:?}");
            assert!(collected_session_container(&bundle, &store).unwrap());
            assert_eq!(fs::read_dir(&bundle).unwrap().count(), 0);
        }
    }

    #[cfg(windows)]
    #[test]
    fn interrupted_empty_purge_reconciles_absence_through_external_authority() {
        let temp = tempfile::tempdir().unwrap();
        let (bundle, store) = fixture(temp.path());
        let preview = inspect_session_collection(&bundle, &store).unwrap();
        collect_session_contents(&bundle, &store, &preview.id).unwrap();
        let preview = inspect_session_collection(&bundle, &store).unwrap();
        FAILURE.with(|failure| *failure.borrow_mut() = Some("container-purged".into()));
        let result = purge_session_container(&bundle, &store, &preview.id).unwrap();
        FAILURE.with(|failure| *failure.borrow_mut() = None);
        assert!(result.purged && !result.complete);
        assert!(!bundle.exists());
        let retry = inspect_session_collection(&bundle, &store).unwrap();
        assert!(retry.resumed && retry.owned);
        assert!(
            purge_session_container(&bundle, &store, &retry.id)
                .unwrap()
                .complete
        );
    }

    #[cfg(windows)]
    #[test]
    fn torn_progress_is_discarded_only_after_a_fresh_validated_preview() {
        let temp = tempfile::tempdir().unwrap();
        let (bundle, store) = fixture(temp.path());
        let preview = inspect_session_collection(&bundle, &store).unwrap();
        FAILURE.with(|failure| *failure.borrow_mut() = Some("removed:manifest.json".into()));
        let result = collect_session_contents(&bundle, &store, &preview.id).unwrap();
        FAILURE.with(|failure| *failure.borrow_mut() = None);
        assert!(!result.complete);
        let path = fs::read_dir(&store)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.extension().is_some_and(|e| e == "json"))
            .unwrap();
        let mut file = fs::OpenOptions::new().append(true).open(&path).unwrap();
        file.write_all(b"{\"schema_version\":1,\"type\":").unwrap();
        file.sync_all().unwrap();
        drop(file);
        let preview = inspect_session_collection(&bundle, &store).unwrap();
        assert!(preview.resumed);
        assert!(
            collect_session_contents(&bundle, &store, &preview.id)
                .unwrap()
                .complete
        );
        assert!(collected_session_container(&bundle, &store).unwrap());
    }

    #[cfg(windows)]
    #[test]
    fn retirement_checkpoint_refuses_a_replaced_original_file() {
        let temp = tempfile::tempdir().unwrap();
        let (bundle, store) = fixture(temp.path());
        let preview = inspect_session_collection(&bundle, &store).unwrap();
        FAILURE.with(|failure| *failure.borrow_mut() = Some("record-prepared".into()));
        assert!(collect_session_contents(&bundle, &store, &preview.id).is_err());
        FAILURE.with(|failure| *failure.borrow_mut() = None);
        let capture = bundle.join("capture.fcapng");
        fs::rename(&capture, temp.path().join("original-file")).unwrap();
        fs::write(&capture, b"replacement must remain").unwrap();
        assert!(inspect_session_collection(&bundle, &store).is_err());
        assert_eq!(fs::read(capture).unwrap(), b"replacement must remain");
    }

    #[cfg(windows)]
    #[test]
    fn failed_initial_retirement_commit_reclaims_only_its_created_temporary_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let (bundle, store) = fixture(temp.path());
        let preview = inspect_session_collection(&bundle, &store).unwrap();
        FAILURE.with(|failure| *failure.borrow_mut() = Some("initial-record-commit".into()));
        assert!(collect_session_contents(&bundle, &store, &preview.id).is_err());
        FAILURE.with(|failure| *failure.borrow_mut() = None);
        assert_eq!(fs::read_dir(&store).unwrap().count(), 0);
        assert!(bundle.join("manifest.json").exists());
        let preview = inspect_session_collection(&bundle, &store).unwrap();
        assert!(!preview.resumed);
        assert!(
            collect_session_contents(&bundle, &store, &preview.id)
                .unwrap()
                .complete
        );
    }

    #[test]
    fn portable_preview_has_no_storage_effect() {
        let temp = tempfile::tempdir().unwrap();
        let (bundle, store) = fixture(temp.path());
        let preview = inspect_session_collection(&bundle, &store).unwrap();
        assert!(preview.owned && preview.files >= 3 && preview.bytes > 0);
        assert!(!store.exists());
        #[cfg(not(windows))]
        assert_eq!(
            collect_session_contents(&bundle, &store, &preview.id)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
    }

    #[cfg(windows)]
    #[test]
    fn retained_ancestor_guard_blocks_live_directory_rename() {
        let temp = tempfile::tempdir().unwrap();
        let ancestor = temp.path().join("held-ancestor");
        fs::create_dir(&ancestor).unwrap();
        let descendant = ancestor.join("selected-session");
        fs::create_dir(&descendant).unwrap();
        let guards = filesystem::guard_ancestors(&descendant).unwrap();
        let replacement = temp.path().join("moved-ancestor");
        assert!(
            fs::rename(&ancestor, &replacement).is_err(),
            "a zero-access handle did not retain ancestor path identity"
        );
        assert!(ancestor.exists() && !replacement.exists());
        drop(guards);
        fs::rename(&ancestor, &replacement).unwrap();
    }
}
