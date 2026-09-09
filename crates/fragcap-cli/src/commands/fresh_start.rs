// SPDX-License-Identifier: Apache-2.0

//! Exact, explicitly confirmed cleanup of canonical fragcap user data.

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use blake3::Hasher;
use serde_json::{json, Value};
use subtle::ConstantTimeEq;

use crate::cli::{FreshStartArgs, FreshStartScopeArg};
use crate::exit::{CliError, Exit};

const SCHEMA_VERSION: u64 = 1;
const MAX_PROFILES: usize = 128;
const MAX_ENTRIES: usize = 1_024;
const MAX_PATH_CHARS: usize = 512;
const MAX_DETAIL_CHARS: usize = 2_048;
const MAX_REPORT_BYTES: usize = 4 * 1024 * 1024;
const OWNED_CATEGORIES: [&str; 9] = [
    "catalog",
    "local-database",
    "profiles",
    "deep-capture-sessions",
    "captures",
    "settings",
    "cache",
    "logs",
    "other-canonical-state",
];

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProfileRoots {
    identity: String,
    roaming: PathBuf,
    local: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OwnedRoot {
    profile: String,
    kind: &'static str,
    path: PathBuf,
    present: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InventoryEntry {
    root: usize,
    relative: String,
    category: &'static str,
    directory: bool,
    redirected: bool,
    len: u64,
    modified_ns: u128,
}

#[derive(Clone, Debug)]
struct Inventory {
    scope: &'static str,
    roots: Vec<OwnedRoot>,
    entries: Vec<InventoryEntry>,
    identifier: String,
}

#[derive(Debug)]
struct ItemOutcome {
    path: PathBuf,
    operation: &'static str,
    status: &'static str,
    detail: String,
}

pub fn run(
    args: &FreshStartArgs,
    json_output: bool,
    out: &mut dyn Write,
) -> Result<Exit, CliError> {
    validate_authorization_shape(args)?;
    let profiles = resolve_profiles(args)?;
    let inventory = build_inventory(scope_name(args.scope), &profiles)
        .map_err(|error| CliError::failure(format!("fresh-start inventory failed: {error}")))?;

    if args.preview {
        emit_inventory(&inventory, json_output, out)?;
        return Ok(Exit::SUCCESS);
    }

    if !args.yes {
        return Err(CliError::usage(
            "fresh-start cleanup is irreversible; preview it, then pass --confirm <inventory-id> --yes",
        ));
    }
    let supplied = args.confirm.as_deref().ok_or_else(|| {
        CliError::usage("fresh-start execution requires --confirm <inventory-id>")
    })?;
    if !identifiers_equal(supplied, &inventory.identifier) {
        return Err(CliError::failure(format!(
            "fresh-start inventory changed; review a new preview (current {})",
            inventory.identifier
        )));
    }

    let report_target = args
        .report
        .as_deref()
        .map(|path| ReportTarget::prepare(path, &inventory.roots))
        .transpose()
        .map_err(|error| CliError::failure(format!("fresh-start report failed: {error}")))?;
    let mut recovery_output = Vec::new();
    let outcomes = execute_inventory(&inventory, &mut recovery_output);
    let complete = outcomes
        .iter()
        .all(|item| matches!(item.status, "removed" | "absent"));
    let report = report_value(&inventory, &outcomes, complete);
    if let Some(target) = report_target {
        target
            .publish(&report)
            .map_err(|error| CliError::failure(format!("fresh-start report failed: {error}")))?;
    }
    if !json_output {
        out.write_all(&recovery_output)
            .map_err(|error| CliError::failure(error.to_string()))?;
    }
    emit_result(&report, json_output, out)?;
    Ok(if complete {
        Exit::SUCCESS
    } else {
        Exit::FAILURE
    })
}

fn validate_authorization_shape(args: &FreshStartArgs) -> Result<(), CliError> {
    if (args.roaming_root.is_some() || args.local_root.is_some()) && !args.installer_adapter {
        return Err(CliError::usage(
            "explicit fresh-start roots are reserved for the installer adapter",
        ));
    }
    if args.installer_adapter
        && (args.scope != FreshStartScopeArg::CurrentUser
            || args.roaming_root.is_none()
            || args.local_root.is_none()
            || (!args.preview && !args.yes))
    {
        return Err(CliError::usage(
            "the installer adapter requires current-user scope, both exact roots, and preview or confirmed execution",
        ));
    }
    if args.preview {
        return Ok(());
    }
    if args.scope == FreshStartScopeArg::AllUsers && args.report.is_none() {
        return Err(CliError::usage(
            "all-users cleanup requires --report <path>; preview remains read-only",
        ));
    }
    Ok(())
}

fn scope_name(scope: FreshStartScopeArg) -> &'static str {
    match scope {
        FreshStartScopeArg::CurrentUser => "current-user",
        FreshStartScopeArg::AllUsers => "all-users",
    }
}

fn resolve_profiles(args: &FreshStartArgs) -> Result<Vec<ProfileRoots>, CliError> {
    match args.scope {
        FreshStartScopeArg::CurrentUser => {
            let default_roaming = crate::paths::default_roaming_data_root().ok_or_else(|| {
                CliError::failure("the current user's roaming data root is unavailable")
            })?;
            let default_local = crate::paths::default_local_data_root().ok_or_else(|| {
                CliError::failure("the current user's local data root is unavailable")
            })?;
            let (roaming, local) = match (&args.roaming_root, &args.local_root) {
                (Some(roaming), Some(local)) => {
                    if !paths_equal_for_platform(roaming, &default_roaming)
                        || !paths_equal_for_platform(local, &default_local)
                    {
                        return Err(CliError::failure(
                            "installer roots do not match the initiating user's canonical data roots",
                        ));
                    }
                    (roaming.clone(), local.clone())
                }
                (None, None) => (default_roaming, default_local),
                _ => unreachable!("clap requires the explicit roots together"),
            };
            Ok(vec![ProfileRoots {
                identity: "current-user".to_string(),
                roaming,
                local,
            }])
        }
        FreshStartScopeArg::AllUsers => {
            if args.roaming_root.is_some() || args.local_root.is_some() || args.installer_adapter {
                return Err(CliError::usage(
                    "all-users scope does not accept the installer current-user adapter",
                ));
            }
            if !platform_is_elevated() {
                return Err(CliError::failure(
                    "all-users fresh-start inventory requires an elevated Administrator session",
                ));
            }
            platform_profiles().map_err(|error| {
                CliError::failure(format!("Windows profile inventory failed: {error}"))
            })
        }
    }
}

#[cfg(windows)]
fn paths_equal_for_platform(left: &Path, right: &Path) -> bool {
    left.to_string_lossy()
        .eq_ignore_ascii_case(&right.to_string_lossy())
}

#[cfg(not(windows))]
fn paths_equal_for_platform(left: &Path, right: &Path) -> bool {
    left == right
}

#[cfg(windows)]
fn platform_is_elevated() -> bool {
    crate::doctor::probe::is_elevated()
}

#[cfg(not(windows))]
fn platform_is_elevated() -> bool {
    false
}

fn build_inventory(scope: &'static str, profiles: &[ProfileRoots]) -> io::Result<Inventory> {
    if profiles.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no eligible user profiles were found",
        ));
    }
    if profiles.len() > MAX_PROFILES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("fresh-start inventory exceeds {MAX_PROFILES} profiles"),
        ));
    }
    let mut roots = Vec::with_capacity(profiles.len() * 2);
    for profile in profiles {
        validate_profile_pair(profile)?;
        let roaming = validated_root(&profile.identity, "roaming", &profile.roaming)?;
        let local = validated_root(&profile.identity, "local", &profile.local)?;
        if paths_overlap(&roaming.path, &local.path) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "profile {} has overlapping data roots {} and {}",
                    profile.identity,
                    roaming.path.display(),
                    local.path.display()
                ),
            ));
        }
        roots.push(roaming);
        roots.push(local);
    }
    for left in 0..roots.len() {
        for right in left + 1..roots.len() {
            if paths_overlap(&roots[left].path, &roots[right].path) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "data roots alias or overlap: {} and {}",
                        roots[left].path.display(),
                        roots[right].path.display()
                    ),
                ));
            }
        }
    }

    roots.sort_by(|a, b| {
        a.profile
            .cmp(&b.profile)
            .then(a.kind.cmp(b.kind))
            .then(a.path.cmp(&b.path))
    });
    let mut entries = Vec::new();
    for (index, root) in roots.iter().enumerate() {
        if root.present {
            inventory_directory(index, &root.path, &root.path, &mut entries)?;
        }
    }
    entries.sort_by(|a, b| a.root.cmp(&b.root).then(a.relative.cmp(&b.relative)));
    let identifier = inventory_identifier(scope, &roots, &entries);
    Ok(Inventory {
        scope,
        roots,
        entries,
        identifier,
    })
}

