// SPDX-License-Identifier: Apache-2.0

//! Where profiles are looked for.
//!
//! Specification section 15.3 resolves a reference over a user profile
//! directory and any directories given on the command line, then a bundled set.
//! The resolution order itself lives in `fragcap-profile`, which takes an
//! ordered [`SearchPath`]; this module only decides what the user directory is
//! on this platform and assembles the search path from it and the repeatable
//! `--profile-dir` values.
//!
//! The user directory is the current user's persistent configuration location,
//! `%USERPROFILE%\.fragcap\roaming\profiles` on Windows. An override, `FRAGCAP_PROFILE_DIR`,
//! exists so a test can point the user directory at a scratch directory without
//! depending on the developer's real one.
//!
//! The bundled set is empty in this release: a bundled profile is a claim about
//! a specific game's current process topology, and the slices that can verify
//! such a claim own shipping one (specification section 15.5).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use fragcap::profile::{BundledSet, SearchPath};

/// The environment variable that overrides the user profile directory.
pub const PROFILE_DIR_ENV: &str = "FRAGCAP_PROFILE_DIR";

/// The environment variable that overrides the catalog store path (slice S050).
/// The `--catalog-db` flag takes precedence over it.
pub const CATALOG_DB_ENV: &str = "FRAGCAP_CATALOG_DB";

/// The environment variable that overrides the local store path (slice S050). The
/// `--local-db` flag takes precedence over it.
pub const LOCAL_DB_ENV: &str = "FRAGCAP_LOCAL_DB";

/// The environment variable that overrides the Deep Capture session directory.
pub const SESSION_DIR_ENV: &str = "FRAGCAP_SESSION_DIR";

/// An explicit whole-store location, primarily useful to isolated automation.
/// The ordinary default remains the current Windows profile's `.fragcap` root.
pub const STORAGE_ROOT_ENV: &str = "FRAGCAP_STORAGE_ROOT";

/// The environment variable that overrides the analyzer extcap directory.
pub const EXTCAP_DIR_ENV: &str = "FRAGCAP_EXTCAP_DIR";

/// The environment variable that overrides the machine-wide (system) analyzer
/// extcap directory.
pub const SYSTEM_EXTCAP_DIR_ENV: &str = "FRAGCAP_SYSTEM_EXTCAP_DIR";

/// The fragcap binary name inside an analyzer extcap directory. Both the
/// `extcap install` command (which writes it) and the `doctor` probe (which
/// checks for it) reference this one constant so the register target and the
/// readiness check cannot drift.
#[cfg(windows)]
pub const EXTCAP_BINARY: &str = "fragcap.exe";
#[cfg(not(windows))]
pub const EXTCAP_BINARY: &str = "fragcap";

/// The analyzer's personal extcap directory, or `None` when the platform
/// location cannot be determined.
///
/// This is where the fragcap binary is copied to register it as an extcap
/// capture source (specification section 14.5). `doctor` reports it read-only and
/// installs nothing. On Windows it is `%APPDATA%\Wireshark\extcap`; elsewhere it
/// is the XDG or HOME Wireshark configuration location. An override,
/// `FRAGCAP_EXTCAP_DIR`, lets a test point it at a scratch directory.
pub fn extcap_dir() -> Option<PathBuf> {
    if let Some(dir) = env::var_os(EXTCAP_DIR_ENV) {
        return Some(PathBuf::from(dir));
    }
    #[cfg(windows)]
    {
        env::var_os("APPDATA").map(|base| PathBuf::from(base).join("Wireshark").join("extcap"))
    }
    #[cfg(not(windows))]
    {
        if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg).join("wireshark").join("extcap"));
        }
        env::var_os("HOME").map(|home| {
            PathBuf::from(home)
                .join(".config")
                .join("wireshark")
                .join("extcap")
        })
    }
}

