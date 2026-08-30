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
use std::time::{Duration, Instant};

#[cfg(any(test, all(feature = "live", windows)))]
use fragcap::core::{is_loopback_adapter, InterfaceInventory, SourceError};

use super::progress::ProbeName;
use super::{DeepCaptureCa, DeepCaptureInputs, Inputs, Privilege, ProxyBackendInfo, Subsystem};

/// Receives progress events around doctor probe groups.
pub trait ProbeObserver {
    /// A probe has started.
    fn begin(&mut self, probe: ProbeName);

    /// A probe has completed after `elapsed`.
    fn complete(&mut self, probe: ProbeName, elapsed: Duration);
}

/// An observer that keeps the existing silent behavior.
pub struct NoopObserver;

impl ProbeObserver for NoopObserver {
    fn begin(&mut self, _probe: ProbeName) {}

    fn complete(&mut self, _probe: ProbeName, _elapsed: Duration) {}
}

fn observe<T>(observer: &mut dyn ProbeObserver, probe: ProbeName, work: impl FnOnce() -> T) -> T {
    observer.begin(probe);
    let started = Instant::now();
    let value = work();
    observer.complete(probe, started.elapsed());
    value
}

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
        tracing_availability_with(|| {
            fragcap::EtwWatcher::probe_session("fragcap-doctor-probe").is_ok()
        })
    }
    #[cfg(not(all(windows, feature = "etw")))]
    {
        None
    }
}

