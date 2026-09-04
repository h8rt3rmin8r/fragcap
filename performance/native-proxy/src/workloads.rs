// SPDX-License-Identifier: Apache-2.0

use std::io::{self, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

use bytes::{Buf, Bytes};
use fragcap::deep_capture::ApplicationArtifactLease;
use fragcap_proxy::{
    DestinationAuthority, DestinationPolicy, LeafCache, NativeProxyBackend, NativeProxyConfig,
    ProtocolLimits, SessionCertificateAuthority, ShutdownReport, build_quic_client_config,
    build_quic_server_config, tls_client_config_with_roots,
};
use h3_quinn::Connection;
use http::{Request, Response};
use quinn::Endpoint;
use rustls::RootCertStore;

const PAYLOAD_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub struct Measurement {
    pub direct_microseconds: u64,
    pub proxy_microseconds: u64,
    pub useful_bytes: u64,
    pub shutdown_microseconds: u64,
    pub clean_shutdown: bool,
    pub resources: ResourceMeasurement,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ResourceMeasurement {
    pub task_peak: u64,
    pub task_current: u64,
    pub task_spawned: u64,
    pub task_completed: u64,
    pub task_aborted: u64,
    pub cache_peak_entries: u64,
    pub cache_peak_bytes: u64,
    pub queue_peak: u64,
    pub queue_current: u64,
    pub failure_details_dropped: u64,
    pub application_events_dropped: u64,
    pub artifact_bytes: u64,
    pub payload_bytes_observed: u64,
    pub payload_bytes_retained: u64,
    pub payload_bytes_omitted: u64,
    pub payload_bytes_queue_dropped: u64,
    pub payload_bytes_storage_dropped: u64,
}

struct ProxyOwner {
    native: fragcap_proxy::NativeProxyLease,
    artifact: ApplicationArtifactLease,
}

static ARTIFACT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub fn measure(protocol: &str, retention: &str) -> io::Result<Measurement> {
    let capture = retention == "on";
    match protocol {
        "http1" => http1(capture),
        "websocket" => websocket(capture),
        "tcp" => tcp(capture),
        "udp" => udp(capture),
        "http2" => h2(false, capture),
        "grpc" => h2(true, capture),
        "quic" => quic(capture),
        _ => Err(io::Error::other(format!(
            "dedicated workload is unavailable for {protocol}"
        ))),
    }
}

fn limits(capture_payloads: bool) -> ProtocolLimits {
    ProtocolLimits {
        capture_payloads,
        max_body_bytes: 2 * PAYLOAD_BYTES as u64,
        max_session_body_bytes: 4 * PAYLOAD_BYTES as u64,
        max_event_chunk_bytes: 16 * 1024,
        idle_timeout: Duration::from_secs(5),
        ..ProtocolLimits::default()
    }
}

fn proxy(origin: SocketAddr, capture: bool) -> io::Result<ProxyOwner> {
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        8,
        16 * 1024,
        Duration::from_secs(3),
    )
    .map_err(|error| io::Error::other(error.to_string()))?
    .with_session_id("s128-performance")
    .map_err(|error| io::Error::other(error.to_string()))?
    .with_protocol_limits(limits(capture));
    let mut policy = DestinationPolicy::new(config.listen());
    policy.grant_for_test(origin);
    let path = performance_artifact_path();
    let artifact = ApplicationArtifactLease::open(&path, "s128-performance", 4096)?;
    let native = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_application_event_sink(artifact.sink())
        .start(Duration::from_secs(3))
        .map_err(|error| io::Error::other(error.to_string()))?;
    Ok(ProxyOwner { native, artifact })
}

fn finish(mut owner: ProxyOwner, started: Instant) -> MeasurementTail {
    let sink = owner.artifact.sink();
    let report = owner.native.cleanup(Duration::from_secs(3));
    let artifact_path = owner.artifact.path().to_path_buf();
    let artifact_finished = owner.artifact.finish().is_ok();
    let artifact_bytes = std::fs::metadata(&artifact_path)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    let artifact_removed = std::fs::remove_file(artifact_path).is_ok();
    let mut resources = resources(&report);
    let sink_accounting = sink.accounting();
    resources.queue_peak = sink_accounting.queue_peak;
    resources.queue_current = sink_accounting.queue_current;
    resources.application_events_dropped = sink_accounting.dropped_events;
    resources.artifact_bytes = artifact_bytes;
    MeasurementTail {
        shutdown_microseconds: micros(started.elapsed()),
        clean_shutdown: report.is_clean()
            && artifact_finished
            && artifact_removed
            && sink_accounting.queue_current == 0,
        resources,
    }
}

fn performance_artifact_path() -> std::path::PathBuf {
    let sequence = ARTIFACT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "fragcap-s128-{}-{sequence}.jsonl",
        std::process::id()
    ))
}

