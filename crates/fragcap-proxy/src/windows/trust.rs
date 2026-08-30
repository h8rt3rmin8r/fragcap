// SPDX-License-Identifier: Apache-2.0

use std::ptr;

use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Security::Cryptography::{
    CertAddEncodedCertificateToStore, CertCloseStore, CertDeleteCertificateFromStore,
    CertFindCertificateInStore, CertFreeCertificateContext, CertGetCertificateContextProperty,
    CertOpenStore, CERT_CONTEXT, CERT_FIND_SHA1_HASH, CERT_SHA256_HASH_PROP_ID, CERT_STORE_ADD_NEW,
    CERT_STORE_OPEN_EXISTING_FLAG, CERT_STORE_PROV_SYSTEM_W, CERT_SYSTEM_STORE_CURRENT_USER_ID,
    CERT_SYSTEM_STORE_LOCAL_MACHINE_ID, CERT_SYSTEM_STORE_LOCATION_SHIFT, CRYPTOAPI_BLOB,
    PKCS_7_ASN_ENCODING, X509_ASN_ENCODING,
};

use crate::{TrustError, TrustMutation, TrustState, CURRENT_USER_ROOT, LOCAL_MACHINE_ROOT};

pub(crate) fn observe(der: &[u8], thumbprint: &str) -> Result<TrustState, TrustError> {
    let hash = parse_thumbprint(thumbprint)?;
    let current = find(CURRENT_USER_ROOT, &hash, der)?;
    if current == Found::Exact {
        return Ok(TrustState::PresentExact);
    }
    if current == Found::Mismatch {
        return Ok(TrustState::Mismatch);
    }
    let machine = find(LOCAL_MACHINE_ROOT, &hash, der)?;
    Ok(match machine {
        Found::Exact | Found::Mismatch => TrustState::PresentWrongStore,
        Found::Absent => TrustState::Absent,
    })
}

pub(crate) fn add(der: &[u8], thumbprint: &str) -> Result<TrustMutation, TrustError> {
    match observe(der, thumbprint)? {
        TrustState::PresentExact => return Ok(TrustMutation::AlreadyPresent),
        TrustState::Mismatch => {
            return Err(error(
                "trust-certificate-mismatch",
                "add",
                None,
                "the authorized thumbprint resolves to different certificate bytes",
            ))
        }
        TrustState::PresentWrongStore | TrustState::Absent => {}
    }
    let store = open(CURRENT_USER_ROOT, false, "add")?;
    let mut added = ptr::null_mut();
    let ok = unsafe {
        CertAddEncodedCertificateToStore(
            store,
            X509_ASN_ENCODING,
            der.as_ptr(),
            der.len() as u32,
            CERT_STORE_ADD_NEW,
            &mut added,
        )
    };
    let os_error = if ok == 0 {
        Some(unsafe { GetLastError() })
    } else {
        None
    };
    if !added.is_null() {
        unsafe { CertFreeCertificateContext(added) };
    }
    unsafe { CertCloseStore(store, 0) };
    if let Some(code) = os_error {
        return Err(error(
            "trust-add-failed",
            "add",
            Some(code),
            format!("CryptoAPI add failed with 0x{code:08X}"),
        ));
    }
    if observe(der, thumbprint)? != TrustState::PresentExact {
        return Err(error(
            "trust-add-verification-failed",
            "add",
            None,
            "the exact certificate was not present after add",
        ));
    }
    Ok(TrustMutation::Added)
}

pub(crate) fn remove(der: &[u8], thumbprint: &str) -> Result<TrustMutation, TrustError> {
    let hash = parse_thumbprint(thumbprint)?;
    let store = open(CURRENT_USER_ROOT, false, "remove")?;
    let context = find_context(store, &hash);
    if context.is_null() {
        unsafe { CertCloseStore(store, 0) };
        return Ok(TrustMutation::AlreadyAbsent);
    }
    if !exact_certificate(context, der) {
        unsafe {
            CertFreeCertificateContext(context);
            CertCloseStore(store, 0);
        }
        return Err(error(
            "trust-certificate-mismatch",
            "remove",
            None,
            "refusing to remove different certificate bytes",
        ));
    }
    let ok = unsafe { CertDeleteCertificateFromStore(context) };
    let os_error = if ok == 0 {
        Some(unsafe { GetLastError() })
    } else {
        None
    };
    unsafe { CertCloseStore(store, 0) };
    if let Some(code) = os_error {
        return Err(error(
            "trust-remove-failed",
            "remove",
            Some(code),
            format!("CryptoAPI delete failed with 0x{code:08X}"),
        ));
    }
    if find(CURRENT_USER_ROOT, &hash, der)? != Found::Absent {
        return Err(error(
            "trust-remove-verification-failed",
            "remove",
            None,
            "the certificate remained after removal",
        ));
    }
    Ok(TrustMutation::Removed)
}

