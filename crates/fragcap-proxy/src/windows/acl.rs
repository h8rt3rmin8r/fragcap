// SPDX-License-Identifier: Apache-2.0

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::os::windows::io::{AsRawHandle, FromRawHandle};
use std::path::{Component, Path, PathBuf, Prefix};
use std::ptr;
use std::slice;

use windows_sys::Win32::Foundation::{GetLastError, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
};
use windows_sys::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPTOAPI_BLOB, CRYPTPROTECT_LOCAL_MACHINE,
    CRYPTPROTECT_UI_FORBIDDEN,
};
use windows_sys::Win32::Security::{
    GetFileSecurityW, GetSecurityDescriptorOwner, GetTokenInformation, IsWellKnownSid, TokenUser,
    WinBuiltinAdministratorsSid, WinLocalSystemSid, DACL_SECURITY_INFORMATION,
    OWNER_SECURITY_INFORMATION, SECURITY_ATTRIBUTES, TOKEN_USER,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateDirectoryW, CreateFileW, FileDispositionInfo, GetFileInformationByHandle,
    SetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, CREATE_NEW, FILE_ATTRIBUTE_DIRECTORY,
    FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_REPARSE_POINT, FILE_DISPOSITION_INFO,
    FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_GENERIC_READ,
    FILE_GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE,
};
use windows_sys::Win32::System::Memory::LocalFree;

use crate::CertificateError;
use zeroize::Zeroizing;

pub(crate) fn protect_and_write(path: &Path, plaintext: &[u8]) -> Result<(), CertificateError> {
    protect_and_write_with_scope(path, plaintext, false)
}

pub(crate) fn protect_and_write_machine(
    path: &Path,
    plaintext: &[u8],
) -> Result<(), CertificateError> {
    protect_and_write_with_scope(path, plaintext, true)
}

fn protect_and_write_with_scope(
    path: &Path,
    plaintext: &[u8],
    machine: bool,
) -> Result<(), CertificateError> {
    let input = CRYPTOAPI_BLOB {
        cbData: plaintext.len() as u32,
        pbData: plaintext.as_ptr().cast_mut(),
    };
    let mut output = CRYPTOAPI_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };
    let ok = unsafe {
        CryptProtectData(
            &input,
            ptr::null(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN
                | if machine {
                    CRYPTPROTECT_LOCAL_MACHINE
                } else {
                    0
                },
            &mut output,
        )
    };
    if ok == 0 {
        return Err(CertificateError::platform(
            "private-material-protect-failed",
            unsafe { GetLastError() },
        ));
    }
    let encrypted = unsafe { slice::from_raw_parts(output.pbData, output.cbData as usize) };
    let write = (|| {
        let mut output = PrivateOutput::create_with_scope(path, machine)?;
        if let Err(error) = output
            .file
            .write_all(encrypted)
            .and_then(|()| output.file.sync_all())
        {
            if let Err(cleanup) = output.remove_on_close() {
                return Err(CertificateError::io(
                    "private-material-write-cleanup-failed",
                    io::Error::other(format!(
                        "{error}; exact private output cleanup failed: {cleanup}"
                    )),
                ));
            }
            return Err(CertificateError::io("private-material-write-failed", error));
        }
        output.committed = true;
        Ok(())
    })();
    unsafe { LocalFree(output.pbData as isize) };
    write
}

pub(crate) fn create_machine_directory(path: &Path) -> Result<(), CertificateError> {
    create_installation_directory(path, MachineAcl::PrivateDirectory)
}

pub(crate) fn create_machine_public_directory(path: &Path) -> Result<(), CertificateError> {
    create_installation_directory(path, MachineAcl::PublicDirectory)
}

fn create_installation_directory(path: &Path, kind: MachineAcl) -> Result<(), CertificateError> {
    if path.exists() {
        return verify_machine_acl(path, kind);
    }
    let descriptor = machine_descriptor(kind)?;
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: 0,
    };
    let name: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    if unsafe { CreateDirectoryW(name.as_ptr(), &attributes) } == 0 {
        return Err(CertificateError::platform(
            "installation-directory-create-failed",
            unsafe { GetLastError() },
        ));
    }
    verify_machine_acl(path, kind)
}

