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
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn pcapng_packet_blocks(path: &Path) -> usize {
    let bytes = std::fs::read(path).expect("native pcapng exists");
    let mut offset = 0;
    let mut packets = 0;
    while offset < bytes.len() {
        assert!(bytes.len() - offset >= 12, "truncated pcapng block");
        let kind = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let length = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
        assert!(length >= 12 && length & 3 == 0);
        assert!(length <= bytes.len() - offset, "pcapng block beyond file");
        assert_eq!(
            u32::from_le_bytes(
                bytes[offset + length - 4..offset + length]
                    .try_into()
                    .unwrap()
            ) as usize,
            length,
            "pcapng block length trailer"
        );
        packets += usize::from(kind == 6);
        offset += length;
    }
    packets
}

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

struct GateFiles(PathBuf, PathBuf);

impl Drop for GateFiles {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
        let _ = std::fs::remove_file(&self.1);
    }
}

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

struct LocalPeer {
    address: SocketAddr,
    stop: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<std::io::Result<(usize, usize)>>>,
}

impl LocalPeer {
    fn start(ip: std::net::IpAddr) -> std::io::Result<Self> {
        if ip.is_loopback() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "controlled non-loopback IP required",
            ));
        }
        let listener = TcpListener::bind(SocketAddr::new(ip, 0))?;
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?;
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(120);
            let mut handlers = Vec::new();
            while handlers.len() < 10 && !worker_stop.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut stream, _)) => handlers.push(std::thread::spawn(move || {
                        stream.set_nonblocking(false)?;
                        stream.set_read_timeout(Some(Duration::from_secs(35)))?;
                        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
                        let mut first = [0; 16];
                        std::io::Read::read_exact(&mut stream, &mut first)?;
                        if &first == b"sentinel-request" {
                            stream.write_all(b"sentinel-reply")?;
                            return Ok::<_, std::io::Error>(false);
                        }
                        let mut remainder = [0; 6];
                        std::io::Read::read_exact(&mut stream, &mut remainder)?;
                        if [&first[..], &remainder[..]].concat() != b"runtime-client-request" {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "unknown controlled request",
                            ));
                        }
                        stream.write_all(b"runtime-origin-reply")?;
                        let mut finish = [0; 21];
                        std::io::Read::read_exact(&mut stream, &mut finish)?;
                        if &finish != b"runtime-client-finish" {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "controlled client did not finish",
                            ));
                        }
                        Ok(true)
                    })),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        if Instant::now() >= deadline {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => return Err(error),
                }
            }
            let mut selected = 0;
            let mut sentinels = 0;
            for handler in handlers {
                if handler
                    .join()
                    .map_err(|_| std::io::Error::other("controlled peer panicked"))??
                {
                    selected += 1;
                } else {
                    sentinels += 1;
                }
            }
            Ok((selected, sentinels))
        });
        Ok(Self {
            address,
            stop,
            worker: Some(worker),
        })
    }

    fn finish(&mut self) -> std::io::Result<()> {
        let counts = self
            .worker
            .take()
            .expect("peer worker exists")
            .join()
            .map_err(|_| std::io::Error::other("controlled peer panicked"))??;
        if counts != (1, 9) {
            return Err(std::io::Error::other(format!(
                "controlled peer counts: {counts:?}"
            )));
        }
        Ok(())
    }
}

impl Drop for LocalPeer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            if let Err(error) = worker
                .join()
                .map_err(|_| std::io::Error::other("controlled peer panicked"))
                .and_then(|result| result.map(|_| ()))
            {
                eprintln!("S211 controlled peer cleanup failed: {error}");
            }
        }
    }
}

