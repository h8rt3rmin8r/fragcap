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

use std::io::Read;
use std::path::PathBuf;

use super::{DeepCaptureCa, DeepCaptureInputs, Inputs, Privilege, ProxyBackendInfo, Subsystem};

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
fn identity_fields() -> (String, Option<PathBuf>, Option<PathBuf>, Option<PathBuf>) {
    (
        env!("CARGO_PKG_VERSION").to_string(),
        std::env::current_exe().ok(),
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
/// check. Resolves the store path the same way the discovery action does, honoring
/// the `FRAGCAP_LOCAL_DB` override, so the count and the action it drives read the
/// same store (the identity section shows the default path, which may differ). A
/// store path that does not exist yet is a real empty store (zero entries),
/// reported without opening anything so the read-only probe creates no file. An
/// existing store that cannot be opened or read returns `None` (undetermined),
/// never a fabricated zero (P-9).
fn read_target_entry_count() -> Option<usize> {
    let path = crate::paths::local_db_path(None).or_else(crate::paths::default_local_db_path)?;
    if !path.exists() {
        return Some(0);
    }
    let store = fragcap::targets::Store::open(&path).ok()?;
    store.targets().ok().map(|targets| targets.len())
}

fn deep_capture_probe() -> DeepCaptureInputs {
    let session_dir = crate::paths::deep_capture_session_dir();
    let session_dir_present = session_dir.as_ref().is_some_and(|p| p.is_dir());
    let mut stale_manifests = Vec::new();
    let mut stale_tls_key_logs = Vec::new();
    let mut sensitive_artifacts = Vec::new();
    if let Some(root) = &session_dir {
        scan_deep_capture_residue(
            root,
            &mut stale_manifests,
            &mut stale_tls_key_logs,
            &mut sensitive_artifacts,
        );
    }
    let (proxy_backend, proxy_backend_error) = proxy_backend_status();
    DeepCaptureInputs {
        session_dir,
        session_dir_present,
        proxy_backend,
        proxy_backend_error,
        analyzer_keylog_configured: std::env::var_os("SSLKEYLOGFILE").is_some(),
        ca: DeepCaptureCa::Absent,
        occupied_proxy_ports: Vec::new(),
        orphaned_proxy_processes: Vec::new(),
        stale_manifests,
        stale_tls_key_logs,
        sensitive_artifacts,
    }
}

fn scan_deep_capture_residue(
    root: &std::path::Path,
    stale_manifests: &mut Vec<PathBuf>,
    stale_tls_key_logs: &mut Vec<PathBuf>,
    sensitive_artifacts: &mut Vec<PathBuf>,
) {
    fn walk(
        dir: &std::path::Path,
        depth: usize,
        seen: &mut usize,
        stale_manifests: &mut Vec<PathBuf>,
        stale_tls_key_logs: &mut Vec<PathBuf>,
        sensitive_artifacts: &mut Vec<PathBuf>,
    ) {
        if depth > 3 || *seen > 200 {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            if *seen > 200 {
                break;
            }
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            if file_type.is_dir() {
                walk(
                    &path,
                    depth + 1,
                    seen,
                    stale_manifests,
                    stale_tls_key_logs,
                    sensitive_artifacts,
                );
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            *seen += 1;
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name == "manifest.json" && manifest_cleanup_unfinished(&path) {
                stale_manifests.push(path.clone());
            }
            if name.eq_ignore_ascii_case("tls-keylog.log")
                || name.eq_ignore_ascii_case("sslkeylog.log")
            {
                stale_tls_key_logs.push(path.clone());
            }
            if matches!(
                name,
                "application.jsonl" | "http.har" | "proxy.jsonl" | "process-trace.jsonl"
            ) {
                sensitive_artifacts.push(path);
            }
        }
    }

    let mut seen = 0;
    walk(
        root,
        0,
        &mut seen,
        stale_manifests,
        stale_tls_key_logs,
        sensitive_artifacts,
    );
}

pub(crate) fn manifest_cleanup_unfinished(path: &std::path::Path) -> bool {
    let Ok(text) = std::fs::read_to_string(path) else {
        return true;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return true;
    };
    let status = value
        .get("cleanup")
        .and_then(|cleanup| cleanup.get("status"))
        .and_then(|status| status.as_str());
    !matches!(status, Some("succeeded" | "not-needed"))
}

fn proxy_backend_status() -> (Option<ProxyBackendInfo>, Option<String>) {
    let Some(path) = find_on_path("mitmdump") else {
        return (None, None);
    };
    match command_stdout_with_timeout(
        std::process::Command::new(&path)
            .arg("--version")
            .stderr(std::process::Stdio::null()),
        std::time::Duration::from_secs(2),
    ) {
        Ok((status, stdout)) if status.success() => {
            let text = String::from_utf8_lossy(&stdout);
            let first = text
                .lines()
                .find(|line| !line.trim().is_empty())
                .unwrap_or("version undetermined")
                .trim()
                .to_string();
            (
                Some(ProxyBackendInfo {
                    name: "mitmdump".to_string(),
                    version: Some(first),
                }),
                None,
            )
        }
        Ok((status, _stdout)) => (
            None,
            Some(format!(
                "mitmdump found at {} but `mitmdump --version` exited {}",
                path.display(),
                status
            )),
        ),
        Err(err) => (
            None,
            Some(format!(
                "mitmdump found at {} but version probing failed: {err}",
                path.display()
            )),
        ),
    }
}

fn command_stdout_with_timeout(
    command: &mut std::process::Command,
    timeout: std::time::Duration,
) -> Result<(std::process::ExitStatus, Vec<u8>), String> {
    let mut child = command
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|err| err.to_string())?;
    let started = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout = Vec::new();
                if let Some(mut pipe) = child.stdout.take() {
                    pipe.read_to_end(&mut stdout)
                        .map_err(|err| err.to_string())?;
                }
                return Ok((status, stdout));
            }
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("timed out after {} ms", timeout.as_millis()));
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(20)),
            Err(err) => return Err(err.to_string()),
        }
    }
}

