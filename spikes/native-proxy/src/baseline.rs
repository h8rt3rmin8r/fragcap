// SPDX-License-Identifier: Apache-2.0

use crate::{
    evidence::{BackendRun, Observation, Status},
    scenario::{self, CaMaterial},
};
use serde::Deserialize;
use std::{
    net::{SocketAddr, TcpListener},
    path::Path,
    process::{Child, Command, Stdio},
    time::Duration,
};

#[derive(Deserialize)]
struct AddonEvent {
    kind: String,
    method: Option<String>,
    path: Option<String>,
    http_version: Option<String>,
    byte_length: usize,
    digest: String,
    direction: Option<String>,
}

fn free_port() -> Result<u16, String> {
    let listener =
        TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).map_err(|error| error.to_string())?;
    listener
        .local_addr()
        .map(|address| address.port())
        .map_err(|error| error.to_string())
}

async fn wait_ready(port: u16, child: &mut Child) -> Result<(), String> {
    for _ in 0..100 {
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("mitmdump exited before readiness with {status}"));
        }
        if tokio::net::TcpStream::connect((std::net::Ipv4Addr::LOCALHOST, port))
            .await
            .is_ok()
        {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Err("mitmdump did not bind loopback within five seconds".to_string())
}

fn start(port: u16, directory: &Path, events: &Path, key_log: &Path) -> Result<Child, String> {
    let addon = Path::new(env!("CARGO_MANIFEST_DIR")).join("baseline-addon.py");
    Command::new("mitmdump")
        .args([
            "--listen-host",
            "127.0.0.1",
            "--listen-port",
            &port.to_string(),
            "--set",
            &format!("confdir={}", directory.display()),
            "--set",
            "ssl_insecure=true",
            "--set",
            &format!("hardump={}", directory.join("baseline.har").display()),
            "--set",
            "termlog_verbosity=error",
            "-w",
            &directory.join("baseline.flows").display().to_string(),
            "-s",
            &addon.display().to_string(),
        ])
        .env("FRAGCAP_SPIKE_EVENTS", events)
        .env("MITMPROXY_SSLKEYLOGFILE", key_log)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| error.to_string())
}

fn version() -> String {
    Command::new("mitmdump")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|text| text.lines().next().map(str::to_string))
        .unwrap_or_else(|| "unavailable".to_string())
}

fn parse_events(path: &Path) -> Vec<Observation> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return vec![Observation::result(
            "matrix",
            "proxy-events",
            Status::NotMeasured,
            "baseline addon produced no event file",
        )];
    };
    content
        .lines()
        .filter_map(|line| serde_json::from_str::<AddonEvent>(line).ok())
        .filter(|event| {
            !event
                .method
                .as_deref()
                .is_some_and(|method| method.eq_ignore_ascii_case("CONNECT"))
        })
        .filter(|event| {
            event.path.as_deref().is_some_and(|path| {
                ["http1", "https-http1", "https-http2", "websocket"]
                    .iter()
                    .any(|scenario| path.contains(scenario))
            })
        })
        .map(|event| {
            let scenario = event
                .path
                .as_deref()
                .map(scenario_from_path)
                .unwrap_or("websocket");
            let kind = match (scenario, event.kind.as_str()) {
                ("websocket", "request") => "proxy-handshake-request",
                ("websocket", "response") => "proxy-handshake-response",
                (_, "request") => "proxy-request",
                (_, "response") => "proxy-response",
                _ => "proxy-message",
            };
            let protocol = if event.kind == "websocket-message" {
                Some("websocket".to_string())
            } else {
                event.http_version
            };
            Observation {
                scenario: scenario.to_string(),
                kind: kind.to_string(),
                status: if scenario == "websocket"
                    && matches!(event.kind.as_str(), "request" | "response")
                {
                    Status::Complete
                } else if event.byte_length == 0 {
                    Status::Empty
                } else {
                    Status::Complete
                },
                protocol,
                direction: event.direction,
                byte_length: event.byte_length,
                digest: Some(event.digest),
                detail: None,
            }
        })
        .collect()
}

fn scenario_from_path(path: &str) -> &'static str {
    if path.contains("https-http2") {
        "https-http2"
    } else if path.contains("https-http1") {
        "https-http1"
    } else if path.contains("websocket") {
        "websocket"
    } else {
        "http1"
    }
}

pub async fn run() -> BackendRun {
    let version = version();
    if version == "unavailable" {
        return BackendRun::failed("mitmdump", &version, "mitmdump is unavailable");
    }
    let directory = match tempfile::tempdir() {
        Ok(directory) => directory,
        Err(error) => return BackendRun::failed("mitmdump", &version, error.to_string()),
    };
    let ca = match CaMaterial::generate() {
        Ok(ca) => ca,
        Err(error) => return BackendRun::failed("mitmdump", &version, error.to_string()),
    };
    let origins = match scenario::Origins::start(&ca).await {
        Ok(origins) => origins,
        Err(error) => return BackendRun::failed("mitmdump", &version, error.to_string()),
    };
    let port = match free_port() {
        Ok(port) => port,
        Err(error) => return BackendRun::failed("mitmdump", &version, error),
    };
    let events = directory.path().join("baseline-events.jsonl");
    let key_log = directory.path().join("baseline.keys");
    let mut child = match start(port, directory.path(), &events, &key_log) {
        Ok(child) => child,
        Err(error) => return BackendRun::failed("mitmdump", &version, error),
    };
    if let Err(error) = wait_ready(port, &mut child).await {
        let _ = child.kill();
        let _ = child.wait();
        return BackendRun::failed("mitmdump", &version, error);
    }
    let mut observations = scenario::exercise(
        SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, port)),
        &origins,
    )
    .await;
    tokio::time::sleep(Duration::from_millis(250)).await;
    let _ = child.kill();
    let shutdown = child
        .wait()
        .map(|_| Status::Bounded)
        .unwrap_or(Status::Failed);
    origins.stop().await;
    observations.extend(parse_events(&events));
    let har = directory.path().join("baseline.har");
    let flows = directory.path().join("baseline.flows");
    if flows.is_file() {
        let _ = Command::new("mitmdump")
            .args([
                "-n",
                "-r",
                &flows.display().to_string(),
                "--set",
                &format!("hardump={}", har.display()),
                "--set",
                "termlog_verbosity=error",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    observations.push(if har.is_file() {
        Observation::result(
            "matrix",
            "har-output",
            Status::Complete,
            "baseline wrote HAR on bounded process termination",
        )
    } else {
        Observation::result(
            "matrix",
            "har-output",
            Status::NotMeasured,
            "baseline did not finalize HAR after forced bounded termination",
        )
    });
    let key_log_lines = std::fs::read_to_string(&key_log)
        .map(|content| content.lines().count())
        .unwrap_or(0);
    let mut run = BackendRun {
        backend: "mitmdump".to_string(),
        version,
        platform: "windows-x86_64".to_string(),
        loopback_only: true,
        trust_store_mutated: false,
        cache_capacity: None,
        key_log_lines,
        shutdown_trials: vec![shutdown],
        observations,
        limitations: vec![
            "the research adapter uses forced bounded termination because mitmdump exposes no embedded cancellation future".to_string(),
        ],
    };
    run.sort();
    run
}
