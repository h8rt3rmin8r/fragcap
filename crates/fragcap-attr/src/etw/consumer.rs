// SPDX-License-Identifier: Apache-2.0

//! Consuming the session, and fanning its events out to subscribers.
//!
//! `ProcessTrace` blocks until the trace is closed, so it runs on a thread of
//! its own and the callback publishes from there.
//!
//! The channel to subscribers is unbounded, and that is a decision rather than
//! an oversight. Specification section 12.4's bounded drop-oldest buffer is the
//! right shape for packets, which arrive faster than they can be written and
//! whose individual loss costs one packet. Process events arrive in the
//! thousands over a session and the loss of one start event costs a subtree.
//! There is therefore no discard path here to count, which is how P-4 is
//! satisfied for this stream; a bound with a counter would be counting the loss
//! the bound introduced.

use std::ffi::c_void;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use windows_sys::core::GUID;
use windows_sys::Win32::System::Diagnostics::Etw::{
    CloseTrace, OpenTraceW, ProcessTrace, EVENT_HEADER_FLAG_32_BIT_HEADER, EVENT_RECORD,
    EVENT_TRACE_LOGFILEW, PROCESS_TRACE_MODE_EVENT_RECORD, PROCESS_TRACE_MODE_REAL_TIME,
};

use fragcap_core::packet::Timestamp;
use fragcap_core::process::{CommandLine, ProcessEvent};

use super::record::{filetime_to_unix_nanos, parse_end, parse_start, PointerWidth};

/// The Process MOF class, which is what the events themselves are tagged with.
///
/// Distinct from the provider GUID passed to `EnableTraceEx2` in
/// [`super::session`]. One turns the events on; this one identifies them.
/// `{3d6fa8d0-fe05-11d0-9dda-00c04fd7ba7c}`, written out field by field because
/// this binding line has no `GUID::from_u128`.
const PROCESS_MOF_CLASS: GUID = GUID {
    data1: 0x3d6f_a8d0,
    data2: 0xfe05,
    data3: 0x11d0,
    data4: [0x9d, 0xda, 0x00, 0xc0, 0x4f, 0xd7, 0xba, 0x7c],
};

const OPCODE_START: u8 = 1;
const OPCODE_END: u8 = 2;
const OPCODE_DC_START: u8 = 3;
const OPCODE_DC_END: u8 = 4;

/// Shared between the consumer thread's callback and the watcher.
pub(super) struct Fanout {
    subscribers: Mutex<Vec<Sender<ProcessEvent>>>,
    /// Records that arrived tagged as process events and did not parse. Counted
    /// because a record fragcap could not read is an observation it failed to
    /// make, and P-4 does not let one go unmentioned merely because it was the
    /// parser rather than the kernel that lost it.
    pub(super) unparsed: AtomicU64,
    /// Rundown events, which the kernel emits at session start to describe
    /// processes that were already running.
    ///
    /// Ignored here because the startup snapshot already covers those
    /// processes, and because a rundown record carries the same stale parent
    /// identifier a running process does, so treating one as a creation event
    /// would claim creation-time ancestry it does not have. Counted rather than
    /// silently dropped.
    pub(super) rundown_ignored: AtomicU64,
}

impl Fanout {
    pub(super) fn new() -> Self {
        Fanout {
            subscribers: Mutex::new(Vec::new()),
            unparsed: AtomicU64::new(0),
            rundown_ignored: AtomicU64::new(0),
        }
    }

    pub(super) fn subscribe(&self) -> Receiver<ProcessEvent> {
        let (tx, rx) = channel();
        if let Ok(mut subs) = self.subscribers.lock() {
            subs.push(tx);
        }
        rx
    }

    fn publish(&self, event: ProcessEvent) {
        if let Ok(mut subs) = self.subscribers.lock() {
            // A subscriber whose receiver has been dropped is removed. Nothing
            // is discarded by that: an event nobody is listening for was never
            // received, which is not the same as one received and thrown away.
            subs.retain(|s| s.send(event.clone()).is_ok());
        }
    }
}

