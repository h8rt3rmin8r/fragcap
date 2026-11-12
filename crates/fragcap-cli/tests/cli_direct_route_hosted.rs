// SPDX-License-Identifier: Apache-2.0

//! Public staged-binary direct-route journey on a disposable Windows runner.

#![cfg(windows)]

use fragcap::profile::FidelityTier;
use fragcap::targets::{
    resolved_client_launch, ClassificationSource, CompatibilityFactKey, Store,
    TargetClassification, TargetEntry,
};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn native_command(binary: &PathBuf) -> Command {
    use std::os::windows::process::CommandExt;

    let mut command = Command::new(binary);
    command.creation_flags(CREATE_NO_WINDOW);
    for override_name in [
        "FRAGCAP_LOCAL_DB",
        "FRAGCAP_SESSION_DIR",
        "FRAGCAP_STORAGE_ROOT",
        "FRAGCAP_PROFILE_DIR",
    ] {
        command.env_remove(override_name);
    }
    command
}

struct RegisteredTarget<'a> {
    binary: &'a PathBuf,
    handle: &'static str,
    removed: bool,
}

impl RegisteredTarget<'_> {
    fn remove(&mut self) -> std::io::Result<()> {
        let result = native_command(self.binary)
            .args(["targets", "remove", self.handle])
            .output()?;
        if !result.status.success() {
            return Err(std::io::Error::other(format!(
                "target removal failed: {result:?}"
            )));
        }
        self.removed = true;
        Ok(())
    }
}

impl Drop for RegisteredTarget<'_> {
    fn drop(&mut self) {
        if !self.removed {
            if let Err(error) = self.remove() {
                eprintln!("S205 controlled target cleanup failed: {error}");
            }
        }
    }
}

struct SessionChild(Child);

impl Drop for SessionChild {
    fn drop(&mut self) {
        if self.0.try_wait().is_ok_and(|status| status.is_none()) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

fn unrelated_sentinel(peer: SocketAddr) -> std::io::Result<Duration> {
    let start = Instant::now();
    let mut stream = TcpStream::connect_timeout(&peer, Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    stream.write_all(b"sentinel-request")?;
    stream.shutdown(Shutdown::Write)?;
    let mut reply = [0; 14];
    std::io::Read::read_exact(&mut stream, &mut reply)?;
    if &reply != b"sentinel-reply" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "sentinel reply differed",
        ));
    }
    Ok(start.elapsed())
}