fn validate_profile_pair(profile: &ProfileRoots) -> io::Result<()> {
    let roaming_parent = profile.roaming.parent().and_then(Path::parent);
    let local_parent = profile.local.parent().and_then(Path::parent);
    let roaming_kind = profile.roaming.parent().and_then(Path::file_name);
    let local_kind = profile.local.parent().and_then(Path::file_name);
    let roaming_appdata = roaming_parent.and_then(Path::file_name);
    let local_appdata = local_parent.and_then(Path::file_name);
    let roaming_profile = roaming_parent.and_then(Path::parent);
    let local_profile = local_parent.and_then(Path::parent);
    let exact_shape = roaming_kind.is_some_and(|name| name.eq_ignore_ascii_case("Roaming"))
        && local_kind.is_some_and(|name| name.eq_ignore_ascii_case("Local"))
        && roaming_appdata.is_some_and(|name| name.eq_ignore_ascii_case("AppData"))
        && local_appdata.is_some_and(|name| name.eq_ignore_ascii_case("AppData"))
        && roaming_profile
            .zip(local_profile)
            .is_some_and(|(left, right)| {
                left.to_string_lossy()
                    .eq_ignore_ascii_case(&right.to_string_lossy())
            });
    if exact_shape {
        ensure_ordinary_ancestors(&profile.roaming)?;
        ensure_ordinary_ancestors(&profile.local)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "profile {} does not supply one canonical AppData/Roaming and AppData/Local pair",
                profile.identity
            ),
        ))
    }
}

fn ensure_ordinary_ancestors(root: &Path) -> io::Result<()> {
    let profile = root
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("canonical shape supplies a profile root");
    for path in [
        profile.to_path_buf(),
        profile.join("AppData"),
        root.parent()
            .expect("canonical shape supplies a kind")
            .to_path_buf(),
    ] {
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "cannot classify cleanup ancestor {}: {error}",
                    path.display()
                ),
            )
        })?;
        if !metadata.is_dir() || is_redirected(&metadata) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "cleanup ancestor {} is redirected or not a directory",
                    path.display()
                ),
            ));
        }
    }
    Ok(())
}