struct MeasurementTail {
    shutdown_microseconds: u64,
    clean_shutdown: bool,
    resources: ResourceMeasurement,
}

fn resources(report: &ShutdownReport) -> ResourceMeasurement {
    let protocol = report.observation.protocol;
    ResourceMeasurement {
        task_peak: report.observation.resources.connection_tasks_peak,
        task_current: report.observation.resources.connection_tasks_current,
        task_spawned: report.observation.resources.connection_tasks_spawned,
        task_completed: report.observation.resources.connection_tasks_completed,
        task_aborted: report.observation.resources.connection_tasks_aborted,
        cache_peak_entries: report.observation.resources.leaf_cache_peak_entries,
        cache_peak_bytes: report.observation.resources.leaf_cache_peak_bytes,
        queue_peak: report.observation.resources.application_queue_peak,
        queue_current: report.observation.resources.application_queue_current,
        failure_details_dropped: report.observation.resources.failure_details_dropped_oldest,
        application_events_dropped: protocol.application_events_dropped,
        artifact_bytes: 0,
        payload_bytes_observed: protocol
            .body_bytes_observed
            .saturating_add(protocol.generic_stream_bytes_observed)
            .saturating_add(protocol.generic_udp_bytes_observed)
            .saturating_add(protocol.quic_stream_bytes_observed)
            .saturating_add(protocol.quic_datagram_bytes_observed),
        payload_bytes_retained: protocol
            .body_bytes_retained
            .saturating_add(protocol.generic_stream_bytes_retained)
            .saturating_add(protocol.generic_udp_bytes_retained)
            .saturating_add(protocol.quic_stream_bytes_retained)
            .saturating_add(protocol.quic_datagram_bytes_retained),
        payload_bytes_omitted: protocol
            .body_bytes_omitted
            .saturating_add(protocol.generic_stream_bytes_omitted)
            .saturating_add(protocol.generic_udp_bytes_omitted)
            .saturating_add(protocol.quic_stream_bytes_omitted)
            .saturating_add(protocol.quic_datagram_bytes_omitted),
        payload_bytes_queue_dropped: protocol
            .body_bytes_queue_dropped
            .saturating_add(protocol.generic_stream_bytes_queue_dropped)
            .saturating_add(protocol.generic_udp_bytes_queue_dropped)
            .saturating_add(protocol.quic_stream_bytes_queue_dropped)
            .saturating_add(protocol.quic_datagram_bytes_queue_dropped),
        payload_bytes_storage_dropped: protocol
            .generic_udp_bytes_storage_dropped
            .saturating_add(protocol.quic_stream_bytes_storage_dropped)
            .saturating_add(protocol.quic_datagram_bytes_storage_dropped),
    }
}

fn http_origin() -> io::Result<(SocketAddr, std::thread::JoinHandle<io::Result<()>>)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let task = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept()?;
        let head = read_head(&mut stream)?;
        let length = content_length(&head)?;
        let mut body = vec![0_u8; length];
        stream.read_exact(&mut body)?;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n"
        )?;
        stream.write_all(&body)
    });
    Ok((address, task))
}

fn http_exchange(endpoint: SocketAddr, origin: SocketAddr, auth: Option<&str>) -> io::Result<u64> {
    let mut stream = TcpStream::connect(endpoint)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let authorization = auth
        .map(|value| format!("Proxy-Authorization: {value}\r\n"))
        .unwrap_or_default();
    let target = if auth.is_some() {
        format!("http://{origin}/payload")
    } else {
        "/payload".to_string()
    };
    let payload = vec![0x5a; 1024 * 1024];
    let started = Instant::now();
    write!(
        stream,
        "POST {target} HTTP/1.1\r\nHost: {origin}\r\n{authorization}Content-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    )?;
    stream.write_all(&payload)?;
    let head = read_head(&mut stream)?;
    let length = content_length(&head)?;
    let mut response = vec![0_u8; length];
    stream.read_exact(&mut response)?;
    if response != payload {
        return Err(io::Error::other("HTTP payload changed"));
    }
    Ok(micros(started.elapsed()))
}

fn http1(capture: bool) -> io::Result<Measurement> {
    let (direct_origin, direct_task) = http_origin()?;
    let direct = http_exchange(direct_origin, direct_origin, None)?;
    direct_task
        .join()
        .map_err(|_| io::Error::other("direct HTTP origin panicked"))??;
    let (origin, task) = http_origin()?;
    let owner = proxy(origin, capture)?;
    let authorization = owner.native.capability_proof().proxy_authorization();
    let proxied = http_exchange(
        owner.native.endpoint(),
        origin,
        Some(authorization.as_str()),
    )?;
    task.join()
        .map_err(|_| io::Error::other("proxy HTTP origin panicked"))??;
    let stop = Instant::now();
    let tail = finish(owner, stop);
    Ok(combine(direct, proxied, tail, 2 * 1024 * 1024))
}

fn websocket_origin() -> io::Result<(SocketAddr, std::thread::JoinHandle<io::Result<()>>)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let task = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept()?;
        let _ = read_head(&mut stream)?;
        stream.write_all(b"HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n")?;
        let payload = read_websocket(&mut stream, true)?;
        write_websocket(&mut stream, &payload, false)?;
        stream.shutdown(Shutdown::Write)
    });
    Ok((address, task))
}

