// SPDX-License-Identifier: Apache-2.0

#![windows_subsystem = "windows"]

use std::{
    env,
    fs::{self, File},
    io::{self, Read, Write},
    net::{IpAddr, SocketAddr},
    os::windows::{io::AsRawHandle, process::CommandExt},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const TEST: &str = "package_public_direct_route_exchanges_external_bytes_with_native_defaults";
const HANDLE: &str = "s205_controlled_route_client";

mod job {
    use super::*;
    use std::{ffi::c_void, mem, ptr};

    type Handle = *mut c_void;
    const BASIC_ACCOUNTING_INFORMATION: i32 = 1;
    const EXTENDED_LIMIT_INFORMATION: i32 = 9;
    const KILL_ON_JOB_CLOSE: u32 = 0x2000;

    #[repr(C)]
    #[derive(Default)]
    struct BasicLimit {
        process_time: i64,
        job_time: i64,
        flags: u32,
        minimum_working_set: usize,
        maximum_working_set: usize,
        active_process_limit: u32,
        affinity: usize,
        priority_class: u32,
        scheduling_class: u32,
    }

    #[repr(C)]
    #[derive(Default)]
    struct IoCounters {
        read_operations: u64,
        write_operations: u64,
        other_operations: u64,
        read_bytes: u64,
        write_bytes: u64,
        other_bytes: u64,
    }

    #[repr(C)]
    #[derive(Default)]
    struct ExtendedLimit {
        basic: BasicLimit,
        io: IoCounters,
        process_memory: usize,
        job_memory: usize,
        peak_process_memory: usize,
        peak_job_memory: usize,
    }

    #[repr(C)]
    #[derive(Default)]
    struct BasicAccounting {
        total_user_time: i64,
        total_kernel_time: i64,
        period_user_time: i64,
        period_kernel_time: i64,
        total_page_faults: u32,
        total_processes: u32,
        active_processes: u32,
        terminated_processes: u32,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateJobObjectW(attributes: Handle, name: *const u16) -> Handle;
        fn SetInformationJobObject(job: Handle, class: i32, info: *const c_void, size: u32) -> i32;
        fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
        fn TerminateJobObject(job: Handle, code: u32) -> i32;
        fn QueryInformationJobObject(
            job: Handle,
            class: i32,
            info: *mut c_void,
            size: u32,
            returned: *mut u32,
        ) -> i32;
        fn CloseHandle(handle: Handle) -> i32;
    }

    pub(super) struct Job(Handle);

    impl Job {
        pub(super) fn new() -> io::Result<Self> {
            let handle = unsafe { CreateJobObjectW(ptr::null_mut(), ptr::null()) };
            if handle.is_null() {
                return Err(io::Error::last_os_error());
            }
            let job = Self(handle);
            let mut limits = ExtendedLimit::default();
            limits.basic.flags = KILL_ON_JOB_CLOSE;
            let installed = unsafe {
                SetInformationJobObject(
                    job.0,
                    EXTENDED_LIMIT_INFORMATION,
                    (&limits as *const ExtendedLimit).cast(),
                    mem::size_of::<ExtendedLimit>() as u32,
                )
            };
            if installed == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(job)
        }

        pub(super) fn assign(&self, child: &Child) -> io::Result<()> {
            if unsafe { AssignProcessToJobObject(self.0, child.as_raw_handle().cast()) } == 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }

        pub(super) fn terminate(&self) -> io::Result<()> {
            if unsafe { TerminateJobObject(self.0, 1) } == 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }

        pub(super) fn active_processes(&self) -> io::Result<u32> {
            let mut accounting = BasicAccounting::default();
            if unsafe {
                QueryInformationJobObject(
                    self.0,
                    BASIC_ACCOUNTING_INFORMATION,
                    (&mut accounting as *mut BasicAccounting).cast(),
                    mem::size_of::<BasicAccounting>() as u32,
                    ptr::null_mut(),
                )
            } == 0
            {
                Err(io::Error::last_os_error())
            } else {
                Ok(accounting.active_processes)
            }
        }
    }

    impl Drop for Job {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }
}

fn native_command(binary: &Path) -> Command {
    let mut command = Command::new(binary);
    command.creation_flags(CREATE_NO_WINDOW);
    for name in [
        "FRAGCAP_LOCAL_DB",
        "FRAGCAP_SESSION_DIR",
        "FRAGCAP_STORAGE_ROOT",
        "FRAGCAP_PROFILE_DIR",
    ] {
        command.env_remove(name);
    }
    command
}

fn assert_target_absent(binary: &Path) -> io::Result<()> {
    let output = native_command(binary)
        .args(["--json", "targets", "show", HANDLE])
        .output()?;
    if !output.status.success()
        || !String::from_utf8_lossy(&output.stdout).contains("\"status\":\"no_match\"")
    {
        return Err(io::Error::other(
            "controlled target handle is already occupied",
        ));
    }
    Ok(())
}

fn run_worker(
    test: &Path,
    binary: &Path,
    client: &Path,
    bind_ip: IpAddr,
    log: &Path,
    external_peer: Option<SocketAddr>,
) -> io::Result<()> {
    let mut go = [0];
    io::stdin().read_exact(&mut go)?;
    if go != [1] {
        return Err(io::Error::other("job assignment gate was not released"));
    }
    let stderr = File::create(log)?;
    let mut command = Command::new(test);
    command
        .args([
            TEST,
            "--exact",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("FRAGCAP_S205_BINARY", binary)
        .env("FRAGCAP_S205_CLIENT", client)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr))
        .creation_flags(CREATE_NO_WINDOW);
    if let Some(peer) = external_peer {
        command
            .env_remove("FRAGCAP_S211_BIND_IP")
            .env("FRAGCAP_S205_PEER", peer.to_string());
    } else {
        command
            .env("FRAGCAP_S211_BIND_IP", bind_ip.to_string())
            .env_remove("FRAGCAP_S205_PEER");
    }
    let mut child = command.spawn()?;
    fs::write(log.with_extension("pid"), child.id().to_string())?;
    if child.wait()?.success() {
        Ok(())
    } else {
        Err(io::Error::other("physical acceptance test failed"))
    }
}

fn cleanup_owned_state(binary: &Path, client: &Path, log: &Path) -> io::Result<()> {
    if let Ok(pid) = fs::read_to_string(log.with_extension("pid")) {
        let parent = client
            .parent()
            .ok_or_else(|| io::Error::other("client has no parent"))?;
        let gate = parent.join(format!("s205-gate-{}.signal", pid.trim()));
        for path in [&gate, &gate.with_extension("release")] {
            match fs::remove_file(path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
            if path.exists() {
                return Err(io::Error::other("controlled gate residue remains"));
            }
        }
    }
    let removal = native_command(binary)
        .args(["targets", "remove", HANDLE])
        .output()?;
    if !removal.status.success() {
        return Err(io::Error::other("controlled target removal failed"));
    }
    assert_target_absent(binary)
}

fn run() -> io::Result<()> {
    let mut args = env::args_os().skip(1);
    let mode = args
        .next()
        .ok_or_else(|| io::Error::other("missing mode"))?;
    let test = PathBuf::from(
        args.next()
            .ok_or_else(|| io::Error::other("missing test"))?,
    );
    let binary = PathBuf::from(
        args.next()
            .ok_or_else(|| io::Error::other("missing binary"))?,
    );
    let client = PathBuf::from(
        args.next()
            .ok_or_else(|| io::Error::other("missing client"))?,
    );
    let bind_ip: IpAddr = args
        .next()
        .ok_or_else(|| io::Error::other("missing bind IP"))?
        .to_string_lossy()
        .parse()
        .map_err(|_| io::Error::other("invalid bind IP"))?;
    let output = PathBuf::from(
        args.next()
            .ok_or_else(|| io::Error::other("missing output"))?,
    );
    let external_peer: Option<SocketAddr> = args
        .next()
        .map(|value| value.to_string_lossy().parse())
        .transpose()
        .map_err(|_| io::Error::other("invalid external peer"))?;
    if args.next().is_some()
        || bind_ip.is_loopback()
        || external_peer.is_some_and(|peer| peer.ip().is_loopback())
    {
        return Err(io::Error::other("extra argument or loopback bind IP"));
    }
    for path in [&test, &binary, &client] {
        if !path.is_file() {
            return Err(io::Error::other(format!(
                "required file absent: {}",
                path.display()
            )));
        }
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    if mode == "probe-grandchild" {
        thread::sleep(Duration::from_secs(30));
        return Ok(());
    }
    if mode == "probe-child" {
        let mut go = [0];
        io::stdin().read_exact(&mut go)?;
        if go != [1] {
            return Err(io::Error::other("job probe gate mismatch"));
        }
        let bind_ip_arg = std::ffi::OsString::from(bind_ip.to_string());
        let grandchild = Command::new(env::current_exe()?)
            .args([
                std::ffi::OsStr::new("probe-grandchild"),
                test.as_os_str(),
                binary.as_os_str(),
                client.as_os_str(),
                bind_ip_arg.as_os_str(),
                output.as_os_str(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()?;
        fs::write(output.with_extension("pid"), grandchild.id().to_string())?;
        thread::sleep(Duration::from_secs(30));
        return Ok(());
    }
    if mode == "probe" {
        let bind_ip_arg = std::ffi::OsString::from(bind_ip.to_string());
        let job = job::Job::new()?;
        let mut child = Command::new(env::current_exe()?)
            .args([
                std::ffi::OsStr::new("probe-child"),
                test.as_os_str(),
                binary.as_os_str(),
                client.as_os_str(),
                bind_ip_arg.as_os_str(),
                output.as_os_str(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()?;
        job.assign(&child)?;
        child
            .stdin
            .take()
            .expect("probe gate stdin")
            .write_all(&[1])?;
        let pid_path = output.with_extension("pid");
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if let Ok(value) = fs::read_to_string(&pid_path) {
                value
                    .trim()
                    .parse::<u32>()
                    .map_err(|_| io::Error::other("bad probe PID"))?;
                break;
            }
            if Instant::now() >= deadline {
                return Err(io::Error::other("job probe grandchild did not start"));
            }
            thread::sleep(Duration::from_millis(10));
        }
        if job.active_processes()? < 2 {
            return Err(io::Error::other("job probe descendant was not contained"));
        }
        job.terminate()?;
        child.wait()?;
        let deadline = Instant::now() + Duration::from_secs(3);
        while job.active_processes()? != 0 {
            if Instant::now() >= deadline {
                return Err(io::Error::other("job probe descendant remains active"));
            }
            thread::sleep(Duration::from_millis(10));
        }
        drop(job);
        fs::write(&output, "S211 job and descendant termination verified\n")?;
        return Ok(());
    }
    if mode == "worker" || mode == "worker-external" {
        if (mode == "worker-external") != external_peer.is_some() {
            return Err(io::Error::other("external worker peer mismatch"));
        }
        return run_worker(&test, &binary, &client, bind_ip, &output, external_peer);
    }
    assert_target_absent(&binary)?;
    let mut summary = File::create(&output)?;
    if mode == "preflight" {
        writeln!(
            summary,
            "S211 preflight ready: test, package binary, client, and bind IP"
        )?;
        return Ok(());
    }
    if mode != "run"
        && mode != "run-refusal"
        && mode != "run-no-payload"
        && mode != "run-all"
        && mode != "run-all-external"
        && mode != "run-refusal-external"
    {
        return Err(io::Error::other("unknown mode"));
    }
    if (mode == "run-all-external" || mode == "run-refusal-external") != external_peer.is_some() {
        return Err(io::Error::other("external runner peer mismatch"));
    }
    let session_root = if external_peer.is_some() || mode == "run-no-payload" {
        let root = PathBuf::from(
            env::var_os("FRAGCAP_S211_SESSION_ROOT")
                .ok_or_else(|| io::Error::other("S211 session root is missing"))?,
        );
        if !root.is_absolute() {
            return Err(io::Error::other("S211 session root is not absolute"));
        }
        let root = root.canonicalize()?;
        let scratch = client
            .parent()
            .ok_or_else(|| io::Error::other("client has no parent"))?
            .join("s211-physical")
            .canonicalize()?;
        if root.parent() != Some(scratch.as_path())
            || !root
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("session-run-"))
        {
            return Err(io::Error::other("S211 session root is outside scratch"));
        }
        Some(root)
    } else {
        None
    };
    let mut failed = false;
    let attempts = if mode == "run-refusal" || mode == "run-refusal-external" {
        1
    } else if mode == "run-no-payload" {
        1
    } else if mode == "run-all" || mode == "run-all-external" {
        4
    } else {
        3
    };
    for attempt in 1..=attempts {
        let session_dir = session_root
            .as_ref()
            .map(|root| -> io::Result<PathBuf> {
                let nonce = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| io::Error::other("system time is before Unix epoch"))?
                    .as_nanos();
                let dir = root.join(format!("attempt-{}-{nonce}-{attempt}", std::process::id()));
                fs::create_dir(&dir)?;
                Ok(dir)
            })
            .transpose()?;
        let refusal = mode == "run-refusal"
            || mode == "run-refusal-external"
            || (mode == "run-all" && attempt == 4)
            || (mode == "run-all-external" && attempt == 1);
        let no_payload = mode == "run-no-payload";
        let case_name = if refusal {
            "refusal"
        } else if no_payload {
            "no-payload"
        } else {
            "transfer"
        };
        let attempt_log = output.with_extension(format!("attempt-{attempt}.log"));
        let bind_ip_arg = std::ffi::OsString::from(bind_ip.to_string());
        let job = job::Job::new()?;
        let mut worker_command = Command::new(env::current_exe()?);
        worker_command
            .args([
                std::ffi::OsStr::new(
                    if mode == "run-all-external" || mode == "run-refusal-external" {
                        "worker-external"
                    } else {
                        "worker"
                    },
                ),
                test.as_os_str(),
                binary.as_os_str(),
                client.as_os_str(),
                bind_ip_arg.as_os_str(),
                attempt_log.as_os_str(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW);
        if let Some(peer) = external_peer {
            worker_command.arg(peer.to_string());
        }
        if let Some(dir) = &session_dir {
            worker_command.env("FRAGCAP_S211_SESSION_DIR", dir);
        }
        if refusal {
            worker_command.env("FRAGCAP_S211_REFUSAL", "1");
        } else {
            worker_command.env_remove("FRAGCAP_S211_REFUSAL");
        }
        if no_payload {
            worker_command.env("FRAGCAP_S211_NO_PAYLOAD", "1");
        } else {
            worker_command.env_remove("FRAGCAP_S211_NO_PAYLOAD");
        }
        let mut worker = worker_command.spawn()?;
        if let Err(error) = job.assign(&worker) {
            worker.kill()?;
            worker.wait()?;
            return Err(error);
        }
        worker
            .stdin
            .take()
            .expect("job gate stdin")
            .write_all(&[1])?;
        let deadline = Instant::now() + Duration::from_secs(180);
        let status = loop {
            if let Some(status) = worker.try_wait()? {
                break Some(status);
            }
            if Instant::now() >= deadline {
                job.terminate()?;
                worker.wait()?;
                writeln!(
                    summary,
                    "attempt {attempt} ({}): timed out; contained descendants terminated",
                    case_name
                )?;
                failed = true;
                break None;
            }
            thread::sleep(Duration::from_millis(50));
        };
        drop(job);
        cleanup_owned_state(&binary, &client, &attempt_log)?;
        if !status.is_some_and(|status| status.success()) {
            writeln!(
                summary,
                "attempt {attempt} ({}): failed; log={}",
                case_name,
                attempt_log.display()
            )?;
            failed = true;
        } else {
            writeln!(
                summary,
                "attempt {attempt} ({}): passed; log={}",
                case_name,
                attempt_log.display()
            )?;
        }
        summary.flush()?;
        if failed && mode != "run-all-external" {
            break;
        }
    }
    if failed {
        Err(io::Error::other("physical acceptance did not complete"))
    } else {
        Ok(())
    }
}

fn main() {
    if run().is_err() {
        std::process::exit(1);
    }
}
