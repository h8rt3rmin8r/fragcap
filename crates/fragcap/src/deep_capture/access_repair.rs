// SPDX-License-Identifier: Apache-2.0

//! Exact, content-preserving inspection and repair of retained bundle access.

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use super::access::{pin_access_path, OutputRecipient, PinnedAccessPath};
use super::{
    read_resource_journal, JournalStatus, ManifestDocument, MANIFEST_PREFIX, RESOURCE_JOURNAL,
};

const MAX_PATHS: usize = 256;
const MAX_DEPTH: usize = 8;
const MAX_PROVENANCE_BYTES: u64 = 4 * 1024 * 1024;

/// One exact object included in the read-only permission proposal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundleAccessInspectionPath {
    pub path: PathBuf,
    pub directory: bool,
    pub traversal_only: bool,
    pub identity: String,
    pub descriptor: String,
}

/// A current recipient-bound proposal. Its identifier authorizes only this snapshot.
#[derive(Clone, Debug)]
pub struct BundleAccessInspection {
    pub id: String,
    pub bundle: PathBuf,
    pub recipient_sid: String,
    pub session_id: String,
    pub repairable: bool,
    pub paths: Vec<BundleAccessInspectionPath>,
    pub access: Value,
    pub limitations: Vec<String>,
}

impl BundleAccessInspection {
    pub fn json(&self) -> Value {
        json!({
            "schema_version": 1, "type": "bundle-access.inspection",
            "inspection_id": self.id, "bundle": self.bundle,
            "recipient_sid": self.recipient_sid, "session_id": self.session_id,
            "repairable": self.repairable, "limitations": self.limitations, "access": self.access,
            "paths": self.paths.iter().map(|path| json!({
                "path": path.path, "directory": path.directory,
                "proposal": if path.traversal_only { "exact-recipient-traversal" } else { "private-recipient-contract" },
                "identity": path.identity, "descriptor": path.descriptor,
            })).collect::<Vec<_>>()
        })
    }
}

/// One actual permission effect, independent of evidence and lifecycle content.
#[derive(Clone, Debug)]
pub struct BundleAccessRepairPath {
    pub path: PathBuf,
    pub status: String,
    pub reason: String,
}

/// Repair results are returned to the caller and never appended to retained evidence.
#[derive(Clone, Debug)]
pub struct BundleAccessRepairReport {
    pub inspection_id: String,
    pub recipient_sid: String,
    pub verified: bool,
    pub verification: Value,
    pub paths: Vec<BundleAccessRepairPath>,
    pub limitations: Vec<String>,
}

impl BundleAccessRepairReport {
    pub fn json(&self) -> Value {
        json!({
            "schema_version": 1, "type": "bundle-access.repair",
            "inspection_id": self.inspection_id, "recipient_sid": self.recipient_sid,
            "verified": self.verified, "verification": self.verification, "limitations": self.limitations,
            "paths": self.paths.iter().map(|path| json!({
                "path": path.path, "status": path.status, "reason": path.reason,
            })).collect::<Vec<_>>()
        })
    }
}

struct Population {
    inspection: BundleAccessInspection,
    pinned: Vec<(PinnedAccessPath, bool)>,
}

/// Inspect one recognized complete population without changing security or content.
///
/// The CLI additionally checks the exact generation-qualified session owner lease.
pub fn inspect_bundle_access(
    bundle: &Path,
    recipient: &OutputRecipient,
    owned_sessions_container: Option<&Path>,
) -> io::Result<BundleAccessInspection> {
    Ok(population(bundle, recipient, owned_sessions_container, false)?.inspection)
}

