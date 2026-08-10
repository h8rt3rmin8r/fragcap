// SPDX-License-Identifier: Apache-2.0

//! The Windows named-pipe streaming transport (specification section 14.2).
//!
//! This is the transport that makes live analysis work with no additional
//! software: Wireshark opens `\\.\pipe\<name>` directly as a capture interface.
//! A listener thread creates a pipe instance, blocks in `ConnectNamedPipe`, and
//! on connection hands the instance to the streaming sink as a consumer, then
//! loops to create the next instance (`PIPE_UNLIMITED_INSTANCES`). This is the
//! classic multi-client named-pipe server; it needs no overlapped IO.
//!
//! P-1: these are ordinary pipe and file calls (`CreateNamedPipeW`,
//! `ConnectNamedPipe`, `WriteFile`). Nothing here intercepts, injects, or opens
//! a process handle, and `cargo xtask lint` forbids the transmit and injection
//! call names outright.

use std::ffi::c_void;
use std::io::{self, Write};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_PIPE_CONNECTED, HANDLE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, WriteFile, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, PIPE_ACCESS_OUTBOUND,
};
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE,
    PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};
use windows_sys::Win32::System::IO::CancelIoEx;

use super::{Acceptor, ConnShutdown, Connection, Stopper};

/// `GENERIC_READ`, taken as a literal so the transport does not pull the
/// `Win32_System_SystemServices` feature for one constant. Used only by the
/// throwaway client that unblocks a pending `ConnectNamedPipe` at shutdown.
const GENERIC_READ: u32 = 0x8000_0000;

/// Per-instance pipe buffer, in bytes, in each direction.
const PIPE_BUFFER: u32 = 64 * 1024;

/// The pipe prefix every name lives under.
fn pipe_path(name: &str) -> String {
    format!(r"\\.\pipe\{name}")
}

/// A UTF-16, null-terminated copy of `s`, for the wide Win32 calls.
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// One connected pipe instance. The handle is closed exactly once, when the
/// last of the writer and the shutdown handle is dropped.
struct PipeInstance {
    handle: HANDLE,
    closed: AtomicBool,
}

// The handle is an OS resource, safe to move and share across threads; the
// close-once flag makes concurrent drop safe.
unsafe impl Send for PipeInstance {}
unsafe impl Sync for PipeInstance {}