#[test]
#[ignore = "requires a local non-loopback bind IP; tests only the disposable peer"]
fn local_peer_protocol_completes_without_driver() {
    let ip = std::env::var("FRAGCAP_S211_BIND_IP")
        .expect("controlled bind IP")
        .parse()
        .expect("IP literal");
    let mut peer = LocalPeer::start(ip).expect("start local peer");
    for _ in 0..3 {
        unrelated_sentinel(peer.address).expect("baseline sentinel");
    }
    let mut selected = TcpStream::connect(peer.address).expect("selected connection");
    selected
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    selected.write_all(b"runtime-client-request").unwrap();
    let mut reply = [0; 20];
    std::io::Read::read_exact(&mut selected, &mut reply).unwrap();
    assert_eq!(&reply, b"runtime-origin-reply");
    for _ in 0..3 {
        unrelated_sentinel(peer.address).expect("active sentinel");
    }
    selected.write_all(b"runtime-client-finish").unwrap();
    selected.shutdown(Shutdown::Write).unwrap();
    let mut tail = Vec::new();
    std::io::Read::read_to_end(&mut selected, &mut tail).unwrap();
    assert!(tail.is_empty());
    for _ in 0..3 {
        unrelated_sentinel(peer.address).expect("post sentinel");
    }
    peer.finish().expect("exact peer exchange counts");
}