#[cfg(any(test, all(windows, feature = "etw")))]
fn tracing_availability_with(probe: impl FnOnce() -> bool) -> Option<bool> {
    Some(probe())
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
        live_probe_with(wpcap_loadable, fragcap::enumerate)
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

#[cfg(any(test, all(feature = "live", windows)))]
fn live_probe_with(
    wpcap_loadable: bool,
    enumerate: impl FnOnce() -> Result<InterfaceInventory, SourceError>,
) -> (Vec<super::IfaceInfo>, Option<bool>, Option<String>) {
    if !wpcap_loadable {
        return (Vec::new(), None, None);
    }

    // A failed enumeration is preserved as an error string rather than
    // flattened to an empty set, so the classifier does not report a probe that
    // could not run as an observed-empty machine (P-9). Loopback stays unknown
    // because no absence was observed.
    let inventory = match enumerate() {
        Ok(inventory) => inventory,
        Err(err) => return (Vec::new(), None, Some(err.to_string())),
    };

    let loopback = Some(
        inventory
            .interfaces
            .iter()
            .any(|record| is_loopback_adapter(record.is_loopback, record.description.as_deref())),
    );
    let interfaces = inventory
        .interfaces
        .iter()
        .map(|record| super::IfaceInfo {
            name: record.name.to_string(),
            addr: record.addresses.first().map(|addr| addr.to_string()),
            up: record.is_up,
            is_virtual: fragcap::core::virtual_verdict(record).is_virtual(),
        })
        .collect();

    (interfaces, loopback, None)
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
    let scan = scan_deep_capture_root(session_dir.as_deref());
    let (proxy_backend, proxy_backend_error) = proxy_backend_status();
    let ca = if scan.errors.is_empty() {
        probe_ca(&scan.manifests)
    } else {
        DeepCaptureCa::Unknown(scan.errors.join("; "))
    };
    DeepCaptureInputs {
        session_dir,
        session_dir_present,
        proxy_backend,
        proxy_backend_error,
        analyzer_keylog_configured: std::env::var_os("SSLKEYLOGFILE").is_some(),
        ca,
        occupied_proxy_ports: None,
        orphaned_proxy_processes: None,
        stale_manifests: scan.stale_manifests,
        stale_tls_key_logs: scan.stale_tls_key_logs,
        sensitive_artifacts: scan.sensitive_artifacts,
    }
}

fn scan_deep_capture_root(root: Option<&std::path::Path>) -> DeepCaptureScan {
    root.filter(|path| path.is_dir())
        .map(scan_deep_capture_residue)
        .unwrap_or_default()
}

#[derive(Default)]
struct DeepCaptureScan {
    stale_manifests: Vec<PathBuf>,
    stale_tls_key_logs: Vec<PathBuf>,
    sensitive_artifacts: Vec<PathBuf>,
    manifests: Vec<PathBuf>,
    errors: Vec<String>,
}

fn scan_deep_capture_residue(root: &std::path::Path) -> DeepCaptureScan {
    struct ScanState<'a> {
        stale_manifests: &'a mut Vec<PathBuf>,
        stale_tls_key_logs: &'a mut Vec<PathBuf>,
        sensitive_artifacts: &'a mut Vec<PathBuf>,
        manifests: &'a mut Vec<PathBuf>,
        errors: &'a mut Vec<String>,
    }

    fn walk(dir: &std::path::Path, depth: usize, visited: &mut usize, state: &mut ScanState<'_>) {
        if depth > 3 {
            push_scan_error(
                state.errors,
                format!("session scan reached the depth limit at {}", dir.display()),
            );
            return;
        }
        if *visited >= 200 {
            push_scan_error(
                state.errors,
                "session scan reached the 200-entry limit".to_string(),
            );
            return;
        }
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(err) => {
                state
                    .errors
                    .push(format!("could not read {}: {err}", dir.display()));
                return;
            }
        };
        for entry in entries {
            if *visited >= 200 {
                push_scan_error(
                    state.errors,
                    "session scan reached the 200-entry limit".to_string(),
                );
                break;
            }
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    state.errors.push(format!(
                        "could not read an entry in {}: {err}",
                        dir.display()
                    ));
                    continue;
                }
            };
            *visited += 1;
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(err) => {
                    push_scan_error(
                        state.errors,
                        format!("could not inspect {}: {err}", entry.path().display()),
                    );
                    continue;
                }
            };
            let path = entry.path();
            if file_type.is_dir() {
                walk(&path, depth + 1, visited, state);
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name == "manifest.json" {
                push_unique(state.manifests, path.clone());
                manifest_declared_artifacts(
                    &path,
                    state.stale_tls_key_logs,
                    state.sensitive_artifacts,
                );
                if manifest_cleanup_unfinished(&path) {
                    push_unique(state.stale_manifests, path.clone());
                }
            }
            if name.eq_ignore_ascii_case("tls-keylog.log")
                || name.eq_ignore_ascii_case("sslkeylog.log")
            {
                push_unique(state.stale_tls_key_logs, path.clone());
            }
            if matches!(
                name,
                "application.jsonl" | "http.har" | "proxy.jsonl" | "process-trace.jsonl"
            ) {
                push_unique(state.sensitive_artifacts, path);
            }
        }
    }

    fn push_scan_error(errors: &mut Vec<String>, error: String) {
        if !errors.contains(&error) {
            errors.push(error);
        }
    }

    let mut scan = DeepCaptureScan::default();
    let mut state = ScanState {
        stale_manifests: &mut scan.stale_manifests,
        stale_tls_key_logs: &mut scan.stale_tls_key_logs,
        sensitive_artifacts: &mut scan.sensitive_artifacts,
        manifests: &mut scan.manifests,
        errors: &mut scan.errors,
    };
    let mut visited = 0;
    walk(root, 0, &mut visited, &mut state);
    scan
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OwnedCaIdentity {
    recorded: String,
    material: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg(any(test, windows))]
struct CaInventory {
    current_user_root: Vec<String>,
    local_machine_root: Vec<String>,
}

fn normalize_thumbprint(value: &str) -> Option<String> {
    let normalized: String = value
        .chars()
        .filter(|ch| !matches!(ch, ':' | '-' | ' ' | '\t' | '\r' | '\n'))
        .map(|ch| ch.to_ascii_uppercase())
        .collect();
    (normalized.len() == 40 && normalized.chars().all(|ch| ch.is_ascii_hexdigit()))
        .then_some(normalized)
}