/// The machine-wide (system) analyzer extcap directory, or `None` when the
/// platform location cannot be determined.
///
/// This is where the MSI's machine-wide option installs the fragcap binary, and
/// where `extcap install --system` writes it, so a registration is visible to
/// every user of the machine. `doctor` reads it read-only and installs nothing.
///
/// Precedence on Windows: the `FRAGCAP_SYSTEM_EXTCAP_DIR` override (which lets a
/// test point it at a scratch directory), then Wireshark's actual install
/// directory from the registry (the value the Wireshark installer records and the
/// fragcap MSI registers into) plus `extcap`, so a non-default install location is
/// recognized, then the `%ProgramFiles%\Wireshark\extcap` default. Off Windows it
/// is the override or a conventional system location. Both `extcap install
/// --system` and the `doctor` probe resolve through this one function, so the
/// register target and the readiness check cannot drift.
pub fn system_extcap_dir() -> Option<PathBuf> {
    if let Some(dir) = env::var_os(SYSTEM_EXTCAP_DIR_ENV) {
        return Some(PathBuf::from(dir));
    }
    #[cfg(windows)]
    {
        if let Some(dir) = wireshark_install_dir() {
            return Some(dir.join("extcap"));
        }
        env::var_os("ProgramFiles").map(|base| PathBuf::from(base).join("Wireshark").join("extcap"))
    }
    #[cfg(not(windows))]
    {
        Some(PathBuf::from("/usr/lib/wireshark/extcap"))
    }
}

/// Wireshark's install directory from `HKLM\SOFTWARE\Wireshark`, or `None` when it
/// is not installed or the value cannot be read.
///
/// This is the location the Wireshark installer records and the fragcap MSI's
/// `RegistrySearch` reads, so resolving the system extcap directory through it
/// keeps `extcap install --system`, the MSI, and `doctor` in agreement even when
/// Wireshark is installed outside Program Files. Read-only: it opens no process
/// handle and modifies nothing (P-1).
#[cfg(windows)]
fn wireshark_install_dir() -> Option<PathBuf> {
    use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};

    let subkey: Vec<u16> = "SOFTWARE\\Wireshark"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    // First call sizes the default value (name null) in bytes.
    let mut bytes: u32 = 0;
    // SAFETY: `subkey` is a live null-terminated UTF-16 string; all output
    // pointers are null except `bytes`, which the API writes.
    let rc = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            subkey.as_ptr(),
            std::ptr::null(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut bytes,
        )
    };
    if rc != 0 || bytes == 0 {
        return None;
    }

    let mut buf: Vec<u16> = vec![0u16; (bytes as usize).div_ceil(2)];
    let mut cap = bytes;
    // SAFETY: `buf` has room for `cap` bytes; `RegGetValueW` writes at most that
    // and null-terminates the string it returns.
    let rc = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            subkey.as_ptr(),
            std::ptr::null(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buf.as_mut_ptr() as *mut core::ffi::c_void,
            &mut cap,
        )
    };
    if rc != 0 {
        return None;
    }

    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    let dir = String::from_utf16_lossy(&buf[..end]);
    if dir.is_empty() {
        None
    } else {
        Some(PathBuf::from(dir))
    }
}

/// The user profile directory, or `None` when the platform location cannot be
/// determined.
pub fn user_profile_dir() -> Option<PathBuf> {
    if let Some(dir) = env::var_os(PROFILE_DIR_ENV) {
        return Some(PathBuf::from(dir));
    }
    default_roaming_data_root().map(|base| base.join("profiles"))
}

/// The catalog store path an override supplies, or `None` when the operator
/// supplied neither the `--catalog-db` flag nor the `FRAGCAP_CATALOG_DB` override.
///
/// The flag takes precedence over the environment variable, mirroring how the
/// profile directory resolves. Returning a path here does not assert the file
/// exists; the caller decides that a missing file means no provider (not an
/// error), while a present-but-unopenable one is surfaced loudly.
pub fn catalog_db_path(flag: Option<&Path>) -> Option<PathBuf> {
    if let Some(path) = flag {
        return Some(path.to_path_buf());
    }
    env::var_os(CATALOG_DB_ENV).map(PathBuf::from)
}

