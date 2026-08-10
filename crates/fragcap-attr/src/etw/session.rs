// SPDX-License-Identifier: Apache-2.0

//! The trace session fragcap creates for itself.
//!
//! Never `NT Kernel Logger`. That session exists once per machine, so
//! contending for it would make fragcap fail whenever any other tool is
//! tracing, and taking it by force would make fragcap the tool that silently
//! breaks the operator's other instrumentation. Windows 8 introduced the system
//! logger mode, under which several sessions may each carry system providers
//! concurrently, and specification section 6.1's platform floor is Windows 10.
//!
//! Nothing here stops, reconfigures, or reuses a session fragcap did not
//! create. `ERROR_ALREADY_EXISTS` is reported, not worked around.

use std::sync::atomic::{AtomicU64, Ordering};

use windows_sys::core::GUID;
use windows_sys::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_SUCCESS};
use windows_sys::Win32::System::Diagnostics::Etw::{
    ControlTraceW, EnableTraceEx2, StartTraceW, ENABLE_TRACE_PARAMETERS,
    EVENT_CONTROL_CODE_DISABLE_PROVIDER, EVENT_CONTROL_CODE_ENABLE_PROVIDER,
    EVENT_TRACE_CONTROL_QUERY, EVENT_TRACE_CONTROL_STOP, EVENT_TRACE_PROPERTIES,
    EVENT_TRACE_REAL_TIME_MODE, EVENT_TRACE_SYSTEM_LOGGER_MODE, TRACE_LEVEL_INFORMATION,
    WNODE_FLAG_TRACED_GUID,
};

use super::WatcherError;

/// The system process provider, enabled on a system-logger-mode session.
///
/// Distinct from the Process MOF class GUID the events themselves carry, which
/// is in [`super::consumer`]. Confusing the two is easy and produces a session
/// that starts, enables nothing, and delivers no events, which looks like a
/// machine on which nothing happened.
/// `{151f55dc-467d-471f-83b5-5f889d46ff66}`, written out field by field because
/// this binding line has no `GUID::from_u128`.
const SYSTEM_PROCESS_PROVIDER: GUID = GUID {
    data1: 0x151f_55dc,
    data2: 0x467d,
    data3: 0x471f,
    data4: [0x83, 0xb5, 0x5f, 0x88, 0x9d, 0x46, 0xff, 0x66],
};

/// `SYSTEM_PROCESS_KW_GENERAL`: process start and end events.
const KW_GENERAL: u64 = 0x0000_0000_0000_0001;

/// Timestamps in the event header are system time, not the performance counter.
///
/// The default for a real-time session is the performance counter, which is
/// monotonic and has no relationship to the wall clock, and therefore none to
/// the timestamps a capture driver puts on packets. Leaving it at the default
/// would produce process events that cannot be placed against the traffic they
/// explain, which is the entire reason for collecting them.
const CLIENT_CONTEXT_SYSTEM_TIME: u32 = 2;

/// Room for the logger name and the (unused) log file name, after the fixed
/// structure. Both are counted in `Wnode.BufferSize`.
const NAME_BYTES: usize = 512;

/// The properties structure the ETW control calls take, with room after it for
/// the logger name they write at `LoggerNameOffset`.
///
/// A single `#[repr(C)]` type rather than a `Vec<u8>` cast so that the
/// allocation carries the alignment `EVENT_TRACE_PROPERTIES` requires. A
/// `Vec<u8>` guarantees only byte alignment, and a custom global allocator is
/// permitted to hand back a merely byte-aligned address, which would make the
/// reference formed from the cast undefined behaviour. Giving the buffer this
/// type moves the alignment guarantee to the compiler. The trailing bytes are
/// contiguous with the structure and hold the name, exactly as the API expects.
#[repr(C)]
struct TraceProps {
    props: EVENT_TRACE_PROPERTIES,
    tail: [u8; NAME_BYTES],
}

impl TraceProps {
    /// A zeroed buffer with `BufferSize` and `LoggerNameOffset` set. All-zero is
    /// a valid `TraceProps`: it is plain data with no invariant a bit pattern
    /// could break.
    fn boxed() -> Box<TraceProps> {
        // SAFETY: `TraceProps` is `repr(C)` over integer and array fields, so an
        // all-zero value is valid and initialized.
        let mut b: Box<TraceProps> = Box::new(unsafe { std::mem::zeroed() });
        b.props.Wnode.BufferSize = std::mem::size_of::<TraceProps>() as u32;
        b.props.LoggerNameOffset = std::mem::size_of::<EVENT_TRACE_PROPERTIES>() as u32;
        b
    }

