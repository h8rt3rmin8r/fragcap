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

/// The subscriber set and the startup backlog, under one lock so that a publish
/// and a subscribe cannot interleave to drop an event between them.
struct Fan {
    subscribers: Vec<Sender<ProcessEvent>>,
    /// Events published before the first subscriber attached.
    ///
    /// The consumer begins delivering the moment `OpenTraceW` returns, which is
    /// inside `EtwWatcher::start`, before the caller can subscribe: a caller
    /// subscribes only after `start` returns. Without this backlog, every event
    /// in that window, including the whole startup burst the snapshot is meant
    /// to overlap with, would be published to nobody and lost, recreating the
    /// gap the subscribe-before-snapshot ordering exists to prevent. The
    /// backlog holds those events until the first subscriber claims them.
    ///
    /// Unbounded, like the live channel and for the same reason (module docs):
    /// losing a start event costs a subtree, so there is no discard path here
    /// to count under P-4.
    backlog: Vec<ProcessEvent>,
    /// Set once the first subscriber has drained the backlog. After that the
    /// backlog is neither filled nor delivered again: later subscribers see
    /// live events only, which is the trait's documented per-subscriber
    /// contract. The bridge is for the first consumer, which is the tree.
    backlog_claimed: bool,
}

/// Shared between the consumer thread's callback and the watcher.
pub(super) struct Fanout {
    fan: Mutex<Fan>,
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
            fan: Mutex::new(Fan {
                subscribers: Vec::new(),
                backlog: Vec::new(),
                backlog_claimed: false,
            }),
            unparsed: AtomicU64::new(0),
            rundown_ignored: AtomicU64::new(0),
        }
    }

    pub(super) fn subscribe(&self) -> Receiver<ProcessEvent> {
        let (tx, rx) = channel();
        if let Ok(mut fan) = self.fan.lock() {
            // The first subscriber inherits every event published since the
            // consumer opened. Draining under the same lock that publish holds
            // means an event published concurrently is either in the backlog
            // (drained here) or sent to the now-registered subscriber, never
            // lost between the two.
            if !fan.backlog_claimed {
                for event in std::mem::take(&mut fan.backlog) {
                    let _ = tx.send(event);
                }
                fan.backlog_claimed = true;
            }
            fan.subscribers.push(tx);
        }
        rx
    }

    fn publish(&self, event: ProcessEvent) {
        if let Ok(mut fan) = self.fan.lock() {
            if !fan.backlog_claimed {
                fan.backlog.push(event.clone());
            }
            // A subscriber whose receiver has been dropped is removed. Nothing
            // is discarded by that: an event nobody is listening for was never
            // received, which is not the same as one received and thrown away.
            fan.subscribers.retain(|s| s.send(event.clone()).is_ok());
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

#[cfg(test)]
mod tests {
    use super::*;

    fn started(pid: u32) -> ProcessEvent {
        ProcessEvent::started(
            pid,
            1,
            "C:\\a.exe",
            "a.exe",
            Timestamp::from_nanos(pid as i64),
        )
    }

    #[test]
    fn the_first_subscriber_inherits_events_published_before_it_attached() {
        // This is the startup gap: the consumer begins publishing the moment
        // the trace opens, inside `EtwWatcher::start`, before any caller can
        // subscribe. Those events must reach the first subscriber rather than
        // being lost.
        let fanout = Fanout::new();
        fanout.publish(started(10));
        fanout.publish(started(20));

        let rx = fanout.subscribe();
        let got: Vec<_> = rx.try_iter().map(|e| e.pid()).collect();
        assert_eq!(
            got,
            vec![10, 20],
            "the backlog reaches the first subscriber"
        );
    }

    #[test]
    fn a_late_second_subscriber_gets_live_events_only() {
        // The backlog bridges the gap for the first consumer, which is the
        // tree. A second subscriber sees only what is published after it, which
        // is the trait's per-subscriber contract, so that two subscribers
        // feeding one tree cannot double-apply the startup burst.
        let fanout = Fanout::new();
        fanout.publish(started(10));
        let first = fanout.subscribe();
        let second = fanout.subscribe();

        fanout.publish(started(20));

        let a: Vec<_> = first.try_iter().map(|e| e.pid()).collect();
        let b: Vec<_> = second.try_iter().map(|e| e.pid()).collect();
        assert_eq!(
            a,
            vec![10, 20],
            "the first sees the backlog and the live event"
        );
        assert_eq!(b, vec![20], "the second sees the live event only");
    }

    #[test]
    fn with_no_subscriber_the_backlog_retains_rather_than_discards() {
        // Unbounded, like the live channel: losing a start event costs a
        // subtree, so nothing here is dropped while waiting for a subscriber.
        let fanout = Fanout::new();
        for pid in 0..1000 {
            fanout.publish(started(pid));
        }
        let rx = fanout.subscribe();
        assert_eq!(rx.try_iter().count(), 1000);
    }
}