/// A live consumer. Dropping it closes the trace, which unblocks
/// `ProcessTrace` on the consumer thread.
pub(super) struct Consumer {
    /// A trace handle. Plain `u64` on this binding line, which predates the
    /// handle newtypes; `u64::MAX` is the invalid value.
    handle: u64,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Consumer {
    /// Open the named session and start consuming it on a new thread.
    pub(super) fn open(name: &[u16], fanout: Arc<Fanout>) -> Option<Self> {
        let mut logfile: EVENT_TRACE_LOGFILEW = unsafe { std::mem::zeroed() };
        logfile.LoggerName = name.as_ptr() as *mut u16;
        logfile.Anonymous1.ProcessTraceMode =
            PROCESS_TRACE_MODE_REAL_TIME | PROCESS_TRACE_MODE_EVENT_RECORD;
        logfile.Anonymous2.EventRecordCallback = Some(on_event);
        // The callback receives this back as `UserContext`. The `Arc` clone
        // below keeps it alive for exactly as long as the consumer thread runs.
        logfile.Context = Arc::as_ptr(&fanout) as *mut c_void;

        // SAFETY: `logfile` is fully initialized above and lives across the
        // call, and `name` is a null-terminated wide string owned by the
        // session, which outlives this consumer.
        let handle = unsafe { OpenTraceW(&mut logfile) };
        if handle == u64::MAX {
            return None;
        }

        let held = Arc::clone(&fanout);
        let raw = handle;
        let thread = std::thread::Builder::new()
            .name("fragcap-etw".into())
            .spawn(move || {
                // Keeps the context pointer valid for the whole blocking call.
                let _held = held;
                let h = raw;
                // SAFETY: the handle came from a successful `OpenTraceW` and is
                // closed only by `Consumer::drop`, which joins this thread
                // afterwards. `ProcessTrace` blocks until then.
                unsafe {
                    ProcessTrace(&h, 1, std::ptr::null(), std::ptr::null());
                }
            })
            .ok()?;

        Some(Consumer {
            handle,
            thread: Some(thread),
        })
    }
}

impl Drop for Consumer {
    fn drop(&mut self) {
        // SAFETY: the handle came from a successful `OpenTraceW`. Closing it is
        // what makes the blocking `ProcessTrace` return.
        unsafe {
            CloseTrace(self.handle);
        }
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn same_guid(a: &GUID, b: &GUID) -> bool {
    a.data1 == b.data1 && a.data2 == b.data2 && a.data3 == b.data3 && a.data4 == b.data4
}

/// The trace callback. Runs on the consumer thread, once per event.
///
/// # Safety
///
/// Called by the platform with a valid `EVENT_RECORD` whose `UserContext` is
/// the pointer installed in `Consumer::open`, which points at an `Arc<Fanout>`
/// held alive by the consumer thread for the duration of `ProcessTrace`.
unsafe extern "system" fn on_event(record: *mut EVENT_RECORD) {
    if record.is_null() {
        return;
    }
    let r = &*record;

    // `GUID` has no `PartialEq` in `windows-sys`, which is a raw-bindings
    // crate. Compared field by field rather than by transmuting to bytes.
    if !same_guid(&r.EventHeader.ProviderId, &PROCESS_MOF_CLASS) {
        return;
    }
    let ctx = r.UserContext as *const Fanout;
    if ctx.is_null() {
        return;
    }
    let fanout = &*ctx;

    let opcode = r.EventHeader.EventDescriptor.Opcode;
    if opcode == OPCODE_DC_START || opcode == OPCODE_DC_END {
        fanout.rundown_ignored.fetch_add(1, Ordering::Relaxed);
        return;
    }
    if opcode != OPCODE_START && opcode != OPCODE_END {
        return;
    }

    let width = if r.EventHeader.Flags & (EVENT_HEADER_FLAG_32_BIT_HEADER as u16) != 0 {
        PointerWidth::Four
    } else {
        PointerWidth::Eight
    };

    if r.UserData.is_null() || r.UserDataLength == 0 {
        fanout.unparsed.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let data = std::slice::from_raw_parts(r.UserData as *const u8, r.UserDataLength as usize);

    // The session sets its client context to system time, so this is a
    // FILETIME rather than a performance counter. See `session.rs`.
    let at = Timestamp::from_nanos(filetime_to_unix_nanos(r.EventHeader.TimeStamp));

    let version = r.EventHeader.EventDescriptor.Version;

    if opcode == OPCODE_START {
        match parse_start(data, version, width) {
            Some(f) => fanout.publish(ProcessEvent::Started {
                pid: f.pid,
                parent: f.parent,
                image: f.image.into(),
                command_line: match f.command_line {
                    Some(c) => CommandLine::observed(c),
                    None => CommandLine::Unavailable,
                },
                at,
            }),
            None => {
                fanout.unparsed.fetch_add(1, Ordering::Relaxed);
            }
        }
    } else {
        match parse_end(data, width) {
            Some(pid) => fanout.publish(ProcessEvent::Exited { pid, at }),
            None => {
                fanout.unparsed.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}