/// The local store path an override supplies, or `None` when the operator
/// supplied neither the `--local-db` flag nor the `FRAGCAP_LOCAL_DB` override.
/// Flag over environment, as with the catalog store.
pub fn local_db_path(flag: Option<&Path>) -> Option<PathBuf> {
    if let Some(path) = flag {
        return Some(path.to_path_buf());
    }
    env::var_os(LOCAL_DB_ENV).map(PathBuf::from)
}

/// The per-user default catalog store location (the ShruggieTech-shipped store),
/// or `None` when the platform application-data base cannot be determined.
///
/// `%USERPROFILE%\.fragcap\roaming\catalog.db` on Windows. The first-run bootstrap seeds it from the
/// template shipped beside the binary; an explicitly named path is never created
/// on the operator's behalf.
pub fn default_catalog_db_path() -> Option<PathBuf> {
    default_roaming_data_root().map(|base| base.join("catalog.db"))
}

/// The per-user default local store location (the user-owned store), or `None`
/// when the platform application-data base cannot be determined.
///
/// `%USERPROFILE%\.fragcap\roaming\local.db` on Windows. The first-run bootstrap creates it empty; it is
/// never seeded and never replaced by a catalog refresh.
pub fn default_local_db_path() -> Option<PathBuf> {
    default_roaming_data_root().map(|base| base.join("local.db"))
}

/// The current user's shared persistent root, outside AppData virtualization.
pub fn storage_root() -> std::io::Result<PathBuf> {
    if let Some(root) = env::var_os(STORAGE_ROOT_ENV) {
        let root = PathBuf::from(root);
        if root.is_absolute() {
            return Ok(root);
        }
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "FRAGCAP_STORAGE_ROOT must be absolute",
        ));
    }
    #[cfg(windows)]
    {
        current_profile_path().map(|path| path.join(".fragcap"))
    }
    #[cfg(not(windows))]
    {
        env::var_os("HOME")
            .map(|path| PathBuf::from(path).join(".fragcap"))
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME is unavailable"))
    }
}

/// The Windows profile path is resolved from the current token, not from a
/// packaged host's APPDATA or USERPROFILE environment variables.
#[cfg(windows)]
pub(crate) fn current_profile_path() -> std::io::Result<PathBuf> {
    use windows_sys::Win32::UI::Shell::FOLDERID_Profile;
    fragcap::deep_capture::OutputRecipient::current_storage_account()?;
    known_folder_path(&FOLDERID_Profile)
}

#[cfg(windows)]
pub(crate) fn legacy_roaming_data_root() -> std::io::Result<PathBuf> {
    use windows_sys::Win32::UI::Shell::FOLDERID_RoamingAppData;
    known_folder_path(&FOLDERID_RoamingAppData).map(|path| path.join("fragcap"))
}

#[cfg(windows)]
pub(crate) fn legacy_local_data_root() -> std::io::Result<PathBuf> {
    use windows_sys::Win32::UI::Shell::FOLDERID_LocalAppData;
    known_folder_path(&FOLDERID_LocalAppData).map(|path| path.join("fragcap"))
}

#[cfg(windows)]
fn known_folder_path(id: &windows_sys::core::GUID) -> std::io::Result<PathBuf> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::System::Com::CoTaskMemFree;
    use windows_sys::Win32::UI::Shell::SHGetKnownFolderPath;

    let mut raw = std::ptr::null_mut();
    // SAFETY: the folder ID and output pointer are valid; a null token selects
    // the current process token. The returned allocation is freed below.
    let result = unsafe { SHGetKnownFolderPath(id, 0, 0, &mut raw) };
    if result < 0 || raw.is_null() {
        return Err(std::io::Error::other(format!(
            "cannot resolve the current Windows user's profile (HRESULT {result:#x})"
        )));
    }
    let length = unsafe { (0..32768).find(|&offset| *raw.add(offset) == 0) };
    let path = length.map(|length| {
        let units = unsafe { std::slice::from_raw_parts(raw, length) };
        PathBuf::from(std::ffi::OsString::from_wide(units))
    });
    // SAFETY: SHGetKnownFolderPath allocated this pointer with the COM allocator.
    unsafe { CoTaskMemFree(raw.cast()) };
    let path = path.ok_or_else(|| std::io::Error::other("Windows profile path is too long"))?;
    if !path.is_absolute() || !path.is_dir() {
        return Err(std::io::Error::other(format!(
            "current Windows profile is unavailable: {}",
            path.display()
        )));
    }
    Ok(path)
}