    /// A pointer to the properties structure, aligned by construction.
    fn as_ptr(b: &mut TraceProps) -> *mut EVENT_TRACE_PROPERTIES {
        &mut b.props as *mut EVENT_TRACE_PROPERTIES
    }
}

/// A running trace session, stopped on drop.
pub struct Session {
    /// A session handle. Plain `u64` on this binding line.
    handle: u64,
    name: Vec<u16>,
    /// The last loss counts a successful query returned. Held so that a later
    /// query which fails does not make an incomplete trace look lossless: the
    /// last known figures are returned instead of zero. Zero before any
    /// successful read means "none observed yet", not "known lossless".
    last_events: AtomicU64,
    last_buffers: AtomicU64,
}

impl std::fmt::Debug for Session {
    // Hand-written because the platform handle is a raw binding with no
    // `Debug`. The value itself is an opaque kernel handle and is not printed.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("name", &String::from_utf16_lossy(&self.name))
            .finish_non_exhaustive()
    }
}

// The handle is an opaque integer and the buffers are owned. Nothing here is
// bound to the thread that created it, and `ProcessTrace` runs on another.
unsafe impl Send for Session {}

impl Session {
    /// Start a session by this name and enable the process provider on it.
    pub fn start(name: &str) -> Result<Self, WatcherError> {
        let mut wide: Vec<u16> = name.encode_utf16().collect();
        wide.push(0);
        if wide.len() * 2 > NAME_BYTES {
            return Err(WatcherError::SessionUnavailable {
                code: 0,
                detail: format!("session name is longer than {NAME_BYTES} bytes"),
            });
        }

        let mut props = TraceProps::boxed();
        props.props.Wnode.Flags = WNODE_FLAG_TRACED_GUID;
        props.props.Wnode.ClientContext = CLIENT_CONTEXT_SYSTEM_TIME;
        props.props.LogFileMode = EVENT_TRACE_REAL_TIME_MODE | EVENT_TRACE_SYSTEM_LOGGER_MODE;
        // Deliberately zero: this session never writes a file. A real-time
        // session with a log file name would put captured process command
        // lines on disk without the operator having asked for it.
        props.props.LogFileNameOffset = 0;

        let mut handle: u64 = 0;

        // SAFETY: `handle` and `props` are live for the call, the name is
        // null-terminated, and `BufferSize` describes the buffer's real length.
        // `StartTraceW` copies the name into the buffer at `LoggerNameOffset`,
        // which is why the buffer carries the trailing bytes. The buffer need
        // not outlive the call: the returned handle is what the session is
        // driven by afterwards.
        let rc = unsafe { StartTraceW(&mut handle, wide.as_ptr(), TraceProps::as_ptr(&mut props)) };
        if rc != ERROR_SUCCESS {
            return Err(if rc == ERROR_ACCESS_DENIED {
                // The one condition with a remedy the operator can act on, so
                // it gets its own variant rather than a code to look up.
                WatcherError::NotElevated
            } else {
                WatcherError::SessionUnavailable {
                    code: rc,
                    detail: describe(rc),
                }
            });
        }

        let session = Session {
            handle,
            name: wide,
            last_events: AtomicU64::new(0),
            last_buffers: AtomicU64::new(0),
        };

        // SAFETY: the session started, so the handle is valid. The provider
        // GUID is a constant and the parameter pointer is null, which the
        // documentation permits.
        let rc = unsafe {
            EnableTraceEx2(
                session.handle,
                &SYSTEM_PROCESS_PROVIDER,
                EVENT_CONTROL_CODE_ENABLE_PROVIDER,
                TRACE_LEVEL_INFORMATION as u8,
                KW_GENERAL,
                0,
                0,
                std::ptr::null::<ENABLE_TRACE_PARAMETERS>() as *const _,
            )
        };
        if rc != ERROR_SUCCESS {
            // `session` drops here and stops what it started.
            return Err(WatcherError::ProviderUnavailable { code: rc });
        }

        Ok(session)
    }

    /// The session name, for the consumer to open.
    pub fn name(&self) -> &[u16] {
        &self.name
    }