fn websocket_exchange(
    endpoint: SocketAddr,
    origin: SocketAddr,
    auth: Option<&str>,
) -> io::Result<u64> {
    let mut stream = TcpStream::connect(endpoint)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let authorization = auth
        .map(|value| format!("Proxy-Authorization: {value}\r\n"))
        .unwrap_or_default();
    let target = if auth.is_some() {
        format!("http://{origin}/socket")
    } else {
        "/socket".into()
    };
    write!(
        stream,
        "GET {target} HTTP/1.1\r\nHost: {origin}\r\n{authorization}Connection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n"
    )?;
    let _ = read_head(&mut stream)?;
    let payload = vec![0x33; PAYLOAD_BYTES];
    let started = Instant::now();
    write_websocket(&mut stream, &payload, true)?;
    let response = read_websocket(&mut stream, false)?;
    if response != payload {
        return Err(io::Error::other("WebSocket payload changed"));
    }
    Ok(micros(started.elapsed()))
}

fn websocket(capture: bool) -> io::Result<Measurement> {
    let (direct_origin, direct_task) = websocket_origin()?;
    let direct = websocket_exchange(direct_origin, direct_origin, None)?;
    direct_task
        .join()
        .map_err(|_| io::Error::other("direct WebSocket origin panicked"))??;
    let (origin, task) = websocket_origin()?;
    let owner = proxy(origin, capture)?;
    let authorization = owner.native.capability_proof().proxy_authorization();
    let proxied = websocket_exchange(
        owner.native.endpoint(),
        origin,
        Some(authorization.as_str()),
    )?;
    task.join()
        .map_err(|_| io::Error::other("proxy WebSocket origin panicked"))??;
    let tail = finish(owner, Instant::now());
    Ok(combine(direct, proxied, tail, 2 * PAYLOAD_BYTES as u64))
}

fn tcp_origin() -> io::Result<(SocketAddr, std::thread::JoinHandle<io::Result<()>>)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let task = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept()?;
        let mut payload = vec![0; PAYLOAD_BYTES];
        stream.read_exact(&mut payload)?;
        stream.write_all(&payload)
    });
    Ok((address, task))
}

fn tcp_exchange(
    endpoint: SocketAddr,
    origin: SocketAddr,
    password: Option<&[u8]>,
) -> io::Result<u64> {
    let mut stream = TcpStream::connect(endpoint)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    if let Some(password) = password {
        socks_authenticate(&mut stream, password)?;
        socks_connect(&mut stream, origin)?;
    }
    let payload = vec![0xa5; PAYLOAD_BYTES];
    let started = Instant::now();
    stream.write_all(&payload)?;
    let mut response = vec![0; payload.len()];
    stream.read_exact(&mut response)?;
    if response != payload {
        return Err(io::Error::other("TCP payload changed"));
    }
    Ok(micros(started.elapsed()))
}

fn tcp(capture: bool) -> io::Result<Measurement> {
    let (direct_origin, direct_task) = tcp_origin()?;
    let direct = tcp_exchange(direct_origin, direct_origin, None)?;
    direct_task
        .join()
        .map_err(|_| io::Error::other("direct TCP origin panicked"))??;
    let (origin, task) = tcp_origin()?;
    let owner = proxy(origin, capture)?;
    let password = owner.native.capability_proof().proxy_password();
    let proxied = tcp_exchange(owner.native.endpoint(), origin, Some(password.as_bytes()))?;
    task.join()
        .map_err(|_| io::Error::other("proxy TCP origin panicked"))??;
    let tail = finish(owner, Instant::now());
    Ok(combine(direct, proxied, tail, 2 * PAYLOAD_BYTES as u64))
}