/// Recompute and authorize one exact preview before any permission change.
pub fn repair_bundle_access(
    bundle: &Path,
    recipient: &OutputRecipient,
    owned_sessions_container: Option<&Path>,
    authorization_id: &str,
) -> io::Result<BundleAccessRepairReport> {
    let population = population(bundle, recipient, owned_sessions_container, true)?;
    if !population.inspection.repairable {
        return Err(invalid(population.inspection.limitations.join("; ")));
    }
    if authorization_id.is_empty() || authorization_id != population.inspection.id {
        return Err(invalid(
            "access inspection changed; inspect again before authorizing repair",
        ));
    }
    apply_population(population, recipient, bundle)
}

fn apply_population(
    population: Population,
    recipient: &OutputRecipient,
    bundle: &Path,
) -> io::Result<BundleAccessRepairReport> {
    recipient.revalidate()?;
    // Every object is already pinned. Recheck all descriptors before the first effect.
    for (object, _) in &population.pinned {
        let (identity, descriptor) = object.refresh()?;
        if identity != object.identity || descriptor != object.descriptor {
            return Err(invalid("access object changed after inspection"));
        }
    }
    recheck_population(&population.inspection)?;
    let mut report = BundleAccessRepairReport {
        inspection_id: population.inspection.id.clone(),
        recipient_sid: recipient.sid().to_string(),
        verified: false,
        verification: json!({"state":"unresolved"}),
        paths: Vec::new(),
        limitations: Vec::new(),
    };
    for (object, traversal_only) in &population.pinned {
        let effect = if *traversal_only {
            object.apply_traversal(recipient)
        } else {
            object.apply(recipient)
        };
        let (status, reason) = match effect {
            Ok(()) => match object.refresh() {
                Ok((identity, descriptor)) if identity == object.identity => (
                    if descriptor == object.descriptor {
                        "unchanged"
                    } else {
                        "applied"
                    },
                    "exact object permission contract applied".to_string(),
                ),
                Ok(_) => (
                    "failed",
                    "object identity changed during permission repair".to_string(),
                ),
                Err(error) => (
                    "failed",
                    format!("permission result could not be read: {error}"),
                ),
            },
            Err(error) => ("failed", error.to_string()),
        };
        report.paths.push(BundleAccessRepairPath {
            path: object.path.clone(),
            status: status.to_string(),
            reason,
        });
    }
    // Exclusive directory handles prevent unchecked propagation and must be released
    // before recipient-context enumeration. No file contents were opened for writing.
    drop(population.pinned);
    if let Err(error) = recheck_population(&population.inspection) {
        report
            .limitations
            .push(format!("bundle population changed during repair: {error}"));
    }
    let mut paths = report
        .paths
        .iter()
        .map(|path| path.path.clone())
        .collect::<Vec<_>>();
    paths.extend(required_ancestors(&population.inspection.bundle));
    paths.sort();
    paths.dedup();
    match recipient.verify_paths(bundle, &paths) {
        Ok(verification) => {
            report.verification = verification.as_json();
            report.verified = verification.state == "verified"
                && report.limitations.is_empty()
                && report.paths.iter().all(|path| path.status != "failed");
            for path in &mut report.paths {
                if matches!(path.status.as_str(), "applied" | "unchanged")
                    && verification
                        .paths
                        .iter()
                        .any(|outcome| outcome.path == path.path && outcome.state != "verified")
                {
                    path.status = "verification-failed".to_string();
                    path.reason
                        .push_str("; this object's recipient access was not verified");
                }
            }
            if !report.verified {
                report
                    .limitations
                    .push("recipient-context access remains unresolved".to_string());
            }
        }
        Err(error) => {
            report
                .limitations
                .push(format!("recipient-context verification failed: {error}"));
            for path in &mut report.paths {
                if matches!(path.status.as_str(), "applied" | "unchanged") {
                    path.status = "verification-failed".to_string();
                    path.reason
                        .push_str("; recipient-context verification could not complete");
                }
            }
        }
    }
    Ok(report)
}

