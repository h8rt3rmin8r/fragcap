// SPDX-License-Identifier: Apache-2.0

//! Gathering the real environment facts `doctor` classifies.
//!
//! Deliberately thin and deliberately not unit tested: everything worth
//! asserting is a pure classifier over an injected [`Inputs`] (see
//! [`super::checks`]). This module only reads the machine, read-only, and it
//! never installs, downloads, or modifies the capture driver, which is the
//! Licensing section's rule made mechanical here rather than remembered.
//!
//! A non-Windows build returns a minimal [`Inputs`] so the command still runs
//! and classifies, which is what keeps `doctor` exercised on any target.

use std::path::PathBuf;

use super::{Inputs, Privilege, Subsystem};

/// The analyzer extcap directories (per-user and machine-wide) and whether a
/// fragcap binary is installed in each, read-only.
///
/// Detection only: this reads the directories to see whether the binary has been
/// copied there and installs, downloads, and copies nothing, which is the
/// Licensing rule and constitution P-1 made mechanical (specification 14.5).
/// Returns `(user_dir, user_installed, system_dir, system_installed)`.
fn extcap_status() -> (Option<PathBuf>, bool, Option<PathBuf>, bool) {
    let present = |dir: &Option<PathBuf>| {
        dir.as_ref()
            .map(|d| d.join(crate::paths::EXTCAP_BINARY).exists())
            .unwrap_or(false)
    };
    let user_dir = crate::paths::extcap_dir();
    let system_dir = crate::paths::system_extcap_dir();
    let user_installed = present(&user_dir);
    let system_installed = present(&system_dir);
    (user_dir, user_installed, system_dir, system_installed)
}