fn udp(capture: bool) -> io::Result<Measurement> {
    let direct_origin = UdpSocket::bind("127.0.0.1:0")?;
    let direct_address = direct_origin.local_addr()?;
    let direct_task = udp_echo(direct_origin, 16);
    let direct_client = UdpSocket::bind("127.0.0.1:0")?;
    direct_client.set_read_timeout(Some(Duration::from_secs(5)))?;
    let payload = vec![0x77; 60 * 1024];
    let start = Instant::now();
    for _ in 0..16 {
        direct_client.send_to(&payload, direct_address)?;
        let mut response = vec![0; payload.len()];
        let (read, _) = direct_client.recv_from(&mut response)?;
        if response[..read] != payload {
            return Err(io::Error::other("direct UDP payload changed"));
        }
    }
    let direct = micros(start.elapsed());
    direct_task
        .join()
        .map_err(|_| io::Error::other("direct UDP origin panicked"))??;

    let origin = UdpSocket::bind("127.0.0.1:0")?;
    let origin_address = origin.local_addr()?;
    let task = udp_echo(origin, 16);
    let owner = proxy(origin_address, capture)?;
    let client = UdpSocket::bind("127.0.0.1:0")?;
    client.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut control = TcpStream::connect(owner.native.endpoint())?;
    socks_authenticate(
        &mut control,
        owner.native.capability_proof().proxy_password().as_bytes(),
    )?;
    let relay = socks_associate(&mut control, client.local_addr()?)?;
    let start = Instant::now();
    for _ in 0..16 {
        let frame = udp_frame(origin_address, &payload);
        client.send_to(&frame, relay)?;
        let mut response = vec![0; payload.len() + 10];
        let (read, _) = client.recv_from(&mut response)?;
        if response[10..read] != payload {
            return Err(io::Error::other("proxy UDP payload changed"));
        }
    }
    let proxied = micros(start.elapsed());
    drop(control);
    task.join()
        .map_err(|_| io::Error::other("proxy UDP origin panicked"))??;
    let tail = finish(owner, Instant::now());
    Ok(combine(direct, proxied, tail, 32 * payload.len() as u64))
}

fn h2(grpc: bool, capture: bool) -> io::Result<Measurement> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async move {
        let (direct_origin, direct_task) = h2_origin(grpc).await?;
        let direct = h2_exchange(direct_origin, direct_origin, None, grpc)
            .await
            .map_err(|error| io::Error::other(format!("direct HTTP/2 exchange: {error}")))?;
        direct_task
            .await
            .map_err(|error| io::Error::other(format!("direct HTTP/2 task join: {error}")))?
            .map_err(|error| io::Error::other(format!("direct HTTP/2 origin: {error}")))?;
        let (origin, task) = h2_origin(grpc).await?;
        let owner = proxy(origin, capture)?;
        let auth = owner.native.capability_proof().proxy_authorization();
        let proxied = h2_exchange(owner.native.endpoint(), origin, Some(auth.as_str()), grpc)
            .await
            .map_err(|error| io::Error::other(format!("proxied HTTP/2 exchange: {error}")))?;
        task.await
            .map_err(|error| io::Error::other(format!("proxied HTTP/2 task join: {error}")))?
            .map_err(|error| io::Error::other(format!("proxied HTTP/2 origin: {error}")))?;
        let tail = finish(owner, Instant::now());
        Ok(combine(direct, proxied, tail, 2 * PAYLOAD_BYTES as u64))
    })
}

