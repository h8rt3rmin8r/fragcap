// SPDX-License-Identifier: Apache-2.0

#![windows_subsystem = "windows"]

use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    net::IpAddr,
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const TEST: &str = "package_public_direct_route_exchanges_external_bytes_with_native_defaults";

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
    if args.next().is_some() || bind_ip.is_loopback() {
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
    let mut summary = File::create(&output)?;
    if mode == "preflight" {
        writeln!(
            summary,
            "S211 preflight ready: test, package binary, client, and bind IP"
        )?;
        return Ok(());
    }
    if mode != "run" {
        return Err(io::Error::other("unknown mode"));
    }
    let mut failed = false;
    for attempt in 1..=3 {
        let attempt_log = output.with_extension(format!("attempt-{attempt}.log"));
        let stderr = File::create(&attempt_log)?;
        let mut child = Command::new(&test)
            .args([
                TEST,
                "--exact",
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("FRAGCAP_S205_BINARY", &binary)
            .env("FRAGCAP_S205_CLIENT", &client)
            .env("FRAGCAP_S211_BIND_IP", bind_ip.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(stderr))
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()?;
        let deadline = Instant::now() + Duration::from_secs(180);
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break Some(status);
            }
            if Instant::now() >= deadline {
                child.kill()?;
                child.wait()?;
                writeln!(
                    summary,
                    "attempt {attempt}: timed out; inspect owned child cleanup"
                )?;
                failed = true;
                break None;
            }
            thread::sleep(Duration::from_millis(50));
        };
        if !status.is_some_and(|status| status.success()) {
            writeln!(
                summary,
                "attempt {attempt}: failed; log={}",
                attempt_log.display()
            )?;
            failed = true;
        } else {
            writeln!(
                summary,
                "attempt {attempt}: passed; log={}",
                attempt_log.display()
            )?;
        }
        summary.flush()?;
        if failed {
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