fn validated_root(profile: &str, kind: &'static str, requested: &Path) -> io::Result<OwnedRoot> {
    if !requested.is_absolute()
        || requested.parent().is_none()
        || requested
            .file_name()
            .is_none_or(|name| !name.to_string_lossy().eq_ignore_ascii_case("fragcap"))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "{} is not an absolute canonical fragcap root",
                requested.display()
            ),
        ));
    }
    let metadata = match fs::symlink_metadata(requested) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    let present = metadata.is_some();
    let path = if let Some(metadata) = metadata {
        if !metadata.is_dir() || is_redirected(&metadata) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} is not an ordinary directory", requested.display()),
            ));
        }
        requested.canonicalize()?
    } else {
        let parent = requested.parent().expect("validated parent");
        let canonical_parent = parent.canonicalize()?;
        canonical_parent.join(requested.file_name().expect("validated name"))
    };
    if path.parent().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "a filesystem root can never be fresh-start data",
        ));
    }
    if path.as_os_str().to_string_lossy().chars().count() > MAX_PATH_CHARS {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("cleanup root exceeds {MAX_PATH_CHARS} characters"),
        ));
    }
    Ok(OwnedRoot {
        profile: profile.to_string(),
        kind,
        path,
        present,
    })
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    left == right || left.starts_with(right) || right.starts_with(left)
}

fn inventory_directory(
    root_index: usize,
    root: &Path,
    directory: &Path,
    entries: &mut Vec<InventoryEntry>,
) -> io::Result<()> {
    let mut children = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    children.sort_by_key(fs::DirEntry::file_name);
    for child in children {
        if entries.len() >= MAX_ENTRIES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("fresh-start inventory exceeds {MAX_ENTRIES} entries"),
            ));
        }
        let path = child.path();
        let metadata = fs::symlink_metadata(&path)?;
        let redirected = is_redirected(&metadata);
        let relative_path = path.strip_prefix(root).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "inventory path escaped its root",
            )
        })?;
        let relative = portable_relative(relative_path);
        if relative.chars().count() > MAX_PATH_CHARS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("fresh-start path exceeds {MAX_PATH_CHARS} characters"),
            ));
        }
        let directory_entry = metadata.is_dir();
        let modified_ns = modified_ns(&metadata);
        entries.push(InventoryEntry {
            root: root_index,
            category: category(relative_path),
            relative,
            directory: directory_entry,
            redirected,
            len: metadata.len(),
            modified_ns,
        });
        if directory_entry && !redirected {
            inventory_directory(root_index, root, &path, entries)?;
        }
    }
    Ok(())
}

