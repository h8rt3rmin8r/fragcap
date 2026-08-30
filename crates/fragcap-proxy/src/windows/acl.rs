// SPDX-License-Identifier: Apache-2.0

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr;
use std::slice;

use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
use windows_sys::Win32::Security::Cryptography::{
    CryptProtectData, CRYPTOAPI_BLOB, CRYPTPROTECT_UI_FORBIDDEN,
};
use windows_sys::Win32::Security::{SetFileSecurityW, DACL_SECURITY_INFORMATION};
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
    let sddl: Vec<u16> = "D:P(A;;FA;;;OW)(A;;FA;;;SY)"
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
    let applied = unsafe { SetFileSecurityW(wide.as_ptr(), DACL_SECURITY_INFORMATION, descriptor) };
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