fn manifest_ca_identities(manifests: &[PathBuf]) -> Result<Vec<OwnedCaIdentity>, String> {
    let mut identities = Vec::new();
    for manifest in manifests {
        let text = std::fs::read_to_string(manifest)
            .map_err(|err| format!("could not read {}: {err}", manifest.display()))?;
        let value: serde_json::Value = serde_json::from_str(&text)
            .map_err(|err| format!("could not parse {}: {err}", manifest.display()))?;
        let Some(raw) = value
            .get("trust")
            .and_then(|trust| trust.get("thumbprint"))
            .and_then(|thumbprint| thumbprint.as_str())
        else {
            continue;
        };
        let recorded = normalize_thumbprint(raw)
            .ok_or_else(|| format!("{} contains an invalid CA thumbprint", manifest.display()))?;
        let material = None;
        let identity = OwnedCaIdentity { recorded, material };
        if let Some(existing) = identities
            .iter_mut()
            .find(|existing: &&mut OwnedCaIdentity| existing.recorded == identity.recorded)
        {
            match (&existing.material, &identity.material) {
                (Some(left), Some(right)) if left != right => {
                    return Err(format!(
                        "multiple bundles for {} contain conflicting CA material",
                        identity.recorded
                    ));
                }
                (None, Some(_)) => existing.material = identity.material,
                _ => {}
            }
        } else {
            identities.push(identity);
        }
    }
    Ok(identities)
}

#[cfg(any(test, windows))]
fn classify_ca(identities: &[OwnedCaIdentity], inventory: &CaInventory) -> DeepCaptureCa {
    let mut findings = Vec::new();
    for identity in identities {
        if let Some(material) = &identity.material {
            if material != &identity.recorded {
                return DeepCaptureCa::Mismatched {
                    expected: identity.recorded.clone(),
                    actual: material.clone(),
                    store: observed_store(material, inventory)
                        .or_else(|| observed_store(&identity.recorded, inventory)),
                };
            }
        }
        if inventory.current_user_root.contains(&identity.recorded) {
            findings.push(DeepCaptureCa::CurrentUser {
                thumbprint: identity.recorded.clone(),
            });
        }
        if inventory.local_machine_root.contains(&identity.recorded) {
            findings.push(DeepCaptureCa::WrongStore {
                store: "LocalMachine/Root".to_string(),
                thumbprint: identity.recorded.clone(),
            });
        }
    }
    match findings.len() {
        0 => DeepCaptureCa::Absent,
        1 => findings.pop().expect("one finding"),
        count => DeepCaptureCa::Unknown(format!(
            "multiple ({count}) fragcap-owned CA trust entries were observed"
        )),
    }
}

#[cfg(any(test, windows))]
fn observed_store(thumbprint: &str, inventory: &CaInventory) -> Option<String> {
    let current = inventory
        .current_user_root
        .iter()
        .any(|item| item == thumbprint);
    let machine = inventory
        .local_machine_root
        .iter()
        .any(|item| item == thumbprint);
    match (current, machine) {
        (true, false) => Some("CurrentUser/Root".to_string()),
        (false, true) => Some("LocalMachine/Root".to_string()),
        _ => None,
    }
}

#[cfg(any(test, windows))]
fn cleanup_targets(
    identities: &[OwnedCaIdentity],
    inventory: &CaInventory,
) -> Vec<(String, String)> {
    let mut targets = Vec::new();
    for identity in identities {
        let mut thumbprints = vec![identity.recorded.as_str()];
        if let Some(material) = identity.material.as_deref() {
            if material != identity.recorded {
                thumbprints.push(material);
            }
        }
        for thumbprint in thumbprints {
            if inventory
                .current_user_root
                .iter()
                .any(|item| item == thumbprint)
            {
                let target = ("CurrentUser/Root".to_string(), thumbprint.to_string());
                if !targets.contains(&target) {
                    targets.push(target);
                }
            }
            if inventory
                .local_machine_root
                .iter()
                .any(|item| item == thumbprint)
            {
                let target = ("LocalMachine/Root".to_string(), thumbprint.to_string());
                if !targets.contains(&target) {
                    targets.push(target);
                }
            }
        }
    }
    targets
}