pub(crate) fn read_and_unprotect_machine(
    path: &Path,
) -> Result<Zeroizing<Vec<u8>>, CertificateError> {
    verify_machine_acl(
        path.parent()
            .ok_or_else(|| CertificateError::new("installation-key-path-invalid"))?,
        MachineAcl::PrivateDirectory,
    )?;
    verify_machine_acl(path, MachineAcl::PrivateFile)?;
    let parents = pinned_parents(path)
        .map_err(|error| CertificateError::io("installation-key-parent-invalid", error))?;
    let file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|error| CertificateError::io("installation-key-read-failed", error))?;
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle() as isize, &mut info) } == 0
        || info.dwFileAttributes & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DIRECTORY) != 0
    {
        return Err(CertificateError::new("installation-key-alias-refused"));
    }
    let mut encrypted = Vec::new();
    file.take(1024 * 1024)
        .read_to_end(&mut encrypted)
        .map_err(|error| CertificateError::io("installation-key-read-failed", error))?;
    drop(parents);
    if encrypted.is_empty() || encrypted.len() >= 1024 * 1024 {
        return Err(CertificateError::new("installation-key-size-invalid"));
    }
    let input = CRYPTOAPI_BLOB {
        cbData: encrypted.len() as u32,
        pbData: encrypted.as_mut_ptr(),
    };
    let mut output = CRYPTOAPI_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };
    if unsafe {
        CryptUnprotectData(
            &input,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    } == 0
    {
        return Err(CertificateError::platform(
            "installation-key-decrypt-failed",
            unsafe { GetLastError() },
        ));
    }
    let private = Zeroizing::new(unsafe {
        slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec()
    });
    unsafe { ptr::write_bytes(output.pbData, 0, output.cbData as usize) };
    unsafe { LocalFree(output.pbData as isize) };
    Ok(private)
}

#[derive(Clone, Copy)]
enum MachineAcl {
    PublicDirectory,
    PrivateDirectory,
    PrivateFile,
}

fn machine_sddl(kind: MachineAcl) -> &'static str {
    match kind {
        MachineAcl::PublicDirectory => "D:P(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)(A;OICI;GR;;;BU)",
        MachineAcl::PrivateDirectory => "D:P(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)",
        MachineAcl::PrivateFile => "D:P(A;;FA;;;SY)(A;;FA;;;BA)",
    }
}

fn machine_descriptor(kind: MachineAcl) -> Result<PrivateDescriptor, CertificateError> {
    descriptor_from_sddl(&format!("O:BA{}", machine_sddl(kind)))
}

fn trusted_machine_owner(owner: windows_sys::Win32::Foundation::PSID) -> bool {
    !owner.is_null()
        && (unsafe { IsWellKnownSid(owner, WinBuiltinAdministratorsSid) } != 0
            || unsafe { IsWellKnownSid(owner, WinLocalSystemSid) } != 0)
}

fn machine_dacl_matches(observed: &str, expected: &str) -> bool {
    observed == expected || observed == expected.replacen("D:P", "D:PAI", 1)
}