pub(crate) fn remove_thumbprint(thumbprint: &str) -> Result<TrustMutation, TrustError> {
    let hash = parse_thumbprint(thumbprint)?;
    let store = open(CURRENT_USER_ROOT, false, "remove")?;
    let context = find_context(store, &hash);
    if context.is_null() {
        unsafe { CertCloseStore(store, 0) };
        return Ok(TrustMutation::AlreadyAbsent);
    }
    let ok = unsafe { CertDeleteCertificateFromStore(context) };
    let os_error = if ok == 0 {
        Some(unsafe { GetLastError() })
    } else {
        None
    };
    unsafe { CertCloseStore(store, 0) };
    if let Some(code) = os_error {
        return Err(error(
            "trust-remove-failed",
            "remove",
            Some(code),
            format!("CryptoAPI delete failed with 0x{code:08X}"),
        ));
    }
    let verify = open(CURRENT_USER_ROOT, true, "observe")?;
    let remaining = find_context(verify, &hash);
    if !remaining.is_null() {
        unsafe { CertFreeCertificateContext(remaining) };
    }
    unsafe { CertCloseStore(verify, 0) };
    if remaining.is_null() {
        Ok(TrustMutation::Removed)
    } else {
        Err(error(
            "trust-remove-verification-failed",
            "remove",
            None,
            "the certificate remained after removal",
        ))
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Found {
    Absent,
    Exact,
    Mismatch,
}

fn find(store_name: &'static str, hash: &[u8; 20], der: &[u8]) -> Result<Found, TrustError> {
    let store = open(store_name, true, "observe")?;
    let context = find_context(store, hash);
    let found = if context.is_null() {
        Found::Absent
    } else if exact_certificate(context, der) {
        Found::Exact
    } else {
        Found::Mismatch
    };
    if !context.is_null() {
        unsafe { CertFreeCertificateContext(context) };
    }
    unsafe { CertCloseStore(store, 0) };
    Ok(found)
}

fn find_context(store: *mut core::ffi::c_void, hash: &[u8; 20]) -> *mut CERT_CONTEXT {
    let blob = CRYPTOAPI_BLOB {
        cbData: hash.len() as u32,
        pbData: hash.as_ptr().cast_mut(),
    };
    unsafe {
        CertFindCertificateInStore(
            store,
            X509_ASN_ENCODING | PKCS_7_ASN_ENCODING,
            0,
            CERT_FIND_SHA1_HASH,
            (&blob as *const CRYPTOAPI_BLOB).cast(),
            ptr::null(),
        )
    }
}

fn exact_certificate(context: *const CERT_CONTEXT, der: &[u8]) -> bool {
    if context.is_null() {
        return false;
    }
    let mut actual = [0_u8; 32];
    let mut actual_len = actual.len() as u32;
    // SAFETY: `context` is live for this call and `actual` has the exact writable
    // capacity supplied through `actual_len`. No context-owned pointer is exposed.
    let read = unsafe {
        CertGetCertificateContextProperty(
            context,
            CERT_SHA256_HASH_PROP_ID,
            actual.as_mut_ptr().cast(),
            &mut actual_len,
        )
    };
    read != 0
        && actual_len == actual.len() as u32
        && actual.as_slice() == ring::digest::digest(&ring::digest::SHA256, der).as_ref()
}

fn open(
    store_name: &'static str,
    readonly: bool,
    operation: &'static str,
) -> Result<*mut core::ffi::c_void, TrustError> {
    let location = if store_name == CURRENT_USER_ROOT {
        CERT_SYSTEM_STORE_CURRENT_USER_ID
    } else {
        CERT_SYSTEM_STORE_LOCAL_MACHINE_ID
    };
    let root: Vec<u16> = "Root".encode_utf16().chain(std::iter::once(0)).collect();
    let mut flags = (location << CERT_SYSTEM_STORE_LOCATION_SHIFT) | CERT_STORE_OPEN_EXISTING_FLAG;
    if readonly {
        flags |= windows_sys::Win32::Security::Cryptography::CERT_STORE_READONLY_FLAG;
    }
    let store =
        unsafe { CertOpenStore(CERT_STORE_PROV_SYSTEM_W, 0, 0, flags, root.as_ptr().cast()) };
    if store.is_null() {
        let code = unsafe { GetLastError() };
        Err(TrustError::new(
            "trust-store-open-failed",
            operation,
            store_name,
            Some(code),
            format!("cannot open {store_name}: 0x{code:08X}"),
        ))
    } else {
        Ok(store)
    }
}

fn parse_thumbprint(value: &str) -> Result<[u8; 20], TrustError> {
    let compact: String = value
        .chars()
        .filter(|value| !matches!(value, ':' | '-' | ' '))
        .collect();
    if compact.len() != 40 || !compact.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(error(
            "invalid-thumbprint",
            "parse",
            None,
            "expected 40 hexadecimal SHA-1 digits",
        ));
    }
    let mut out = [0_u8; 20];
    for (index, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&compact[index * 2..index * 2 + 2], 16)
            .map_err(|_| error("invalid-thumbprint", "parse", None, "invalid SHA-1 digit"))?;
    }
    Ok(out)
}

fn error(
    code: &'static str,
    operation: &'static str,
    os_error: Option<u32>,
    detail: impl Into<String>,
) -> TrustError {
    TrustError::new(code, operation, CURRENT_USER_ROOT, os_error, detail)
}
