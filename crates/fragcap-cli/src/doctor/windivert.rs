// SPDX-License-Identifier: Apache-2.0

//! Read-only identity check for the packaged WinDivert component.

use std::path::Path;

#[cfg(test)]
use std::fs;

use super::Check;

/// Inspect files beside the executable. Exact hashes include the official signed
/// driver, but no driver handle is opened and load-time policy is not inferred.
pub fn check() -> Check {
    if !cfg!(windows) {
        return Check::skip(
            "Deep Capture",
            "WinDivert component",
            "Windows package only",
        );
    }
    match std::env::current_exe() {
        Ok(exe) => match exe.parent() {
            Some(parent) => check_directory(parent),
            None => Check::warn(
                "Deep Capture",
                "WinDivert component",
                "executable directory unavailable",
            ),
        },
        Err(error) => Check::warn(
            "Deep Capture",
            "WinDivert component",
            format!("executable path unavailable: {error}"),
        ),
    }
}

fn check_directory(directory: &Path) -> Check {
    if let Err(detail) = fragcap::deep_capture::verify_windivert_component(directory) {
        return Check::warn("Deep Capture", "WinDivert component", detail);
    }
    if service_start_disabled() {
        return Check::warn(
            "Deep Capture",
            "WinDivert component",
            "WinDivert service is disabled; ask an administrator to inspect the service before a direct-route session",
        );
    }
    Check::ok(
        "Deep Capture",
        "WinDivert component",
        "certified 2.2.2-A files present; driver load and policy compatibility unverified",
    )
}

#[cfg(windows)]
fn service_start_disabled() -> bool {
    use windows_sys::Win32::System::Registry::{
        RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD,
    };

    let key: Vec<u16> = "SYSTEM\\CurrentControlSet\\Services\\WinDivert\0"
        .encode_utf16()
        .collect();
    let value: Vec<u16> = "Start\0".encode_utf16().collect();
    let mut start = 0u32;
    let mut bytes = std::mem::size_of::<u32>() as u32;
    // SAFETY: the key/value strings are terminated; `start` is a live four-byte
    // output buffer and `bytes` advertises its exact size. This only reads the
    // service configuration and never opens or starts the driver.
    let status = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            key.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&raw mut start).cast(),
            &mut bytes,
        )
    };
    status == 0 && bytes == 4 && start == 4
}

#[cfg(not(windows))]
fn service_start_disabled() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_and_altered_files_are_not_ready() {
        let directory = tempfile::tempdir().unwrap();
        let missing = check_directory(directory.path());
        assert!(missing.detail.contains("WinDivert.dll missing"));
        fs::write(directory.path().join("WinDivert.dll"), b"altered").unwrap();
        let altered = check_directory(directory.path());
        assert!(altered
            .detail
            .contains("WinDivert.dll has no x64 PE header"));
    }
}