fn verify_machine_acl(path: &Path, kind: MachineAcl) -> Result<(), CertificateError> {
    use windows_sys::Win32::Security::Authorization::ConvertSecurityDescriptorToStringSecurityDescriptorW;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| CertificateError::io("installation-resource-inaccessible", error))?;
    if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || matches!(kind, MachineAcl::PrivateFile) != metadata.is_file()
    {
        return Err(CertificateError::new("installation-resource-alias-refused"));
    }
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut needed = 0;
    unsafe {
        GetFileSecurityW(
            wide.as_ptr(),
            DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION,
            ptr::null_mut(),
            0,
            &mut needed,
        )
    };
    if needed == 0 || needed > 65536 {
        return Err(CertificateError::platform(
            "installation-acl-read-failed",
            unsafe { GetLastError() },
        ));
    }
    let mut buffer = vec![0_usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
    if unsafe {
        GetFileSecurityW(
            wide.as_ptr(),
            DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION,
            buffer.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    } == 0
    {
        return Err(CertificateError::platform(
            "installation-acl-read-failed",
            unsafe { GetLastError() },
        ));
    }
    let mut owner = ptr::null_mut();
    let mut owner_defaulted = 0;
    if unsafe {
        GetSecurityDescriptorOwner(buffer.as_mut_ptr().cast(), &mut owner, &mut owner_defaulted)
    } == 0
        || !trusted_machine_owner(owner)
    {
        return Err(CertificateError::new("installation-owner-invalid"));
    }
    let mut text = ptr::null_mut();
    if unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            buffer.as_mut_ptr().cast(),
            1,
            DACL_SECURITY_INFORMATION,
            &mut text,
            ptr::null_mut(),
        )
    } == 0
    {
        return Err(CertificateError::platform(
            "installation-acl-read-failed",
            unsafe { GetLastError() },
        ));
    }
    let mut len = 0;
    while unsafe { *text.add(len) } != 0 {
        len += 1;
    }
    let observed = String::from_utf16(unsafe { slice::from_raw_parts(text, len) })
        .map_err(|_| CertificateError::new("installation-acl-invalid"));
    unsafe { LocalFree(text as isize) };
    let observed = observed?;
    let expected = machine_sddl(kind);
    // NTFS can set SE_DACL_AUTO_INHERITED when persisting a protected DACL.
    // That control bit changes no ACE; all grants and protection must still
    // match the closed descriptor exactly.
    if !machine_dacl_matches(&observed, expected) {
        return Err(CertificateError {
            code: "installation-acl-invalid",
            detail: format!("expected {expected}, observed {observed}"),
        });
    }
    Ok(())
}

struct PrivateDescriptor(*mut core::ffi::c_void);

impl Drop for PrivateDescriptor {
    fn drop(&mut self) {
        unsafe { LocalFree(self.0 as isize) };
    }
}

fn private_descriptor() -> Result<PrivateDescriptor, CertificateError> {
    // DPAPI issuer material belongs to its producer, independently of object owner
    // and of the selected recipient for retained packet and application evidence.
    descriptor_from_sddl(&format!("D:P(A;;FA;;;{})(A;;FA;;;SY)", producer_sid()?))
}

fn descriptor_from_sddl(sddl: &str) -> Result<PrivateDescriptor, CertificateError> {
    let sddl: Vec<u16> = sddl.encode_utf16().chain(std::iter::once(0)).collect();
    let mut descriptor = ptr::null_mut();
    let mut length = 0_u32;
    let converted = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut descriptor,
            &mut length,
        )
    };
    if converted == 0 {
        return Err(CertificateError::platform(
            "private-material-acl-build-failed",
            unsafe { GetLastError() },
        ));
    }
    Ok(PrivateDescriptor(descriptor))
}

/// Exclusive, producer-private creation. Failure deletes only this exact new object.
struct PrivateOutput {
    file: File,
    _parents: Vec<File>,
    committed: bool,
}

impl PrivateOutput {
    #[cfg(test)]
    fn create(path: &Path) -> Result<Self, CertificateError> {
        Self::create_with_scope(path, false)
    }