#[test]
#[ignore = "requires an elevated package binary, cold controlled client, and finite peer"]
fn package_public_direct_route_exchanges_external_bytes_with_native_defaults() {
    let refusal = std::env::var_os("FRAGCAP_S211_REFUSAL").is_some();
    let binary = PathBuf::from(std::env::var_os("FRAGCAP_S205_BINARY").expect("package binary"));
    let client = PathBuf::from(std::env::var_os("FRAGCAP_S205_CLIENT").expect("cold client"));
    let mut local_peer = std::env::var("FRAGCAP_S211_BIND_IP")
        .ok()
        .map(|ip| LocalPeer::start(ip.parse().expect("controlled bind IP")))
        .transpose()
        .expect("controlled non-loopback peer");
    let peer: SocketAddr = if let Some(peer) = &local_peer {
        peer.address
    } else {
        std::env::var("FRAGCAP_S205_PEER")
            .expect("external peer")
            .parse()
            .expect("IP:port")
    };
    assert!(
        !peer.ip().is_loopback(),
        "the peer IP must not be a loopback literal"
    );
    let binary = binary.canonicalize().expect("package binary exists");
    let client = client.canonicalize().expect("controlled client exists");
    let baseline = (0..3)
        .map(|_| unrelated_sentinel(peer).expect("baseline unrelated sentinel"))
        .collect::<Vec<_>>();
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

    let gate_signal = client
        .parent()
        .unwrap()
        .join(format!("s205-gate-{}.signal", std::process::id()));
    let gate_release = gate_signal.with_extension("release");
    let _gate_files = GateFiles(gate_signal.clone(), gate_release.clone());
    let gate_for_sentinels = gate_signal.clone();
    let route_active = Arc::new(AtomicBool::new(false));
    let gate_active = Arc::clone(&route_active);
    let sentinels = std::thread::spawn(move || -> std::io::Result<Vec<Duration>> {
        let deadline = Instant::now() + Duration::from_secs(40);
        loop {
            if std::fs::read(&gate_for_sentinels).is_ok_and(|bytes| bytes == b"reply-received") {
                break;
            }
            if Instant::now() >= deadline {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "selected client did not reach the reply gate",
                ));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        if !gate_active.load(Ordering::Acquire) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "selected reply arrived without confirmed route activation",
            ));
        }
        let mut elapsed = Vec::new();
        for _ in 0..3 {
            elapsed.push(unrelated_sentinel(peer)?);
        }
        std::fs::write(&gate_release, b"release")?;
        Ok(elapsed)
    });

    let mut capture_args = vec![
        "--json".to_owned(),
        "deep-capture".to_owned(),
        handle.to_owned(),
        "--launch".to_owned(),
        "--direct-route".to_owned(),
    ];
    if !refusal {
        capture_args.extend(["--proxy-allow-destination".to_owned(), peer.to_string()]);
    }
    capture_args.extend([
        "--authorize-stdin".to_owned(),
        "--duration".to_owned(),
        "8s".to_owned(),
        "--wait".to_owned(),
        "30s".to_owned(),
    ]);
    let mut child = SessionChild(
        native_command(&binary)
            .args(&capture_args)
            .env("S205_CONTROLLED_PEER", peer.to_string())
            .env("S205_CONTROLLED_GATE", &gate_signal)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("public direct-route command"),
    );
    let mut stdin = child.0.stdin.take().unwrap();
    let stderr = child.0.stderr.take().unwrap();
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let reader_active = Arc::clone(&route_active);
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            let line = line.expect("JSON event line");
            if let Ok(event) = serde_json::from_str::<Value>(&line) {
                if event["event"] == "deep_capture.authorization_plan" {
                    writeln!(stdin, "{}", event["plan_id"].as_str().unwrap())
                        .expect("authorize exact plan");
                    stdin.flush().expect("flush authorization");
                }
                if event["event"] == "deep_capture.proxy_started" {
                    reader_active.store(true, Ordering::Release);
                }
                event_tx.send(event).expect("event receiver");
            }
        }
    });
    let deadline = Instant::now() + Duration::from_secs(120);
    let mut events = Vec::new();
    let status = loop {
        while let Ok(event) = event_rx.try_recv() {
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
    let sentinels = sentinels
        .join()
        .expect("sentinel gate thread")
        .unwrap_or_else(|error| {
            panic!("unrelated sentinels during selected transfer: {error}; events={events:#?}")
        });
    let post = (0..3)
        .map(|_| unrelated_sentinel(peer).expect("post-route unrelated sentinel"))
        .collect::<Vec<_>>();
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
        if refusal {
            assert_eq!(report["route"]["bidirectional_flows"], 0);
            assert!(
                pcapng_packet_blocks(&bundle.join("capture.fcapng")) > 0,
                "Npcap did not retain a reinjected original packet"
            );
            let compatibility: Value =
                serde_json::from_slice(&std::fs::read(bundle.join("compatibility.json")).unwrap())
                    .unwrap();
            assert!(
                compatibility["observations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|observation| {
                        observation["direct_route"]["receipt_state"] == "unconsumed"
                            && observation["direct_route"]["original_packet_state"]
                                == "observed-owned"
                    }),
                "refused selected packet was not retained with exact owner: {compatibility:#?}"
            );
        } else {
            assert_eq!(report["route"]["bidirectional_flows"], 1);
            assert_eq!(report["flows"][0]["process_generation_join"], "exact");
            assert_eq!(report["flows"][0]["final_client_generation_proven"], true);
            assert_eq!(
                report["flows"][0]["client_to_upstream_bytes"],
                (b"runtime-client-request".len() + b"runtime-client-finish".len()) as u64
            );
            assert_eq!(
                report["flows"][0]["upstream_to_client_bytes"],
                b"runtime-origin-reply".len() as u64
            );
        }
        let limit = std::cmp::max(
            Duration::from_millis(50),
            baseline.iter().copied().max().unwrap().saturating_mul(4),
        );
        assert!(
            sentinels.iter().all(|elapsed| *elapsed <= limit),
            "active unrelated sentinel exceeded {limit:?}: baseline={baseline:?}, active={sentinels:?}"
        );
    }
    assert!(bundle.join("capture.fcapng").is_file());
    eprintln!(
        "S211 mode={} package={} peer={} bundle={} baseline_ms={:?} active_ms={:?} post_ms={:?}",
        if refusal { "refusal" } else { "transfer" },
        binary.display(),
        peer,
        bundle.display(),
        baseline.iter().map(Duration::as_millis).collect::<Vec<_>>(),
        sentinels
            .iter()
            .map(Duration::as_millis)
            .collect::<Vec<_>>(),
        post.iter().map(Duration::as_millis).collect::<Vec<_>>()
    );
    registration_guard
        .remove()
        .expect("remove controlled target");
    if let Some(peer) = local_peer.as_mut() {
        peer.finish()
            .expect("controlled peer completed exact exchanges");
    }
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
            "--inspect-scope",
            "launch-chain",
            "--launcher-proxy",
            "http-and-socks",
            "--authorize-stdin",
            "--controlled-target",
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