#[cfg(windows)]
fn read_ca_inventory() -> Result<CaInventory, String> {
    Ok(CaInventory {
        current_user_root: crate::windows_cert::store_thumbprints(
            crate::windows_cert::CURRENT_USER_ROOT,
        )?,
        local_machine_root: crate::windows_cert::store_thumbprints(
            crate::windows_cert::LOCAL_MACHINE_ROOT,
        )?,
    })
}

#[cfg(windows)]
fn probe_ca(manifests: &[PathBuf]) -> DeepCaptureCa {
    let identities = match manifest_ca_identities(manifests) {
        Ok(identities) => identities,
        Err(reason) => return DeepCaptureCa::Unknown(reason),
    };
    if identities.is_empty() {
        return DeepCaptureCa::Absent;
    }
    match read_ca_inventory() {
        Ok(inventory) => classify_ca(&identities, &inventory),
        Err(reason) => DeepCaptureCa::Unknown(reason),
    }
}

#[cfg(not(windows))]
fn probe_ca(manifests: &[PathBuf]) -> DeepCaptureCa {
    match manifest_ca_identities(manifests) {
        Err(reason) => DeepCaptureCa::Unknown(reason),
        Ok(identities) if identities.is_empty() => DeepCaptureCa::Absent,
        Ok(_) => DeepCaptureCa::Unknown(
            "Windows certificate stores are unavailable on this platform".to_string(),
        ),
    }
}

#[cfg(windows)]
pub(crate) fn ca_cleanup_targets(root: &std::path::Path) -> Result<Vec<(String, String)>, String> {
    let scan = scan_deep_capture_residue(root);
    if !scan.errors.is_empty() {
        return Err(scan.errors.join("; "));
    }
    let identities = manifest_ca_identities(&scan.manifests)?;
    if identities.is_empty() {
        return Ok(Vec::new());
    }
    let inventory = read_ca_inventory()?;
    Ok(cleanup_targets(&identities, &inventory))
}

#[cfg(not(windows))]
pub(crate) fn ca_cleanup_targets(root: &std::path::Path) -> Result<Vec<(String, String)>, String> {
    let scan = scan_deep_capture_residue(root);
    if !scan.errors.is_empty() {
        return Err(scan.errors.join("; "));
    }
    if manifest_ca_identities(&scan.manifests)?.is_empty() {
        return Ok(Vec::new());
    }
    Err("Windows certificate stores are unavailable on this platform".to_string())
}

pub(crate) fn manifest_declared_artifacts(
    manifest: &std::path::Path,
    stale_tls_key_logs: &mut Vec<PathBuf>,
    sensitive_artifacts: &mut Vec<PathBuf>,
) {
    let Ok(text) = std::fs::read_to_string(manifest) else {
        return;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return;
    };
    let Some(artifacts) = value
        .get("artifacts")
        .and_then(|artifacts| artifacts.as_array())
    else {
        return;
    };
    let bundle_root = manifest
        .parent()
        .unwrap_or_else(|| std::path::Path::new(""));
    for artifact in artifacts {
        let role = artifact.get("role").and_then(|role| role.as_str());
        let path = artifact.get("path").and_then(|path| path.as_str());
        let Some(path) = path.and_then(safe_manifest_relative_path) else {
            continue;
        };
        let resolved = bundle_root.join(path);
        if !resolved.is_file() {
            continue;
        }
        match role {
            Some("tls-key-log") => push_unique(stale_tls_key_logs, resolved),
            Some("application-jsonl" | "har" | "proxy-log" | "process-trace") => {
                push_unique(sensitive_artifacts, resolved);
            }
            _ => {}
        }
    }
}