    fn create_with_scope(path: &Path, machine: bool) -> Result<Self, CertificateError> {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|error| CertificateError::io("private-material-path-failed", error))?
                .join(path)
        };
        validate_private_path(&path)
            .map_err(|error| CertificateError::io("private-material-path-failed", error))?;
        let parents = pinned_parents(&path)
            .map_err(|error| CertificateError::io("private-material-parent-failed", error))?;
        if machine {
            verify_machine_acl(
                path.parent()
                    .ok_or_else(|| CertificateError::new("installation-key-path-invalid"))?,
                MachineAcl::PrivateDirectory,
            )?;
        }
        let descriptor = if machine {
            machine_descriptor(MachineAcl::PrivateFile)?
        } else {
            private_descriptor()?
        };
        let attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor.0,
            bInheritHandle: 0,
        };
        let name: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        // DELETE (0x10000) permits exact handle-based cleanup on write failure.
        // CREATE_NEW refuses all existing files and aliases without opening them.
        let handle = unsafe {
            CreateFileW(
                name.as_ptr(),
                FILE_GENERIC_READ | FILE_GENERIC_WRITE | 0x0001_0000,
                0,
                &attributes,
                CREATE_NEW,
                FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT,
                0,
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(CertificateError::platform(
                "private-material-create-failed",
                unsafe { GetLastError() },
            ));
        }
        // SAFETY: The new exclusive handle is transferred once to its owning File.
        let file = unsafe { File::from_raw_handle(handle as *mut core::ffi::c_void) };
        Ok(Self {
            file,
            _parents: parents,
            committed: false,
        })
    }

    fn remove_on_close(&self) -> io::Result<()> {
        let disposition = FILE_DISPOSITION_INFO { DeleteFileA: 1 };
        if unsafe {
            SetFileInformationByHandle(
                self.file.as_raw_handle() as isize,
                FileDispositionInfo,
                (&disposition as *const FILE_DISPOSITION_INFO).cast(),
                std::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
            )
        } == 0
        {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl Drop for PrivateOutput {
    fn drop(&mut self) {
        if !self.committed {
            let _ = self.remove_on_close();
        }
    }
}

fn validate_private_path(path: &Path) -> io::Result<()> {
    let invalid = || {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "private output refuses stream, device, or normalized path aliases",
        )
    };
    if !path.is_absolute() || path.as_os_str().encode_wide().any(|unit| unit == 0) {
        return Err(invalid());
    }
    for component in path.components() {
        match component {
            Component::ParentDir => return Err(invalid()),
            Component::Prefix(prefix)
                if !matches!(
                    prefix.kind(),
                    Prefix::Disk(_)
                        | Prefix::UNC(_, _)
                        | Prefix::VerbatimDisk(_)
                        | Prefix::VerbatimUNC(_, _)
                ) =>
            {
                return Err(invalid());
            }
            Component::Normal(name) => {
                let units = name.encode_wide().collect::<Vec<_>>();
                if units.contains(&(b':' as u16)) || matches!(units.last(), Some(0x2e | 0x20)) {
                    return Err(invalid());
                }
                // DOS device names remain aliases even with an ordinary extension.
                let text = name.to_string_lossy().to_ascii_uppercase();
                let base = text.split('.').next().unwrap_or_default();
                if matches!(base, "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$")
                    || ((base.starts_with("COM") || base.starts_with("LPT"))
                        && matches!(
                            base.get(3..),
                            Some(
                                "1" | "2"
                                    | "3"
                                    | "4"
                                    | "5"
                                    | "6"
                                    | "7"
                                    | "8"
                                    | "9"
                                    | "\u{b9}"
                                    | "\u{b2}"
                                    | "\u{b3}"
                            )
                        ))
                {
                    return Err(invalid());
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn pinned_parents(path: &Path) -> io::Result<Vec<File>> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "private output has no parent")
    })?;
    let mut paths = parent
        .ancestors()
        .map(Path::to_path_buf)
        .collect::<Vec<PathBuf>>();
    paths.reverse();
    let mut handles = Vec::new();
    for path in paths {
        let file = OpenOptions::new()
            .access_mode(0)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&path)?;
        let mut information: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe { GetFileInformationByHandle(file.as_raw_handle() as isize, &mut information) }
            == 0
        {
            return Err(io::Error::last_os_error());
        }
        if information.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
            || information.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "private output refuses a reparse or non-directory parent",
            ));
        }
        handles.push(file);
    }
    Ok(handles)
}