fn portable_relative(path: &Path) -> String {
    path.components()
        .map(|part| part.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn category(path: &Path) -> &'static str {
    let first = path
        .components()
        .next()
        .map(|part| part.as_os_str().to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    match first.as_str() {
        "catalog.db" | "catalog.bootstrap.json" => "catalog",
        "local.db" => "local-database",
        "profiles" => "profiles",
        "sessions" => "deep-capture-sessions",
        "captures" => "captures",
        "settings.json" | "settings" => "settings",
        "cache" | "caches" => "cache",
        "log" | "logs" => "logs",
        _ => "other-canonical-state",
    }
}

#[cfg(windows)]
fn is_redirected(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_type().is_symlink()
        || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_redirected(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

fn inventory_identifier(scope: &str, roots: &[OwnedRoot], entries: &[InventoryEntry]) -> String {
    let mut hasher = Hasher::new();
    for value in ["fresh-start-v1", scope] {
        hasher.update(value.as_bytes());
        hasher.update(&[0]);
    }
    for root in roots {
        for value in [
            root.profile.as_str(),
            root.kind,
            root.path.to_string_lossy().as_ref(),
            if root.present { "present" } else { "absent" },
        ] {
            hasher.update(value.as_bytes());
            hasher.update(&[0]);
        }
    }
    for entry in entries {
        for value in [
            entry.root.to_string(),
            entry.relative.clone(),
            entry.category.to_string(),
            entry.directory.to_string(),
            entry.redirected.to_string(),
            entry.len.to_string(),
            entry.modified_ns.to_string(),
        ] {
            hasher.update(value.as_bytes());
            hasher.update(&[0]);
        }
    }
    format!("fresh-start-v1:{}", hasher.finalize().to_hex())
}

fn identifiers_equal(supplied: &str, expected: &str) -> bool {
    supplied.len() == expected.len()
        && supplied.as_bytes().ct_eq(expected.as_bytes()).unwrap_u8() == 1
}

fn execute_inventory(inventory: &Inventory, recovery_out: &mut dyn Write) -> Vec<ItemOutcome> {
    let mut outcomes = Vec::new();
    let mut recovery_blocked = BTreeSet::new();
    let mut recovery_succeeded = BTreeSet::new();
    let mut root_blocked = BTreeSet::new();
    for (index, root) in inventory.roots.iter().enumerate() {
        if !root.present {
            continue;
        }
        let valid = fs::symlink_metadata(&root.path)
            .ok()
            .filter(|metadata| metadata.is_dir() && !is_redirected(metadata))
            .and_then(|_| root.path.canonicalize().ok())
            .is_some_and(|current| current == root.path);
        if !valid {
            root_blocked.insert(index);
            outcomes.push(ItemOutcome {
                path: root.path.clone(),
                operation: "retain-root",
                status: "refused",
                detail: "canonical root changed or became redirected after inventory".to_string(),
            });
        }
    }
    for (index, root) in inventory.roots.iter().enumerate() {
        let sessions = root.path.join("sessions");
        let ordinary_sessions = fs::symlink_metadata(&sessions)
            .ok()
            .is_some_and(|metadata| metadata.is_dir() && !is_redirected(&metadata));
        if root.present && ordinary_sessions && !root_blocked.contains(&index) {
            if inventory.scope == "all-users" {
                recovery_blocked.insert(index);
                outcomes.push(ItemOutcome {
                    path: sessions,
                    operation: "deep-capture-recovery",
                    status: "retained",
                    detail: "cross-profile CurrentUser trust recovery is refused; run current-user fresh-start in that profile first".to_string(),
                });
                continue;
            }
            match crate::doctor::fix::recover_for_fresh_start(&sessions, recovery_out) {
                Ok(()) => {
                    recovery_succeeded.insert(index);
                    outcomes.push(ItemOutcome {
                        path: sessions,
                        operation: "deep-capture-recovery",
                        status: "removed",
                        detail: "exact recorded obligations reached safe terminal states"
                            .to_string(),
                    });
                }
                Err(errors) => {
                    recovery_blocked.insert(index);
                    outcomes.push(ItemOutcome {
                        path: sessions,
                        operation: "deep-capture-recovery",
                        status: "retained",
                        detail: format!(
                            "{}; run `fragcap doctor --fix` before retrying",
                            bounded_detail(&errors.join("; "))
                        ),
                    });
                }
            }
        }
    }

    let mut entries: Vec<_> = inventory.entries.iter().collect();
    entries.sort_by(|a, b| {
        b.relative
            .matches('/')
            .count()
            .cmp(&a.relative.matches('/').count())
            .then(b.relative.cmp(&a.relative))
    });
    for entry in entries {
        let root = &inventory.roots[entry.root];
        let path = root
            .path
            .join(entry.relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        let is_session = entry
            .relative
            .split('/')
            .next()
            .is_some_and(|part| part.eq_ignore_ascii_case("sessions"));
        if root_blocked.contains(&entry.root) {
            outcomes.push(ItemOutcome {
                path,
                operation: "retain",
                status: "retained",
                detail: "canonical root changed after inventory".to_string(),
            });
            continue;
        }
        if entry.redirected {
            outcomes.push(ItemOutcome {
                path,
                operation: "retain",
                status: "refused",
                detail: "link or reparse point is outside cleanup authority".to_string(),
            });
            continue;
        }
        if is_session && recovery_blocked.contains(&entry.root) {
            outcomes.push(ItemOutcome {
                path,
                operation: "retain",
                status: "retained",
                detail: "Deep Capture recovery evidence remains required".to_string(),
            });
            continue;
        }
        let recovery_removed_owner_record =
            if is_session && recovery_succeeded.contains(&entry.root) {
                let mut parts = entry.relative.split('/');
                parts
                    .next()
                    .is_some_and(|part| part.eq_ignore_ascii_case("sessions"))
                    && parts
                        .next()
                        .is_some_and(|part| part.eq_ignore_ascii_case("session-owners"))
                    && parts.next().is_some()
            } else {
                false
            };
        if recovery_removed_owner_record
            && fs::symlink_metadata(&path)
                .is_err_and(|error| error.kind() == io::ErrorKind::NotFound)
        {
            outcomes.push(ItemOutcome {
                path,
                operation: "remove",
                status: "absent",
                detail: "session-owner record was retired during exact recovery".to_string(),
            });
            continue;
        }
        if let Err(error) = validate_deletion_ancestors(&root.path, &path) {
            outcomes.push(ItemOutcome {
                path,
                operation: "retain",
                status: "refused",
                detail: bounded_detail(&format!(
                    "cleanup ancestor changed after inventory: {error}"
                )),
            });
            continue;
        }
        let current = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                outcomes.push(ItemOutcome {
                    path,
                    operation: "remove",
                    status: "absent",
                    detail: "item was already absent".to_string(),
                });
                continue;
            }
            Err(error) => {
                outcomes.push(ItemOutcome {
                    path,
                    operation: "retain",
                    status: "failed",
                    detail: bounded_detail(&error.to_string()),
                });
                continue;
            }
        };
        let changed = is_redirected(&current)
            || current.is_dir() != entry.directory
            || (!(entry.directory || is_session && recovery_succeeded.contains(&entry.root))
                && (current.len() != entry.len || modified_ns(&current) != entry.modified_ns));
        if changed {
            outcomes.push(ItemOutcome {
                path,
                operation: "retain",
                status: "refused",
                detail: "item changed or became redirected after inventory".to_string(),
            });
            continue;
        }
        if let Err(error) = validate_deletion_ancestors(&root.path, &path) {
            outcomes.push(ItemOutcome {
                path,
                operation: "retain",
                status: "refused",
                detail: bounded_detail(&format!(
                    "cleanup ancestor changed before deletion: {error}"
                )),
            });
            continue;
        }
        let removal = if entry.directory {
            fs::remove_dir(&path)
        } else {
            fs::remove_file(&path)
        };
        match removal {
            Ok(()) => outcomes.push(ItemOutcome {
                path,
                operation: if entry.directory {
                    "remove-directory"
                } else {
                    "remove-file"
                },
                status: "removed",
                detail: "removed exact inventoried item".to_string(),
            }),
            Err(error) if error.kind() == io::ErrorKind::NotFound => outcomes.push(ItemOutcome {
                path,
                operation: "remove",
                status: "absent",
                detail: "item was already absent".to_string(),
            }),
            Err(error) => outcomes.push(ItemOutcome {
                path,
                operation: "remove",
                status: "failed",
                detail: bounded_detail(&error.to_string()),
            }),
        }
    }
    for (index, root) in inventory.roots.iter().enumerate() {
        if !root.present {
            match fs::symlink_metadata(&root.path) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    outcomes.push(ItemOutcome {
                        path: root.path.clone(),
                        operation: "remove-root",
                        status: "absent",
                        detail: "canonical root did not exist".to_string(),
                    });
                }
                Ok(_) => outcomes.push(ItemOutcome {
                    path: root.path.clone(),
                    operation: "retain-root",
                    status: "refused",
                    detail: "canonical root appeared after inventory".to_string(),
                }),
                Err(error) => outcomes.push(ItemOutcome {
                    path: root.path.clone(),
                    operation: "retain-root",
                    status: "failed",
                    detail: bounded_detail(&error.to_string()),
                }),
            }
            continue;
        }
        if root_blocked.contains(&index) {
            continue;
        }
        let ordinary_root = fs::symlink_metadata(&root.path)
            .ok()
            .is_some_and(|metadata| metadata.is_dir() && !is_redirected(&metadata));
        if !ordinary_root {
            outcomes.push(ItemOutcome {
                path: root.path.clone(),
                operation: "retain-root",
                status: "refused",
                detail: "canonical root changed or became redirected during cleanup".to_string(),
            });
            continue;
        }
        match fs::remove_dir(&root.path) {
            Ok(()) => outcomes.push(ItemOutcome {
                path: root.path.clone(),
                operation: "remove-root",
                status: "removed",
                detail: "removed empty canonical root".to_string(),
            }),
            Err(error) if error.kind() == io::ErrorKind::NotFound => outcomes.push(ItemOutcome {
                path: root.path.clone(),
                operation: "remove-root",
                status: "absent",
                detail: "canonical root was already absent".to_string(),
            }),
            Err(error) => outcomes.push(ItemOutcome {
                path: root.path.clone(),
                operation: "remove-root",
                status: "failed",
                detail: bounded_detail(&error.to_string()),
            }),
        }
    }
    outcomes
}

fn validate_deletion_ancestors(root: &Path, target: &Path) -> io::Result<()> {
    let relative = target.strip_prefix(root).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "deletion target escaped its canonical root",
        )
    })?;
    let parent = relative.parent().unwrap_or_else(|| Path::new(""));
    let mut current = root.to_path_buf();
    for component in std::iter::once(None).chain(parent.components().map(Some)) {
        if let Some(component) = component {
            current.push(component.as_os_str());
        }
        let metadata = fs::symlink_metadata(&current)?;
        if !metadata.is_dir() || is_redirected(&metadata) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} is redirected or not a directory", current.display()),
            ));
        }
        if current.canonicalize()? != current {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} no longer resolves exactly", current.display()),
            ));
        }
    }
    Ok(())
}

