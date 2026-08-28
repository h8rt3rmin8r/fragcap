// SPDX-License-Identifier: Apache-2.0

//! Narrow Windows certificate helpers shared by Deep Capture and doctor.

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows_sys::Win32::Foundation::{GetLastError, SetLastError, CRYPT_E_NOT_FOUND};
use windows_sys::Win32::Security::Cryptography::{
    CertCloseStore, CertEnumCertificatesInStore, CertFreeCertificateContext,
    CertGetCertificateContextProperty, CertOpenStore, CryptQueryObject, CERT_CONTEXT,
    CERT_QUERY_CONTENT_FLAG_CERT, CERT_QUERY_FORMAT_FLAG_ALL, CERT_QUERY_OBJECT_FILE,
    CERT_SHA1_HASH_PROP_ID, CERT_STORE_OPEN_EXISTING_FLAG, CERT_STORE_PROV_SYSTEM_W,
    CERT_STORE_READONLY_FLAG,
};

pub(crate) const CURRENT_USER_ROOT: &str = "CurrentUser/Root";
pub(crate) const LOCAL_MACHINE_ROOT: &str = "LocalMachine/Root";

pub(crate) fn file_thumbprint(path: &Path) -> Result<String, String> {
    let path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut context: *mut core::ffi::c_void = core::ptr::null_mut();
    // SAFETY: `path` is a live, null-terminated UTF-16 filename; unused outputs
    // are null and `context` is a live out pointer freed below.
    let queried = unsafe {
        CryptQueryObject(
            CERT_QUERY_OBJECT_FILE,
            path.as_ptr().cast(),
            CERT_QUERY_CONTENT_FLAG_CERT,
            CERT_QUERY_FORMAT_FLAG_ALL,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut context,
        )
    };
    if queried == 0 || context.is_null() {
        return Err(format!(
            "Windows could not parse certificate {}",
            path_display(&path)
        ));
    }
    let certificate = context.cast::<CERT_CONTEXT>();
    let result = context_thumbprint(certificate);
    // SAFETY: this function owns the live context returned by CryptQueryObject.
    unsafe { CertFreeCertificateContext(certificate) };
    result
}

pub(crate) fn store_thumbprints(location: &str) -> Result<Vec<String>, String> {
    use windows_sys::Win32::Security::Cryptography::{
        CERT_SYSTEM_STORE_CURRENT_USER_ID, CERT_SYSTEM_STORE_LOCAL_MACHINE_ID,
        CERT_SYSTEM_STORE_LOCATION_SHIFT,
    };

    let flags = match location {
        CURRENT_USER_ROOT => CERT_SYSTEM_STORE_CURRENT_USER_ID << CERT_SYSTEM_STORE_LOCATION_SHIFT,
        LOCAL_MACHINE_ROOT => {
            CERT_SYSTEM_STORE_LOCAL_MACHINE_ID << CERT_SYSTEM_STORE_LOCATION_SHIFT
        }
        _ => return Err(format!("unsupported certificate store {location}")),
    } | CERT_STORE_READONLY_FLAG
        | CERT_STORE_OPEN_EXISTING_FLAG;
    let root: Vec<u16> = "Root".encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: the provider constant is defined by CryptoAPI and `root` is a live,
    // null-terminated UTF-16 store name. The returned handle is closed below.
    let store =
        unsafe { CertOpenStore(CERT_STORE_PROV_SYSTEM_W, 0, 0, flags, root.as_ptr().cast()) };
    if store.is_null() {
        return Err(format!("Windows could not open {location} read-only"));
    }
    let mut out = Vec::new();
    let mut previous = core::ptr::null();
    loop {
        // SAFETY: resetting thread-local last-error state immediately before the
        // API call makes its null-result reason unambiguous.
        unsafe { SetLastError(0) };
        // SAFETY: `store` is live. CryptoAPI frees `previous` as it advances and
        // returns either a live context owned by the enumeration or null at end.
        let context = unsafe { CertEnumCertificatesInStore(store, previous) };
        if context.is_null() {
            // SAFETY: read immediately after the failed enumeration call, before
            // any other Windows API can replace the thread-local value.
            let error = unsafe { GetLastError() };
            if error != CRYPT_E_NOT_FOUND as u32 {
                // SAFETY: enumeration has released `previous`; `store` is still
                // the live handle opened above.
                unsafe { CertCloseStore(store, 0) };
                return Err(format!(
                    "Windows stopped enumerating {location} with error 0x{error:08X}"
                ));
            }
            break;
        }
        match context_thumbprint(context) {
            Ok(value) => out.push(value),
            Err(err) => {
                // SAFETY: enumeration returned this live context and ownership has
                // not advanced to another call.
                unsafe { CertFreeCertificateContext(context) };
                unsafe { CertCloseStore(store, 0) };
                return Err(format!("could not read a certificate in {location}: {err}"));
            }
        }
        previous = context;
    }
    // SAFETY: `store` is the live handle opened above and enumeration is complete.
    unsafe { CertCloseStore(store, 0) };
    out.sort();
    out.dedup();
    Ok(out)
}

fn context_thumbprint(context: *const CERT_CONTEXT) -> Result<String, String> {
    let mut hash = [0u8; 20];
    let mut hash_len = hash.len() as u32;
    // SAFETY: `context` is a live certificate context and the output buffer has
    // the exact writable size passed through `hash_len`.
    let read = unsafe {
        CertGetCertificateContextProperty(
            context,
            CERT_SHA1_HASH_PROP_ID,
            hash.as_mut_ptr().cast(),
            &mut hash_len,
        )
    };
    if read == 0 || hash_len != hash.len() as u32 {
        return Err("Windows could not read the SHA-1 thumbprint".to_string());
    }
    Ok(hash.iter().map(|byte| format!("{byte:02X}")).collect())
}

fn path_display(path: &[u16]) -> String {
    String::from_utf16_lossy(path)
        .trim_end_matches('\0')
        .to_string()
}