fn quic(capture: bool) -> io::Result<Measurement> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()?;
    runtime.block_on(async move {
        let (direct_origin, direct_ca, direct_task) = quic_origin(201).await?;
        let mut direct_roots = RootCertStore::empty();
        direct_roots
            .add(direct_ca.der().clone())
            .map_err(io::Error::other)?;
        let direct = quic_exchange(direct_origin, direct_roots, None)
            .await
            .map_err(|error| io::Error::other(format!("direct QUIC exchange: {error}")))?;
        direct_task
            .await
            .map_err(|error| io::Error::other(format!("direct QUIC task join: {error}")))?
            .map_err(|error| io::Error::other(format!("direct QUIC origin: {error}")))?;

        let (origin, origin_ca, task) = quic_origin(202).await?;
        let mut config = NativeProxyConfig::new(
            "127.0.0.1:0".parse().unwrap(),
            8,
            16 * 1024,
            Duration::from_secs(3),
        )
        .map_err(|error| io::Error::other(error.to_string()))?
        .with_session_id("s128-quic-performance")
        .map_err(|error| io::Error::other(error.to_string()))?;
        let mut quic_limits = limits(capture);
        quic_limits.max_socks_udp_datagram_bytes = 8192;
        quic_limits.max_concurrent_streams = 8;
        config = config.with_protocol_limits(quic_limits);
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(origin);
        let mut origin_roots = RootCertStore::empty();
        origin_roots
            .add(origin_ca.der().clone())
            .map_err(io::Error::other)?;
        let artifact_path = performance_artifact_path();
        let artifact =
            ApplicationArtifactLease::open(&artifact_path, "s128-quic-performance", 4096)?;
        let native = NativeProxyBackend::new(config)
            .with_destination_policy(policy)
            .with_tls_client_config(
                tls_client_config_with_roots(origin_roots)
                    .map_err(|error| io::Error::other(error.to_string()))?,
            )
            .with_application_event_sink(artifact.sink())
            .start(Duration::from_secs(3))
            .map_err(|error| io::Error::other(error.to_string()))?;
        let shuttle = UdpSocket::bind("127.0.0.1:0")?;
        shuttle.set_read_timeout(Some(Duration::from_millis(50)))?;
        let shuttle_address = shuttle.local_addr()?;
        let client_probe = UdpSocket::bind("127.0.0.1:0")?;
        let client_address = client_probe.local_addr()?;
        drop(client_probe);
        let (control, relay) = authenticate_and_associate(
            native.endpoint(),
            native.capability_proof().proxy_password().as_bytes(),
            shuttle_address,
        )?;
        let stop = Arc::new(AtomicBool::new(false));
        let shuttle_stop = Arc::clone(&stop);
        let shuttle_task = std::thread::spawn(move || -> io::Result<u64> {
            let mut buffer = vec![0_u8; 8192];
            let mut connection_resets = 0_u64;
            while !shuttle_stop.load(Ordering::Acquire) {
                match shuttle.recv_from(&mut buffer) {
                    Ok((read, source)) if source == client_address => {
                        shuttle.send_to(&domain_frame(origin.port(), &buffer[..read]), relay)?;
                    }
                    Ok((read, source)) if source == relay => {
                        shuttle.send_to(response_payload(&buffer[..read])?, client_address)?;
                    }
                    Ok(_) => {}
                    Err(error)
                        if matches!(
                            error.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                        ) => {}
                    Err(error) if error.kind() == io::ErrorKind::ConnectionReset => {
                        connection_resets = connection_resets.saturating_add(1);
                    }
                    Err(error) => return Err(error),
                }
            }
            Ok(connection_resets)
        });
        let mut client_roots = RootCertStore::empty();
        client_roots
            .add(native.ca_der().to_vec().into())
            .map_err(io::Error::other)?;
        let proxied = quic_exchange(shuttle_address, client_roots, Some(client_address))
            .await
            .map_err(|error| io::Error::other(format!("proxied QUIC exchange: {error}")));
        stop.store(true, Ordering::Release);
        let shuttle_result = shuttle_task
            .join()
            .map_err(|_| io::Error::other("QUIC shuttle panicked"))?
            .map_err(|error| io::Error::other(format!("QUIC shuttle: {error}")))?;
        let proxied = proxied?;
        let _connection_resets = shuttle_result;
        drop(control);
        task.await
            .map_err(|error| io::Error::other(format!("proxied QUIC task join: {error}")))?
            .map_err(|error| io::Error::other(format!("proxied QUIC origin: {error}")))?;
        let tail = finish(ProxyOwner { native, artifact }, Instant::now());
        Ok(combine(direct, proxied, tail, 2 * 1024 * 1024))
    })
}

async fn quic_origin(
    lineage: u64,
) -> io::Result<(
    SocketAddr,
    SessionCertificateAuthority,
    tokio::task::JoinHandle<io::Result<()>>,
)> {
    let authority = DestinationAuthority::parse("localhost:443").map_err(io::Error::other)?;
    let ca = SessionCertificateAuthority::generate(
        lineage,
        SystemTime::now(),
        Duration::from_secs(3600),
    )
    .map_err(|error| io::Error::other(error.to_string()))?;
    let mut cache = LeafCache::new(
        NonZeroUsize::new(4).unwrap(),
        NonZeroUsize::new(128 * 1024).unwrap(),
        Duration::from_secs(1800),
        lineage,
    )
    .map_err(|error| io::Error::other(error.to_string()))?;
    let mut quic_limits = limits(true);
    quic_limits.max_concurrent_streams = 8;
    quic_limits.max_socks_udp_datagram_bytes = 8192;
    let server_config = build_quic_server_config(&authority, &ca, &mut cache, None, &quic_limits)
        .map_err(|error| io::Error::other(format!("{error:?}")))?;
    let server = Endpoint::server(server_config, "127.0.0.1:0".parse().unwrap())?;
    let address = server.local_addr()?;
    let task = tokio::spawn(async move {
        let connection = server
            .accept()
            .await
            .ok_or_else(|| io::Error::other("QUIC origin closed"))?
            .await
            .map_err(io::Error::other)?;
        let mut h3 = h3::server::Connection::new(Connection::new(connection))
            .await
            .map_err(io::Error::other)?;
        let resolver = h3
            .accept()
            .await
            .map_err(io::Error::other)?
            .ok_or_else(|| io::Error::other("HTTP/3 request missing"))?;
        let (_, mut stream) = resolver.resolve_request().await.map_err(io::Error::other)?;
        let mut body = Vec::new();
        while let Some(mut chunk) = stream.recv_data().await.map_err(io::Error::other)? {
            while chunk.has_remaining() {
                let bytes = chunk.chunk();
                body.extend_from_slice(bytes);
                let length = bytes.len();
                chunk.advance(length);
            }
        }
        stream
            .send_response(Response::builder().status(200).body(()).unwrap())
            .await
            .map_err(io::Error::other)?;
        stream
            .send_data(Bytes::from(body))
            .await
            .map_err(io::Error::other)?;
        stream.finish().await.map_err(io::Error::other)?;
        let _ = h3.accept().await;
        server.wait_idle().await;
        Ok(())
    });
    Ok((address, ca, task))
}