/// Count the `.json` profiles directly in a directory, or zero when it cannot
/// be read.
fn count_profiles(dir: &Option<PathBuf>) -> usize {
    let Some(dir) = dir else {
        return 0;
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .count()
}

/// Whether the process-event tracing capability is built into this binary, and
/// whether its session can open.
///
/// `None` when the `etw` feature is not compiled in, a non-blocking skip. When
/// it is, this reports whether a session could be opened, which is what the
/// tracing check turns a blocking verdict on while elevated.
fn tracing_availability() -> Option<bool> {
    #[cfg(all(windows, feature = "etw"))]
    {
        match fragcap::EtwWatcher::start("fragcap-doctor-probe") {
            Ok(watcher) => {
                let _ = watcher.stop();
                Some(true)
            }
            Err(_) => Some(false),
        }
    }
    #[cfg(not(all(windows, feature = "etw")))]
    {
        None
    }
}

/// Whether the live capture backend is compiled into this binary.
///
/// Presence is a compile-time fact, so this is `Some(true)` when the `live`
/// feature is on and `None` when it is not, mirroring [`tracing_availability`].
/// A binary built without it cannot capture, which the check turns into a
/// blocking verdict rather than a downstream "no interfaces" symptom.
fn live_availability() -> Option<bool> {
    #[cfg(feature = "live")]
    {
        Some(true)
    }
    #[cfg(not(feature = "live"))]
    {
        None
    }
}

/// Whether the socket-table attribution backend is compiled into this binary.
///
/// `Some(true)` when the `socket-table` feature is on, `None` when it is not.
/// Absence degrades attribution (ETW may still cover it), so the check treats it
/// as a non-blocking concern.
fn socket_table_availability() -> Option<bool> {
    #[cfg(feature = "socket-table")]
    {
        Some(true)
    }
    #[cfg(not(feature = "socket-table"))]
    {
        None
    }
}

/// The enumerated interfaces and the three-valued loopback state, read from the
/// live capture backend.
///
/// The backend is linked only under the `live` feature on Windows, so the
/// `#[cfg(not(...))]` fallback returns an empty interface set and an undetermined
/// loopback. That fallback is load-bearing: it keeps the default `cargo test
/// --workspace` (no features) and the platform-neutral `fragcap-core` build
/// compiling (constitution P-2). This mirrors [`live_availability`]. The old
/// probe hardcoded the empty set and guessed loopback from an unrelated file;
/// both are gone.
fn live_probe(wpcap_loadable: bool) -> (Vec<super::IfaceInfo>, Option<bool>, Option<String>) {
    #[cfg(all(feature = "live", windows))]
    {
        use fragcap::core::virtual_verdict;
        // wpcap.dll is a delay-load import (crates/fragcap-cli/build.rs), so the
        // first call that reaches it forces the load. Both fragcap::enumerate and
        // fragcap::detect_driver call pcap::Device::list, and when the DLL cannot
        // be resolved that load raises a delay-load exception (0xC06D007E,
        // MOD_NOT_FOUND) that aborts the process before the Result-based guards
        // inside those functions can run. So the live backend is touched only when
        // wpcap.dll is loadable (see the gate in gather_windows). When it is not,
        // nothing was attempted, so this is not a probe failure: the interface set
        // is empty with no error, and the npcap / winpcap-api checks (not the
        // interface check) carry the remediation. Without this gate `doctor` could
        // not run on the very machine that most needs it to run and say what to
        // install.
        if !wpcap_loadable {
            return (Vec::new(), None, None);
        }
        // A failed enumeration is preserved as an error string rather than
        // flattened to an empty set, so the classifier does not report a probe
        // that could not run as an observed-empty machine (P-9).
        let (interfaces, error) = match fragcap::enumerate() {
            Ok(inventory) => (
                inventory
                    .interfaces
                    .iter()
                    .map(|record| super::IfaceInfo {
                        name: record.name.to_string(),
                        addr: record.addresses.first().map(|addr| addr.to_string()),
                        up: record.is_up,
                        is_virtual: virtual_verdict(record).is_virtual(),
                    })
                    .collect(),
                None,
            ),
            Err(err) => (Vec::new(), Some(err.to_string())),
        };
        let loopback = fragcap::detect_driver().loopback_supported;
        (interfaces, loopback, error)
    }
    #[cfg(not(all(feature = "live", windows)))]
    {
        // Enumeration was not attempted (the backend is not linked); that is not
        // a failure, so the error is None and the classifier attributes the empty
        // set to the missing backend.
        let _ = wpcap_loadable;
        (Vec::new(), None, None)
    }
}

/// The identity facts: which fragcap produced this report and where it keeps its
/// per-user data. All read-only and computed regardless of whether the paths
/// exist yet.
fn identity_fields() -> (
    String,
    Option<PathBuf>,
    Option<PathBuf>,
    Option<PathBuf>,
    Option<PathBuf>,
) {
    (
        env!("CARGO_PKG_VERSION").to_string(),
        std::env::current_exe().ok(),
        crate::paths::user_profile_dir(),
        crate::paths::default_catalog_db_path(),
        crate::paths::default_local_db_path(),
    )
}

/// The npcap version, read from the `wpcap.dll` FileVersion resource, or the
/// literal "installed" when it cannot be read.
///
/// This is a read-only version-resource query on a file path: it opens no
/// process handle, loads no library, and needs no elevation, so constitution
/// P-1 is not engaged. Any failure falls back to "installed" so the report never
/// claims a version it did not actually read (P-9).
#[cfg(windows)]
fn npcap_version(wpcap: &std::path::Path) -> String {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
    };

    const FALLBACK: &str = "installed";

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    let path: Vec<u16> = wpcap
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // SAFETY: `path` is a live, null-terminated UTF-16 string; `handle` is a
    // live out-param the API writes and the size query otherwise ignores.
    let size = unsafe {
        let mut handle: u32 = 0;
        GetFileVersionInfoSizeW(path.as_ptr(), &mut handle)
    };
    if size == 0 {
        return FALLBACK.to_string();
    }

    let mut buf: Vec<u8> = vec![0u8; size as usize];
    // SAFETY: `buf` is exactly `size` bytes, matching the value just returned;
    // `path` is the same live null-terminated string.
    let ok = unsafe { GetFileVersionInfoW(path.as_ptr(), 0, size, buf.as_mut_ptr() as *mut _) };
    if ok == 0 {
        return FALLBACK.to_string();
    }

    // The translation table: the first (language, codepage) pair names the
    // StringFileInfo sub-block that carries the human FileVersion string.
    let mut val_ptr: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut val_len: u32 = 0;
    let translation = wide("\\VarFileInfo\\Translation");
    // SAFETY: `buf` holds a valid version-info block of `size` bytes;
    // `translation` is a live null-terminated string; the out params are live.
    let ok = unsafe {
        VerQueryValueW(
            buf.as_ptr() as *const _,
            translation.as_ptr(),
            &mut val_ptr,
            &mut val_len,
        )
    };
    if ok == 0 || val_ptr.is_null() || val_len < 4 {
        return FALLBACK.to_string();
    }
    // SAFETY: the len check guarantees at least two u16 at `val_ptr`.
    let (lang, codepage) = unsafe {
        let p = val_ptr as *const u16;
        (*p, *p.add(1))
    };

    let subblock = format!("\\StringFileInfo\\{lang:04x}{codepage:04x}\\FileVersion");
    let subblock = wide(&subblock);
    let mut str_ptr: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut str_len: u32 = 0;
    // SAFETY: as the translation query above; `subblock` is live and null
    // terminated.
    let ok = unsafe {
        VerQueryValueW(
            buf.as_ptr() as *const _,
            subblock.as_ptr(),
            &mut str_ptr,
            &mut str_len,
        )
    };
    if ok == 0 || str_ptr.is_null() || str_len == 0 {
        return FALLBACK.to_string();
    }
    // `str_len` is the length in characters, including the trailing null.
    // SAFETY: `str_ptr` points to `str_len` u16 code units per the API contract.
    let value = unsafe {
        let slice = std::slice::from_raw_parts(str_ptr as *const u16, str_len as usize);
        String::from_utf16_lossy(slice)
    };
    let value = value.trim_end_matches('\0').trim();
    if value.is_empty() {
        FALLBACK.to_string()
    } else {
        value.to_string()
    }
}