fn modified_ns(metadata: &fs::Metadata) -> u128 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |duration| duration.as_nanos())
}

fn bounded_detail(detail: &str) -> String {
    let mut value = detail.chars().take(MAX_DETAIL_CHARS).collect::<String>();
    if detail.chars().count() > MAX_DETAIL_CHARS {
        value.push_str(" [truncated]");
    }
    value
}

fn inventory_value(inventory: &Inventory) -> Value {
    json!({
        "schema_version": SCHEMA_VERSION,
        "type": "fresh-start.inventory",
        "scope": inventory.scope,
        "inventory_id": inventory.identifier,
        "roots": inventory.roots.iter().map(|root| json!({
            "profile": root.profile,
            "kind": root.kind,
            "path": root.path,
            "status": if root.present { "eligible" } else { "absent" },
        })).collect::<Vec<_>>(),
        "categories": OWNED_CATEGORIES,
        "entries": inventory.entries.iter().map(|entry| json!({
            "root": entry.root,
            "relative": entry.relative,
            "category": entry.category,
            "kind": if entry.directory { "directory" } else { "file" },
            "status": if entry.redirected { "refused-redirection" } else { "eligible" },
        })).collect::<Vec<_>>(),
    })
}

fn report_value(inventory: &Inventory, outcomes: &[ItemOutcome], complete: bool) -> Value {
    json!({
        "schema_version": SCHEMA_VERSION,
        "type": "fresh-start.cleanup",
        "scope": inventory.scope,
        "inventory_id": inventory.identifier,
        "status": if complete { "complete" } else { "partial" },
        "roots": inventory.roots.iter().map(|root| json!({
            "profile": root.profile,
            "kind": root.kind,
            "path": root.path,
        })).collect::<Vec<_>>(),
        "items": outcomes.iter().map(|item| json!({
            "path": item.path,
            "operation": item.operation,
            "status": item.status,
            "detail": item.detail,
        })).collect::<Vec<_>>(),
        "guidance": if complete {
            Value::Null
        } else {
            json!("Review retained paths and run `fragcap doctor --fix` for Deep Capture recovery before retrying.")
        },
    })
}

fn emit_inventory(
    inventory: &Inventory,
    json_output: bool,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    if json_output {
        writeln!(out, "{}", inventory_value(inventory))
            .map_err(|error| CliError::failure(error.to_string()))?;
        return Ok(());
    }
    writeln!(out, "Fresh-start preview ({})", inventory.scope)
        .map_err(|error| CliError::failure(error.to_string()))?;
    writeln!(out, "Irreversible after confirmation. Exact roots:")
        .map_err(|error| CliError::failure(error.to_string()))?;
    for root in &inventory.roots {
        writeln!(
            out,
            "  {} {}: {} ({})",
            root.profile,
            root.kind,
            root.path.display(),
            if root.present { "eligible" } else { "absent" }
        )
        .map_err(|error| CliError::failure(error.to_string()))?;
    }
    writeln!(out, "Categories: {}", OWNED_CATEGORIES.join(", "))
        .map_err(|error| CliError::failure(error.to_string()))?;
    writeln!(out, "Inventory: {}", inventory.identifier)
        .map_err(|error| CliError::failure(error.to_string()))?;
    Ok(())
}

fn emit_result(report: &Value, json_output: bool, out: &mut dyn Write) -> Result<(), CliError> {
    if json_output {
        writeln!(out, "{report}").map_err(|error| CliError::failure(error.to_string()))?;
    } else {
        writeln!(
            out,
            "Fresh-start cleanup: {}",
            report["status"].as_str().unwrap_or("partial")
        )
        .map_err(|error| CliError::failure(error.to_string()))?;
        if let Some(guidance) = report["guidance"].as_str() {
            writeln!(out, "{guidance}").map_err(|error| CliError::failure(error.to_string()))?;
        }
    }
    Ok(())
}

