// SPDX-License-Identifier: Apache-2.0

use std::process::Child;

#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessSample {
    pub available: bool,
    pub cpu_microseconds: u64,
    pub working_set_bytes: u64,
    pub private_bytes: u64,
}

impl ProcessSample {
    pub fn merge(&mut self, other: Self) {
        self.available |= other.available;
        self.cpu_microseconds = self.cpu_microseconds.max(other.cpu_microseconds);
        self.working_set_bytes = self.working_set_bytes.max(other.working_set_bytes);
        self.private_bytes = self.private_bytes.max(other.private_bytes);
    }
}

#[cfg(windows)]
pub fn sample(child: &Child) -> ProcessSample {
    use std::mem::{size_of, zeroed};
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::ProcessStatus::{
        K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows_sys::Win32::System::Threading::GetProcessTimes;

    let handle = child.as_raw_handle() as isize;
    let mut memory: PROCESS_MEMORY_COUNTERS_EX = unsafe { zeroed() };
    memory.cb = size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
    let memory_ok = unsafe {
        K32GetProcessMemoryInfo(
            handle,
            (&mut memory as *mut PROCESS_MEMORY_COUNTERS_EX).cast::<PROCESS_MEMORY_COUNTERS>(),
            memory.cb,
        )
    } != 0;
    let mut creation: FILETIME = unsafe { zeroed() };
    let mut exit: FILETIME = unsafe { zeroed() };
    let mut kernel: FILETIME = unsafe { zeroed() };
    let mut user: FILETIME = unsafe { zeroed() };
    let cpu_ok =
        unsafe { GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) } != 0;
    let filetime =
        |value: FILETIME| (u64::from(value.dwHighDateTime) << 32) | u64::from(value.dwLowDateTime);
    ProcessSample {
        available: memory_ok && cpu_ok,
        cpu_microseconds: if cpu_ok {
            filetime(kernel).saturating_add(filetime(user)) / 10
        } else {
            0
        },
        working_set_bytes: if memory_ok {
            memory.PeakWorkingSetSize as u64
        } else {
            0
        },
        private_bytes: if memory_ok {
            memory.PrivateUsage as u64
        } else {
            0
        },
    }
}

#[cfg(target_os = "linux")]
pub fn sample(child: &Child) -> ProcessSample {
    let status =
        std::fs::read_to_string(format!("/proc/{}/status", child.id())).unwrap_or_default();
    let stat = std::fs::read_to_string(format!("/proc/{}/stat", child.id())).unwrap_or_default();
    let working_set_bytes = status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .and_then(|value| value.split_whitespace().next())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0)
        .saturating_mul(1024);
    let clock_ticks = linux_clock_ticks();
    let cpu_ticks = stat
        .rfind(')')
        .and_then(|end| stat.get(end + 2..))
        .map(|fields| fields.split_whitespace().collect::<Vec<_>>())
        .and_then(|fields| {
            let user = fields.get(11)?.parse::<u64>().ok()?;
            let system = fields.get(12)?.parse::<u64>().ok()?;
            Some(user.saturating_add(system))
        });
    ProcessSample {
        available: !status.is_empty() && cpu_ticks.is_some() && clock_ticks > 0,
        cpu_microseconds: cpu_ticks.unwrap_or(0).saturating_mul(1_000_000) / clock_ticks.max(1),
        working_set_bytes,
        private_bytes: working_set_bytes,
    }
}

#[cfg(target_os = "linux")]
fn linux_clock_ticks() -> u64 {
    use std::ffi::{c_int, c_long};

    unsafe extern "C" {
        fn sysconf(name: c_int) -> c_long;
    }
    const SC_CLK_TCK: c_int = 2;
    let value = unsafe { sysconf(SC_CLK_TCK) };
    u64::try_from(value).unwrap_or(0)
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn sample(_: &Child) -> ProcessSample {
    ProcessSample::default()
}
