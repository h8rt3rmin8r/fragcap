// SPDX-License-Identifier: Apache-2.0

//! Image names, by query-only process enumeration.
//!
//! Specification section 19.2 permits query-only process enumeration, and
//! constitution P-1 requires that any process handle state its access rights
//! explicitly at the call site. This module opens no process handle at all, so
//! there is nothing to state.
//!
//! That is a deliberate choice rather than a happy accident. The obvious
//! alternative, `OpenProcess` followed by `QueryFullProcessImageNameW`, would
//! also comply: `PROCESS_QUERY_LIMITED_INFORMATION` carries no memory rights,
//! and naming it at the call site is exactly what P-1 asks for. But P-1's
//! requirement exists because a handle request is a thing a reviewer has to
//! check, and this path removes the thing to check rather than documenting it.
//! `cargo xtask lint` can then assert that fragcap names no process-opening
//! call anywhere, which is a stronger and cheaper guarantee than asserting that
//! every one it does name requests the right rights.
//!
//! The snapshot handle this does take is a handle to a snapshot object, not to
//! any process. It is closed on every path, including the error ones.
//!
//! # A note for slice S11
//!
//! `PROCESSENTRY32W` carries `th32ParentProcessID` alongside the image name, so
//! the ancestry S11 needs is available here. S11 should still not use it: a
//! snapshot of a tree says who a process's parent is *now*, and specification
//! section 10 wants creation-time ancestry, which is a record of how the tree
//! was built. A parent that has exited leaves an identifier that may already
//! have been reused. Noted so the enumeration does not have to be rediscovered.

use std::collections::HashMap;
use std::sync::Arc;

use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

use crate::seam::ProcessNamer;

/// Names processes by enumerating them.
///
/// Enumerates once per call rather than once per identifier, which is what
/// requirement FR-033a asks for: the caller hands over every identifier the
/// socket table reported, and the result becomes part of the published
/// snapshot. Resolving lazily would put this enumeration on the acquisition
/// path at the start of a session, when the most sockets are opening at once.
#[derive(Clone, Debug, Default)]
pub struct ToolhelpNamer;

impl ToolhelpNamer {
    pub fn new() -> Self {
        ToolhelpNamer
    }

    /// Every process the machine reports, as identifier and image name.
    ///
    /// Returns an empty map on failure rather than an error. A name that cannot
    /// be resolved is a missing name and not a failure: requirement FR-032 and
    /// constitution P-9 require the attribution be produced carrying the
    /// observed identifier regardless, because the identifier is what was
    /// observed.
    fn enumerate() -> HashMap<u32, Arc<str>> {
        let mut out = HashMap::new();

        // A snapshot object, not a process. Nothing here requests any right
        // against any target.
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if snapshot == INVALID_HANDLE_VALUE {
            return out;
        }

        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..unsafe { std::mem::zeroed() }
        };

        // SAFETY: `snapshot` is valid and `entry.dwSize` is set, which is the
        // platform's documented precondition for both calls.
        let mut ok = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
        while ok {
            out.insert(entry.th32ProcessID, image_name(&entry.szExeFile));
            ok = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
        }

        // Closed on every path, including the loop ending in a failure.
        unsafe { CloseHandle(snapshot) };
        out
    }
}

/// The image name from a fixed-size, null-terminated wide buffer.
///
/// A name that is not valid UTF-16 is reported with replacement characters
/// rather than discarded. It was observed, and P-9 says what was observed is
/// what gets reported; a process with an undecodable name still has an
/// identifier and still owns sockets.
fn image_name(buf: &[u16; 260]) -> Arc<str> {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Arc::from(String::from_utf16_lossy(&buf[..end]))
}

impl ProcessNamer for ToolhelpNamer {
    fn names(&mut self, pids: &[u32]) -> HashMap<u32, Arc<str>> {
        let all = Self::enumerate();
        // Only what was asked for. The socket table's identifiers are a small
        // fraction of a machine's processes, and carrying the rest into the
        // published snapshot would grow it for nothing.
        pids.iter()
            .filter_map(|pid| all.get(pid).map(|n| (*pid, Arc::clone(n))))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_image_name_stops_at_the_terminator() {
        let mut buf = [0u16; 260];
        for (i, c) in "eso64.exe".encode_utf16().enumerate() {
            buf[i] = c;
        }
        // Trailing garbage past the terminator must not appear.
        buf[20] = b'X' as u16;
        assert_eq!(&*image_name(&buf), "eso64.exe");
    }

    #[test]
    fn an_empty_buffer_yields_an_empty_name() {
        assert_eq!(&*image_name(&[0u16; 260]), "");
    }

    #[test]
    fn an_unterminated_buffer_uses_the_whole_of_it() {
        let buf = [b'a' as u16; 260];
        assert_eq!(image_name(&buf).len(), 260);
    }

    #[test]
    fn an_undecodable_name_is_reported_rather_than_discarded() {
        // An unpaired surrogate. The process exists and owns sockets; reporting
        // nothing for it would discard an observation.
        let mut buf = [0u16; 260];
        buf[0] = 0xD800;
        buf[1] = b'.' as u16;
        let name = image_name(&buf);
        assert!(!name.is_empty());
        assert!(name.ends_with('.'));
    }

    // Tier 2 by specification section 25.2. Needs a Windows machine.
    #[test]
    #[ignore = "tier 2: enumerates the machine's real process list"]
    fn the_real_enumeration_names_this_process() {
        let me = std::process::id();
        let mut n = ToolhelpNamer::new();
        let named = n.names(&[me]);
        let name = named.get(&me).expect("this process is in the list");
        assert!(name.to_ascii_lowercase().ends_with(".exe"), "got {name:?}");
    }

    #[test]
    #[ignore = "tier 2: enumerates the machine's real process list"]
    fn an_identifier_that_does_not_exist_resolves_to_no_name() {
        let mut n = ToolhelpNamer::new();
        assert!(n.names(&[u32::MAX]).is_empty());
    }
}