#[derive(Debug)]
struct ReportTarget {
    destination: PathBuf,
    temporary: PathBuf,
    file: Option<fs::File>,
}

impl ReportTarget {
    fn prepare(path: &Path, roots: &[OwnedRoot]) -> io::Result<Self> {
        let parent = path
            .parent()
            .filter(|value| !value.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let parent = parent.canonicalize()?;
        let file_name = path.file_name().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "report path has no file name")
        })?;
        let destination = parent.join(file_name);
        if roots
            .iter()
            .any(|root| destination == root.path || destination.starts_with(&root.path))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "report path must be outside every fresh-start cleanup root",
            ));
        }
        match fs::symlink_metadata(&destination) {
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "fresh-start report destination already exists",
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        let temporary = destination.with_extension(format!(
            "tmp-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        Ok(Self {
            destination,
            temporary,
            file: Some(file),
        })
    }

    fn publish(mut self, report: &Value) -> io::Result<()> {
        let mut bytes = serde_json::to_vec_pretty(report)?;
        bytes.push(b'\n');
        if bytes.len() > MAX_REPORT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("fresh-start report exceeds {MAX_REPORT_BYTES} bytes"),
            ));
        }
        let mut file = self.file.take().expect("prepared report file");
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&self.temporary, &self.destination)
    }
}

impl Drop for ReportTarget {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.temporary);
    }
}

#[cfg(windows)]
fn platform_profiles() -> io::Result<Vec<ProfileRoots>> {
    use windows_sys::Win32::Foundation::ERROR_NO_MORE_ITEMS;
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegEnumKeyExW, RegGetValueW, RegOpenKeyExW, HKEY, HKEY_LOCAL_MACHINE,
        KEY_READ, RRF_RT_REG_EXPAND_SZ, RRF_RT_REG_SZ,
    };

    const PROFILE_LIST: &str = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList";
    let wide = |value: &str| {
        value
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>()
    };
    let mut key: HKEY = 0;
    let profile_list = wide(PROFILE_LIST);
    let opened = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            profile_list.as_ptr(),
            0,
            KEY_READ,
            &mut key,
        )
    };
    if opened != 0 {
        return Err(io::Error::from_raw_os_error(opened as i32));
    }
    struct Key(HKEY);
    impl Drop for Key {
        fn drop(&mut self) {
            unsafe { RegCloseKey(self.0) };
        }
    }
    let key = Key(key);
    let mut profiles = Vec::new();
    for index in 0..4096u32 {
        let mut name = vec![0u16; 256];
        let mut length = name.len() as u32;
        let result = unsafe {
            RegEnumKeyExW(
                key.0,
                index,
                name.as_mut_ptr(),
                &mut length,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if result == ERROR_NO_MORE_ITEMS {
            break;
        }
        if result != 0 {
            return Err(io::Error::from_raw_os_error(result as i32));
        }
        let sid = String::from_utf16_lossy(&name[..length as usize]);
        if !sid.starts_with("S-1-5-21-") || sid.ends_with(".bak") {
            continue;
        }
        let subkey = wide(&format!("{PROFILE_LIST}\\{sid}"));
        let value = wide("ProfileImagePath");
        let mut bytes = 0u32;
        let sized = unsafe {
            RegGetValueW(
                HKEY_LOCAL_MACHINE,
                subkey.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut bytes,
            )
        };
        if sized != 0 || bytes == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("profile {sid} has no readable ProfileImagePath"),
            ));
        }
        let mut buffer = vec![0u16; (bytes as usize).div_ceil(2)];
        let mut capacity = bytes;
        let read = unsafe {
            RegGetValueW(
                HKEY_LOCAL_MACHINE,
                subkey.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ,
                std::ptr::null_mut(),
                buffer.as_mut_ptr().cast(),
                &mut capacity,
            )
        };
        if read != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("profile {sid} ProfileImagePath could not be read"),
            ));
        }
        let end = buffer
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(buffer.len());
        let profile = expand_registry_environment(&String::from_utf16_lossy(&buffer[..end]));
        let base = PathBuf::from(profile);
        if !base.is_absolute() || !base.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("profile {sid} has no safely classifiable profile directory"),
            ));
        }
        profiles.push(ProfileRoots {
            identity: sid,
            roaming: base.join("AppData").join("Roaming").join("fragcap"),
            local: base.join("AppData").join("Local").join("fragcap"),
        });
    }
    profiles.sort_by(|a, b| a.identity.cmp(&b.identity));
    Ok(profiles)
}

#[cfg(windows)]
fn expand_registry_environment(value: &str) -> String {
    let mut expanded = value.to_string();
    for (name, replacement) in std::env::vars() {
        expanded = expanded.replace(&format!("%{name}%"), &replacement);
        expanded = expanded.replace(&format!("%{}%", name.to_ascii_lowercase()), &replacement);
        expanded = expanded.replace(&format!("%{}%", name.to_ascii_uppercase()), &replacement);
    }
    expanded
}