async fn quic_exchange(
    endpoint: SocketAddr,
    roots: RootCertStore,
    bind: Option<SocketAddr>,
) -> io::Result<u64> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let tls = Arc::new(
        rustls::ClientConfig::builder_with_provider(provider)
            .with_protocol_versions(&[&rustls::version::TLS13])
            .map_err(io::Error::other)?
            .with_root_certificates(roots)
            .with_no_client_auth(),
    );
    let mut quic_limits = limits(true);
    quic_limits.max_socks_udp_datagram_bytes = 8192;
    let client_config = build_quic_client_config(&tls, &quic_limits)
        .map_err(|error| io::Error::other(format!("{error:?}")))?;
    let local = bind.unwrap_or("127.0.0.1:0".parse().unwrap());
    let mut client = Endpoint::client(local)?;
    let client_address = client.local_addr()?;
    if let Some(expected) = bind {
        if client_address != expected {
            return Err(io::Error::other("QUIC client reservation changed"));
        }
    }
    client.set_default_client_config(client_config);
    let connection = client
        .connect(endpoint, "localhost")
        .map_err(io::Error::other)?
        .await
        .map_err(io::Error::other)?;
    let (mut driver, mut sender) = h3::client::new(Connection::new(connection.clone()))
        .await
        .map_err(io::Error::other)?;
    let driver_task =
        tokio::spawn(async move { std::future::poll_fn(|cx| driver.poll_close(cx)).await });
    let mut stream = sender
        .send_request(Request::post("https://localhost/payload").body(()).unwrap())
        .await
        .map_err(io::Error::other)?;
    let payload = Bytes::from(vec![0x66; 1024 * 1024]);
    let started = Instant::now();
    for chunk in payload.chunks(16 * 1024) {
        stream
            .send_data(Bytes::copy_from_slice(chunk))
            .await
            .map_err(io::Error::other)?;
    }
    stream.finish().await.map_err(io::Error::other)?;
    let _ = stream.recv_response().await.map_err(io::Error::other)?;
    let mut observed = Vec::new();
    while let Some(mut chunk) = stream.recv_data().await.map_err(io::Error::other)? {
        while chunk.has_remaining() {
            let bytes = chunk.chunk();
            observed.extend_from_slice(bytes);
            let length = bytes.len();
            chunk.advance(length);
        }
    }
    let elapsed = micros(started.elapsed());
    if observed != payload {
        return Err(io::Error::other("QUIC payload changed"));
    }
    connection.close(0_u32.into(), b"complete");
    client.wait_idle().await;
    driver_task.abort();
    let _ = driver_task.await;
    Ok(elapsed)
}

fn authenticate_and_associate(
    proxy: SocketAddr,
    password: &[u8],
    udp_client: SocketAddr,
) -> io::Result<(TcpStream, SocketAddr)> {
    let mut control = TcpStream::connect(proxy)?;
    control.set_read_timeout(Some(Duration::from_secs(3)))?;
    socks_authenticate(&mut control, password)?;
    let relay = socks_associate(&mut control, udp_client)?;
    Ok((control, relay))
}