fn population(
    bundle: &Path,
    recipient: &OutputRecipient,
    owned_sessions_container: Option<&Path>,
    write_dacl: bool,
) -> io::Result<Population> {
    recipient.revalidate()?;
    let mut root_object = pin_access_path(bundle, write_dacl)?;
    if !root_object.identity.directory {
        return Err(invalid("bundle access requires a directory"));
    }
    let root = bundle.canonicalize()?;
    if pin_access_path(&root, false)?.identity != root_object.identity {
        return Err(invalid(
            "bundle location changed while establishing its exact root",
        ));
    }
    root_object.path = root.clone();
    let mut pinned = vec![(root_object, false)];
    let mut inventory = BTreeSet::new();
    inventory_paths(&root, &root, 0, &mut inventory)?;
    // Pin the complete inventory before parsing provenance or applying a descriptor.
    for path in &inventory {
        pinned.push((pin_access_path(path, write_dacl)?, false));
    }
    let manifest_path = if inventory.contains(&root.join("manifest.json")) {
        root.join("manifest.json")
    } else {
        root.join(MANIFEST_PREFIX)
    };
    let manifest_bytes = bounded_read(&manifest_path)?;
    let manifest = ManifestDocument::parse(&manifest_bytes)?;
    let session_id = manifest.value()["session_id"]
        .as_str()
        .filter(|id| !id.is_empty())
        .ok_or_else(|| invalid("bundle manifest has no exact session identity"))?;
    let journal_path = root.join(RESOURCE_JOURNAL);
    let journal_bytes = bounded_read(&journal_path)?;
    let journal = read_resource_journal(&journal_path)?;
    if journal.status == JournalStatus::UnknownVersion || journal.session_id != session_id {
        return Err(invalid(
            "bundle and resource journal identities do not agree",
        ));
    }
    // A descriptor-only operation does not acquire the live session's lease.
    // Without a library owner-lease capability, only closed historical output
    // can supply inactive-writer evidence. The CLI additionally rejects active
    // generation leases even when a terminal manifest is already present.
    if journal.status != JournalStatus::Complete
        || !matches!(
            manifest.value()["state"].as_str(),
            Some("complete" | "partial" | "failed" | "interrupted")
        )
    {
        return Err(invalid("access repair requires a terminal manifest and closed resource journal; inactive crash-prefix ownership is unproven"));
    }
    // The general crash-prefix reader accepts a trailer by type alone. Repair
    // needs its exact closure identity and count before treating it as authority.
    let journal_text = std::str::from_utf8(&journal_bytes).map_err(io::Error::other)?;
    let trailer: Value = serde_json::from_str(
        journal_text
            .lines()
            .last()
            .ok_or_else(|| invalid("resource journal trailer is absent"))?,
    )
    .map_err(io::Error::other)?;
    if !journal_bytes.ends_with(b"\n")
        || trailer["type"] != "resource-journal.trailer"
        || trailer["schema_version"] != 1
        || trailer["session_id"] != session_id
        || trailer["records"].as_u64() != Some(journal.transitions.len() as u64)
    {
        return Err(invalid(
            "resource journal terminal identity or record count is invalid",
        ));
    }
    let mut allowed = auxiliary_paths();
    for artifact in manifest.value()["artifacts"]
        .as_array()
        .ok_or_else(|| invalid("bundle manifest has no artifact population"))?
    {
        if let Some(path) = artifact["path"].as_str() {
            let path = super::validate_relative_path(path)?;
            validate_alias_free(&path)?;
            allowed.insert(path);
        }
    }
    let mut directories = BTreeSet::new();
    for path in &allowed {
        let mut parent = path.parent();
        while let Some(path) = parent.filter(|path| !path.as_os_str().is_empty()) {
            directories.insert(path.to_path_buf());
            parent = path.parent();
        }
    }
    for (object, _) in pinned.iter().skip(1) {
        let relative = object
            .path
            .strip_prefix(&root)
            .map_err(|_| invalid("bundle object escaped its root"))?;
        validate_alias_free(relative)?;
        let recognized = if object.identity.directory {
            directories.contains(relative)
        } else {
            allowed.contains(relative)
        };
        if !recognized {
            return Err(invalid(format!(
                "unrecognized bundle object: {}",
                relative.display()
            )));
        }
    }
    if let Some(container) = owned_sessions_container {
        add_owned_parents(&root, container, write_dacl, &mut pinned)?;
    }
    if pinned.len() > MAX_PATHS {
        return Err(invalid("bundle access population exceeds 256 objects"));
    }
    let paths = pinned
        .iter()
        .map(|(object, traversal_only)| BundleAccessInspectionPath {
            path: object.path.clone(),
            directory: object.identity.directory,
            traversal_only: *traversal_only,
            identity: format!("{:?}", object.identity),
            descriptor: object.descriptor.clone(),
        })
        .collect::<Vec<_>>();
    let ancestors = required_ancestors(&root);
    let ancestor_access = recipient.verify_paths(&root, &ancestors)?;
    let mut limitations = Vec::new();
    for outcome in &ancestor_access.paths {
        if outcome.state != "verified"
            && !paths
                .iter()
                .any(|path| path.traversal_only && path.path == outcome.path)
        {
            limitations.push(format!(
                "unowned parent access is unresolved at {}: {}",
                outcome.path.display(),
                outcome.reason
            ));
        }
    }
    let mut verification_paths = paths
        .iter()
        .map(|path| path.path.clone())
        .collect::<Vec<_>>();
    verification_paths.extend(ancestors);
    verification_paths.sort();
    verification_paths.dedup();
    let access = recipient
        .verify_paths(&root, &verification_paths)?
        .as_json();
    let snapshot = json!({
        "bundle": root, "recipient": recipient.sid(), "producer": recipient.producer_sid(),
        "session_id": session_id, "provenance": [blake3::hash(&manifest_bytes).to_hex().to_string(), blake3::hash(&journal_bytes).to_hex().to_string()],
        "paths": paths.iter().map(|path| json!({"path":path.path,"identity":path.identity,"descriptor":path.descriptor,"traversal_only":path.traversal_only})).collect::<Vec<_>>()
    });
    let id = blake3::hash(&serde_json::to_vec(&snapshot).map_err(io::Error::other)?)
        .to_hex()
        .to_string();
    Ok(Population {
        inspection: BundleAccessInspection {
            id,
            bundle: root,
            recipient_sid: recipient.sid().to_string(),
            session_id: session_id.to_string(),
            repairable: limitations.is_empty(),
            paths,
            access,
            limitations,
        },
        pinned,
    })
}

