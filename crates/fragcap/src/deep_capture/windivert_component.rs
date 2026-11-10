// SPDX-License-Identifier: Apache-2.0

//! Read-only exact package identity before the authorized driver load.

use std::{
    fs,
    path::{Path, PathBuf},
};

use ring::digest::{digest, SHA256};

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

/// Confirm the complete S199 component without opening a driver handle.
pub fn verify_windivert_component(directory: &Path) -> Result<PathBuf, String> {
    for (name, expected) in FILES {
        let path = directory.join(name);
        let bytes = fs::read(&path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                format!("{name} missing; reinstall or extract the complete package")
            } else {
                format!("{name} inaccessible: {error}")
            }
        })?;
        if matches!(name, "WinDivert.dll" | "WinDivert64.sys") && !is_x64_pe(&bytes) {
            return Err(format!("{name} has no x64 PE header"));
        }
        let actual: String = digest(&SHA256, &bytes)
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        if actual != expected {
            return Err(format!("{name} differs from the certified signed 2.2.2-A release; reinstall or extract the complete package"));
        }
    }
    Ok(directory.join("WinDivert.dll"))
}

fn is_x64_pe(bytes: &[u8]) -> bool {
    if bytes.len() < 66 || bytes[0..2] != *b"MZ" {
        return false;
    }
    let offset = u32::from_le_bytes(bytes[0x3c..0x40].try_into().unwrap()) as usize;
    bytes
        .get(offset..offset + 6)
        .is_some_and(|header| header[0..4] == *b"PE\0\0" && header[4..6] == [0x64, 0x86])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_and_altered_package_never_qualify() {
        let directory = tempfile::tempdir().unwrap();
        assert!(verify_windivert_component(directory.path())
            .unwrap_err()
            .starts_with("WinDivert.dll missing"));
        fs::write(directory.path().join("WinDivert.dll"), b"altered").unwrap();
        assert!(verify_windivert_component(directory.path())
            .unwrap_err()
            .starts_with("WinDivert.dll has no x64 PE header"));
    }
}