pub(crate) fn manifest_declared_cleanup_paths(manifest: &std::path::Path) -> Vec<PathBuf> {
    let mut tls = Vec::new();
    let mut sensitive = Vec::new();
    manifest_declared_artifacts(manifest, &mut tls, &mut sensitive);
    tls.extend(sensitive);
    tls
}

fn safe_manifest_relative_path(path: &str) -> Option<PathBuf> {
    let path = std::path::Path::new(path);
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(part) => out.push(part),
            std::path::Component::CurDir => {}
            _ => return None,
        }
    }
    if out.as_os_str().is_empty() {
        None
    } else {
        Some(out)
    }
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

pub(crate) fn push_unique(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

fn proxy_backend_status() -> (Option<ProxyBackendInfo>, Option<String>) {
    (
        Some(ProxyBackendInfo {
            name: "fragcap-native".to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }),
        None,
    )
}

/// Gather the environment facts for `doctor`.
pub fn gather() -> Inputs {
    let mut observer = NoopObserver;
    gather_with(&mut observer)
}

/// Gather the environment facts for `doctor`, reporting progress to `observer`.
pub fn gather_with(observer: &mut dyn ProbeObserver) -> Inputs {
    #[cfg(windows)]
    {
        gather_windows(observer)
    }
    #[cfg(not(windows))]
    {
        let (extcap_dir, extcap_installed, extcap_system_dir, extcap_system_installed) =
            observe(observer, ProbeName::AnalyzerIntegration, extcap_status);
        let (fragcap_version, binary_path, catalog_db_path, local_db_path) =
            observe(observer, ProbeName::Identity, identity_fields);
        // wpcap.dll is not loadable on a non-Windows build; the live backend is
        // not linked anyway.
        let (interfaces, _loopback, interface_error) =
            observe(observer, ProbeName::CaptureDriverInterfaces, || {
                live_probe(false)
            });
        let target_entry_count =
            observe(observer, ProbeName::TargetStores, read_target_entry_count);
        let deep_capture = observe(
            observer,
            ProbeName::DeepCaptureReadiness,
            deep_capture_probe,
        );
        let etw_available = observe(
            observer,
            ProbeName::ProcessEventTracing,
            tracing_availability,
        );
        let (os, subsystem, privilege) = observe(observer, ProbeName::Platform, || {
            (
                format!("{} (capture is Windows-only)", std::env::consts::OS),
                Subsystem::Native,
                Privilege::NotElevated,
            )
        });
        Inputs {
            fragcap_version,
            binary_path,
            catalog_db_present: catalog_db_path.as_ref().is_some_and(|p| p.exists()),
            catalog_db_path,
            local_db_present: local_db_path.as_ref().is_some_and(|p| p.exists()),
            local_db_path,
            os,
            subsystem,
            privilege,
            npcap: None,
            etw_available,
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
fn gather_windows(observer: &mut dyn ProbeObserver) -> Inputs {
    use super::NpcapInfo;

    let (system32, npcap_wpcap, system_wpcap, npcap_present, privilege) =
        observe(observer, ProbeName::Platform, || {
            let system_root =
                std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
            let system32 = PathBuf::from(&system_root).join("System32");
            let npcap_dir = system32.join("Npcap");
            let npcap_wpcap = npcap_dir.join("wpcap.dll");
            let system_wpcap = system32.join("wpcap.dll");

            // npcap is present when its own wpcap.dll exists in the Npcap directory; this
            // drives the installation and version report below.
            let npcap_present = npcap_wpcap.exists();
            let privilege = if is_elevated() {
                Privilege::Elevated
            } else {
                Privilege::NotElevated
            };
            (
                system32,
                npcap_wpcap,
                system_wpcap,
                npcap_present,
                privilege,
            )
        });
    let _ = system32;

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
    let (interfaces, loopback_supported, interface_error) =
        observe(observer, ProbeName::CaptureDriverInterfaces, || {
            live_probe(wpcap_loadable)
        });

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
        observe(observer, ProbeName::AnalyzerIntegration, extcap_status);
    let (fragcap_version, binary_path, catalog_db_path, local_db_path) =
        observe(observer, ProbeName::Identity, identity_fields);
    let target_entry_count = observe(observer, ProbeName::TargetStores, read_target_entry_count);
    let deep_capture = observe(
        observer,
        ProbeName::DeepCaptureReadiness,
        deep_capture_probe,
    );
    let etw_available = observe(
        observer,
        ProbeName::ProcessEventTracing,
        tracing_availability,
    );
    Inputs {
        fragcap_version,
        binary_path,
        catalog_db_present: catalog_db_path.as_ref().is_some_and(|p| p.exists()),
        catalog_db_path,
        local_db_present: local_db_path.as_ref().is_some_and(|p| p.exists()),
        local_db_path,
        os: "Windows".to_string(),
        subsystem: Subsystem::Native,
        privilege,
        npcap,
        etw_available,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::net::IpAddr;
    use std::sync::Arc;

    use fragcap::core::{InterfaceRecord, LinkType};

    fn test_inventory(records: Vec<InterfaceRecord>) -> InterfaceInventory {
        InterfaceInventory {
            interfaces: records,
            default_route_source: None,
        }
    }

    fn test_record(name: &str) -> InterfaceRecord {
        InterfaceRecord {
            addresses: vec![IpAddr::from([192, 0, 2, 10])],
            is_up: true,
            is_running: true,
            ..InterfaceRecord::new(name, LinkType::ETHERNET)
        }
    }

    #[test]
    fn observe_reports_begin_before_work_completes() {
        struct RecordingObserver {
            started: Instant,
            events: Vec<(&'static str, Duration)>,
        }

        impl ProbeObserver for RecordingObserver {
            fn begin(&mut self, _probe: ProbeName) {
                self.events.push(("begin", self.started.elapsed()));
            }

            fn complete(&mut self, _probe: ProbeName, elapsed: Duration) {
                self.events.push(("complete", elapsed));
            }
        }

        let mut observer = RecordingObserver {
            started: Instant::now(),
            events: Vec::new(),
        };
        let value = observe(&mut observer, ProbeName::Identity, || {
            std::thread::sleep(Duration::from_millis(50));
            7
        });

        assert_eq!(value, 7);
        assert_eq!(observer.events.len(), 2);
        assert_eq!(observer.events[0].0, "begin");
        assert!(
            observer.events[0].1 < Duration::from_millis(20),
            "begin was delayed until {:?}",
            observer.events[0].1
        );
        assert_eq!(observer.events[1].0, "complete");
        assert!(
            observer.events[1].1 >= Duration::from_millis(50),
            "completion did not include slow work: {:?}",
            observer.events[1].1
        );
    }

    #[test]
    fn live_probe_uses_one_enumeration_for_interfaces_and_loopback() {
        let calls = Cell::new(0);
        let (interfaces, loopback, error) = live_probe_with(true, || {
            calls.set(calls.get() + 1);
            let mut record = test_record("NPF_Loopback");
            record.is_loopback = true;
            Ok(test_inventory(vec![record]))
        });

        assert_eq!(calls.get(), 1);
        assert_eq!(interfaces.len(), 1);
        assert_eq!(interfaces[0].name, "NPF_Loopback");
        assert_eq!(loopback, Some(true));
        assert_eq!(error, None);
    }

    #[test]
    fn live_probe_accepts_loopback_description_marker() {
        let mut record = test_record("NPF_{1234}");
        record.description = Some(Arc::from("Npcap Loopback Adapter"));

        let (_, loopback, error) = live_probe_with(true, || Ok(test_inventory(vec![record])));

        assert_eq!(loopback, Some(true));
        assert_eq!(error, None);
    }

    #[test]
    fn live_probe_reports_observed_loopback_absence_only_after_success() {
        let record = test_record("NPF_{ETH}");

        let (interfaces, loopback, error) =
            live_probe_with(true, || Ok(test_inventory(vec![record])));

        assert_eq!(interfaces.len(), 1);
        assert_eq!(loopback, Some(false));
        assert_eq!(error, None);
    }

    #[test]
    fn live_probe_keeps_loopback_unknown_when_enumeration_fails() {
        let (interfaces, loopback, error) = live_probe_with(true, || {
            Err(SourceError::Backend {
                detail: "interface enumeration failed: boom".to_string(),
            })
        });

        assert_eq!(interfaces, Vec::<super::super::IfaceInfo>::new());
        assert_eq!(loopback, None);
        assert_eq!(
            error,
            Some("capture backend failure: interface enumeration failed: boom".to_string())
        );
    }

    #[test]
    fn live_probe_does_not_enumerate_when_wpcap_is_not_loadable() {
        let calls = Cell::new(0);
        let (interfaces, loopback, error) = live_probe_with(false, || {
            calls.set(calls.get() + 1);
            Ok(test_inventory(Vec::new()))
        });

        assert_eq!(calls.get(), 0);
        assert_eq!(interfaces, Vec::<super::super::IfaceInfo>::new());
        assert_eq!(loopback, None);
        assert_eq!(error, None);
    }

    #[test]
    fn tracing_availability_maps_probe_success_and_failure() {
        assert_eq!(tracing_availability_with(|| true), Some(true));
        assert_eq!(tracing_availability_with(|| false), Some(false));
    }

    #[test]
    fn tracing_availability_does_not_start_the_full_watcher() {
        let source = include_str!("probe.rs");
        let full_watcher_probe_call = concat!("EtwWatcher::", "start(\"fragcap-doctor-probe\")");

        assert!(
            !source.contains(full_watcher_probe_call),
            "doctor readiness must use the probe-only ETW entry point"
        );
        assert!(source.contains("EtwWatcher::probe_session(\"fragcap-doctor-probe\")"));
    }

    #[test]
    fn manifest_declared_artifacts_use_manifest_paths_and_reject_parent_paths() {
        let dir = tempfile::tempdir().expect("tempdir");
        let bundle = dir.path().join("bundle");
        std::fs::create_dir_all(bundle.join("logs")).expect("bundle dirs");
        let tls = bundle.join("logs").join("proxy.keys");
        let app = bundle.join("logs").join("app.events");
        let outside = dir.path().join("outside.har");
        std::fs::write(&tls, "keys").expect("tls");
        std::fs::write(&app, "{}").expect("app");
        std::fs::write(&outside, "{}").expect("outside");
        let manifest = bundle.join("manifest.json");
        std::fs::write(
            &manifest,
            r#"{"artifacts":[{"role":"tls-key-log","path":"logs/proxy.keys"},{"role":"application-jsonl","path":"logs/app.events"},{"role":"har","path":"../outside.har"}]}"#,
        )
        .expect("manifest");

        let mut tls_paths = Vec::new();
        let mut sensitive = Vec::new();
        manifest_declared_artifacts(&manifest, &mut tls_paths, &mut sensitive);

        assert_eq!(tls_paths, vec![tls]);
        assert_eq!(sensitive, vec![app]);
    }

    fn identity(recorded: &str, material: Option<&str>) -> OwnedCaIdentity {
        OwnedCaIdentity {
            recorded: recorded.to_string(),
            material: material.map(str::to_string),
        }
    }

    fn inventory(current_user: &[&str], local_machine: &[&str]) -> CaInventory {
        CaInventory {
            current_user_root: current_user
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            local_machine_root: local_machine
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        }
    }

    #[test]
    fn ca_classifier_covers_absent_supported_wrong_store_and_unrelated() {
        let owned = "00112233445566778899AABBCCDDEEFF00112233";
        let unrelated = "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";

        assert_eq!(
            classify_ca(
                &[identity(owned, Some(owned))],
                &inventory(&[unrelated], &[])
            ),
            DeepCaptureCa::Absent
        );
        assert_eq!(
            classify_ca(&[identity(owned, Some(owned))], &inventory(&[owned], &[])),
            DeepCaptureCa::CurrentUser {
                thumbprint: owned.to_string()
            }
        );
        assert_eq!(
            classify_ca(&[identity(owned, Some(owned))], &inventory(&[], &[owned])),
            DeepCaptureCa::WrongStore {
                store: "LocalMachine/Root".to_string(),
                thumbprint: owned.to_string()
            }
        );
    }

    #[test]
    fn ca_classifier_reports_material_mismatch_and_ambiguity() {
        let recorded = "00112233445566778899AABBCCDDEEFF00112233";
        let material = "112233445566778899AABBCCDDEEFF0011223344";
        assert_eq!(
            classify_ca(
                &[identity(recorded, Some(material))],
                &inventory(&[material], &[])
            ),
            DeepCaptureCa::Mismatched {
                expected: recorded.to_string(),
                actual: material.to_string(),
                store: Some("CurrentUser/Root".to_string())
            }
        );

        let second = "2233445566778899AABBCCDDEEFF001122334455";
        assert!(matches!(
            classify_ca(
                &[identity(recorded, None), identity(second, None)],
                &inventory(&[recorded, second], &[])
            ),
            DeepCaptureCa::Unknown(reason) if reason.contains("multiple")
        ));
    }

    #[test]
    fn thumbprint_normalization_is_strict_and_canonical() {
        assert_eq!(
            normalize_thumbprint("00:11 22-33 44 55 66 77 88 99 aa bb cc dd ee ff 00 11 22 33"),
            Some("00112233445566778899AABBCCDDEEFF00112233".to_string())
        );
        assert_eq!(normalize_thumbprint("controlled-thumbprint"), None);
        assert_eq!(normalize_thumbprint("0011"), None);
    }

    #[test]
    fn manifest_identities_are_exact_and_deduplicated() {
        let dir = tempfile::tempdir().expect("tempdir");
        let first = dir.path().join("one.json");
        let second = dir.path().join("two.json");
        std::fs::write(
            &first,
            r#"{"trust":{"thumbprint":"00:11:22:33:44:55:66:77:88:99:aa:bb:cc:dd:ee:ff:00:11:22:33"}}"#,
        )
        .expect("first manifest");
        std::fs::write(
            &second,
            r#"{"trust":{"thumbprint":"00112233445566778899AABBCCDDEEFF00112233"}}"#,
        )
        .expect("second manifest");

        assert_eq!(
            manifest_ca_identities(&[first, second]).expect("identities"),
            vec![identity("00112233445566778899AABBCCDDEEFF00112233", None)]
        );
    }

    #[test]
    fn an_absent_session_root_is_an_empty_scan() {
        let dir = tempfile::tempdir().expect("tempdir");
        let scan = scan_deep_capture_root(Some(&dir.path().join("never-created")));

        assert!(scan.manifests.is_empty());
        assert!(scan.errors.is_empty());
    }

    #[test]
    fn a_truncated_session_scan_is_incomplete() {
        let dir = tempfile::tempdir().expect("tempdir");
        for index in 0..201 {
            std::fs::write(dir.path().join(format!("artifact-{index:03}")), "x").expect("artifact");
        }

        let scan = scan_deep_capture_residue(dir.path());

        assert!(scan
            .errors
            .iter()
            .any(|error| error.contains("200-entry limit")));
    }

    #[test]
    fn cleanup_targets_include_every_exact_observed_owned_entry() {
        let first = "00112233445566778899AABBCCDDEEFF00112233";
        let second = "112233445566778899AABBCCDDEEFF0011223344";
        let unrelated = "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";
        let targets = cleanup_targets(
            &[identity(first, None), identity(second, None)],
            &inventory(&[first, unrelated], &[second]),
        );

        assert_eq!(
            targets,
            vec![
                ("CurrentUser/Root".to_string(), first.to_string()),
                ("LocalMachine/Root".to_string(), second.to_string()),
            ]
        );
    }
}
