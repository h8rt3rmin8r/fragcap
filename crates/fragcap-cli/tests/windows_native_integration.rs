// SPDX-License-Identifier: Apache-2.0

//! Finite S129 probes driven only by `cargo xtask windows-integration`.

#![cfg(windows)]

use serde_json::Value;
use std::fs;
use std::net::{Ipv4Addr, Ipv6Addr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn staged_binary() -> PathBuf {
    PathBuf::from(std::env::var_os("FRAGCAP_WINDOWS_BINARY").expect("matrix staged binary"))
}

fn scratch() -> PathBuf {
    PathBuf::from(std::env::var_os("FRAGCAP_WINDOWS_SCRATCH").expect("matrix scratch root"))
}

fn run(binary: &Path, arguments: &[&str]) -> Output {
    use std::os::windows::process::CommandExt;
    Command::new(binary)
        .args(arguments)
        .env("FRAGCAP_SESSION_DIR", scratch())
        .env("FRAGCAP_LOCAL_DB", scratch().join("local.db"))
        .env("FRAGCAP_CATALOG_DB", scratch().join("catalog.db"))
        .stdin(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .expect("start staged fragcap without a console window")
}

fn doctor() -> Vec<Value> {
    let output = run(&staged_binary(), &["--json", "doctor"]);
    assert!(matches!(output.status.code(), Some(0 | 1)));
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn named<'a>(records: &'a [Value], name: &str) -> &'a Value {
    records
        .iter()
        .find(|record| record["name"] == name)
        .unwrap_or_else(|| panic!("Doctor omitted {name}"))
}

#[test]
#[ignore = "S129 matrix runner only"]
fn staged_binary_runs_from_relocated_layout() {
    let binary = staged_binary();
    assert!(binary.is_file());
    assert!(binary.parent().unwrap().ends_with("stage"));
    let output = run(&binary, &["--version"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "fragcap 0.8.0"
    );
}

#[test]
#[ignore = "S129 matrix runner only"]
fn staged_doctor_reports_npcap_absent_without_weakening_deep_capture() {
    assert_eq!(
        std::env::var("FRAGCAP_WINDOWS_EXPECT_NPCAP").unwrap(),
        "absent"
    );
    let records = doctor();
    assert_eq!(named(&records, "npcap")["status"], "fail");
    assert_eq!(named(&records, "proxy backend")["status"], "ok");
    assert_eq!(named(&records, "IPv4 loopback listener")["status"], "ok");
    assert_eq!(named(&records, "IPv6 loopback listener")["status"], "ok");
}

#[test]
#[ignore = "S129 matrix runner only"]
fn staged_doctor_reports_npcap_present_and_deep_capture_independently() {
    assert_eq!(
        std::env::var("FRAGCAP_WINDOWS_EXPECT_NPCAP").unwrap(),
        "present"
    );
    let records = doctor();
    assert_eq!(named(&records, "npcap")["status"], "ok");
    assert_eq!(named(&records, "proxy backend")["status"], "ok");
}

#[test]
#[ignore = "S129 matrix runner only"]
fn loopback_families_preserve_system_network_state() {
    let proxy_before = proxy_environment();
    let ipv4 = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let ipv6 = TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap();
    assert!(ipv4.local_addr().unwrap().ip().is_loopback());
    assert!(ipv6.local_addr().unwrap().ip().is_loopback());
    drop((ipv4, ipv6));
    assert_eq!(proxy_environment(), proxy_before);
}

#[test]
#[ignore = "S129 matrix runner only"]
fn non_admin_host_refuses_elevated_process_watching() {
    let records = doctor();
    assert_eq!(named(&records, "privilege")["status"], "warn");
    assert_ne!(named(&records, "process events")["status"], "ok");
}

#[test]
#[ignore = "S129 matrix runner only"]
fn physical_run_finishes_without_owned_residue() {
    let entries = fs::read_dir(scratch())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(
        entries.iter().all(|entry| matches!(
            entry.file_name().to_str(),
            Some("local.db" | "local.db-shm" | "local.db-wal")
        )),
        "unexpected owned residue: {entries:?}"
    );
}

fn proxy_environment() -> Vec<(String, Option<std::ffi::OsString>)> {
    [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "NO_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
        "no_proxy",
    ]
    .into_iter()
    .map(|name| (name.to_owned(), std::env::var_os(name)))
    .collect()
}