fn domain_frame(port: u16, payload: &[u8]) -> Vec<u8> {
    let mut frame = vec![0, 0, 0, 3, 9];
    frame.extend_from_slice(b"localhost");
    frame.extend_from_slice(&port.to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}

fn response_payload(frame: &[u8]) -> io::Result<&[u8]> {
    match frame.get(3).copied() {
        Some(1) if frame.len() >= 10 => Ok(&frame[10..]),
        Some(4) if frame.len() >= 22 => Ok(&frame[22..]),
        Some(3) if frame.len() >= 7 + usize::from(frame[4]) => {
            Ok(&frame[7 + usize::from(frame[4])..])
        }
        _ => Err(io::Error::other("invalid SOCKS UDP response")),
    }
}

async fn h2_origin(
    grpc: bool,
) -> io::Result<(SocketAddr, tokio::task::JoinHandle<io::Result<()>>)> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let task = tokio::spawn(async move {
        let (stream, _) = listener.accept().await?;
        let mut connection = h2::server::handshake(stream)
            .await
            .map_err(io::Error::other)?;
        let (request, mut respond) = connection
            .accept()
            .await
            .ok_or_else(|| io::Error::other("missing h2 request"))?
            .map_err(io::Error::other)?;
        let mut handler = tokio::spawn(async move {
            if grpc
                && request
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    != Some("application/grpc")
            {
                return Err(io::Error::other("missing gRPC content type"));
            }
            let mut body = request.into_body();
            let mut payload = Vec::new();
            while let Some(chunk) = body.data().await {
                let chunk = chunk.map_err(io::Error::other)?;
                body.flow_control()
                    .release_capacity(chunk.len())
                    .map_err(io::Error::other)?;
                payload.extend_from_slice(&chunk);
            }
            let mut builder = Response::builder().status(200);
            if grpc {
                builder = builder.header("content-type", "application/grpc");
            }
            let mut send = respond
                .send_response(builder.body(()).unwrap(), false)
                .map_err(io::Error::other)?;
            send_h2_all(&mut send, Bytes::from(payload)).await
        });
        loop {
            tokio::select! {
                result = &mut handler => {
                    let result = result.map_err(io::Error::other)?;
                    loop {
                        match tokio::time::timeout(Duration::from_secs(1), connection.accept()).await {
                            Ok(None) => return result,
                            Ok(Some(Ok(_))) => {
                                return Err(io::Error::other("unexpected second h2 request"));
                            }
                            Ok(Some(Err(_))) => return result,
                            Err(_) => {
                                return Err(io::Error::other("HTTP/2 origin drain timed out"));
                            }
                        }
                    }
                },
                extra = connection.accept() => {
                    match extra {
                        Some(Ok(_)) => {
                            return Err(io::Error::other("unexpected second h2 request"));
                        }
                        Some(Err(error)) => {
                            return Err(io::Error::other(format!(
                                "HTTP/2 origin connection: {error}"
                            )));
                        }
                        None => {
                            return Err(io::Error::other(
                                "HTTP/2 peer closed before origin handler completed",
                            ));
                        }
                    }
                }
            }
        }
    });
    Ok((address, task))
}

async fn h2_exchange(
    endpoint: SocketAddr,
    origin: SocketAddr,
    auth: Option<&str>,
    grpc: bool,
) -> io::Result<u64> {
    let stream = tokio::net::TcpStream::connect(endpoint).await?;
    let (mut sender, connection) = h2::client::handshake(stream)
        .await
        .map_err(|error| io::Error::other(format!("client handshake: {error}")))?;
    let driver = tokio::spawn(connection);
    let uri = format!("http://{origin}/payload");
    let mut builder = Request::builder().method("POST").uri(uri);
    if let Some(auth) = auth {
        builder = builder.header("proxy-authorization", auth);
    }
    if grpc {
        builder = builder.header("content-type", "application/grpc");
    }
    let (response, mut send) = sender
        .send_request(builder.body(()).unwrap(), false)
        .map_err(|error| io::Error::other(format!("request headers: {error}")))?;
    let payload = Bytes::from(vec![0x44; PAYLOAD_BYTES]);
    let started = Instant::now();
    send_h2_all(&mut send, payload.clone())
        .await
        .map_err(|error| io::Error::other(format!("request body: {error}")))?;
    let mut body = response
        .await
        .map_err(|error| io::Error::other(format!("response headers: {error}")))?
        .into_body();
    let mut observed = Vec::new();
    while let Some(chunk) = body.data().await {
        let chunk = chunk.map_err(|error| io::Error::other(format!("response body: {error}")))?;
        body.flow_control()
            .release_capacity(chunk.len())
            .map_err(io::Error::other)?;
        observed.extend_from_slice(&chunk);
    }
    let elapsed = micros(started.elapsed());
    if observed != payload {
        return Err(io::Error::other("HTTP/2 payload changed"));
    }
    drop(sender);
    driver.abort();
    let _ = driver.await;
    Ok(elapsed)
}

async fn send_h2_all(send: &mut h2::SendStream<Bytes>, payload: Bytes) -> io::Result<()> {
    let mut offset = 0;
    while offset < payload.len() {
        send.reserve_capacity(payload.len() - offset);
        let capacity = std::future::poll_fn(|context| send.poll_capacity(context))
            .await
            .ok_or_else(|| io::Error::other("HTTP/2 send stream closed"))?
            .map_err(io::Error::other)?;
        let length = capacity.min(payload.len() - offset);
        let end = offset + length == payload.len();
        send.send_data(payload.slice(offset..offset + length), end)
            .map_err(io::Error::other)?;
        offset += length;
    }
    Ok(())
}

