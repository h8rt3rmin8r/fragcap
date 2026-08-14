// SPDX-License-Identifier: Apache-2.0

//! Steam platform integration: library discovery, profile scaffolding, and
//! managed launch (specification section 16, roadmap slice S17).
//!
//! Contains no capture logic and no attribution logic. `fragcap-core` has no
//! notion of Steam; this crate reads Steam's local metadata and issues a launch
//! through the operating system's protocol handler, and nothing more.
//!
//! # What is portable and what is not
//!
//! Everything except two steps is portable and compiled and tested on any host:
//! the VDF parser ([`vdf`]), the whole of discovery given a root
//! ([`library::discover_in`]), the scaffolding classifier and renderer
//! ([`scaffold`]), and the launch-request decision ([`launch_request`]). Only
//! finding the Steam root through the Windows registry ([`steam_root`]) and
//! issuing the `steam://` protocol handler ([`launch`]) are `#[cfg(windows)]`;
//! the non-Windows arms return [`SteamError::UnsupportedPlatform`]. This keeps
//! the crate building on the neutral non-Windows target (P-2, FR-014).
//!
//! # It opens no process handle
//!
//! Section 16.5 (environment inheritance) would require a process handle
//! carrying memory-read rights, which the constitution's technique denylist and
//! the `OpenProcess` lint forbid. It is deferred (S17 D6): it is a corroborating
//! signal only, and section 10 ancestry already attributes reliably.

use std::fmt;
use std::path::PathBuf;

pub mod appinfo;
pub mod vdf;

mod launch;
mod library;
mod scaffold;
mod walker;

#[cfg(test)]
mod test_support;

pub use appinfo::{
    read_appinfo, read_appinfo_bytes, AppInfoApp, AppInfoFailure, AppInfoParse, AppInfoReader,
    SectionInfo, SteamLaunchEntry,
};
pub use launch::{launch, launch_request, LaunchConfigError, LaunchRequest};
pub use library::{
    discover_in, install_root_in, InstallLookup, InstalledTitle, SteamInstallation, SteamLibrary,
};
pub use scaffold::scaffold;
pub use walker::SteamWalkerProvider;

/// A failure in a Steam integration operation.
///
/// No variant carries a process handle, a captured packet, or an attribution:
/// the crate holds none of those (P-1, P-3). A malformed manifest is not an
/// error here; discovery reports and skips it (see [`SteamInstallation::warnings`]).
#[derive(Debug)]
pub enum SteamError {
    /// No Steam installation was found (registry entry or root absent).
    NotInstalled,
    /// The app_id is not installed in any discovered library.
    TitleNotFound {
        /// The app_id that was asked for.
        app_id: String,
    },
    /// A Windows-only operation was called on a non-Windows build.
    UnsupportedPlatform,
    /// A filesystem error reading Steam metadata or scanning an install
    /// directory.
    Io {
        /// The path being read when the error occurred.
        path: PathBuf,
        /// The underlying error.
        source: std::io::Error,
    },
    /// An install directory held no executable image to propose as a stage.
    NoExecutables {
        /// The install directory that was scanned.
        install_dir: PathBuf,
    },
    /// The rendered scaffold failed its own validation. This is a bug in the
    /// scaffolder (D4 makes validity hold by construction), surfaced as an error
    /// so an invalid profile is never emitted.
    Scaffold(String),
    /// The Steam protocol handler could not be invoked for a managed launch.
    LaunchFailed {
        /// The `steam://` URL that failed.
        url: String,
        /// The `ShellExecuteW` result code (32 or less means failure).
        code: i64,
    },
}

impl fmt::Display for SteamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SteamError::NotInstalled => {
                write!(f, "no Steam installation found")
            }
            SteamError::TitleNotFound { app_id } => {
                write!(f, "no installed Steam title with app_id {app_id}")
            }
            SteamError::UnsupportedPlatform => {
                write!(f, "Steam integration is only supported on Windows")
            }
            SteamError::Io { path, source } => {
                write!(f, "reading {}: {source}", path.display())
            }
            SteamError::NoExecutables { install_dir } => {
                write!(
                    f,
                    "no executable image found under {} to propose as a stage",
                    install_dir.display()
                )
            }
            SteamError::Scaffold(detail) => {
                write!(f, "scaffolded profile failed validation: {detail}")
            }
            SteamError::LaunchFailed { url, code } => {
                write!(f, "could not start {url} through Steam (code {code})")
            }
        }
    }
}

