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

/// The analyzer extcap directory and whether a fragcap binary is installed in
/// it, read-only.
///
/// Detection only: this reads the directory to see whether the binary has been
/// copied there and installs, downloads, and copies nothing, which is the
/// Licensing rule and constitution P-1 made mechanical (specification 14.5).
fn extcap_status() -> (Option<PathBuf>, bool) {
    let dir = crate::paths::extcap_dir();
    let installed = dir
        .as_ref()
        .map(|d| d.join(EXTCAP_BINARY).exists())
        .unwrap_or(false);
    (dir, installed)
}

/// The fragcap binary name in the analyzer's extcap directory.
#[cfg(windows)]
const EXTCAP_BINARY: &str = "fragcap.exe";
#[cfg(not(windows))]
const EXTCAP_BINARY: &str = "fragcap";

/// Count the `.toml` profiles directly in a directory, or zero when it cannot
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
        .filter(|e| e.path().extension().is_some_and(|x| x == "toml"))
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
        let (extcap_dir, extcap_installed) = extcap_status();
        Inputs {
            os: format!("{} (capture is Windows-only)", std::env::consts::OS),
            subsystem: Subsystem::Native,
            privilege: Privilege::NotElevated,
            npcap: None,
            etw_available: tracing_availability(),
            interfaces: Vec::new(),
            extcap_installed,
            extcap_dir,
            bundled_count: crate::paths::bundled().len(),
            user_count,
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

    // npcap is present when its own wpcap.dll exists in the Npcap directory. The
    // WinPcap API compatibility option additionally installs wpcap.dll directly
    // into System32, so its presence there is the signal for that option.
    let npcap = if npcap_wpcap.exists() {
        Some(NpcapInfo {
            version: "installed".to_string(),
            // Loopback support installs an "NPF_Loopback" adapter; without an
            // adapter enumeration this probe cannot see it, so it reports the
            // conservative answer and the operator confirms.
            loopback_adapter: npcap_dir.join("npcap_wifi.sys").exists(),
            winpcap_api_mode: system_wpcap.exists(),
        })
    } else {
        None
    };

    let (extcap_dir, extcap_installed) = extcap_status();
    Inputs {
        os: "Windows".to_string(),
        subsystem: Subsystem::Native,
        privilege: if is_elevated() {
            Privilege::Elevated
        } else {
            Privilege::NotElevated
        },
        npcap,
        etw_available: tracing_availability(),
        // Interface enumeration belongs to the capture backend, which is not
        // linked here; the check warns rather than fails on an empty set.
        interfaces: Vec::new(),
        extcap_installed,
        extcap_dir,
        bundled_count: crate::paths::bundled().len(),
        user_count,
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
fn is_elevated() -> bool {
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