fn inventory_paths(
    root: &Path,
    directory: &Path,
    depth: usize,
    paths: &mut BTreeSet<PathBuf>,
) -> io::Result<()> {
    if depth > MAX_DEPTH {
        return Err(invalid("bundle access depth exceeds eight levels"));
    }
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        // Reject every reparse component before enumerating a child directory.
        let _object = pin_access_path(&path, false)?;
        let metadata = path.symlink_metadata()?;
        if metadata.file_type().is_symlink() || !(metadata.is_file() || metadata.is_dir()) {
            return Err(invalid(
                "bundle access refuses non-regular or reparse objects",
            ));
        }
        if !path.starts_with(root) {
            return Err(invalid("bundle object escaped its root"));
        }
        paths.insert(path.clone());
        if paths.len() >= MAX_PATHS {
            return Err(invalid("bundle access population exceeds 256 objects"));
        }
        if metadata.is_dir() {
            inventory_paths(root, &path, depth + 1, paths)?;
        }
    }
    Ok(())
}

fn add_owned_parents(
    root: &Path,
    container: &Path,
    write_dacl: bool,
    pinned: &mut Vec<(PinnedAccessPath, bool)>,
) -> io::Result<()> {
    let _container_object = pin_access_path(container, false)?;
    let container = container.canonicalize()?;
    let application = container
        .parent()
        .ok_or_else(|| invalid("owned container has no application parent"))?;
    if !container
        .file_name()
        .is_some_and(|name| name.as_encoded_bytes().eq_ignore_ascii_case(b"sessions"))
        || !application
            .file_name()
            .is_some_and(|name| name.as_encoded_bytes().eq_ignore_ascii_case(b"fragcap"))
    {
        return Err(invalid(
            "owned parent correction requires the exact fragcap/sessions container",
        ));
    }
    if !root.starts_with(&container) || root == container {
        return Err(invalid(
            "bundle is outside the proven owned sessions container",
        ));
    }
    let mut parent = root.parent();
    let mut count = 0;
    while let Some(path) = parent {
        if path != application && !path.starts_with(&container) {
            break;
        }
        count += 1;
        if count > MAX_DEPTH {
            return Err(invalid("owned parent correction exceeds eight objects"));
        }
        let object = pin_access_path(path, write_dacl)?;
        if !object.identity.directory {
            return Err(invalid("owned parent is not a directory"));
        }
        pinned.push((object, true));
        if path == application {
            break;
        }
        parent = path.parent();
    }
    Ok(())
}