impl std::error::Error for SteamError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SteamError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Discover the Steam installation and its installed titles.
///
/// Locates the Steam root through the registry (Windows only), then reads
/// `libraryfolders.vdf` and every `appmanifest_*.acf` across every library. A
/// malformed manifest is reported in [`SteamInstallation::warnings`] and skipped
/// (FR-004). Returns [`SteamError::NotInstalled`] when no Steam root is found and
/// [`SteamError::UnsupportedPlatform`] on a non-Windows build.
pub fn discover() -> Result<SteamInstallation, SteamError> {
    let root = steam_root()?;
    library::discover_in(&root)
}

/// Look up the install directory of the installed title with `app_id`, carrying
/// any enumeration warnings.
///
/// Locates Steam through the registry (Windows only) then reads its libraries.
/// The platform walker (S030) and the capture path use this to enrich a
/// resolution request with the title's install directory, so the engine rule and
/// the walker can resolve the socket-holding client from it, and to surface the
/// enumeration warnings (FR-008). Reads the filesystem and registry only; opens no
/// process handle (P-1).
pub fn install_root_for(app_id: &str) -> Result<InstallLookup, SteamError> {
    let root = steam_root()?;
    library::install_root_in(&root, app_id)
}

/// The Steam install directory, from the Windows registry.
#[cfg(windows)]
fn steam_root() -> Result<PathBuf, SteamError> {
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

    // 32-bit Steam records its path under the 32-bit registry view; read that
    // view explicitly so the lookup works from a 64-bit process.
    if let Some(p) = read_reg_sz(HKEY_LOCAL_MACHINE, "SOFTWARE\\Valve\\Steam", "InstallPath") {
        if p.exists() {
            return Ok(p);
        }
    }
    if let Some(p) = read_reg_sz(HKEY_CURRENT_USER, "SOFTWARE\\Valve\\Steam", "SteamPath") {
        if p.exists() {
            return Ok(p);
        }
    }
    Err(SteamError::NotInstalled)
}

/// Non-Windows: there is no registry and no Steam protocol handler.
#[cfg(not(windows))]
fn steam_root() -> Result<PathBuf, SteamError> {
    Err(SteamError::UnsupportedPlatform)
}

/// Read a `REG_SZ` value, returning it as a path.
///
/// Kept small and total: any failure (missing key, missing value, wrong type)
/// yields `None`, and the caller falls back or reports `NotInstalled`. This is
/// the crate's only registry access.
#[cfg(windows)]
fn read_reg_sz(
    root: windows_sys::Win32::System::Registry::HKEY,
    subkey: &str,
    value: &str,
) -> Option<PathBuf> {
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, KEY_READ, KEY_WOW64_32KEY,
    };

    let subkey_w = to_wide(subkey);
    let value_w = to_wide(value);

    unsafe {
        let mut hkey: HKEY = 0;
        if RegOpenKeyExW(
            root,
            subkey_w.as_ptr(),
            0,
            KEY_READ | KEY_WOW64_32KEY,
            &mut hkey,
        ) != 0
        {
            return None;
        }

        let mut kind: u32 = 0;
        let mut len: u32 = 0;
        let sized = RegQueryValueExW(
            hkey,
            value_w.as_ptr(),
            std::ptr::null_mut(),
            &mut kind,
            std::ptr::null_mut(),
            &mut len,
        );
        if sized != 0 || len == 0 {
            RegCloseKey(hkey);
            return None;
        }

        let mut buf = vec![0u8; len as usize];
        let read = RegQueryValueExW(
            hkey,
            value_w.as_ptr(),
            std::ptr::null_mut(),
            &mut kind,
            buf.as_mut_ptr(),
            &mut len,
        );
        RegCloseKey(hkey);
        if read != 0 {
            return None;
        }

        let u16s: Vec<u16> = buf[..len as usize]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let s = String::from_utf16_lossy(&u16s);
        let s = s.trim_end_matches('\0');
        if s.is_empty() {
            None
        } else {
            Some(PathBuf::from(s))
        }
    }
}

/// A NUL-terminated UTF-16 string for a wide Win32 call.
#[cfg(windows)]
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