/// Count the registered target entries in the local store, for the no-targets
/// check. A store path that does not exist yet is a real empty store (zero
/// entries), reported without opening anything so the read-only probe creates no
/// file. An existing store that cannot be opened or read returns `None`
/// (undetermined), never a fabricated zero (P-9).
fn read_target_entry_count(local_db_path: &Option<PathBuf>) -> Option<usize> {
    let path = local_db_path.as_ref()?;
    if !path.exists() {
        return Some(0);
    }
    let store = fragcap::targets::Store::open(path).ok()?;
    store.targets().ok().map(|targets| targets.len())
}

/// Gather the environment facts for `doctor`.
pub fn gather() -> Inputs {
    let user_dir = crate::paths::user_profile_dir();
    let user_count = count_profiles(&user_dir);

    #[cfg(windows)]
    {
        gather_windows(user_count)
    }
    #[cfg(not(windows))]
    {
        let (extcap_dir, extcap_installed, extcap_system_dir, extcap_system_installed) =
            extcap_status();
        let (fragcap_version, binary_path, profile_dir, catalog_db_path, local_db_path) =
            identity_fields();
        // wpcap.dll is not loadable on a non-Windows build; the live backend is
        // not linked anyway.
        let (interfaces, _loopback, interface_error) = live_probe(false);
        let target_entry_count = read_target_entry_count(&local_db_path);
        Inputs {
            fragcap_version,
            binary_path,
            profile_dir,
            catalog_db_present: catalog_db_path.as_ref().is_some_and(|p| p.exists()),
            catalog_db_path,
            local_db_present: local_db_path.as_ref().is_some_and(|p| p.exists()),
            local_db_path,
            os: format!("{} (capture is Windows-only)", std::env::consts::OS),
            subsystem: Subsystem::Native,
            privilege: Privilege::NotElevated,
            npcap: None,
            etw_available: tracing_availability(),
            live_available: live_availability(),
            socket_table_available: socket_table_availability(),
            interfaces,
            interface_error,
            extcap_installed,
            extcap_dir,
            extcap_system_installed,
            extcap_system_dir,
            bundled_count: crate::paths::bundled().len(),
            user_count,
            target_entry_count,
        }
    }
}