#[test]
#[ignore = "requires an elevated package binary, cold controlled client, and finite external peer"]
fn package_public_direct_route_exchanges_external_bytes_with_native_defaults() {
    let binary = PathBuf::from(std::env::var_os("FRAGCAP_S205_BINARY").expect("package binary"));
    let client = PathBuf::from(std::env::var_os("FRAGCAP_S205_CLIENT").expect("cold client"));
    let peer: SocketAddr = std::env::var("FRAGCAP_S205_PEER")
        .expect("external peer")
        .parse()
        .expect("IP:port");
    assert!(!peer.ip().is_loopback(), "the peer must be external");
    let binary = binary.canonicalize().expect("package binary exists");
    let client = client.canonicalize().expect("controlled client exists");
    let handle = "s205_controlled_route_client";
    let registration = native_command(&binary)
        .args([
            "targets",
            "add",
            "S205 Controlled Route Client",
            "--handle",
            handle,
            "--exe",
            client.to_str().unwrap(),
            "--socket-holder",
            "yes",
        ])
        .output()
        .expect("public target registration");
    assert!(registration.status.success(), "{registration:?}");
    let mut registration_guard = RegisteredTarget {
        binary: &binary,
        handle,
        removed: false,
    };

    let mut child = SessionChild(
        native_command(&binary)
            .args([
                "--json",
                "deep-capture",
                handle,
                "--launch",
                "--direct-route",
                "--authorize-stdin",
                "--calibrate",
                "reachability",
                "--calibration-protocol",
                "routing",
                "--launch-case",
                "direct-exe-cold",
                "--duration",
                "8s",
                "--wait",
                "30s",
            ])
            .env("S205_CONTROLLED_PEER", peer.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("public direct-route command"),
    );
    let mut stdin = child.0.stdin.take().unwrap();
    let stderr = child.0.stderr.take().unwrap();
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            let line = line.expect("JSON event line");
            if let Ok(event) = serde_json::from_str::<Value>(&line) {
                if event["event"] == "deep_capture.authorization_plan" {
                    writeln!(stdin, "{}", event["plan_id"].as_str().unwrap())
                        .expect("authorize exact plan");
                    stdin.flush().expect("flush authorization");
                }
                event_tx.send(event).expect("event receiver");
            }
        }
    });
    let deadline = Instant::now() + Duration::from_secs(120);
    let mut events = Vec::new();
    let mut sentinels = Vec::new();
    let status = loop {
        while let Ok(event) = event_rx.try_recv() {
            if event["event"] == "deep_capture.authorization_plan" {
                for _ in 0..3 {
                    sentinels.push(unrelated_sentinel(peer).expect("unrelated sentinel"));
                }
            }
            events.push(event);
        }
        if let Some(status) = child.0.try_wait().expect("session status") {
            break status;
        }
        if Instant::now() >= deadline {
            child
                .0
                .kill()
                .expect("terminate timed-out acceptance session");
            child.0.wait().expect("reap timed-out acceptance session");
            panic!("public direct-route session timed out: {events:#?}");
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    reader.join().expect("event reader");
    events.extend(event_rx.try_iter());
    assert_eq!(sentinels.len(), 3, "unrelated sentinel count");
    assert!(status.success(), "session failed: {events:#?}");
    assert_eq!(
        events
            .iter()
            .filter(|event| event["event"] == "deep_capture.authorization_plan")
            .count(),
        1,
        "authorization plan count: {events:#?}"
    );
    let complete = events
        .iter()
        .find(|event| event["event"] == "deep_capture.complete")
        .expect("terminal completion event");
    let bundle = PathBuf::from(
        complete["bundle_effective"]
            .as_str()
            .expect("effective native bundle path"),
    );
    assert_eq!(complete["bundle_redirected"], false);
    let report = fragcap::deep_capture::build_route_debug_report(&bundle).unwrap();
    if std::env::var_os("FRAGCAP_S205_PUBLISHED_BASELINE").is_some() {
        assert_eq!(report["route"]["state"], "unavailable");
    } else {
        assert_eq!(report["route"]["state"], "complete");
        assert_eq!(report["route"]["dropped_events"], 0);
        assert_eq!(report["route"]["bidirectional_flows"], 1);
        assert_eq!(
            report["flows"][0]["client_to_upstream_bytes"],
            (b"runtime-client-request".len() + b"runtime-client-finish".len()) as u64
        );
        assert_eq!(
            report["flows"][0]["upstream_to_client_bytes"],
            b"runtime-origin-reply".len() as u64
        );
    }
    assert!(bundle.join("capture.fcapng").is_file());
    eprintln!(
        "S205 package={} peer={} bundle={} sentinel_ms={:?}",
        binary.display(),
        peer,
        bundle.display(),
        sentinels
            .iter()
            .map(Duration::as_millis)
            .collect::<Vec<_>>()
    );
    registration_guard
        .remove()
        .expect("remove controlled target");
}

#[test]
#[ignore = "requires the signed staged component and disposable elevated Windows runner"]
fn staged_public_direct_route_authorizes_one_composite_plan_and_cleans_up() {
    use std::os::windows::process::CommandExt;

    let binary = PathBuf::from(std::env::var_os("FRAGCAP_WINDOWS_BINARY").unwrap());
    let scratch = PathBuf::from(std::env::var_os("FRAGCAP_WINDOWS_SCRATCH").unwrap());
    std::fs::create_dir_all(&scratch).unwrap();
    let local = scratch.join("direct-cli.db");
    let bundle = scratch.join("direct-cli-bundle");
    let mut store = Store::open(&local).unwrap();
    let row_id = store
        .insert_target(&TargetEntry {
            id: None,
            stable_id: 75_000,
            handle: "sample-target".into(),
            name: "Sample Target".into(),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::User,
            fidelity: FidelityTier::Authored,
            provenance: None,
            anchor: None,
            launch_entries: Some(resolved_client_launch("client.exe")),
            install_root: None,
            evidence: None,
            detection_scan: None,
            folder_name: None,
            executable_hint: Some("client.exe".into()),
        })
        .unwrap();
    drop(store);

    let mut child = Command::new(&binary)
        .args([
            "--json",
            "deep-capture",
            "--id",
            "75000",
            "--launch",
            "--direct-route",
            "--authorize-stdin",
            "--controlled-target",
            "--calibrate",
            "reachability",
            "--calibration-protocol",
            "routing",
            "--launch-case",
            "direct-exe-cold",
            "--local-db",
            local.to_str().unwrap(),
            "--bundle",
            bundle.to_str().unwrap(),
        ])
        .env("FRAGCAP_SESSION_DIR", &scratch)
        .env("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE", &binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut events = Vec::new();
    for line in BufReader::new(child.stderr.take().unwrap()).lines() {
        let record: Value = serde_json::from_str(&line.unwrap()).unwrap();
        if record["event"] == "deep_capture.authorization_plan" {
            assert_eq!(
                record["plan"]["proxy"]["route_identity"],
                "child-environment+direct-socket-v1"
            );
            assert!(record["plan"]["proxy"]["direct_socket"].is_object());
            writeln!(stdin, "{}", record["plan_id"].as_str().unwrap()).unwrap();
            stdin.flush().unwrap();
        }
        events.push(record);
    }
    let status = child.wait().unwrap();
    assert!(status.success(), "events: {events:#?}");
    assert_eq!(
        events
            .iter()
            .filter(|event| event["event"] == "deep_capture.authorization_plan")
            .count(),
        1
    );
    assert!(events
        .iter()
        .any(|event| event["event"] == "deep_capture.complete"));
    let store = Store::open(&local).unwrap();
    let facts = store.compatibility_facts_for_target(row_id).unwrap();
    assert!(
        facts.iter().all(|fact| {
            fact.key != CompatibilityFactKey::ProxyRouting || fact.value != "reached-client"
        }),
        "environment-only controlled traffic must not qualify a direct receipt"
    );
}
