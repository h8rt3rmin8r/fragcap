// SPDX-License-Identifier: Apache-2.0

//! Exact process-object receipts for pre-authorized managed Deep Capture shutdown.
//!
//! A PID or image name is never a control authority. The narrow query and wait
//! handle retained here pins one Windows process object across later checks.

use std::io;
use std::path::PathBuf;

#[cfg(windows)]
mod windows {
    use super::*;
    use std::ptr;
    use std::sync::Arc;
    use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, HANDLE, WAIT_TIMEOUT};
    use windows_sys::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows_sys::Win32::Security::{GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER};
    use windows_sys::Win32::Storage::FileSystem::SYNCHRONIZE;
    use windows_sys::Win32::System::Memory::LocalFree;
    use windows_sys::Win32::System::RemoteDesktop::ProcessIdToSessionId;
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcessId, GetProcessTimes, OpenProcess, OpenProcessToken,
        QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject,
        PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE, WAIT_OBJECT_0,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE,
    };

    #[derive(Debug)]
    struct Handle(HANDLE);

    // SAFETY: the owned handle is immutable, used only through thread-safe
    // Windows process queries, and closed once the final Arc is dropped.
    unsafe impl Send for Handle {}
    unsafe impl Sync for Handle {}

    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }

    #[derive(Clone, Debug)]
    pub struct ProcessIdentity {
        pub pid: u32,
        pub created: u64,
        pub path: PathBuf,
        pub user_sid: String,
        pub session: u32,
        handle: Arc<Handle>,
    }

    fn bad(message: &'static str) -> io::Error {
        io::Error::new(io::ErrorKind::PermissionDenied, message)
    }

    fn opened(pid: u32, rights: u32) -> io::Result<Handle> {
        let observation = PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE;
        if rights != observation && rights != observation | PROCESS_TERMINATE {
            return Err(bad("unsupported managed process access rights"));
        }
        let value = unsafe { OpenProcess(rights, 0, pid) };
        if value == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(Handle(value))
        }
    }

    fn creation(handle: HANDLE) -> io::Result<u64> {
        let mut created = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut exited = created;
        let mut kernel = created;
        let mut user = created;
        if unsafe { GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user) }
            == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok((u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime))
    }

    fn image_path(handle: HANDLE) -> io::Result<PathBuf> {
        let mut value = vec![0_u16; 32768];
        let mut length = value.len() as u32;
        if unsafe { QueryFullProcessImageNameW(handle, 0, value.as_mut_ptr(), &mut length) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let path = PathBuf::from(String::from_utf16_lossy(&value[..length as usize]));
        path.canonicalize()
    }

    fn token_user(handle: HANDLE) -> io::Result<String> {
        let mut token = 0;
        if unsafe { OpenProcessToken(handle, TOKEN_QUERY, &mut token) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let token = Handle(token);
        let mut needed = 0;
        unsafe { GetTokenInformation(token.0, TokenUser, ptr::null_mut(), 0, &mut needed) };
        if needed == 0 || needed > 65536 {
            return Err(io::Error::last_os_error());
        }
        let mut data = vec![0_usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
        if unsafe {
            GetTokenInformation(
                token.0,
                TokenUser,
                data.as_mut_ptr().cast(),
                needed,
                &mut needed,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let sid = unsafe { (*data.as_ptr().cast::<TOKEN_USER>()).User.Sid };
        let mut text = ptr::null_mut();
        if unsafe { ConvertSidToStringSidW(sid, &mut text) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut len = 0;
        while len < 65536 && unsafe { *text.add(len) } != 0 {
            len += 1;
        }
        let result = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(text, len) });
        unsafe { LocalFree(text as isize) };
        Ok(result)
    }

    fn process_session(pid: u32) -> io::Result<u32> {
        let mut session = 0;
        if unsafe { ProcessIdToSessionId(pid, &mut session) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(session)
        }
    }

    fn same_identity(left: &ProcessIdentity, right: &ProcessIdentity) -> bool {
        left.pid == right.pid
            && left.created == right.created
            && left
                .path
                .to_string_lossy()
                .eq_ignore_ascii_case(&right.path.to_string_lossy())
            && left.user_sid == right.user_sid
            && left.session == right.session
    }

    impl ProcessIdentity {
        /// Acquire only query and synchronize rights for one exact process.
        pub fn observe(pid: u32) -> io::Result<Self> {
            let handle = Arc::new(opened(
                pid,
                PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE,
            )?);
            let created = creation(handle.0)?;
            let path = image_path(handle.0)?;
            let user_sid = token_user(handle.0)?;
            let session = process_session(pid)?;
            Ok(Self {
                pid,
                created,
                path,
                user_sid,
                session,
                handle,
            })
        }

        /// The current process identity is a comparison anchor, not a target.
        pub fn current() -> io::Result<Self> {
            let pid = unsafe { GetCurrentProcessId() };
            Self::observe(pid)
        }

        pub fn same_user_session(&self, other: &Self) -> bool {
            self.user_sid == other.user_sid && self.session == other.session
        }

        pub fn created_unix_nanos(&self) -> i128 {
            (i128::from(self.created) - 116_444_736_000_000_000) * 100
        }

        pub fn exited(&self) -> io::Result<bool> {
            match unsafe { WaitForSingleObject(self.handle.0, 0) } {
                WAIT_OBJECT_0 => Ok(true),
                WAIT_TIMEOUT => Ok(false),
                _ => Err(io::Error::last_os_error()),
            }
        }

        pub fn request_graceful_close(&self) -> io::Result<usize> {
            if self.exited()? {
                return Ok(0);
            }
            let live = Self::observe(self.pid)?;
            if !same_identity(self, &live) {
                return Err(bad(
                    "managed process identity changed before graceful close",
                ));
            }
            struct Context {
                pid: u32,
                sent: usize,
            }
            unsafe extern "system" fn callback(window: isize, state: isize) -> i32 {
                let context = unsafe { &mut *(state as *mut Context) };
                let mut pid = 0;
                unsafe { GetWindowThreadProcessId(window, &mut pid) };
                if pid == context.pid && unsafe { PostMessageW(window, WM_CLOSE, 0, 0) } != 0 {
                    context.sent += 1;
                }
                1
            }
            let mut context = Context {
                pid: self.pid,
                sent: 0,
            };
            if unsafe { EnumWindows(Some(callback), (&mut context as *mut Context) as isize) } == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(context.sent)
        }

        /// Reopen with the exact named terminate right, revalidate, then issue
        /// a force request. The caller must independently observe exit.
        pub fn force_close(&self) -> io::Result<()> {
            if self.exited()? {
                return Ok(());
            }
            let control = opened(
                self.pid,
                PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE | PROCESS_TERMINATE,
            )?;
            let live = Self {
                pid: self.pid,
                created: creation(control.0)?,
                path: image_path(control.0)?,
                user_sid: token_user(control.0)?,
                session: process_session(self.pid)?,
                handle: Arc::new(control),
            };
            if !same_identity(self, &live) {
                return Err(bad("managed process identity changed before termination"));
            }
            if unsafe { TerminateProcess(live.handle.0, 1) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }
    }
}

#[cfg(windows)]
pub use windows::ProcessIdentity;

#[cfg(not(windows))]
#[derive(Clone, Debug)]
pub struct ProcessIdentity {
    pub pid: u32,
    pub created: u64,
    pub path: PathBuf,
    pub user_sid: String,
    pub session: u32,
}

#[cfg(not(windows))]
impl ProcessIdentity {
    pub fn observe(_pid: u32) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "managed shutdown requires Windows",
        ))
    }
    pub fn current() -> io::Result<Self> {
        Self::observe(0)
    }
    pub fn same_user_session(&self, other: &Self) -> bool {
        self.user_sid == other.user_sid && self.session == other.session
    }
    pub fn created_unix_nanos(&self) -> i128 {
        0
    }
    pub fn exited(&self) -> io::Result<bool> {
        Self::observe(self.pid).map(|_| false)
    }
    pub fn request_graceful_close(&self) -> io::Result<usize> {
        Self::observe(self.pid).map(|_| 0)
    }
    pub fn force_close(&self) -> io::Result<()> {
        Self::observe(self.pid).map(|_| ())
    }
}
