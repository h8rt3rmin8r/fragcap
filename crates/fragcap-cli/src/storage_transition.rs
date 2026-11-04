// SPDX-License-Identifier: Apache-2.0

//! Non-destructive transition from the pre-S192 AppData stores.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use rusqlite::backup::{Backup, StepResult};
use rusqlite::{Connection, OpenFlags};
use serde_json::{json, Value};

const RECORD_PREFIX: &str = "storage-transition-v1-";
const MAX_PACKAGES: usize = 4096;
const MAX_RECORD_BYTES: u64 = 64 * 1024;

/// Exact pre-S192 product roots visible to the current Windows account.
pub(crate) fn legacy_roots() -> io::Result<Vec<PathBuf>> {
    #[cfg(windows)]
    {
        let roaming = crate::paths::legacy_roaming_data_root()?;
        let local = crate::paths::legacy_local_data_root()?;
        let packages = local
            .parent()
            .ok_or_else(|| io::Error::other("legacy local data parent is unavailable"))?
            .join("Packages");
        let profile = crate::paths::current_profile_path()?;
        let ordinary_roaming = profile.join("AppData").join("Roaming").join("fragcap");
        let ordinary_local = profile.join("AppData").join("Local").join("fragcap");
        let ordinary_packages = profile.join("AppData").join("Local").join("Packages");
        let mut roots = legacy_roots_from(&roaming, &local, &packages)?;
        roots.extend(legacy_roots_from(
            &ordinary_roaming,
            &ordinary_local,
            &ordinary_packages,
        )?);
        roots.sort();
        roots.dedup();
        Ok(roots)
    }
    #[cfg(not(windows))]
    {
        Ok(Vec::new())
    }
}