impl PipeInstance {
    fn write_chunk(&self, buf: &[u8]) -> io::Result<usize> {
        let mut written: u32 = 0;
        let len = buf.len().min(u32::MAX as usize) as u32;
        // SAFETY: `handle` is a valid pipe instance until this instance is
        // dropped; `buf` is valid for `len` bytes; `written` is a valid out
        // pointer; the overlapped pointer is null for a synchronous write.
        let ok = unsafe {
            WriteFile(
                self.handle,
                buf.as_ptr() as *const c_void,
                len,
                &mut written,
                ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(written as usize)
    }

    fn unblock(&self) {
        // Cancels any pending `WriteFile` so a writer parked on a stalled
        // reader returns and its thread exits. Unlike `DisconnectNamedPipe`,
        // this does not discard bytes already accepted into the pipe buffer, so
        // a consumer that kept up still delivers its stream: `CloseHandle` on
        // drop lets the client read the remaining buffered data before end of
        // stream.
        // SAFETY: `handle` is valid until this instance is dropped.
        unsafe {
            let _ = CancelIoEx(self.handle, ptr::null());
        }
    }
}

impl Drop for PipeInstance {
    fn drop(&mut self) {
        if !self.closed.swap(true, Ordering::AcqRel) {
            // SAFETY: closed exactly once, guarded by the swap above.
            unsafe {
                let _ = CloseHandle(self.handle);
            }
        }
    }
}

/// The `Write` half of a pipe consumer.
struct PipeWriter(Arc<PipeInstance>);

impl Write for PipeWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write_chunk(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        // A byte-mode `WriteFile` hands its bytes to the pipe buffer directly;
        // there is nothing buffered in process to flush.
        Ok(())
    }
}

/// The shutdown half: disconnect the instance to unblock a stuck writer.
struct PipeShutdown(Arc<PipeInstance>);

impl ConnShutdown for PipeShutdown {
    fn shutdown(&self) {
        self.0.unblock();
    }
}

/// A named-pipe server that yields each connected instance as a consumer.
pub struct NamedPipeAcceptor {
    name: String,
    wide_path: Vec<u16>,
    stop: Arc<AtomicBool>,
    next_ordinal: AtomicU64,
}

impl NamedPipeAcceptor {
    /// Validate the name by creating and immediately closing a probe instance,
    /// so a malformed name surfaces before capture starts.
    pub fn bind(name: &str) -> io::Result<Self> {
        let wide_path = wide(&pipe_path(name));
        let probe = create_instance(&wide_path)?;
        // SAFETY: `probe` is a valid handle we just created and now own.
        unsafe {
            let _ = CloseHandle(probe);
        }
        Ok(NamedPipeAcceptor {
            name: name.to_string(),
            wide_path,
            stop: Arc::new(AtomicBool::new(false)),
            next_ordinal: AtomicU64::new(0),
        })
    }
}

/// Create one pipe instance, or return the OS error.
fn create_instance(wide_path: &[u16]) -> io::Result<HANDLE> {
    // SAFETY: `wide_path` is a valid null-terminated UTF-16 string; a null
    // security attributes pointer requests the default descriptor.
    let handle = unsafe {
        CreateNamedPipeW(
            wide_path.as_ptr(),
            PIPE_ACCESS_OUTBOUND,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
            PIPE_UNLIMITED_INSTANCES,
            PIPE_BUFFER,
            PIPE_BUFFER,
            0,
            ptr::null(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    Ok(handle)
}

impl Acceptor for NamedPipeAcceptor {
    fn accept(&mut self) -> Option<Connection> {
        loop {
            if self.stop.load(Ordering::Acquire) {
                return None;
            }
            let handle = match create_instance(&self.wide_path) {
                Ok(handle) => handle,
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    continue;
                }
            };
            // SAFETY: `handle` is a valid instance; a null overlapped pointer
            // makes this a synchronous, blocking connect.
            let connected = unsafe { ConnectNamedPipe(handle, ptr::null_mut()) };
            // SAFETY: reading the thread's last error is always sound.
            let already = connected == 0 && unsafe { GetLastError() } == ERROR_PIPE_CONNECTED;
            if connected == 0 && !already {
                // SAFETY: closing the handle we just created and own.
                unsafe {
                    let _ = CloseHandle(handle);
                }
                continue;
            }
            if self.stop.load(Ordering::Acquire) {
                // The connection that unblocked us is the shutdown poke (or a
                // real client arriving during shutdown); drop it.
                // SAFETY: `handle` is valid and owned here.
                unsafe {
                    let _ = DisconnectNamedPipe(handle);
                    let _ = CloseHandle(handle);
                }
                return None;
            }
            let instance = Arc::new(PipeInstance {
                handle,
                closed: AtomicBool::new(false),
            });
            let ordinal = self.next_ordinal.fetch_add(1, Ordering::Relaxed);
            let id = format!(r"pipe:\\.\pipe\{}#{ordinal}", self.name);
            return Some(Connection {
                id,
                writer: Box::new(PipeWriter(Arc::clone(&instance))),
                shutdown: Box::new(PipeShutdown(instance)),
            });
        }
    }

    fn stopper(&self) -> Stopper {
        let stop = Arc::clone(&self.stop);
        let wide_path = self.wide_path.clone();
        Arc::new(move || {
            stop.store(true, Ordering::Release);
            // Poke the pipe: a throwaway client unblocks a pending
            // `ConnectNamedPipe` so the acceptor loop can observe the flag.
            // SAFETY: `wide_path` is a valid null-terminated UTF-16 string;
            // null security attributes and template handle are permitted.
            let client = unsafe {
                CreateFileW(
                    wide_path.as_ptr(),
                    GENERIC_READ,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    ptr::null(),
                    OPEN_EXISTING,
                    0,
                    0,
                )
            };
            if client != INVALID_HANDLE_VALUE {
                // SAFETY: closing the client handle we just opened.
                unsafe {
                    let _ = CloseHandle(client);
                }
            }
        })
    }

    fn describe(&self) -> String {
        format!(r"pipe:\\.\pipe\{}", self.name)
    }
}
