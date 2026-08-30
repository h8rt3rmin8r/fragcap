// SPDX-License-Identifier: Apache-2.0

//! Narrow Windows certificate helpers shared by Deep Capture and doctor.

use std::path::Path;

use windows_sys::Win32::Foundation::{GetLastError, SetLastError, CRYPT_E_NOT_FOUND};
use windows_sys::Win32::Security::Cryptography::{
    CertCloseStore, CertCreateCertificateContext, CertEnumCertificatesInStore,
    CertFreeCertificateContext, CertGetCertificateContextProperty, CertOpenStore,
    CryptStringToBinaryA, CERT_CONTEXT, CERT_SHA1_HASH_PROP_ID, CERT_STORE_OPEN_EXISTING_FLAG,
    CERT_STORE_PROV_SYSTEM_W, CERT_STORE_READONLY_FLAG, CRYPT_STRING_BASE64HEADER,
    PKCS_7_ASN_ENCODING, X509_ASN_ENCODING,
};

pub(crate) const CURRENT_USER_ROOT: &str = "CurrentUser/Root";
pub(crate) const LOCAL_MACHINE_ROOT: &str = "LocalMachine/Root";

pub(crate) fn file_thumbprint(path: &Path) -> Result<String, String> {
    file_der_and_thumbprint(path).map(|(_, thumbprint)| thumbprint)
}

pub(crate) fn file_der_and_thumbprint(path: &Path) -> Result<(Vec<u8>, String), String> {
    let encoded = std::fs::read(path)
        .map_err(|error| format!("could not read certificate {}: {error}", path.display()))?;
    let der = if encoded.starts_with(b"-----BEGIN CERTIFICATE-----") {
        decode_pem(&encoded).map_err(|error| format!("{}: {error}", path.display()))?
    } else {
        encoded
    };
    // SAFETY: `der` remains live for the call and supplies exactly the byte count
    // passed. CryptoAPI validates the encoding and returns an owned context.
    let certificate = unsafe {
        CertCreateCertificateContext(
            X509_ASN_ENCODING | PKCS_7_ASN_ENCODING,
            der.as_ptr(),
            der.len() as u32,
        )
    };
    if certificate.is_null() {
        return Err(format!(
            "Windows could not parse certificate {}",
            path.display()
        ));
    }
    let result = context_thumbprint(certificate).map(|thumbprint| (der, thumbprint));
    // SAFETY: this function owns the context returned by CertCreateCertificateContext.
    unsafe { CertFreeCertificateContext(certificate) };
    result
}

fn decode_pem(encoded: &[u8]) -> Result<Vec<u8>, &'static str> {
    let length = u32::try_from(encoded.len()).map_err(|_| "certificate file is too large")?;
    let mut decoded_len = 0_u32;
    // SAFETY: `encoded` is live for the call; a null destination requests the
    // required output size and every optional out pointer is intentionally null.
    let measured = unsafe {
        CryptStringToBinaryA(
            encoded.as_ptr(),
            length,
            CRYPT_STRING_BASE64HEADER,
            core::ptr::null_mut(),
            &mut decoded_len,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )
    };
    if measured == 0 || decoded_len == 0 {
        return Err("Windows could not decode the PEM certificate");
    }
    let mut der = vec![0_u8; decoded_len as usize];
    // SAFETY: `der` has the exact writable capacity reported by the first call.
    let decoded = unsafe {
        CryptStringToBinaryA(
            encoded.as_ptr(),
            length,
            CRYPT_STRING_BASE64HEADER,
            der.as_mut_ptr(),
            &mut decoded_len,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )
    };
    if decoded == 0 {
        return Err("Windows could not decode the PEM certificate");
    }
    der.truncate(decoded_len as usize);
    Ok(der)
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