/// Historical session roots are retained at their original paths because
/// manifests and recovery journals may contain absolute path authority.
pub(crate) fn legacy_session_roots() -> io::Result<Vec<PathBuf>> {
    if std::env::var_os(crate::paths::STORAGE_ROOT_ENV).is_some()
        || std::env::var_os(crate::paths::SESSION_DIR_ENV).is_some()
    {
        return Ok(Vec::new());
    }
    let mut sessions = Vec::new();
    for root in legacy_roots()? {
        let path = root.join("sessions");
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_dir() => {
                #[cfg(windows)]
                if is_redirected(&metadata) {
                    return Err(io::Error::other(format!(
                        "legacy sessions root is redirected: {}",
                        path.display()
                    )));
                }
                sessions.push(path);
            }
            Ok(_) => {
                return Err(io::Error::other(format!(
                    "legacy sessions root is not a directory: {}",
                    path.display()
                )))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    sessions.sort();
    sessions.dedup();
    Ok(sessions)
}

/// Write an append-only marker for each retained root. A marker is never used
/// as deletion authority; each later scan revalidates the exact legacy path.
pub(crate) fn index_legacy_sessions() -> Result<(), String> {
    let roots = legacy_session_roots().map_err(|error| error.to_string())?;
    if roots.is_empty() {
        return Ok(());
    }
    let current = crate::paths::default_roaming_data_root()
        .ok_or_else(|| "current-user storage root is unavailable".to_string())?;
    let index = current.join("legacy-session-roots-v1");
    fs::create_dir_all(&index)
        .map_err(|error| format!("cannot create legacy session index: {error}"))?;
    for root in roots {
        let key = blake3::hash(root.to_string_lossy().as_bytes())
            .to_hex()
            .to_string();
        let path = index.join(format!("{key}.json"));
        if path.exists() {
            continue;
        }
        let stage = index.join(format!(".{key}-{}.tmp", std::process::id()));
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&stage)
            .map_err(|error| format!("cannot stage legacy session index: {error}"))?;
        serde_json::to_writer(
            &mut file,
            &json!({"version":1,"root":root,"disposition":"retained-in-place"}),
        )
        .map_err(|error| error.to_string())?;
        file.write_all(b"\n").map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        drop(file);
        match fs::hard_link(&stage, &path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(format!("cannot index legacy session root: {error}")),
        }
        fs::remove_file(&stage).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(windows)]
fn legacy_roots_from(roaming: &Path, local: &Path, packages: &Path) -> io::Result<Vec<PathBuf>> {
    let mut roots = vec![roaming.to_path_buf(), local.to_path_buf()];
    let entries = match fs::read_dir(packages) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(roots),
        Err(error) => return Err(error),
    };
    for (index, entry) in entries.enumerate() {
        if index >= MAX_PACKAGES {
            return Err(io::Error::other(
                "package-root inventory exceeds 4096 entries",
            ));
        }
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if !metadata.is_dir() || is_redirected(&metadata) {
            continue;
        }
        for suffix in [
            ["LocalCache", "Roaming", "fragcap"],
            ["LocalCache", "Local", "fragcap"],
        ] {
            let candidate = suffix
                .iter()
                .fold(entry.path(), |path, part| path.join(part));
            match fs::symlink_metadata(&candidate) {
                Ok(_) => {
                    ensure_plain_ancestors(&candidate, packages)?;
                    roots.push(candidate);
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
    }
    roots.sort();
    roots.dedup();
    Ok(roots)
}

#[cfg(windows)]
fn ensure_plain_ancestors(path: &Path, stop: &Path) -> io::Result<()> {
    let mut current = path;
    while current != stop {
        let metadata = fs::symlink_metadata(current)?;
        if !metadata.is_dir() || is_redirected(&metadata) {
            return Err(io::Error::other(format!(
                "legacy path is redirected or not a directory: {}",
                current.display()
            )));
        }
        current = current
            .parent()
            .ok_or_else(|| io::Error::other("legacy path escaped"))?;
    }
    Ok(())
}

#[cfg(windows)]
fn is_redirected(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}

/// Transfer one default database, or refuse when the legacy history is ambiguous.
/// The caller invokes this only for an unoverridden default data operation.
pub(crate) fn prepare_database(destination: &Path) -> Result<(), String> {
    if std::env::var_os(crate::paths::STORAGE_ROOT_ENV).is_some() {
        // An explicitly selected whole-store location owns its own history.
        return Ok(());
    }
    let roots =
        legacy_roots().map_err(|error| format!("legacy store inventory failed: {error}"))?;
    prepare_database_from(destination, &roots)?;
    index_legacy_sessions()
}

/// Preserve an authored profile directory before the first default profile read.
pub(crate) fn prepare_profiles() -> Result<(), String> {
    if std::env::var_os(crate::paths::STORAGE_ROOT_ENV).is_some()
        || std::env::var_os(crate::paths::PROFILE_DIR_ENV).is_some()
    {
        return Ok(());
    }
    let roots = legacy_roots().map_err(|error| error.to_string())?;
    let destination = crate::paths::default_roaming_data_root()
        .ok_or_else(|| "current-user profile storage is unavailable".to_string())?
        .join("profiles");
    prepare_profiles_from(&destination, &roots)
}

fn prepare_profiles_from(destination: &Path, roots: &[PathBuf]) -> Result<(), String> {
    let sources = roots
        .iter()
        .map(|root| root.join("profiles"))
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    if sources.len() > 1 {
        return Err(format!(
            "multiple legacy profile directories require an explicit choice: {}",
            sources
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let Some(source) = sources.first() else {
        return Ok(());
    };
    let before = profile_fingerprint(source)?;
    let record = destination
        .parent()
        .expect("profile directory parent")
        .join("storage-transition-v1-profiles.json");
    if destination.exists() {
        let file = File::open(&record).map_err(|error| format!(
            "profile destination {} exists with legacy source {} but no readable transition record: {error}",
            destination.display(), source.display()))?;
        if file.metadata().map_err(|e| e.to_string())?.len() > MAX_RECORD_BYTES {
            return Err("profile transition record is oversized".to_string());
        }
        let prior: Value = serde_json::from_reader(file).map_err(|error| error.to_string())?;
        if prior["version"] == 1
            && prior["source"] == source.to_string_lossy().as_ref()
            && prior["fingerprint"] == before
        {
            return Ok(());
        }
        return Err(format!(
            "legacy profiles changed after transition or conflict with {}",
            destination.display()
        ));
    }
    let parent = destination.parent().expect("profile directory parent");
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let stage = parent.join(format!(".profiles.s192-{}.tmp", std::process::id()));
    fs::create_dir(&stage)
        .map_err(|error| format!("cannot stage profiles at {}: {error}", stage.display()))?;
    let mut count = 0usize;
    copy_profile_tree(source, &stage, 0, &mut count)?;
    if before != profile_fingerprint(source)? {
        return Err(format!(
            "legacy profiles changed during transition: {}",
            source.display()
        ));
    }
    fs::rename(&stage, destination)
        .map_err(|error| format!("cannot publish profiles without replacement: {error}"))?;
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&record)
        .map_err(|error| format!("cannot record profile transition: {error}"))?;
    serde_json::to_writer(&mut file, &json!({"version":1,"source":source,"fingerprint":before,"disposition":"copied-source-retained"}))
        .map_err(|error| error.to_string())?;
    file.write_all(b"\n").map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    Ok(())
}

fn copy_profile_tree(
    source: &Path,
    destination: &Path,
    depth: usize,
    count: &mut usize,
) -> Result<(), String> {
    if depth > 8 {
        return Err(format!(
            "legacy profiles exceed depth limit: {}",
            source.display()
        ));
    }
    let metadata = fs::symlink_metadata(source).map_err(|error| error.to_string())?;
    if !metadata.is_dir() {
        return Err(format!(
            "legacy profiles path is not a directory: {}",
            source.display()
        ));
    }
    #[cfg(windows)]
    if is_redirected(&metadata) {
        return Err(format!(
            "legacy profiles path is redirected: {}",
            source.display()
        ));
    }
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        *count += 1;
        if *count > 1024 {
            return Err("legacy profiles exceed 1024 entries".to_string());
        }
        let path = entry.path();
        let target = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        #[cfg(windows)]
        if is_redirected(&metadata) {
            return Err(format!(
                "legacy profile entry is redirected: {}",
                path.display()
            ));
        }
        if metadata.is_dir() {
            fs::create_dir(&target).map_err(|error| error.to_string())?;
            copy_profile_tree(&path, &target, depth + 1, count)?;
        } else if metadata.file_type().is_file() {
            fs::copy(&path, &target).map_err(|error| error.to_string())?;
            fs::OpenOptions::new()
                .write(true)
                .open(&target)
                .and_then(|file| file.sync_all())
                .map_err(|error| error.to_string())?;
        } else {
            return Err(format!(
                "legacy profile entry is not a regular file: {}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn profile_fingerprint(root: &Path) -> Result<String, String> {
    let mut paths = Vec::new();
    fn collect(here: &Path, depth: usize, paths: &mut Vec<PathBuf>) -> Result<(), String> {
        if depth > 8 {
            return Err("legacy profiles exceed depth limit".to_string());
        }
        let metadata = fs::symlink_metadata(here).map_err(|error| error.to_string())?;
        #[cfg(windows)]
        if is_redirected(&metadata) {
            return Err(format!(
                "legacy profile entry is redirected: {}",
                here.display()
            ));
        }
        if !metadata.is_dir() {
            return Err(format!(
                "legacy profile entry is not a directory: {}",
                here.display()
            ));
        }
        for entry in fs::read_dir(here).map_err(|error| error.to_string())? {
            let path = entry.map_err(|error| error.to_string())?.path();
            if paths.len() >= 1024 {
                return Err("legacy profiles exceed 1024 entries".to_string());
            }
            paths.push(path.clone());
            if fs::symlink_metadata(&path)
                .map_err(|error| error.to_string())?
                .is_dir()
            {
                collect(&path, depth + 1, paths)?;
            }
        }
        Ok(())
    }
    collect(root, 0, &mut paths)?;
    paths.sort();
    let mut hash = blake3::Hasher::new();
    for path in paths {
        let relative = path.strip_prefix(root).map_err(|error| error.to_string())?;
        hash.update(relative.to_string_lossy().as_bytes());
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        #[cfg(windows)]
        if is_redirected(&metadata) {
            return Err(format!(
                "legacy profile entry is redirected: {}",
                path.display()
            ));
        }
        if metadata.file_type().is_file() {
            let mut file = File::open(&path).map_err(|error| error.to_string())?;
            let mut bytes = [0u8; 64 * 1024];
            loop {
                let count = file.read(&mut bytes).map_err(|error| error.to_string())?;
                if count == 0 {
                    break;
                }
                hash.update(&bytes[..count]);
            }
        } else if !metadata.is_dir() {
            return Err(format!(
                "legacy profile entry is not a regular file: {}",
                path.display()
            ));
        }
    }
    Ok(hash.finalize().to_hex().to_string())
}

fn prepare_database_from(destination: &Path, roots: &[PathBuf]) -> Result<(), String> {
    let name = destination
        .file_name()
        .ok_or_else(|| "default database has no file name".to_string())?;
    let sources = roots
        .iter()
        .map(|root| root.join(name))
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    for source in &sources {
        let metadata = fs::symlink_metadata(source)
            .map_err(|error| format!("cannot inspect {}: {error}", source.display()))?;
        if !metadata.file_type().is_file() {
            return Err(format!(
                "legacy database is not a regular file: {}",
                source.display()
            ));
        }
        #[cfg(windows)]
        if is_redirected(&metadata) {
            return Err(format!(
                "legacy database is redirected: {}",
                source.display()
            ));
        }
    }
    if sources.len() > 1 {
        return Err(format!(
            "multiple legacy {} stores require an explicit choice: {}",
            name.to_string_lossy(),
            sources
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let Some(source) = sources.first() else {
        return Ok(());
    };
    let record_path = destination
        .parent()
        .ok_or_else(|| "default database has no parent".to_string())?
        .join(format!("{RECORD_PREFIX}{}.json", name.to_string_lossy()));
    if !destination.exists() && record_path.exists() {
        return Err(format!(
            "transition record exists without destination {}; inspect before retry",
            destination.display()
        ));
    }
    if destination.exists() {
        let record = read_record(&record_path)?;
        let key = name.to_string_lossy();
        let prior = &record["databases"][key.as_ref()];
        if prior["source"] == source.to_string_lossy().as_ref()
            && prior["fingerprint"] == source_fingerprint(source).map_err(|e| e.to_string())?
        {
            return Ok(());
        }
        return Err(format!(
            "default {} already exists while legacy state remains at {}; automatic replacement is refused",
            destination.display(),
            source.display()
        ));
    }
    let parent = destination.parent().expect("checked parent");
    fs::create_dir_all(parent)
        .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    let staged = parent.join(format!(
        ".{}.s192-{}.tmp",
        name.to_string_lossy(),
        std::process::id()
    ));
    if staged.exists() {
        return Err(format!(
            "stale transition stage requires inspection: {}",
            staged.display()
        ));
    }
    let result = (|| -> Result<(), String> {
        sqlite_snapshot(source, &staged)?;
        let after = source_fingerprint(source).map_err(|error| error.to_string())?;
        fs::OpenOptions::new()
            .write(true)
            .open(&staged)
            .and_then(|file| file.sync_all())
            .map_err(|error| format!("cannot sync staged database: {error}"))?;
        fs::hard_link(&staged, destination).map_err(|error| {
            format!(
                "cannot publish {} without replacing existing data: {error}",
                destination.display()
            )
        })?;
        let mut record = read_record(&record_path)?;
        record["databases"][name.to_string_lossy().as_ref()] = json!({
            "source": source,
            "fingerprint": after,
            "disposition": "copied-source-retained"
        });
        write_record(&record_path, &record)?;
        Ok(())
    })();
    let _ = fs::remove_file(&staged);
    result
}

fn sqlite_snapshot(source: &Path, staged: &Path) -> Result<(), String> {
    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let source_db = Connection::open_with_flags(source, flags).map_err(|error| {
        format!(
            "cannot open legacy SQLite store {}: {error}",
            source.display()
        )
    })?;
    source_db
        .busy_timeout(std::time::Duration::ZERO)
        .map_err(|error| error.to_string())?;
    source_db
        .execute_batch("BEGIN IMMEDIATE; ROLLBACK;")
        .map_err(|error| {
            format!(
                "legacy SQLite store is active or inaccessible at {}: {error}",
                source.display()
            )
        })?;
    let before_version: i64 = source_db
        .query_row("PRAGMA data_version", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    let check: String = source_db
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(|error| format!("legacy SQLite check failed: {error}"))?;
    if check != "ok" {
        return Err(format!("legacy SQLite store failed quick_check: {check}"));
    }
    let mut target = Connection::open(staged)
        .map_err(|error| format!("cannot open staged SQLite store: {error}"))?;
    let backup = Backup::new(&source_db, &mut target)
        .map_err(|error| format!("cannot start SQLite snapshot: {error}"))?;
    match backup
        .step(-1)
        .map_err(|error| format!("SQLite snapshot failed: {error}"))?
    {
        StepResult::Done => {}
        state => {
            return Err(format!(
                "legacy SQLite store was active during snapshot: {state:?}"
            ))
        }
    }
    drop(backup);
    let check: String = target
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(|error| format!("staged SQLite check failed: {error}"))?;
    if check != "ok" {
        return Err(format!("staged SQLite store failed quick_check: {check}"));
    }
    let after_version: i64 = source_db
        .query_row("PRAGMA data_version", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if before_version != after_version {
        return Err(format!(
            "legacy SQLite store changed during snapshot: {}",
            source.display()
        ));
    }
    Ok(())
}

fn source_fingerprint(source: &Path) -> io::Result<String> {
    let mut hasher = blake3::Hasher::new();
    for path in [
        source.to_path_buf(),
        source.with_file_name({
            let mut name = source.file_name().unwrap_or_default().to_os_string();
            name.push("-wal");
            name
        }),
    ] {
        if !path.exists() {
            hasher.update(b"absent");
            continue;
        }
        let mut file = File::open(&path)?;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn read_record(path: &Path) -> Result<Value, String> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(json!({"version":1,"databases":{}}))
        }
        Err(error) => {
            return Err(format!(
                "cannot read transition record {}: {error}",
                path.display()
            ))
        }
    };
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_RECORD_BYTES {
        return Err(format!(
            "transition record is oversized: {}",
            path.display()
        ));
    }
    let value: Value = serde_json::from_reader(file)
        .map_err(|error| format!("invalid transition record {}: {error}", path.display()))?;
    if value["version"] != 1 || !value["databases"].is_object() {
        return Err(format!("unsupported transition record: {}", path.display()));
    }
    Ok(value)
}

fn write_record(path: &Path, value: &Value) -> Result<(), String> {
    let staged = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&staged)
        .map_err(|error| format!("cannot stage transition record: {error}"))?;
    serde_json::to_writer(&mut file, value).map_err(|error| error.to_string())?;
    file.write_all(b"\n").map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    drop(file);
    let result = fs::hard_link(&staged, path)
        .map_err(|error| format!("cannot publish transition record without replacement: {error}"));
    let _ = fs::remove_file(&staged);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn package_private_legacy_root_is_inventoried_without_other_package_files() {
        let temp = tempfile::tempdir().unwrap();
        let roaming = temp.path().join("AppData").join("Roaming").join("fragcap");
        let local = temp.path().join("AppData").join("Local").join("fragcap");
        let packages = temp.path().join("AppData").join("Local").join("Packages");
        let product = packages
            .join("host.example")
            .join("LocalCache")
            .join("Roaming")
            .join("fragcap");
        fs::create_dir_all(&product).unwrap();
        fs::write(product.join("local.db"), b"synthetic").unwrap();
        let unrelated = packages
            .join("host.example")
            .join("LocalCache")
            .join("other");
        fs::create_dir_all(&unrelated).unwrap();
        let roots = legacy_roots_from(&roaming, &local, &packages).unwrap();
        assert!(roots.contains(&product));
        assert!(!roots.contains(&unrelated));
    }

    #[test]
    fn sqlite_wal_rows_survive_transition_and_repeat_start() {
        let temp = tempfile::tempdir().unwrap();
        let legacy = temp.path().join("old");
        fs::create_dir(&legacy).unwrap();
        let source = legacy.join("local.db");
        let connection = Connection::open(&source).unwrap();
        connection.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE marker(value TEXT); INSERT INTO marker VALUES ('retained');").unwrap();
        let destination = temp.path().join("new").join("local.db");
        prepare_database_from(&destination, std::slice::from_ref(&legacy)).unwrap();
        let value: String = Connection::open(&destination)
            .unwrap()
            .query_row("SELECT value FROM marker", [], |row| row.get(0))
            .unwrap();
        assert_eq!(value, "retained");
        prepare_database_from(&destination, &[legacy]).unwrap();
        assert!(source.exists());
    }

    #[test]
    fn conflicting_sources_do_not_create_destination() {
        let temp = tempfile::tempdir().unwrap();
        let first = temp.path().join("one");
        let second = temp.path().join("two");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        Connection::open(first.join("local.db")).unwrap();
        Connection::open(second.join("local.db")).unwrap();
        let destination = temp.path().join("new").join("local.db");
        assert!(prepare_database_from(&destination, &[first, second])
            .unwrap_err()
            .contains("multiple legacy"));
        assert!(!destination.exists());
    }

    #[test]
    fn existing_destination_without_transition_record_preserves_both_stores() {
        let temp = tempfile::tempdir().unwrap();
        let legacy = temp.path().join("old");
        let current = temp.path().join("new");
        fs::create_dir(&legacy).unwrap();
        fs::create_dir(&current).unwrap();
        let source = legacy.join("local.db");
        let destination = current.join("local.db");
        Connection::open(&source)
            .unwrap()
            .execute_batch("CREATE TABLE marker(value TEXT); INSERT INTO marker VALUES ('old')")
            .unwrap();
        Connection::open(&destination)
            .unwrap()
            .execute_batch("CREATE TABLE marker(value TEXT); INSERT INTO marker VALUES ('new')")
            .unwrap();
        let error = prepare_database_from(&destination, &[legacy]).unwrap_err();
        assert!(
            error.contains("automatic replacement is refused"),
            "{error}"
        );
        for (path, expected) in [(source, "old"), (destination, "new")] {
            let value: String = Connection::open(path)
                .unwrap()
                .query_row("SELECT value FROM marker", [], |row| row.get(0))
                .unwrap();
            assert_eq!(value, expected);
        }
    }

    #[test]
    fn closed_wal_database_transfers_without_a_false_conflict() {
        let temp = tempfile::tempdir().unwrap();
        let legacy = temp.path().join("old");
        fs::create_dir(&legacy).unwrap();
        let source = legacy.join("local.db");
        {
            let connection = Connection::open(&source).unwrap();
            connection.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE marker(value TEXT); INSERT INTO marker VALUES ('closed');").unwrap();
        }
        let destination = temp.path().join("new").join("local.db");
        prepare_database_from(&destination, &[legacy]).unwrap();
        let value: String = Connection::open(&destination)
            .unwrap()
            .query_row("SELECT value FROM marker", [], |row| row.get(0))
            .unwrap();
        assert_eq!(value, "closed");
    }

    #[test]
    fn active_legacy_writer_is_preserved_and_reported() {
        let temp = tempfile::tempdir().unwrap();
        let legacy = temp.path().join("old");
        fs::create_dir(&legacy).unwrap();
        let source = legacy.join("local.db");
        let connection = Connection::open(&source).unwrap();
        connection.execute_batch("CREATE TABLE marker(value TEXT); BEGIN IMMEDIATE; INSERT INTO marker VALUES ('uncommitted');").unwrap();
        let destination = temp.path().join("new").join("local.db");
        let error = prepare_database_from(&destination, &[legacy]).unwrap_err();
        assert!(error.contains("active or inaccessible"), "{error}");
        assert!(!destination.exists());
        connection.execute_batch("ROLLBACK").unwrap();
        assert!(source.exists());
    }

    #[test]
    fn authored_profiles_transfer_without_removing_the_source() {
        let temp = tempfile::tempdir().unwrap();
        let legacy = temp.path().join("old");
        let source = legacy.join("profiles");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("game.toml"), b"name = 'sample'").unwrap();
        let destination = temp.path().join("new").join("profiles");
        prepare_profiles_from(&destination, std::slice::from_ref(&legacy)).unwrap();
        prepare_profiles_from(&destination, &[legacy]).unwrap();
        assert_eq!(
            fs::read(source.join("game.toml")).unwrap(),
            fs::read(destination.join("game.toml")).unwrap()
        );
    }
}