fn find_on_path(program: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let candidates = std::env::split_paths(&path).flat_map(|dir| {
        #[cfg(windows)]
        {
            vec![dir.join(format!("{program}.exe")), dir.join(program)]
        }
        #[cfg(not(windows))]
        {
            vec![dir.join(program)]
        }
    });
    candidates.into_iter().find(|path| path.is_file())
}

/// Gather the environment facts for `doctor`.
pub fn gather() -> Inputs {
    #[cfg(windows)]
    {
        gather_windows()
    }
    #[cfg(not(windows))]
    {
        let (extcap_dir, extcap_installed, extcap_system_dir, extcap_system_installed) =
            extcap_status();
        let (fragcap_version, binary_path, catalog_db_path, local_db_path) = identity_fields();
        // wpcap.dll is not loadable on a non-Windows build; the live backend is
        // not linked anyway.
        let (interfaces, _loopback, interface_error) = live_probe(false);
        let target_entry_count = read_target_entry_count();
        let deep_capture = deep_capture_probe();
        Inputs {
            fragcap_version,
            binary_path,
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
            target_entry_count,
            deep_capture,
        }
    }
}

/// The Windows probe. Reads the filesystem for the npcap markers it can see
/// without a registry API, which is a best-effort detection the operator reads
/// as guidance; it installs nothing.
#[cfg(windows)]
fn gather_windows() -> Inputs {
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
    let (fragcap_version, binary_path, catalog_db_path, local_db_path) = identity_fields();
    let target_entry_count = read_target_entry_count();
    let deep_capture = deep_capture_probe();
    Inputs {
        fragcap_version,
        binary_path,
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
        target_entry_count,
        deep_capture,
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