/// The canonical current-user roaming data root used by fresh-start cleanup.
pub fn default_roaming_data_root() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        storage_root().ok().map(|base| base.join("roaming"))
    }
    #[cfg(not(windows))]
    {
        if env::var_os(STORAGE_ROOT_ENV).is_some() {
            return storage_root().ok().map(|base| base.join("roaming"));
        }
        env::var_os("APPDATA").map(|base| PathBuf::from(base).join("fragcap"))
    }
}

/// The canonical current-user local data root used by fresh-start cleanup.
pub fn default_local_data_root() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        storage_root().ok().map(|base| base.join("local"))
    }
    #[cfg(not(windows))]
    {
        if env::var_os(STORAGE_ROOT_ENV).is_some() {
            return storage_root().ok().map(|base| base.join("local"));
        }
        env::var_os("LOCALAPPDATA").map(|base| PathBuf::from(base).join("fragcap"))
    }
}

/// The default Deep Capture session-bundle root, or `None` when the platform
/// application-data base cannot be determined.
///
/// `%USERPROFILE%\.fragcap\roaming\sessions` on Windows. An override,
/// `FRAGCAP_SESSION_DIR`, lets tests and operators point doctor at a scratch
/// location. Resolving the path is read-only; the directory is not created here.
pub fn deep_capture_session_dir() -> Option<PathBuf> {
    if let Some(dir) = env::var_os(SESSION_DIR_ENV) {
        return Some(PathBuf::from(dir));
    }
    default_roaming_data_root().map(|base| base.join("sessions"))
}

/// Observe a selected storage path without creating it. On Windows the
/// effective path comes from the opened file or directory handle.
pub(crate) fn observe_storage(
    name: &str,
    path: PathBuf,
    source: &str,
) -> crate::doctor::StorageObservation {
    use crate::doctor::StorageObservation;
    let mut observation = StorageObservation {
        name: name.to_string(),
        selected: path.clone(),
        source: source.to_string(),
        existence: "unavailable".to_string(),
        effective: None,
        redirected: None,
        reason: None,
    };
    match fs::symlink_metadata(&path) {
        Ok(metadata) => {
            observation.existence = "present".to_string();
            match effective_opened_path(&path, metadata.is_dir()) {
                Ok(effective) => {
                    #[cfg(windows)]
                    let redirected = {
                        let normalized = |path: &Path| {
                            let value = path.to_string_lossy();
                            if let Some(network) = value.strip_prefix("\\\\?\\UNC\\") {
                                format!("\\\\{network}").to_lowercase()
                            } else {
                                value.trim_start_matches("\\\\?\\").to_lowercase()
                            }
                        };
                        long_selected_path(&path)
                            .map(|selected| normalized(&effective) != normalized(&selected))
                    };
                    #[cfg(not(windows))]
                    let redirected: std::io::Result<bool> = Ok(effective != path);
                    if path.is_absolute() {
                        match redirected {
                            Ok(redirected) => observation.redirected = Some(redirected),
                            Err(error) => {
                                observation.reason =
                                    Some(format!("selected-path normalization failed: {error}"))
                            }
                        }
                    } else {
                        observation.reason =
                            Some("relative selected path cannot establish redirection".to_string());
                    }
                    observation.effective = Some(effective);
                }
                Err(error) => {
                    observation.reason = Some(format!("opened-path verification failed: {error}"))
                }
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            observation.existence = "absent".to_string();
            observation.reason = Some("the path has not been created".to_string());
        }
        Err(error) => observation.reason = Some(format!("path inspection failed: {error}")),
    }
    observation
}

#[cfg(windows)]
fn long_selected_path(path: &Path) -> std::io::Result<PathBuf> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use windows_sys::Win32::Storage::FileSystem::GetLongPathNameW;

    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    // SAFETY: `wide` is a live null-terminated UTF-16 path; a null output
    // with zero capacity asks Windows for the required character count.
    let needed = unsafe { GetLongPathNameW(wide.as_ptr(), std::ptr::null_mut(), 0) };
    if needed == 0 || needed > 32768 {
        return Err(std::io::Error::last_os_error());
    }
    let mut buffer = vec![0u16; needed as usize + 1];
    // SAFETY: the output has at least the reported capacity; the input remains live.
    let written =
        unsafe { GetLongPathNameW(wide.as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32) };
    if written == 0 || written as usize >= buffer.len() {
        return Err(std::io::Error::last_os_error());
    }
    Ok(PathBuf::from(std::ffi::OsString::from_wide(
        &buffer[..written as usize],
    )))
}

