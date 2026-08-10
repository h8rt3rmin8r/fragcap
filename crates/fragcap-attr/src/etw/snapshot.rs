// SPDX-License-Identifier: Apache-2.0

//! The startup snapshot: what was already running when fragcap began.
//!
//! One call, and the handle it takes is to a snapshot object rather than to any
//! process. `CreateToolhelp32Snapshot` yields every process with its
//! identifier, its recorded parent, and its executable file name, and opens no
//! handle on a target at all.
//!
//! **No process handle is opened here, and that is stronger than opening a
//! narrow one.** Slice S11 first wrote this module with
//! `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` and `GetProcessTimes`, to
//! obtain a start time for each running process. That complies with P-1: the
//! right carries no memory access and naming it at the call site is exactly
//! what the principle asks for. It was withdrawn during integration with slice
//! S10, whose `platform::toolhelp` module had already made the stronger
//! argument and backed it with a lint: P-1's requirement exists because a
//! handle request is a thing a reviewer has to check, and opening nothing
//! removes the thing to check rather than documenting it. `cargo xtask lint`
//! now asserts that fragcap names no process-opening call anywhere, which is
//! cheaper to trust than asserting that every one it does name asks for the
//! right rights.
//!
//! The cost is real and small: a process found already running has no start
//! time. `ProcessRecord::started` is `None`, which requirement FR-009 permits
//! and FR-024 gives a defined meaning in resolution, so nothing downstream has
//! to guess. It is never replaced by the session's own start, which would be
//! the comfortable untruth P-9 forbids and would sort a process running for
//! hours after one created a second ago.
//!
//! **No command line either.** Reading a running process's command line means
//! reading its process environment block, which requires `PROCESS_VM_READ`.
//! That right is denylisted, so a process the snapshot finds records
//! `CommandLine::Unavailable` and the tree says so. Slice S11 research R-3
//! records the alternatives: the object-model query supplies command lines
//! without a memory right, but brings a component-object dependency and costs
//! over a second, for a field that only ever applies to processes which started
//! before fragcap did and which are therefore not members of the launcher chain
//! fragcap was started to watch.
//!
//! # Why this is not slice S10's enumeration
//!
//! `platform::toolhelp` answers a different question with the same call. It
//! maps identifier to image name for the socket table's benefit, behind the
//! `socket-table` feature, and deliberately ignores `th32ParentProcessID`
//! because attribution has no use for ancestry. This needs the parent, records
//! it as [`Ancestry::Snapshot`](fragcap_core::Ancestry::Snapshot) rather than
//! trusting it, and lives behind `etw`. Sharing one module would tie two
//! independent features together to save a dozen lines.

use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

use fragcap_core::process::ProcessRecord;

/// Enumerate the running processes.
///
/// Returns an empty vector rather than an error when the platform refuses. A
/// snapshot that could not be taken is not a run that must fail: the event
/// stream is already subscribed by this point and carries everything created
/// from here on, which is the part that matters for a launcher chain.
pub(super) fn take() -> Vec<ProcessRecord> {
    // SAFETY: the flags are constants and the second argument is documented as
    // ignored for a system-wide process snapshot.
    let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snap == INVALID_HANDLE_VALUE || snap == 0 {
        return Vec::new();
    }

    let mut out = Vec::new();
    // SAFETY: `PROCESSENTRY32W` is a plain C structure with no pointers, so an
    // all-zero value is valid; `dwSize` is then set as the API requires.
    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

    // SAFETY: `snap` is a live snapshot handle and `entry` has its `dwSize` set.
    let mut ok = unsafe { Process32FirstW(snap, &mut entry) } != 0;
    while ok {
        // The parent is recorded, not trusted. Specification section 5.3 says a
        // parent identifier read from a running process may name an unrelated
        // process or nothing at all, which is why the tree marks a node built
        // from this as `Ancestry::Snapshot` rather than `Observed`.
        out.push(ProcessRecord::new(
            entry.th32ProcessID,
            entry.th32ParentProcessID,
            wide_to_string(&entry.szExeFile),
        ));

        // SAFETY: as above.
        ok = unsafe { Process32NextW(snap, &mut entry) } != 0;
    }

    // SAFETY: `snap` came from `CreateToolhelp32Snapshot` and is not used again.
    // Closed on every path, including the error ones above.
    unsafe {
        CloseHandle(snap);
    }
    out
}

/// A fixed-size null-terminated wide buffer as a `String`.
fn wide_to_string(buf: &[u16]) -> String {
    let end = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_buffer_stops_at_its_terminator() {
        let mut buf = [0u16; 8];
        for (i, c) in "a.exe".encode_utf16().enumerate() {
            buf[i] = c;
        }
        assert_eq!(wide_to_string(&buf), "a.exe");
    }

    #[test]
    fn a_full_wide_buffer_with_no_terminator_is_read_whole() {
        let buf: Vec<u16> = "abcd".encode_utf16().collect();
        assert_eq!(wide_to_string(&buf), "abcd");
    }

    #[test]
    fn the_snapshot_finds_this_process() {
        // Checkable without elevation: enumeration needs no privilege, so the
        // test process must appear.
        let me = std::process::id();
        let found = take();
        assert!(!found.is_empty(), "enumeration returned nothing");
        let mine = found.iter().find(|r| r.pid == me).expect("this process");
        assert!(mine.image.to_lowercase().ends_with(".exe"));
    }

    #[test]
    fn the_snapshot_reports_a_parent_for_this_process() {
        // Recorded, not trusted. The value may already be stale, which is the
        // whole reason the tree keeps snapshot ancestry apart from observed.
        let me = std::process::id();
        let found = take();
        let mine = found.iter().find(|r| r.pid == me).expect("this process");
        assert_ne!(mine.parent, me, "a process is not its own parent");
    }

    #[test]
    fn no_snapshot_record_claims_a_start_time_or_a_command_line() {
        // Both would need a process handle, and this module opens none.
        // Requirement FR-009 permits the first absence and FR-036 the second.
        for r in take() {
            assert!(r.started.is_none(), "pid {} claimed a start time", r.pid);
            assert!(
                !r.command_line.is_available(),
                "pid {} claimed a command line",
                r.pid
            );
        }
    }
}
