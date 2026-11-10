// SPDX-License-Identifier: Apache-2.0

//! Read-only exact package identity before the authorized driver load.

use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(all(windows, feature = "direct-route"))]
use std::{
    fs::{File, OpenOptions},
    io::{self, Read},
    os::windows::{ffi::OsStrExt, fs::OpenOptionsExt, io::FromRawHandle},
};

#[cfg(all(windows, feature = "direct-route"))]
use windows_sys::Win32::{
    Foundation::INVALID_HANDLE_VALUE,
    Storage::FileSystem::{
        CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_LIST_DIRECTORY,
        FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    },
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
        verify_bytes(name, expected, &bytes)?;
    }
    Ok(directory.join("WinDivert.dll"))
}

fn verify_bytes(name: &str, expected: &str, bytes: &[u8]) -> Result<(), String> {
    if matches!(name, "WinDivert.dll" | "WinDivert64.sys") && !is_x64_pe(bytes) {
        return Err(format!("{name} has no x64 PE header"));
    }
    let actual: String = digest(&SHA256, bytes)
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if actual != expected {
        return Err(format!("{name} differs from the certified signed 2.2.2-A release; reinstall or extract the complete package"));
    }
    Ok(())
}

/// Keep every path component and package file pinned from verification through
/// driver use. The read-sharing handles reject concurrent writes and deletes;
/// ancestor directory handles reject a rename that would redirect the pathname.
#[cfg(all(windows, feature = "direct-route"))]
pub(super) struct LockedWinDivertComponent {
    pub dll_path: PathBuf,
    _directories: Vec<File>,
    _files: Vec<File>,
}

#[cfg(all(windows, feature = "direct-route"))]
fn lock_directory(path: &Path) -> io::Result<File> {
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            FILE_LIST_DIRECTORY,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
            0,
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: CreateFileW returned a fresh owned handle transferred once.
    Ok(unsafe { File::from_raw_handle(handle as _) })
}

#[cfg(all(windows, feature = "direct-route"))]
pub(super) fn verify_locked_windivert_component(
    directory: &Path,
) -> io::Result<LockedWinDivertComponent> {
    let directory = fs::canonicalize(directory)?;
    let mut directories = Vec::new();
    for ancestor in directory.ancestors().collect::<Vec<_>>().into_iter().rev() {
        directories.push(lock_directory(ancestor)?);
    }
    let mut files = Vec::new();
    for (name, expected) in FILES {
        let path = directory.join(name);
        let mut file = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(&path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        verify_bytes(name, expected, &bytes).map_err(io::Error::other)?;
        files.push(file);
    }
    Ok(LockedWinDivertComponent {
        dll_path: directory.join("WinDivert.dll"),
        _directories: directories,
        _files: files,
    })
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