fn producer_sid() -> Result<String, CertificateError> {
    // The current-process token pseudo-handle is query-only and opens no target process.
    const CURRENT_PROCESS_TOKEN: isize = -4;
    let mut needed = 0;
    unsafe {
        GetTokenInformation(
            CURRENT_PROCESS_TOKEN,
            TokenUser,
            ptr::null_mut(),
            0,
            &mut needed,
        )
    };
    if needed == 0 || needed > 65536 {
        return Err(CertificateError::platform(
            "private-material-user-query-failed",
            unsafe { GetLastError() },
        ));
    }
    let mut storage = vec![0_usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
    if unsafe {
        GetTokenInformation(
            CURRENT_PROCESS_TOKEN,
            TokenUser,
            storage.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    } == 0
    {
        return Err(CertificateError::platform(
            "private-material-user-query-failed",
            unsafe { GetLastError() },
        ));
    }
    let user = unsafe { &*storage.as_ptr().cast::<TOKEN_USER>() };
    let mut sid = ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(user.User.Sid, &mut sid) } == 0 {
        return Err(CertificateError::platform(
            "private-material-user-sid-failed",
            unsafe { GetLastError() },
        ));
    }
    let mut length = 0;
    while unsafe { *sid.add(length) } != 0 {
        length += 1;
    }
    let result =
        String::from_utf16(unsafe { slice::from_raw_parts(sid, length) }).map_err(|error| {
            CertificateError::io(
                "private-material-user-sid-invalid",
                std::io::Error::new(std::io::ErrorKind::InvalidData, error),
            )
        });
    unsafe { LocalFree(sid as isize) };
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::Security::Authorization::{
        ConvertSecurityDescriptorToStringSecurityDescriptorW, ConvertStringSidToSidW,
    };
    use windows_sys::Win32::Security::{
        EqualSid, GetAce, GetFileSecurityW, GetSecurityDescriptorDacl, IsWellKnownSid,
        SetFileSecurityW, WinLocalSystemSid, ACCESS_ALLOWED_ACE, DACL_SECURITY_INFORMATION,
        PROTECTED_DACL_SECURITY_INFORMATION,
    };
    use windows_sys::Win32::Storage::FileSystem::FILE_ALL_ACCESS;

    #[test]
    fn installation_owner_refuses_an_ordinary_user_even_with_an_exact_dacl() {
        let descriptor = machine_descriptor(MachineAcl::PrivateFile).unwrap();
        let mut configured_owner = ptr::null_mut();
        let mut owner_defaulted = 0;
        assert_ne!(
            unsafe {
                GetSecurityDescriptorOwner(
                    descriptor.0,
                    &mut configured_owner,
                    &mut owner_defaulted,
                )
            },
            0
        );
        assert_ne!(
            unsafe { IsWellKnownSid(configured_owner, WinBuiltinAdministratorsSid) },
            0
        );
        for (sid_text, trusted) in [
            ("S-1-5-18", true),
            ("S-1-5-32-544", true),
            ("S-1-5-21-111-222-333-1001", false),
        ] {
            let wide: Vec<u16> = sid_text.encode_utf16().chain(Some(0)).collect();
            let mut sid = ptr::null_mut();
            assert_ne!(
                unsafe { ConvertStringSidToSidW(wide.as_ptr(), &mut sid) },
                0
            );
            assert_eq!(trusted_machine_owner(sid), trusted, "{sid_text}");
            unsafe { LocalFree(sid as isize) };
        }
    }

    #[test]
    fn installation_dacl_accepts_only_its_exact_aces_and_optional_auto_inherit_marker() {
        for kind in [
            MachineAcl::PublicDirectory,
            MachineAcl::PrivateDirectory,
            MachineAcl::PrivateFile,
        ] {
            let expected = machine_sddl(kind);
            assert!(machine_dacl_matches(expected, expected));
            assert!(machine_dacl_matches(
                &expected.replacen("D:P", "D:PAI", 1),
                expected
            ));
            assert!(!machine_dacl_matches(
                &expected.replacen("D:P", "D:", 1),
                expected
            ));
            assert!(!machine_dacl_matches(
                &format!("{expected}(A;;FA;;;BU)"),
                expected
            ));
        }
    }

    #[test]
    fn issuer_private_dacl_names_exact_producer_without_owner_relative_access() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("issuer-private.dpapi");
        let mut output = PrivateOutput::create(&path).unwrap();
        let wide = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let mut needed = 0;
        unsafe {
            GetFileSecurityW(
                wide.as_ptr(),
                DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                0,
                &mut needed,
            )
        };
        let mut buffer = vec![0_usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
        assert_ne!(
            unsafe {
                GetFileSecurityW(
                    wide.as_ptr(),
                    DACL_SECURITY_INFORMATION,
                    buffer.as_mut_ptr().cast(),
                    needed,
                    &mut needed,
                )
            },
            0
        );
        let mut descriptor = ptr::null_mut();
        assert_ne!(
            unsafe {
                ConvertSecurityDescriptorToStringSecurityDescriptorW(
                    buffer.as_mut_ptr().cast(),
                    1,
                    DACL_SECURITY_INFORMATION,
                    &mut descriptor,
                    ptr::null_mut(),
                )
            },
            0
        );
        let mut length = 0;
        while unsafe { *descriptor.add(length) } != 0 {
            length += 1;
        }
        let text =
            String::from_utf16(unsafe { slice::from_raw_parts(descriptor, length) }).unwrap();
        unsafe { LocalFree(descriptor as isize) };
        let mut acl = ptr::null_mut();
        let mut present = 0;
        let mut defaulted = 0;
        assert_ne!(
            unsafe {
                GetSecurityDescriptorDacl(
                    buffer.as_mut_ptr().cast(),
                    &mut present,
                    &mut acl,
                    &mut defaulted,
                )
            },
            0
        );
        assert_ne!(present, 0);
        assert!(!acl.is_null());
        assert_eq!(unsafe { (*acl).AceCount }, 2, "{text}");
        let producer = producer_sid().unwrap();
        let mut producer_pointer = ptr::null_mut();
        let producer_wide: Vec<u16> = producer.encode_utf16().chain(Some(0)).collect();
        assert_ne!(
            unsafe { ConvertStringSidToSidW(producer_wide.as_ptr(), &mut producer_pointer) },
            0
        );
        let mut producer_count = 0;
        let mut system_count = 0;
        for index in 0..2 {
            let mut ace = std::mem::MaybeUninit::<*mut core::ffi::c_void>::uninit();
            assert_ne!(unsafe { GetAce(acl, index, ace.as_mut_ptr()) }, 0);
            // SAFETY: GetAce succeeded and initialized its documented output pointer.
            let ace = unsafe { ace.assume_init() };
            assert!(!ace.is_null());
            let allowed = unsafe { &*ace.cast::<ACCESS_ALLOWED_ACE>() };
            assert_eq!(allowed.Header.AceType, 0);
            assert_eq!(allowed.Mask, FILE_ALL_ACCESS);
            let sid = (&allowed.SidStart as *const u32).cast_mut().cast();
            producer_count += u32::from(unsafe { EqualSid(sid, producer_pointer) } != 0);
            system_count += u32::from(unsafe { IsWellKnownSid(sid, WinLocalSystemSid) } != 0);
        }
        unsafe { LocalFree(producer_pointer as isize) };
        assert_eq!((producer_count, system_count), (1, 1), "{text}");
        assert!(!text.contains(";;;OW)"), "{text}");
        assert!(text.contains(";;;SY)"), "{text}");
        assert!(
            !text.contains(";;;WD)") && !text.contains(";;;BU)"),
            "{text}"
        );
        assert_eq!(output.file.metadata().unwrap().len(), 0);
        output.file.write_all(b"synthetic encrypted state").unwrap();
        output.file.sync_all().unwrap();
        output.committed = true;
    }

    #[test]
    fn issuer_persistence_refuses_existing_content_without_overwriting_it() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("existing-issuer.private");
        std::fs::write(&path, b"existing synthetic evidence").unwrap();
        let error = protect_and_write(&path, b"new synthetic private key");
        assert!(error.is_err(), "existing issuer paths must be refused");
        assert_eq!(std::fs::read(path).unwrap(), b"existing synthetic evidence");
    }

    #[test]
    fn issuer_creation_does_not_inherit_retained_recipient_grants() {
        let root = tempfile::tempdir().unwrap();
        let producer = producer_sid().unwrap();
        let parent_sddl = format!("D:P(A;OICI;FA;;;{producer})(A;OICI;FA;;;SY)(A;OICI;FA;;;AN)");
        let sddl = parent_sddl
            .encode_utf16()
            .chain(Some(0))
            .collect::<Vec<_>>();
        let mut pointer = ptr::null_mut();
        assert_ne!(
            unsafe {
                ConvertStringSecurityDescriptorToSecurityDescriptorW(
                    sddl.as_ptr(),
                    1,
                    &mut pointer,
                    ptr::null_mut(),
                )
            },
            0
        );
        let parent = root
            .path()
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        assert_ne!(
            unsafe {
                SetFileSecurityW(
                    parent.as_ptr(),
                    DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                    pointer,
                )
            },
            0
        );
        unsafe { LocalFree(pointer as isize) };
        let inherited = root.path().join("retained-recipient-control");
        std::fs::write(&inherited, b"synthetic retained evidence").unwrap();
        assert!(descriptor_text(&inherited).contains(";;;AN)"));
        let issuer = root.path().join("issuer-private.dpapi");
        let output = PrivateOutput::create(&issuer).unwrap();
        // Inspect before any byte is written, so post-write retrofit cannot pass.
        let text = descriptor_text(&issuer);
        assert!(text.starts_with("D:P"), "{text}");
        assert!(
            !text.contains(";;;AN)") && !text.contains(";;;OW)"),
            "{text}"
        );
        assert!(text.contains(";;;SY)"), "{text}");
        assert_eq!(output.file.metadata().unwrap().len(), 0);
        drop(output);
        assert!(
            !issuer.exists(),
            "uncommitted exact private output must be removed"
        );
        assert_eq!(
            std::fs::read(inherited).unwrap(),
            b"synthetic retained evidence"
        );
    }

    #[test]
    fn issuer_existing_hard_link_alias_is_never_overwritten() {
        let root = tempfile::tempdir().unwrap();
        let outside = root.path().join("outside-evidence");
        std::fs::write(&outside, b"unrelated synthetic evidence").unwrap();
        let alias = root.path().join("issuer-private.dpapi");
        std::fs::hard_link(&outside, &alias).unwrap();
        assert!(protect_and_write(&alias, b"synthetic key").is_err());
        assert_eq!(
            std::fs::read(&outside).unwrap(),
            b"unrelated synthetic evidence"
        );
        assert_eq!(
            std::fs::read(alias).unwrap(),
            b"unrelated synthetic evidence"
        );
    }

    #[test]
    fn issuer_alternate_stream_cannot_extend_an_existing_retained_file() {
        let root = tempfile::tempdir().unwrap();
        let retained = root.path().join("retained-evidence");
        std::fs::write(&retained, b"synthetic retained evidence").unwrap();
        let descriptor = descriptor_text(&retained);
        let stream = root.path().join("retained-evidence:issuer-private");
        assert!(protect_and_write(&stream, b"synthetic private key").is_err());
        assert_eq!(
            std::fs::read(&retained).unwrap(),
            b"synthetic retained evidence"
        );
        assert_eq!(descriptor_text(&retained), descriptor);
        assert!(
            std::fs::read(stream).is_err(),
            "no private alternate stream was created"
        );
    }

    #[test]
    fn issuer_creation_refuses_windows_normalized_and_device_aliases() {
        let root = tempfile::tempdir().unwrap();
        for name in [
            "issuer-private.",
            "issuer-private ",
            "NUL.private",
            "COM1.key",
        ] {
            assert!(
                PrivateOutput::create(&root.path().join(name)).is_err(),
                "{name}"
            );
        }
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }

    fn descriptor_text(path: &Path) -> String {
        let wide = path
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        let mut needed = 0;
        unsafe {
            GetFileSecurityW(
                wide.as_ptr(),
                DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                0,
                &mut needed,
            )
        };
        let mut storage = vec![0_usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
        assert_ne!(
            unsafe {
                GetFileSecurityW(
                    wide.as_ptr(),
                    DACL_SECURITY_INFORMATION,
                    storage.as_mut_ptr().cast(),
                    needed,
                    &mut needed,
                )
            },
            0
        );
        let mut text = ptr::null_mut();
        assert_ne!(
            unsafe {
                ConvertSecurityDescriptorToStringSecurityDescriptorW(
                    storage.as_mut_ptr().cast(),
                    1,
                    DACL_SECURITY_INFORMATION,
                    &mut text,
                    ptr::null_mut(),
                )
            },
            0
        );
        let mut length = 0;
        while unsafe { *text.add(length) } != 0 {
            length += 1;
        }
        let result = String::from_utf16(unsafe { slice::from_raw_parts(text, length) }).unwrap();
        unsafe { LocalFree(text as isize) };
        result
    }
}