#[cfg(not(windows))]
fn platform_profiles() -> io::Result<Vec<ProfileRoots>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "all-users fresh start is Windows-only",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roots(base: &Path) -> ProfileRoots {
        let appdata = base.join("User").join("AppData");
        ProfileRoots {
            identity: "test-user".to_string(),
            roaming: appdata.join("Roaming").join("fragcap"),
            local: appdata.join("Local").join("fragcap"),
        }
    }

    #[test]
    fn inventory_is_stable_and_classifies_owned_categories() {
        let temp = tempfile::tempdir().unwrap();
        let profile = roots(temp.path());
        fs::create_dir_all(profile.roaming.join("profiles")).unwrap();
        fs::create_dir_all(profile.local.join("logs")).unwrap();
        fs::write(profile.roaming.join("local.db"), b"target state").unwrap();
        fs::write(
            profile.roaming.join("profiles").join("game.toml"),
            b"profile",
        )
        .unwrap();
        fs::write(profile.local.join("logs").join("fragcap.log"), b"log").unwrap();

        let first = build_inventory("current-user", std::slice::from_ref(&profile)).unwrap();
        let second = build_inventory("current-user", &[profile]).unwrap();
        assert_eq!(first.identifier, second.identifier);
        let categories: BTreeSet<_> = first.entries.iter().map(|entry| entry.category).collect();
        assert!(categories.contains("local-database"));
        assert!(categories.contains("profiles"));
        assert!(categories.contains("logs"));
    }

    #[test]
    fn changed_inventory_changes_confirmation_identifier() {
        let temp = tempfile::tempdir().unwrap();
        let profile = roots(temp.path());
        fs::create_dir_all(&profile.roaming).unwrap();
        fs::create_dir_all(&profile.local).unwrap();
        let first = build_inventory("current-user", std::slice::from_ref(&profile)).unwrap();
        fs::write(profile.roaming.join("local.db"), b"changed").unwrap();
        let second = build_inventory("current-user", &[profile]).unwrap();
        assert_ne!(first.identifier, second.identifier);
    }

    #[test]
    fn exact_inventory_deletes_only_the_two_canonical_roots() {
        let temp = tempfile::tempdir().unwrap();
        let profile = roots(temp.path());
        fs::create_dir_all(&profile.roaming).unwrap();
        fs::create_dir_all(&profile.local).unwrap();
        fs::write(profile.roaming.join("local.db"), b"state").unwrap();
        let excluded = temp.path().join("exported.fcapng");
        fs::write(&excluded, b"capture").unwrap();
        let inventory = build_inventory("current-user", std::slice::from_ref(&profile)).unwrap();

        let outcomes = execute_inventory(&inventory, &mut io::sink());

        assert!(outcomes
            .iter()
            .all(|item| matches!(item.status, "removed" | "absent")));
        assert!(!profile.roaming.exists());
        assert!(!profile.local.exists());
        assert_eq!(fs::read(excluded).unwrap(), b"capture");
    }

    #[test]
    fn custom_shaped_pair_is_refused_before_inventory() {
        let temp = tempfile::tempdir().unwrap();
        let profile = ProfileRoots {
            identity: "test-user".to_string(),
            roaming: temp.path().join("custom").join("fragcap"),
            local: temp.path().join("other").join("fragcap"),
        };
        let error = build_inventory("current-user", &[profile]).unwrap_err();
        assert!(error.to_string().contains("canonical AppData/Roaming"));
    }

    #[test]
    fn multiple_profiles_are_sorted_and_remain_distinct() {
        let temp = tempfile::tempdir().unwrap();
        let first = roots(&temp.path().join("one"));
        let mut second = roots(&temp.path().join("two"));
        second.identity = "another-user".to_string();
        for profile in [&first, &second] {
            fs::create_dir_all(&profile.roaming).unwrap();
            fs::create_dir_all(&profile.local).unwrap();
        }
        let inventory = build_inventory("all-users", &[first, second]).unwrap();
        assert_eq!(inventory.roots.len(), 4);
        assert_eq!(inventory.roots[0].profile, "another-user");
        assert_eq!(inventory.roots[2].profile, "test-user");
    }

    #[test]
    fn failed_deep_capture_recovery_retains_sessions_but_removes_independent_state() {
        let temp = tempfile::tempdir().unwrap();
        let profile = roots(temp.path());
        let broken = profile.roaming.join("sessions").join("broken");
        fs::create_dir_all(&broken).unwrap();
        fs::create_dir_all(&profile.local).unwrap();
        fs::write(
            broken.join(fragcap::deep_capture::RESOURCE_JOURNAL),
            b"not-json\n",
        )
        .unwrap();
        fs::write(profile.local.join("settings.json"), b"settings").unwrap();
        let inventory = build_inventory("current-user", std::slice::from_ref(&profile)).unwrap();

        let outcomes = execute_inventory(&inventory, &mut io::sink());

        assert!(outcomes.iter().any(|item| {
            item.operation == "deep-capture-recovery" && item.status == "retained"
        }));
        assert!(profile.roaming.join("sessions").exists());
        assert!(!profile.local.exists());
    }

    #[test]
    fn successful_recovery_accepts_a_retired_session_owner_record() {
        let temp = tempfile::tempdir().unwrap();
        let profile = roots(temp.path());
        let sessions = profile.roaming.join("sessions");
        let bundle = temp.path().join("completed-bundle");
        fs::create_dir_all(&bundle).unwrap();
        fs::create_dir_all(&profile.local).unwrap();
        let lease = crate::doctor::fix::register_session_owner(&sessions, &bundle).unwrap();
        drop(lease);
        let inventory = build_inventory("current-user", std::slice::from_ref(&profile)).unwrap();

        let outcomes = execute_inventory(&inventory, &mut io::sink());

        assert!(outcomes
            .iter()
            .all(|item| matches!(item.status, "removed" | "absent")));
        assert!(outcomes.iter().any(|item| {
            item.path
                .parent()
                .is_some_and(|parent| parent.ends_with("session-owners"))
                && item.status == "absent"
                && item.detail.contains("retired during exact recovery")
        }));
        assert!(!profile.roaming.exists());
        assert!(!profile.local.exists());
        assert!(bundle.exists());
    }

    #[test]
    fn all_users_retains_session_evidence_for_profile_local_recovery() {
        let temp = tempfile::tempdir().unwrap();
        let profile = roots(temp.path());
        fs::create_dir_all(profile.roaming.join("sessions").join("one")).unwrap();
        fs::create_dir_all(&profile.local).unwrap();
        fs::write(profile.local.join("settings.json"), b"settings").unwrap();
        let inventory = build_inventory("all-users", std::slice::from_ref(&profile)).unwrap();

        let outcomes = execute_inventory(&inventory, &mut io::sink());

        assert!(outcomes.iter().any(|item| {
            item.operation == "deep-capture-recovery"
                && item.status == "retained"
                && item.detail.contains("CurrentUser trust recovery")
        }));
        assert!(profile.roaming.join("sessions").exists());
        assert!(!profile.local.exists());
    }

    #[test]
    fn report_inside_cleanup_root_is_refused_before_execution() {
        let temp = tempfile::tempdir().unwrap();
        let profile = roots(temp.path());
        fs::create_dir_all(&profile.roaming).unwrap();
        fs::create_dir_all(&profile.local).unwrap();
        let inventory = build_inventory("current-user", &[profile]).unwrap();

        let error = ReportTarget::prepare(
            &inventory.roots[0].path.join("cleanup-report.json"),
            &inventory.roots,
        )
        .unwrap_err();

        assert!(error.to_string().contains("outside every fresh-start"));
    }

    #[test]
    fn report_preserves_an_existing_destination() {
        let temp = tempfile::tempdir().unwrap();
        let destination = temp.path().join("fresh-start.json");
        fs::write(&destination, b"prior report").unwrap();

        let error = ReportTarget::prepare(&destination, &[]).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(&destination).unwrap(), b"prior report");
    }

    #[test]
    fn root_created_after_inventory_prevents_complete_result() {
        let temp = tempfile::tempdir().unwrap();
        let profile = roots(temp.path());
        fs::create_dir_all(profile.roaming.parent().unwrap()).unwrap();
        fs::create_dir_all(profile.local.parent().unwrap()).unwrap();
        let inventory = build_inventory("current-user", std::slice::from_ref(&profile)).unwrap();
        fs::create_dir(&profile.roaming).unwrap();

        let outcomes = execute_inventory(&inventory, &mut io::sink());

        assert!(outcomes.iter().any(|item| {
            item.operation == "retain-root"
                && item.status == "refused"
                && item.detail.contains("appeared after inventory")
        }));
        assert!(profile.roaming.exists());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_is_retained_without_following_its_target() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let profile = roots(temp.path());
        fs::create_dir_all(&profile.roaming).unwrap();
        fs::create_dir_all(&profile.local).unwrap();
        let outside = temp.path().join("outside.txt");
        fs::write(&outside, b"outside").unwrap();
        symlink(&outside, profile.roaming.join("redirect")).unwrap();
        let inventory = build_inventory("current-user", &[profile]).unwrap();

        let outcomes = execute_inventory(&inventory, &mut io::sink());

        assert!(outcomes.iter().any(|item| item.status == "refused"));
        assert_eq!(fs::read(outside).unwrap(), b"outside");
    }

    #[cfg(unix)]
    #[test]
    fn redirected_sessions_are_never_passed_to_recovery() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let profile = roots(temp.path());
        fs::create_dir_all(&profile.roaming).unwrap();
        fs::create_dir_all(&profile.local).unwrap();
        let outside = temp.path().join("outside-sessions");
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("evidence.jsonl"), b"evidence").unwrap();
        symlink(&outside, profile.roaming.join("sessions")).unwrap();
        let inventory = build_inventory("current-user", &[profile]).unwrap();

        let outcomes = execute_inventory(&inventory, &mut io::sink());

        assert!(outcomes.iter().any(|item| {
            item.path.ends_with("sessions")
                && item.operation == "retain"
                && item.status == "refused"
        }));
        assert_eq!(
            fs::read(outside.join("evidence.jsonl")).unwrap(),
            b"evidence"
        );
    }

    #[cfg(unix)]
    #[test]
    fn ancestor_redirected_after_inventory_is_refused_before_child_deletion() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let profile = roots(temp.path());
        let owned = profile.roaming.join("cache");
        fs::create_dir_all(&owned).unwrap();
        fs::create_dir_all(&profile.local).unwrap();
        fs::write(owned.join("entry.bin"), b"owned").unwrap();
        let inventory = build_inventory("current-user", std::slice::from_ref(&profile)).unwrap();
        fs::rename(&owned, profile.roaming.join("cache-original")).unwrap();
        let outside = temp.path().join("outside-cache");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("entry.bin"), b"outside").unwrap();
        symlink(&outside, &owned).unwrap();

        let outcomes = execute_inventory(&inventory, &mut io::sink());

        assert!(outcomes.iter().any(|item| {
            item.path.ends_with("cache/entry.bin")
                && item.status == "refused"
                && item.detail.contains("cleanup ancestor changed")
        }));
        assert_eq!(fs::read(outside.join("entry.bin")).unwrap(), b"outside");
    }

    #[cfg(unix)]
    #[test]
    fn redirected_appdata_ancestor_is_refused() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let user = temp.path().join("User");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&user).unwrap();
        fs::create_dir_all(outside.join("Roaming")).unwrap();
        fs::create_dir_all(outside.join("Local")).unwrap();
        symlink(&outside, user.join("AppData")).unwrap();
        let profile = roots(temp.path());

        let error = build_inventory("current-user", &[profile]).unwrap_err();

        assert!(error.to_string().contains("cleanup ancestor"));
    }

    #[cfg(unix)]
    #[test]
    fn dangling_canonical_root_is_refused_instead_of_reported_absent() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let profile = roots(temp.path());
        fs::create_dir_all(profile.roaming.parent().unwrap()).unwrap();
        fs::create_dir_all(profile.local.parent().unwrap()).unwrap();
        symlink(temp.path().join("missing"), &profile.roaming).unwrap();

        let error = build_inventory("current-user", &[profile]).unwrap_err();

        assert!(error.to_string().contains("not an ordinary directory"));
    }
}
