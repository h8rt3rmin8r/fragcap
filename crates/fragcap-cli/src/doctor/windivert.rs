// SPDX-License-Identifier: Apache-2.0

//! Read-only identity check for the packaged WinDivert component.

use std::fs;
use std::path::Path;

use ring::digest::{digest, SHA256};

use super::Check;

const FILES: [(&str, &str); 4] = [
    (
        "WinDivert.dll",
        "c1e060ee19444a259b2162f8af0f3fe8c4428a1c6f694dce20de194ac8d7d9a2",
    ),
    (
        "WinDivert64.sys",
        "8da085332782708d8767bcace5327a6ec7283c17cfb85e40b03cd2323a90ddc2",
    ),
    (
        "WinDivert-LICENSE.txt",
        "14a0cb5214d536e4fdae6aa3f5696f981eeda106cd026e9794bba489ee79d628",
    ),
    (
        "WinDivert-source.zip",
        "65ec79c9e6afa99f648a3f4d1f6db794640b40d0b65bd438770ea503ee14ecb7",
    ),
];

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
    for (name, expected) in FILES {
        let path = directory.join(name);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Check::warn(
                    "Deep Capture",
                    "WinDivert component",
                    format!("{name} missing; reinstall or extract the complete package"),
                );
            }
            Err(error) => {
                return Check::warn(
                    "Deep Capture",
                    "WinDivert component",
                    format!("{name} inaccessible: {error}"),
                );
            }
        };
        if matches!(name, "WinDivert.dll" | "WinDivert64.sys")
            && (bytes.len() < 66 || u16::from_le_bytes([bytes[0], bytes[1]]) != 0x5a4d || {
                let offset = u32::from_le_bytes(bytes[0x3c..0x40].try_into().unwrap()) as usize;
                bytes
                    .get(offset..offset + 6)
                    .is_none_or(|header| header[0..4] != *b"PE\0\0" || header[4..6] != [0x64, 0x86])
            })
        {
            return Check::warn(
                "Deep Capture",
                "WinDivert component",
                format!("{name} has no x64 PE header"),
            );
        }
        let actual: String = digest(&SHA256, &bytes)
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        if actual != expected {
            return Check::warn("Deep Capture", "WinDivert component", format!("{name} differs from the certified signed 2.2.2-A release; reinstall or extract the complete package"));
        }
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