/// The Windows probe. Reads the filesystem for the npcap markers it can see
/// without a registry API, which is a best-effort detection the operator reads
/// as guidance; it installs nothing.
#[cfg(windows)]
fn gather_windows(user_count: usize) -> Inputs {
    use super::NpcapInfo;

    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    let system32 = PathBuf::from(&system_root).join("System32");
    let npcap_dir = system32.join("Npcap");
    let npcap_wpcap = npcap_dir.join("wpcap.dll");
    let system_wpcap = system32.join("wpcap.dll");

    // npcap is present when its own wpcap.dll exists in the Npcap directory; this
    // drives the installation and version report below.
    let npcap_present = npcap_wpcap.exists();

    // Whether the live backend can be touched at all. enumerate and detect_driver
    // reach the delay-loaded `wpcap.dll` by name, which the loader resolves from
    // the default DLL search path (System32), not from System32\Npcap: fragcap
    // adds no npcap DLL directory and requires the WinPcap API compatibility
    // option, which installs wpcap.dll into System32 (spec 20.3). When that copy
    // is absent the delay-load raises a MOD_NOT_FOUND exception at the first pcap
    // call and aborts the process before doctor can report it, so the gate is the
    // presence of the System32 copy, not of npcap itself. npcap installed without
    // the compatibility option (Npcap\wpcap.dll present, System32\wpcap.dll
    // absent) is therefore not probed here, and the winpcap-api check names the
    // fix.
    let wpcap_loadable = system_wpcap.exists();

    // The live backend answers what exists: the interface set, and whether a
    // loopback adapter is among them. Probed only when wpcap.dll is loadable, and
    // linked only under the `live` feature, so this returns empty and
    // undetermined otherwise.
    let (interfaces, loopback_supported, interface_error) = live_probe(wpcap_loadable);

    let npcap = if npcap_present {
        Some(NpcapInfo {
            version: npcap_version(&npcap_wpcap),
            // From enumerating the loopback adapter, never a proxy file.
            loopback_supported,
            winpcap_api_mode: system_wpcap.exists(),
        })
    } else {
        None
    };

    let (extcap_dir, extcap_installed, extcap_system_dir, extcap_system_installed) =
        extcap_status();
    let (fragcap_version, binary_path, profile_dir, catalog_db_path, local_db_path) =
        identity_fields();
    let target_entry_count = read_target_entry_count(&local_db_path);
    Inputs {
        fragcap_version,
        binary_path,
        profile_dir,
        catalog_db_present: catalog_db_path.as_ref().is_some_and(|p| p.exists()),
        catalog_db_path,
        local_db_present: local_db_path.as_ref().is_some_and(|p| p.exists()),
        local_db_path,
        os: "Windows".to_string(),
        subsystem: Subsystem::Native,
        privilege: if is_elevated() {
            Privilege::Elevated
        } else {
            Privilege::NotElevated
        },
        npcap,
        etw_available: tracing_availability(),
        live_available: live_availability(),
        socket_table_available: socket_table_availability(),
        interfaces,
        interface_error,
        extcap_installed,
        extcap_dir,
        extcap_system_installed,
        extcap_system_dir,
        bundled_count: crate::paths::bundled().len(),
        user_count,
        target_entry_count,
    }
}

/// Whether this process runs elevated, read from its own access token.
///
/// Reads the elevation flag of the current process's primary token through the
/// documented current-process token pseudo handle, so no handle is opened
/// against any process and nothing is closed: there is no handle to audit
/// against P-1, and the query carries no rights beyond read. A query that
/// genuinely fails defaults to not elevated, so the blocking doctor branch that
/// pairs elevation with an unavailable trace session is never entered on a false
/// positive.
#[cfg(windows)]
pub(crate) fn is_elevated() -> bool {
    use windows_sys::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION};

    // The current process's primary token, a documented pseudo handle whose
    // value is (HANDLE)-4. HANDLE is an isize in this binding.
    const CURRENT_PROCESS_TOKEN: isize = -4;

    let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
    let mut returned: u32 = 0;
    let size = core::mem::size_of::<TOKEN_ELEVATION>() as u32;
    // SAFETY: the token is the well-known current-process pseudo handle, the
    // information class matches the out buffer type, the buffer is a live
    // `TOKEN_ELEVATION` whose exact size is passed, and `returned` is a live
    // `u32`. On failure the call returns 0 and leaves `TokenIsElevated` at its
    // initialized 0, which reads as not elevated.
    let ok = unsafe {
        GetTokenInformation(
            CURRENT_PROCESS_TOKEN,
            TokenElevation,
            (&mut elevation as *mut TOKEN_ELEVATION).cast(),
            size,
            &mut returned,
        )
    };
    ok != 0 && elevation.TokenIsElevated != 0
}