fn required_ancestors(root: &Path) -> Vec<PathBuf> {
    // Explorer must enumerate the output container. Higher arbitrary ancestors
    // only require actual traversal, already demonstrated by opening the bundle;
    // requiring their enumeration would add unrelated usage restrictions.
    root.parent().map(Path::to_path_buf).into_iter().collect()
}

fn recheck_population(inspection: &BundleAccessInspection) -> io::Result<()> {
    let mut current = BTreeSet::new();
    inventory_paths(&inspection.bundle, &inspection.bundle, 0, &mut current)?;
    let expected = inspection
        .paths
        .iter()
        .filter(|path| !path.traversal_only && path.path != inspection.bundle)
        .map(|path| path.path.clone())
        .collect::<BTreeSet<_>>();
    if current != expected {
        return Err(invalid("recognized bundle inventory changed"));
    }
    Ok(())
}

fn auxiliary_paths() -> BTreeSet<PathBuf> {
    [
        "manifest.json",
        MANIFEST_PREFIX,
        RESOURCE_JOURNAL,
        ".sensitive-actions.jsonl",
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

fn validate_alias_free(path: &Path) -> io::Result<()> {
    for component in path.components() {
        let text = component
            .as_os_str()
            .to_str()
            .ok_or_else(|| invalid("bundle path is not valid Unicode"))?;
        if text.contains(':') || text.ends_with('.') || text.ends_with(' ') {
            return Err(invalid(
                "bundle access refuses alternate-stream or normalized path aliases",
            ));
        }
    }
    Ok(())
}

fn bounded_read(path: &Path) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(MAX_PROVENANCE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_PROVENANCE_BYTES {
        return Err(invalid("bundle access provenance exceeds four MiB"));
    }
    Ok(bytes)
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

#[cfg(all(test, windows))]
mod tests {
    use super::super::{prepare_bundle, ResourceJournal, MANIFEST_SCHEMA};
    use super::*;

    fn fixture(parent: &Path) -> PathBuf {
        let bundle = parent.join("bundle");
        prepare_bundle(&bundle).unwrap();
        let mut journal =
            ResourceJournal::create(&bundle, "repair-fixture", "fixture-plan").unwrap();
        journal.finish().unwrap();
        fs::write(bundle.join("capture.fcapng"), b"synthetic packet content").unwrap();
        fs::write(
            bundle.join("cleanup.jsonl"),
            b"synthetic independent sidecar\n",
        )
        .unwrap();
        fs::write(bundle.join("tls-keylog.log"), []).unwrap();
        fs::write(
            bundle.join("manifest.json"),
            serde_json::to_vec(&json!({
                "$schema":MANIFEST_SCHEMA,"manifest_version":2,
                "product":{"name":"fragcap","version":"fixture"},
                "session_id":"repair-fixture","state":"complete","artifacts":[],"omissions":[],
            }))
            .unwrap(),
        )
        .unwrap();
        bundle
    }

    #[test]
    fn exact_inspection_includes_protected_auxiliaries_and_repair_conserves_all_bytes() {
        let root = tempfile::tempdir().unwrap();
        let bundle = fixture(root.path());
        let recipient = OutputRecipient::current_account().unwrap();
        let before = fs::read_dir(&bundle)
            .unwrap()
            .map(|entry| {
                let path = entry.unwrap().path();
                (path.clone(), fs::read(path).unwrap())
            })
            .collect::<Vec<_>>();
        let inspection = inspect_bundle_access(&bundle, &recipient, None).unwrap();
        assert!(inspection.repairable);
        assert!(inspection
            .paths
            .iter()
            .any(|path| path.path.ends_with(".sensitive-actions.jsonl")));
        assert!(inspection
            .paths
            .iter()
            .any(|path| path.path.ends_with("cleanup.jsonl")));
        let report = repair_bundle_access(&bundle, &recipient, None, &inspection.id).unwrap();
        assert!(report.verified, "{:?}", report.limitations);
        for (path, bytes) in &before {
            assert_eq!(&fs::read(path).unwrap(), bytes);
        }
        assert_eq!(fs::read_dir(&bundle).unwrap().count(), before.len());
        let fresh = inspect_bundle_access(&bundle, &recipient, None).unwrap();
        let retry = repair_bundle_access(&bundle, &recipient, None, &fresh.id).unwrap();
        assert!(retry.verified);
        assert!(retry.paths.iter().all(|path| path.status == "unchanged"));
    }

    #[test]
    fn unknown_population_and_stale_authorization_change_nothing() {
        let root = tempfile::tempdir().unwrap();
        let bundle = fixture(root.path());
        let recipient = OutputRecipient::current_account().unwrap();
        let inspection = inspect_bundle_access(&bundle, &recipient, None).unwrap();
        assert!(repair_bundle_access(&bundle, &recipient, None, "incorrect-preview").is_err());
        fs::write(bundle.join("unrelated.txt"), b"unrelated").unwrap();
        assert!(inspect_bundle_access(&bundle, &recipient, None).is_err());
        assert!(repair_bundle_access(&bundle, &recipient, None, &inspection.id).is_err());
        assert_eq!(
            fs::read(bundle.join("unrelated.txt")).unwrap(),
            b"unrelated"
        );
    }

    #[test]
    fn provenance_mismatch_and_unowned_parent_refuse() {
        let root = tempfile::tempdir().unwrap();
        let bundle = fixture(root.path());
        let recipient = OutputRecipient::current_account().unwrap();
        assert!(inspect_bundle_access(&bundle, &recipient, Some(root.path())).is_err());
        let manifest_path = bundle.join("manifest.json");
        let mut manifest: Value =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest["session_id"] = json!("another-session");
        fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert!(inspect_bundle_access(&bundle, &recipient, None).is_err());
    }

    #[test]
    fn hard_link_alias_is_refused_before_any_permission_change() {
        let root = tempfile::tempdir().unwrap();
        let bundle = fixture(root.path());
        let recipient = OutputRecipient::current_account().unwrap();
        let outside = root.path().join("outside.fcapng");
        fs::hard_link(bundle.join("capture.fcapng"), &outside).unwrap();
        assert!(inspect_bundle_access(&bundle, &recipient, None).is_err());
        assert_eq!(fs::read(outside).unwrap(), b"synthetic packet content");
    }

    #[test]
    fn open_journal_cannot_supply_inactive_historical_ownership() {
        let root = tempfile::tempdir().unwrap();
        let bundle = fixture(root.path());
        let recipient = OutputRecipient::current_account().unwrap();
        let path = bundle.join(RESOURCE_JOURNAL);
        let text = fs::read_to_string(&path).unwrap();
        let header = text.lines().next().unwrap();
        fs::write(&path, format!("{header}\n")).unwrap();
        let before = fs::read(&path).unwrap();
        let error = inspect_bundle_access(&bundle, &recipient, None).unwrap_err();
        assert!(error.to_string().contains("closed resource journal"));
        assert_eq!(fs::read(path).unwrap(), before);
    }

    #[test]
    fn journal_terminal_identity_mismatch_is_refused() {
        let root = tempfile::tempdir().unwrap();
        let bundle = fixture(root.path());
        let recipient = OutputRecipient::current_account().unwrap();
        let path = bundle.join(RESOURCE_JOURNAL);
        let text = fs::read_to_string(&path).unwrap();
        let mut records = text
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        records.last_mut().unwrap()["session_id"] = json!("another-session");
        fs::write(
            &path,
            records
                .iter()
                .map(|value| format!("{value}\n"))
                .collect::<String>(),
        )
        .unwrap();
        assert!(inspect_bundle_access(&bundle, &recipient, None).is_err());
    }

    #[test]
    fn actual_partial_descriptor_failure_preserves_bytes_and_fresh_retry_verifies() {
        let root = tempfile::tempdir().unwrap();
        let bundle = fixture(root.path()).canonicalize().unwrap();
        let recipient = OutputRecipient::current_account().unwrap();
        let earlier = bundle.join(".sensitive-actions.jsonl");
        old_owner_relative_dacl(&earlier);
        let before = fs::read_dir(&bundle)
            .unwrap()
            .map(|entry| {
                let path = entry.unwrap().path();
                (path.clone(), fs::read(path).unwrap())
            })
            .collect::<Vec<_>>();
        let mut pending = population(&bundle, &recipient, None, true).unwrap();
        let failed = bundle.join("capture.fcapng");
        let replacement = pin_access_path(&failed, false).unwrap();
        let slot = pending
            .pinned
            .iter_mut()
            .find(|(object, _)| object.path == failed)
            .unwrap();
        // This is a real READ_CONTROL-only file handle. SetSecurityInfo must
        // fail with access denied after the earlier exact ACL change succeeds.
        slot.0 = replacement;
        let report = apply_population(pending, &recipient, &bundle).unwrap();
        assert!(!report.verified);
        assert!(report
            .paths
            .iter()
            .any(|path| path.path == earlier && path.status == "applied"));
        assert!(report
            .paths
            .iter()
            .any(|path| path.path == failed && path.status == "failed"));
        assert!(report
            .paths
            .iter()
            .find(|path| path.path == failed)
            .unwrap()
            .reason
            .contains("os error 5"));
        for (path, bytes) in &before {
            assert_eq!(&fs::read(path).unwrap(), bytes);
        }
        let fresh = inspect_bundle_access(&bundle, &recipient, None).unwrap();
        let retry = repair_bundle_access(&bundle, &recipient, None, &fresh.id).unwrap();
        assert!(retry.verified, "{:?}", retry.limitations);
        for (path, bytes) in &before {
            assert_eq!(&fs::read(path).unwrap(), bytes);
        }
    }

    fn old_owner_relative_dacl(path: &Path) {
        use std::os::windows::ffi::OsStrExt;
        use std::ptr;
        use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
        use windows_sys::Win32::Security::{
            SetFileSecurityW, DACL_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION,
        };
        use windows_sys::Win32::System::Memory::LocalFree;
        let sddl = "D:P(A;;FA;;;OW)(A;;FA;;;SY)"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let mut descriptor = ptr::null_mut();
        assert_ne!(
            unsafe {
                ConvertStringSecurityDescriptorToSecurityDescriptorW(
                    sddl.as_ptr(),
                    1,
                    &mut descriptor,
                    ptr::null_mut(),
                )
            },
            0
        );
        let path = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        assert_ne!(
            unsafe {
                SetFileSecurityW(
                    path.as_ptr(),
                    DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                    descriptor,
                )
            },
            0
        );
        unsafe { LocalFree(descriptor as isize) };
    }
}