    /// Ask the platform what it has lost.
    ///
    /// Relayed, never accumulated: the query returns counters from the start of
    /// the session, so fragcap copies a cumulative value rather than summing
    /// deltas, and there is no arithmetic in which an alteration could hide.
    ///
    /// A failed query returns the last figures a query did succeed in reading,
    /// not zero. Reporting zero losses because the question could not be asked
    /// would be the comfortable untruth P-9 forbids, and it would make a
    /// transient failure erase a loss the trace really suffered. Before any
    /// successful read the cached figures are zero, which is honest: none has
    /// been observed.
    pub fn lost(&self) -> (u64, u64) {
        let mut props = TraceProps::boxed();

        // SAFETY: the handle is valid for the lifetime of `self`, and `props`
        // is live and correctly sized and aligned for the call.
        let rc = unsafe {
            ControlTraceW(
                self.handle,
                std::ptr::null(),
                TraceProps::as_ptr(&mut props),
                EVENT_TRACE_CONTROL_QUERY,
            )
        };
        if rc != ERROR_SUCCESS {
            return (
                self.last_events.load(Ordering::Relaxed),
                self.last_buffers.load(Ordering::Relaxed),
            );
        }

        let events = props.props.EventsLost as u64;
        let buffers = props.props.RealTimeBuffersLost as u64;
        self.last_events.store(events, Ordering::Relaxed);
        self.last_buffers.store(buffers, Ordering::Relaxed);
        (events, buffers)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // A trace session outlives the process that started it. Leaving one
        // running is a resource leak the operator cannot see and cannot easily
        // find, so this is not best-effort cleanup: it is the only place the
        // session is ever stopped.

        // SAFETY: the handle was returned by a successful `StartTraceW` and has
        // not been closed. Failures are ignored because there is nothing left
        // to do about them at drop time.
        unsafe {
            EnableTraceEx2(
                self.handle,
                &SYSTEM_PROCESS_PROVIDER,
                EVENT_CONTROL_CODE_DISABLE_PROVIDER,
                0,
                0,
                0,
                0,
                std::ptr::null::<ENABLE_TRACE_PARAMETERS>() as *const _,
            );
        }

        let mut props = TraceProps::boxed();
        // SAFETY: the handle is valid and `props` is live and correctly sized
        // and aligned for the call.
        unsafe {
            ControlTraceW(
                self.handle,
                self.name.as_ptr(),
                TraceProps::as_ptr(&mut props),
                EVENT_TRACE_CONTROL_STOP,
            );
        }
    }
}

/// What to tell the operator about a start failure that is not privilege.
///
/// Specification section 26.4 requires an error to say what was attempted, what
/// happened, and what to do next. The platform's own code is always carried
/// alongside this, so a condition without a sentence here is still reportable.
fn describe(code: u32) -> String {
    const ERROR_ALREADY_EXISTS: u32 = 183;
    const ERROR_BAD_LENGTH: u32 = 24;
    const ERROR_NO_SYSTEM_RESOURCES: u32 = 1450;
    const ERROR_INVALID_PARAMETER: u32 = 87;

    match code {
        ERROR_ALREADY_EXISTS => {
            "a trace session by this name already exists. fragcap does not stop \
             or reuse a session it did not create; stop it, or choose another name."
                .into()
        }
        ERROR_NO_SYSTEM_RESOURCES => {
            "the machine is already running the maximum number of system trace \
             sessions. Stop another tracing tool and try again."
                .into()
        }
        ERROR_BAD_LENGTH | ERROR_INVALID_PARAMETER => {
            "the platform rejected the session parameters. This is a defect in \
             fragcap rather than in the environment; please report it."
                .into()
        }
        _ => "the platform refused to start the trace session.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_failure_code_has_something_to_say() {
        for code in [183u32, 24, 1450, 87, 5, 0, 9999] {
            let d = describe(code);
            assert!(!d.is_empty());
            // Section 26.4: what happened, and what to do next.
            assert!(d.ends_with('.'), "code {code}: {d}");
        }
    }

    #[test]
    fn a_session_name_longer_than_the_buffer_is_refused_before_any_call() {
        let long = "x".repeat(NAME_BYTES);
        match Session::start(&long) {
            Err(WatcherError::SessionUnavailable { code, .. }) => assert_eq!(code, 0),
            other => panic!("expected a refusal without a platform call, got {other:?}"),
        }
    }
}
