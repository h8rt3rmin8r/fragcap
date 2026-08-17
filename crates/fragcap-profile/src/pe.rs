// SPDX-License-Identifier: Apache-2.0

//! A bounded reader for a Windows PE binary's version-information strings (slice
//! S053), for the `pe-version-string` detection signature kind.
//!
//! It confirms the file is a PE image (the DOS `MZ` stub and the `PE\0\0`
//! signature the `e_lfanew` offset points at), locates the `VS_VERSION_INFO`
//! version-information block, and reads the UTF-16LE string fields it carries
//! (`CompanyName`, `ProductName`, `FileDescription`, and the structural keys). A
//! `pe-version-string` signature matches its needle against those strings.
//!
//! # Passive
//!
//! It reads the bytes of a file the operator already has on disk. It opens no
//! process handle, reads no process memory, and calls no operating-system API
//! (constitution P-1); a wrong offset yields no match, never a wrong process. It is
//! deliberately not a full resource-directory walk: extracting the version block's
//! string fields is all detection needs, and navigating the `.rsrc` tree would add
//! fragility for no detection benefit. What it does not find, it reports as absent,
//! never guessed (P-9).

/// The UTF-16LE encoding of the version block's root key, `VS_VERSION_INFO`. The
/// block is located by this signature.
fn vs_version_info_marker() -> Vec<u8> {
    utf16le("VS_VERSION_INFO")
}

/// Encode an ASCII/Unicode string as UTF-16LE bytes (no terminator).
fn utf16le(s: &str) -> Vec<u8> {
    s.encode_utf16().flat_map(|u| u.to_le_bytes()).collect()
}

/// Whether `bytes` is a PE image: the `MZ` stub, an `e_lfanew` in range, and the
/// `PE\0\0` signature at that offset.
fn is_pe(bytes: &[u8]) -> bool {
    if bytes.len() < 0x40 || &bytes[0..2] != b"MZ" {
        return false;
    }
    let e_lfanew =
        u32::from_le_bytes([bytes[0x3C], bytes[0x3D], bytes[0x3E], bytes[0x3F]]) as usize;
    bytes.len() >= e_lfanew + 4 && &bytes[e_lfanew..e_lfanew + 4] == b"PE\0\0"
}

/// The UTF-16LE string fields of the file's `VS_VERSIONINFO` version block, or an
/// empty vector if the file is not a PE image or carries no version block.
///
/// Strings shorter than two characters are dropped as noise. The structural keys
/// (`VS_VERSION_INFO`, `StringFileInfo`, and the like) are returned alongside the
/// value fields; a detection needle is a product name, which does not collide with
/// them in practice.
pub fn version_strings(bytes: &[u8]) -> Vec<String> {
    if !is_pe(bytes) {
        return Vec::new();
    }
    let marker = vs_version_info_marker();
    let Some(start) = find_subslice(bytes, &marker) else {
        return Vec::new();
    };
    utf16le_strings(&bytes[start..])
}

/// Find the first index of `needle` in `haystack`.
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Extract every run of two or more printable UTF-16LE code units from `bytes`,
/// splitting on NUL (the version block's field terminator) and on any non-printable
/// unit. A lone unpaired trailing byte is ignored.
fn utf16le_strings(bytes: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    let mut current: Vec<u16> = Vec::new();
    let mut i = 0;
    while i + 1 < bytes.len() {
        let unit = u16::from_le_bytes([bytes[i], bytes[i + 1]]);
        i += 2;
        let printable = unit >= 0x20 && unit != 0xFFFF;
        if printable {
            current.push(unit);
        } else {
            flush(&mut current, &mut out);
        }
    }
    flush(&mut current, &mut out);
    out
}

/// Decode and push the accumulated units as a string if it is at least two units,
/// then clear the accumulator.
fn flush(current: &mut Vec<u16>, out: &mut Vec<String>) {
    if current.len() >= 2 {
        if let Ok(s) = String::from_utf16(current) {
            out.push(s);
        }
    }
    current.clear();
}

#[cfg(test)]
/// Test-support fixture builders. Fixtures are generated, not hand-made
/// (`fixtures/README.md`): this builds a minimal PE image carrying one
/// version-information string so the `pe-version-string` matcher is exercised
/// without a real binary.
pub mod tests_support {
    use super::utf16le;

    /// Build a minimal PE image whose version block carries `key` = `value`.
    ///
    /// It is not a loadable executable and carries no real resource directory; it is
    /// the smallest byte sequence [`super::version_strings`] accepts: the `MZ` stub,
    /// an `e_lfanew` pointing at a `PE\0\0` signature, and a `VS_VERSION_INFO` block
    /// with the key and value as UTF-16LE fields.
    pub fn minimal_pe_with_version_string(key: &str, value: &str) -> Vec<u8> {
        let mut buf = vec![0u8; 0x40];
        buf[0] = b'M';
        buf[1] = b'Z';
        // e_lfanew at 0x3C points at the PE signature we place at 0x40.
        buf[0x3C..0x40].copy_from_slice(&0x40u32.to_le_bytes());
        buf.extend_from_slice(b"PE\0\0");
        // The version block: the root key, then the key and value as NUL-terminated
        // UTF-16LE fields, the shape version_strings reads.
        buf.extend_from_slice(&utf16le("VS_VERSION_INFO"));
        buf.extend_from_slice(&[0, 0]);
        buf.extend_from_slice(&utf16le(key));
        buf.extend_from_slice(&[0, 0]);
        buf.extend_from_slice(&utf16le(value));
        buf.extend_from_slice(&[0, 0]);
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_non_pe_file_yields_no_strings() {
        assert!(version_strings(b"not a pe file at all").is_empty());
    }

    #[test]
    fn a_pe_with_no_version_block_yields_no_strings() {
        let mut buf = vec![0u8; 0x40];
        buf[0] = b'M';
        buf[1] = b'Z';
        buf[0x3C..0x40].copy_from_slice(&0x40u32.to_le_bytes());
        buf.extend_from_slice(b"PE\0\0");
        assert!(version_strings(&buf).is_empty());
    }

    #[test]
    fn a_version_block_string_is_read_back() {
        let bytes = tests_support::minimal_pe_with_version_string("ProductName", "Frostbite");
        let strings = version_strings(&bytes);
        assert!(
            strings.iter().any(|s| s == "Frostbite"),
            "read version strings: {strings:?}"
        );
    }
}
