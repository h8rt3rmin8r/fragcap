// SPDX-License-Identifier: Apache-2.0

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr;
use std::slice;

use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
};
use windows_sys::Win32::Security::Cryptography::{
    CryptProtectData, CRYPTOAPI_BLOB, CRYPTPROTECT_UI_FORBIDDEN,
};
use windows_sys::Win32::Security::{
    GetTokenInformation, SetFileSecurityW, TokenUser, DACL_SECURITY_INFORMATION,
    PROTECTED_DACL_SECURITY_INFORMATION, TOKEN_USER,
};
use windows_sys::Win32::System::Memory::LocalFree;

use crate::CertificateError;

pub(crate) fn protect_and_write(path: &Path, plaintext: &[u8]) -> Result<(), CertificateError> {
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
            CRYPTPROTECT_UI_FORBIDDEN,
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
    let write = std::fs::write(path, encrypted)
        .map_err(|error| CertificateError::io("private-material-write-failed", error));
    unsafe { LocalFree(output.pbData as isize) };
    write?;
    apply_private_dacl(path)
}

fn apply_private_dacl(path: &Path) -> Result<(), CertificateError> {
    // DPAPI issuer material belongs to its producer, independently of object owner
    // and of the selected recipient for retained packet and application evidence.
    let sddl: Vec<u16> = format!("D:P(A;;FA;;;{})(A;;FA;;;SY)", producer_sid()?)
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
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
    let wide: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let applied = unsafe {
        SetFileSecurityW(
            wide.as_ptr(),
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            descriptor,
        )
    };
    let error = if applied == 0 {
        Some(unsafe { GetLastError() })
    } else {
        None
    };
    unsafe { LocalFree(descriptor as isize) };
    match error {
        Some(code) => Err(CertificateError::platform(
            "private-material-acl-failed",
            code,
        )),
        None => Ok(()),
    }
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
    use windows_sys::Win32::Security::Authorization::ConvertSecurityDescriptorToStringSecurityDescriptorW;
    use windows_sys::Win32::Security::GetFileSecurityW;

    #[test]
    fn issuer_private_dacl_names_exact_producer_without_owner_relative_access() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("issuer-private.dpapi");
        fs_write_for_test(&path);
        apply_private_dacl(&path).unwrap();
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
        assert!(
            text.contains(&format!(";;;{})", producer_sid().unwrap())),
            "{text}"
        );
        assert!(!text.contains(";;;OW)"), "{text}");
        assert!(text.contains(";;;SY)"), "{text}");
        assert!(
            !text.contains(";;;WD)") && !text.contains(";;;BU)"),
            "{text}"
        );
    }

    fn fs_write_for_test(path: &Path) {
        std::fs::write(path, b"synthetic encrypted state").unwrap();
    }
}