fn read_head(stream: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut head = Vec::new();
    let mut byte = [0];
    while !head.ends_with(b"\r\n\r\n") {
        stream.read_exact(&mut byte)?;
        head.push(byte[0]);
        if head.len() > 64 * 1024 {
            return Err(io::Error::other("header exceeded bound"));
        }
    }
    Ok(head)
}
fn content_length(head: &[u8]) -> io::Result<usize> {
    let text = std::str::from_utf8(head).map_err(io::Error::other)?;
    text.lines()
        .find_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("content-length: ")
                .map(str::to_string)
        })
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| io::Error::other("content length missing"))
}
fn write_websocket(stream: &mut impl Write, payload: &[u8], masked: bool) -> io::Result<()> {
    let mut head = vec![0x82, 127 | if masked { 0x80 } else { 0 }];
    head.extend_from_slice(&(payload.len() as u64).to_be_bytes());
    let key = [1, 2, 3, 4];
    if masked {
        head.extend_from_slice(&key);
    }
    stream.write_all(&head)?;
    if masked {
        let encoded: Vec<u8> = payload
            .iter()
            .enumerate()
            .map(|(i, b)| b ^ key[i % 4])
            .collect();
        stream.write_all(&encoded)
    } else {
        stream.write_all(payload)
    }
}
fn read_websocket(stream: &mut impl Read, masked: bool) -> io::Result<Vec<u8>> {
    let mut head = [0; 2];
    stream.read_exact(&mut head)?;
    if head[1] & 0x7f != 127 {
        return Err(io::Error::other("expected long WebSocket frame"));
    }
    let mut length = [0; 8];
    stream.read_exact(&mut length)?;
    let length = usize::try_from(u64::from_be_bytes(length)).map_err(io::Error::other)?;
    let mut key = [0; 4];
    if masked {
        stream.read_exact(&mut key)?;
    }
    let mut payload = vec![0; length];
    stream.read_exact(&mut payload)?;
    if masked {
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= key[i % 4];
        }
    }
    Ok(payload)
}
fn socks_authenticate(stream: &mut TcpStream, password: &[u8]) -> io::Result<()> {
    stream.write_all(&[5, 1, 2])?;
    let mut reply = [0; 2];
    stream.read_exact(&mut reply)?;
    let mut request = vec![1, 7];
    request.extend_from_slice(b"fragcap");
    request.push(password.len() as u8);
    request.extend_from_slice(password);
    stream.write_all(&request)?;
    stream.read_exact(&mut reply)?;
    if reply != [1, 0] {
        return Err(io::Error::other("SOCKS authentication failed"));
    }
    Ok(())
}
fn socks_connect(stream: &mut TcpStream, address: SocketAddr) -> io::Result<()> {
    let SocketAddr::V4(address) = address else {
        return Err(io::Error::other("benchmark requires IPv4"));
    };
    let mut request = vec![5, 1, 0, 1];
    request.extend_from_slice(&address.ip().octets());
    request.extend_from_slice(&address.port().to_be_bytes());
    stream.write_all(&request)?;
    let mut reply = [0; 10];
    stream.read_exact(&mut reply)?;
    if reply[1] != 0 {
        return Err(io::Error::other("SOCKS connect failed"));
    }
    Ok(())
}
fn socks_associate(stream: &mut TcpStream, client: SocketAddr) -> io::Result<SocketAddr> {
    let SocketAddr::V4(client) = client else {
        return Err(io::Error::other("benchmark requires IPv4"));
    };
    let mut request = vec![5, 3, 0, 1];
    request.extend_from_slice(&client.ip().octets());
    request.extend_from_slice(&client.port().to_be_bytes());
    stream.write_all(&request)?;
    let mut reply = [0; 10];
    stream.read_exact(&mut reply)?;
    if reply[1] != 0 {
        return Err(io::Error::other("SOCKS associate failed"));
    }
    Ok(SocketAddr::from((
        [reply[4], reply[5], reply[6], reply[7]],
        u16::from_be_bytes([reply[8], reply[9]]),
    )))
}
fn udp_frame(address: SocketAddr, payload: &[u8]) -> Vec<u8> {
    let SocketAddr::V4(address) = address else {
        unreachable!()
    };
    let mut frame = vec![0, 0, 0, 1];
    frame.extend_from_slice(&address.ip().octets());
    frame.extend_from_slice(&address.port().to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}
fn udp_echo(socket: UdpSocket, count: usize) -> std::thread::JoinHandle<io::Result<()>> {
    std::thread::spawn(move || {
        let mut buffer = vec![0; 65535];
        for _ in 0..count {
            let (read, peer) = socket.recv_from(&mut buffer)?;
            socket.send_to(&buffer[..read], peer)?;
        }
        Ok(())
    })
}
fn combine(direct: u64, proxied: u64, tail: MeasurementTail, useful_bytes: u64) -> Measurement {
    Measurement {
        direct_microseconds: direct,
        proxy_microseconds: proxied,
        useful_bytes,
        shutdown_microseconds: tail.shutdown_microseconds,
        clean_shutdown: tail.clean_shutdown,
        resources: tail.resources,
    }
}
fn micros(value: Duration) -> u64 {
    value.as_micros().min(u128::from(u64::MAX)) as u64
}