#[cfg(windows)]
fn effective_opened_path(path: &Path, directory: bool) -> std::io::Result<PathBuf> {
    use std::os::windows::ffi::OsStringExt;
    use std::os::windows::fs::OpenOptionsExt;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFinalPathNameByHandleW, FILE_FLAG_BACKUP_SEMANTICS,
    };

    let mut options = fs::OpenOptions::new();
    options.read(true);
    if directory {
        options.custom_flags(FILE_FLAG_BACKUP_SEMANTICS);
    }
    let file = options.open(path)?;
    let handle = file.as_raw_handle() as isize;
    let needed = unsafe { GetFinalPathNameByHandleW(handle, std::ptr::null_mut(), 0, 0) };
    if needed == 0 || needed > 32768 {
        return Err(std::io::Error::last_os_error());
    }
    let mut buffer = vec![0u16; needed as usize + 1];
    let written =
        unsafe { GetFinalPathNameByHandleW(handle, buffer.as_mut_ptr(), buffer.len() as u32, 0) };
    if written == 0 || written as usize >= buffer.len() {
        return Err(std::io::Error::last_os_error());
    }
    Ok(PathBuf::from(std::ffi::OsString::from_wide(
        &buffer[..written as usize],
    )))
}

#[cfg(not(windows))]
fn effective_opened_path(path: &Path, _directory: bool) -> std::io::Result<PathBuf> {
    path.canonicalize()
}

/// Assemble the section 15.3 search path from the command-line directories and
/// the user directory.
pub fn search_path(command_line: &[PathBuf]) -> SearchPath {
    SearchPath {
        command_line: command_line.to_vec(),
        user: user_profile_dir(),
    }
}

/// The bundled profile set for this release: empty.
pub fn bundled() -> BundledSet {
    BundledSet::empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opened_destination_preserves_non_ascii_and_absent_is_distinct() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("caf\u{e9}").join("local.db");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, b"synthetic").unwrap();
        let present = observe_storage("local", file.clone(), "controlled");
        assert_eq!(present.existence, "present");
        assert!(present
            .effective
            .as_ref()
            .unwrap()
            .to_string_lossy()
            .contains("caf\u{e9}"));
        assert_eq!(present.redirected, Some(false), "{present:?}");

        let absent = observe_storage("local", file.with_file_name("missing.db"), "controlled");
        assert_eq!(absent.existence, "absent");
        assert!(absent.effective.is_none());
        assert!(absent.redirected.is_none());
    }

    #[test]
    fn the_flag_supplies_each_store_path() {
        // Each explicit flag is returned as-is, independent of the environment.
        let catalog = Path::new("C:/scratch/catalog.db");
        assert_eq!(catalog_db_path(Some(catalog)), Some(catalog.to_path_buf()));
        let local = Path::new("C:/scratch/local.db");
        assert_eq!(local_db_path(Some(local)), Some(local.to_path_buf()));
    }

    #[cfg(windows)]
    #[test]
    fn the_default_stores_share_the_current_root() {
        let root = default_roaming_data_root().expect("current-user root");
        assert_eq!(default_catalog_db_path(), Some(root.join("catalog.db")));
        assert_eq!(default_local_db_path(), Some(root.join("local.db")));
        assert_eq!(deep_capture_session_dir(), Some(root.join("sessions")));
    }
}
