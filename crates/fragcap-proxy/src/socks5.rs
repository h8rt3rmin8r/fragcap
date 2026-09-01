// SPDX-License-Identifier: Apache-2.0

//! Bounded SOCKS5 TCP negotiation and byte-transparent relay.

use std::future::Future;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{watch, Notify};
use tokio::time::{timeout, Instant};

use crate::http1::{HttpRun, ProtocolError};
use crate::{
    connect_upstream, ApplicationEvent, ApplicationEventKind, DestinationAuthority,
    DestinationPolicy, ProtocolAccounting, ProtocolLimits, ProxyObservation, SessionCapability,
    SocksClassification, SocksConnectEvent, SocksNegotiationEvent, SocksTransferEvent,
    UpstreamError, UpstreamStage,
};

const SOCKS_VERSION: u8 = 5;
const AUTH_VERSION: u8 = 1;
const USERNAME_PASSWORD: u8 = 2;
const NO_ACCEPTABLE_METHODS: u8 = 0xff;
const CONNECT: u8 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SocksAddressType {
    Ipv4,
    Domain,
    Ipv6,
}

impl SocksAddressType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ipv4 => "ipv4",
            Self::Domain => "domain",
            Self::Ipv6 => "ipv6",
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SocksReplyCode {
    Succeeded = 0,
    GeneralFailure = 1,
    NetworkUnreachable = 3,
    HostUnreachable = 4,
    ConnectionRefused = 5,
    TtlExpired = 6,
    CommandNotSupported = 7,
    AddressTypeNotSupported = 8,
}

#[derive(Debug)]
struct ConnectRequest {
    authority: DestinationAuthority,
    address_type: SocksAddressType,
}

pub(crate) struct SocksConnectionContext<'a> {
    pub limits: &'a ProtocolLimits,
    pub shutdown: watch::Receiver<bool>,
    pub session_id: &'a str,
    pub connection_id: u64,
    pub peer: SocketAddr,
    pub local: SocketAddr,
    pub sink: &'a crate::application::SharedEventSink,
    pub buffer_bytes: usize,
}

pub(crate) async fn is_socks5(stream: &TcpStream, budget: Duration) -> bool {
    let mut byte = [0_u8; 1];
    matches!(timeout(budget, stream.peek(&mut byte)).await, Ok(Ok(1))) && byte[0] == SOCKS_VERSION
}

pub(crate) async fn serve_socks5(
    mut client: TcpStream,
    capability: &SessionCapability,
    policy: &DestinationPolicy,
    context: SocksConnectionContext<'_>,
) -> HttpRun {
    let SocksConnectionContext {
        limits,
        mut shutdown,
        session_id,
        connection_id,
        peer,
        local,
        sink,
        buffer_bytes,
    } = context;
    let mut accounting = ProtocolAccounting {
        socks_negotiations: 1,
        ..Default::default()
    };
    let result = tokio::select! {
        _ = shutdown.changed() => Err(SocksFailure::cancelled()),
        result = negotiate(&mut client, capability, limits.header_timeout) => result,
    };
    if let Err(failure) = result {
        if failure.authentication_refused {
            accounting.socks_auth_refused = 1;
        } else {
            accounting.parse_refused = 1;
        }
        accounting.timed_out = u64::from(failure.timed_out);
        crate::application::emit(
            sink,
            ApplicationEvent::now(
                session_id,
                connection_id,
                None,
                None,
                ApplicationEventKind::SocksNegotiation(SocksNegotiationEvent {
                    authenticated: false,
                }),
            ),
        );
        return failed(accounting, failure);
    }
    accounting.socks_auth_succeeded = 1;
    crate::application::emit(
        sink,
        ApplicationEvent::now(
            session_id,
            connection_id,
            None,
            None,
            ApplicationEventKind::SocksNegotiation(SocksNegotiationEvent {
                authenticated: true,
            }),
        ),
    );

    let request = tokio::select! {
        _ = shutdown.changed() => Err(SocksFailure::cancelled()),
        result = read_connect_request(&mut client, limits.header_timeout) => result,
    };
    let request = match request {
        Ok(request) => request,
        Err(failure) => {
            accounting.parse_refused = 1;
            accounting.socks_connect_refused = 1;
            crate::application::emit(
                sink,
                ApplicationEvent::now(
                    session_id,
                    connection_id,
                    None,
                    None,
                    ApplicationEventKind::Error { code: failure.code },
                ),
            );
            let _ = write_reply(&mut client, failure.reply, None).await;
            return failed(accounting, failure);
        }
    };
    accounting.socks_connect_requested = 1;
    match request.address_type {
        SocksAddressType::Ipv4 => accounting.socks_ipv4 = 1,
        SocksAddressType::Ipv6 => accounting.socks_ipv6 = 1,
        SocksAddressType::Domain => {
            accounting.socks_domain = 1;
            accounting.socks_dns_owned = 1;
        }
    }
    let authority_text = request.authority.lookup_host();
    let upstream = tokio::select! {
        _ = shutdown.changed() => Err(SocksFailure::cancelled()),
        result = connect_upstream(&request.authority, policy, limits.upstream) => {
            result.map_err(SocksFailure::upstream)
        }
    };
    let mut upstream = match upstream {
        Ok(upstream) => upstream,
        Err(failure) => {
            accounting.socks_connect_refused = 1;
            accounting.policy_refused = u64::from(failure.policy_refused);
            accounting.timed_out = u64::from(failure.timed_out);
            let _ = write_reply(&mut client, failure.reply, None).await;
            emit_connect(
                sink,
                session_id,
                connection_id,
                &authority_text,
                request.address_type,
                "refused",
                None,
            );
            return failed_with_observation(
                accounting,
                failure,
                observation(
                    session_id,
                    connection_id,
                    peer,
                    local,
                    &authority_text,
                    "inconclusive",
                ),
            );
        }
    };
    let bound = upstream.local_addr().ok();
    if let Err(error) = write_reply(&mut client, SocksReplyCode::Succeeded, bound).await {
        accounting.socks_connect_refused = 1;
        return failed(
            accounting,
            SocksFailure::new("socks-reply-write-failed", error.to_string()),
        );
    }
    accounting.socks_connect_succeeded = 1;
    let classification = classify_prefix(
        &client,
        limits.header_timeout.min(Duration::from_millis(25)),
    )
    .await;
    match classification {
        SocksClassification::Http => accounting.socks_http = 1,
        SocksClassification::Tls => accounting.socks_tls = 1,
        SocksClassification::OpaqueTcp => accounting.socks_tcp_opaque = 1,
    }
    emit_connect(
        sink,
        session_id,
        connection_id,
        &authority_text,
        request.address_type,
        "connected",
        Some(classification),
    );
    let client_bytes = AtomicU64::new(0);
    let upstream_bytes = AtomicU64::new(0);
    let activity = Notify::new();
    let forwarding = relay_with_idle_timeout(
        &mut client,
        &mut upstream,
        RelayContext {
            buffer_bytes,
            idle_timeout: limits.idle_timeout,
            upstream_read_timeout: limits.upstream.read,
            upstream_write_timeout: limits.upstream.write,
            shutdown: &mut shutdown,
            client_bytes: &client_bytes,
            upstream_bytes: &upstream_bytes,
            activity: &activity,
        },
    )
    .await;
    let client_bytes = client_bytes.load(Ordering::Relaxed);
    let upstream_bytes = upstream_bytes.load(Ordering::Relaxed);
    accounting.socks_client_bytes = client_bytes;
    accounting.socks_upstream_bytes = upstream_bytes;
    crate::application::emit(
        sink,
        ApplicationEvent::now(
            session_id,
            connection_id,
            None,
            None,
            ApplicationEventKind::SocksTransfer(SocksTransferEvent {
                client_to_upstream_bytes: client_bytes,
                upstream_to_client_bytes: upstream_bytes,
            }),
        ),
    );
    match forwarding {
        Ok(()) => HttpRun {
            observations: vec![observation(
                session_id,
                connection_id,
                peer,
                local,
                &authority_text,
                classification.as_str(),
            )],
            accounting,
            failure: None,
        },
        Err(failure) => {
            accounting.timed_out = u64::from(failure.timed_out);
            failed_with_observation(
                accounting,
                failure,
                observation(
                    session_id,
                    connection_id,
                    peer,
                    local,
                    &authority_text,
                    classification.as_str(),
                ),
            )
        }
    }
}

async fn negotiate(
    stream: &mut TcpStream,
    capability: &SessionCapability,
    budget: Duration,
) -> Result<(), SocksFailure> {
    let deadline = Instant::now() + budget;
    let mut head = [0_u8; 2];
    read_exact_until(stream, &mut head, deadline, "socks-greeting").await?;
    if head[0] != SOCKS_VERSION || head[1] == 0 {
        return Err(SocksFailure::auth("socks-greeting-invalid"));
    }
    let mut methods = vec![0_u8; usize::from(head[1])];
    read_exact_until(stream, &mut methods, deadline, "socks-methods").await?;
    if !methods.contains(&USERNAME_PASSWORD) {
        let _ = stream
            .write_all(&[SOCKS_VERSION, NO_ACCEPTABLE_METHODS])
            .await;
        return Err(SocksFailure::auth("socks-auth-method-required"));
    }
    stream
        .write_all(&[SOCKS_VERSION, USERNAME_PASSWORD])
        .await
        .map_err(|error| SocksFailure::new("socks-method-write-failed", error.to_string()))?;
    let mut auth_head = [0_u8; 2];
    read_exact_until(stream, &mut auth_head, deadline, "socks-auth-head").await?;
    if auth_head[0] != AUTH_VERSION || auth_head[1] == 0 {
        let _ = stream.write_all(&[AUTH_VERSION, 1]).await;
        return Err(SocksFailure::auth("socks-auth-malformed"));
    }
    let mut username = vec![0_u8; usize::from(auth_head[1])];
    read_exact_until(stream, &mut username, deadline, "socks-username").await?;
    let mut password_len = [0_u8; 1];
    read_exact_until(stream, &mut password_len, deadline, "socks-password-length").await?;
    if password_len[0] == 0 {
        let _ = stream.write_all(&[AUTH_VERSION, 1]).await;
        return Err(SocksFailure::auth("socks-auth-malformed"));
    }
    let mut password = zeroize::Zeroizing::new(vec![0_u8; usize::from(password_len[0])]);
    read_exact_until(stream, password.as_mut_slice(), deadline, "socks-password").await?;
    if !capability.authenticates_socks_credentials(&username, &password) {
        let _ = stream.write_all(&[AUTH_VERSION, 1]).await;
        return Err(SocksFailure::auth("socks-auth-refused"));
    }
    stream
        .write_all(&[AUTH_VERSION, 0])
        .await
        .map_err(|error| SocksFailure::new("socks-auth-write-failed", error.to_string()))
}

async fn read_connect_request(
    stream: &mut TcpStream,
    budget: Duration,
) -> Result<ConnectRequest, SocksFailure> {
    let deadline = Instant::now() + budget;
    let mut head = [0_u8; 4];
    read_exact_until(stream, &mut head, deadline, "socks-request-head").await?;
    if head[0] != SOCKS_VERSION || head[2] != 0 {
        return Err(SocksFailure::reply(
            "socks-request-invalid",
            SocksReplyCode::GeneralFailure,
        ));
    }
    let (host, address_type) = match head[3] {
        1 => {
            let mut octets = [0_u8; 4];
            read_exact_until(stream, &mut octets, deadline, "socks-ipv4").await?;
            (
                IpAddr::V4(Ipv4Addr::from(octets)).to_string(),
                SocksAddressType::Ipv4,
            )
        }
        3 => {
            let mut length = [0_u8; 1];
            read_exact_until(stream, &mut length, deadline, "socks-domain-length").await?;
            if length[0] == 0 {
                return Err(SocksFailure::reply(
                    "socks-domain-empty",
                    SocksReplyCode::AddressTypeNotSupported,
                ));
            }
            let mut bytes = vec![0_u8; usize::from(length[0])];
            read_exact_until(stream, &mut bytes, deadline, "socks-domain").await?;
            let domain = std::str::from_utf8(&bytes).map_err(|_| {
                SocksFailure::reply(
                    "socks-domain-invalid",
                    SocksReplyCode::AddressTypeNotSupported,
                )
            })?;
            (domain.to_string(), SocksAddressType::Domain)
        }
        4 => {
            let mut octets = [0_u8; 16];
            read_exact_until(stream, &mut octets, deadline, "socks-ipv6").await?;
            (
                format!("[{}]", Ipv6Addr::from(octets)),
                SocksAddressType::Ipv6,
            )
        }
        _ => {
            return Err(SocksFailure::reply(
                "socks-address-type-unsupported",
                SocksReplyCode::AddressTypeNotSupported,
            ))
        }
    };
    let mut port = [0_u8; 2];
    read_exact_until(stream, &mut port, deadline, "socks-port").await?;
    if head[1] != CONNECT {
        return Err(SocksFailure::reply(
            "socks-command-unsupported",
            SocksReplyCode::CommandNotSupported,
        ));
    }
    let port = u16::from_be_bytes(port);
    let authority = DestinationAuthority::parse(&format!("{host}:{port}")).map_err(|error| {
        SocksFailure::reply(error.code, SocksReplyCode::AddressTypeNotSupported)
    })?;
    Ok(ConnectRequest {
        authority,
        address_type,
    })
}

async fn read_exact(
    stream: &mut TcpStream,
    bytes: &mut [u8],
    budget: Duration,
    stage: &'static str,
) -> Result<(), SocksFailure> {
    timeout(budget, stream.read_exact(bytes))
        .await
        .map_err(|_| SocksFailure::timeout(stage, SocksReplyCode::TtlExpired))?
        .map(|_| ())
        .map_err(|error| SocksFailure::new(stage, error.to_string()))
}

async fn read_exact_until(
    stream: &mut TcpStream,
    bytes: &mut [u8],
    deadline: Instant,
    stage: &'static str,
) -> Result<(), SocksFailure> {
    read_exact(
        stream,
        bytes,
        deadline.saturating_duration_since(Instant::now()),
        stage,
    )
    .await
}

struct RelayContext<'a> {
    buffer_bytes: usize,
    idle_timeout: Duration,
    upstream_read_timeout: Duration,
    upstream_write_timeout: Duration,
    shutdown: &'a mut watch::Receiver<bool>,
    client_bytes: &'a AtomicU64,
    upstream_bytes: &'a AtomicU64,
    activity: &'a Notify,
}

async fn relay_with_idle_timeout<U>(
    client: &mut TcpStream,
    upstream: &mut U,
    context: RelayContext<'_>,
) -> Result<(), SocksFailure>
where
    U: AsyncRead + AsyncWrite + Unpin,
{
    let RelayContext {
        buffer_bytes,
        idle_timeout,
        upstream_read_timeout,
        upstream_write_timeout,
        shutdown,
        client_bytes,
        upstream_bytes,
        activity,
    } = context;
    let progress = RelayProgress {
        client_bytes,
        upstream_bytes,
        activity,
    };
    let relay = relay_bidirectional(
        client,
        upstream,
        buffer_bytes,
        upstream_read_timeout,
        upstream_write_timeout,
        &progress,
    );
    tokio::pin!(relay);
    let idle = tokio::time::sleep(idle_timeout);
    tokio::pin!(idle);
    loop {
        tokio::select! {
            biased;
            _ = shutdown.changed() => return Err(SocksFailure::cancelled()),
            _ = activity.notified() => idle.as_mut().reset(Instant::now() + idle_timeout),
            result = &mut relay => {
                return result.map_err(|error| match error.kind() {
                    io::ErrorKind::TimedOut => SocksFailure::timeout(
                        "socks-forward-operation-timeout",
                        SocksReplyCode::TtlExpired,
                    ),
                    _ => SocksFailure::new("socks-forward-failed", error.to_string()),
                });
            }
            _ = &mut idle => {
                return Err(SocksFailure::timeout(
                    "socks-forward-timeout",
                    SocksReplyCode::TtlExpired,
                ));
            }
        }
    }
}

struct RelayProgress<'a> {
    client_bytes: &'a AtomicU64,
    upstream_bytes: &'a AtomicU64,
    activity: &'a Notify,
}

async fn relay_bidirectional<U>(
    client: &mut TcpStream,
    upstream: &mut U,
    buffer_bytes: usize,
    upstream_read_timeout: Duration,
    upstream_write_timeout: Duration,
    progress: &RelayProgress<'_>,
) -> io::Result<()>
where
    U: AsyncRead + AsyncWrite + Unpin,
{
    let (client_read, client_write) = tokio::io::split(client);
    let (upstream_read, upstream_write) = tokio::io::split(upstream);
    tokio::try_join!(
        relay_direction(
            client_read,
            upstream_write,
            buffer_bytes,
            None,
            Some(upstream_write_timeout),
            progress.client_bytes,
            progress.activity,
        ),
        relay_direction(
            upstream_read,
            client_write,
            buffer_bytes,
            Some(upstream_read_timeout),
            None,
            progress.upstream_bytes,
            progress.activity,
        ),
    )?;
    Ok(())
}

async fn relay_direction<R, W>(
    mut reader: R,
    mut writer: W,
    buffer_bytes: usize,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
    transferred: &AtomicU64,
    activity: &Notify,
) -> io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buffer = vec![0_u8; buffer_bytes];
    loop {
        let read = timed_io(read_timeout, reader.read(&mut buffer)).await?;
        if read == 0 {
            timed_io(write_timeout, writer.shutdown()).await?;
            return Ok(());
        }
        activity.notify_one();
        let mut written = 0;
        while written < read {
            let count = timed_io(write_timeout, writer.write(&buffer[written..read])).await?;
            if count == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "SOCKS relay write made no progress",
                ));
            }
            written += count;
            transferred.fetch_add(count as u64, Ordering::Relaxed);
            activity.notify_one();
        }
    }
}

async fn timed_io<T>(
    budget: Option<Duration>,
    operation: impl Future<Output = io::Result<T>>,
) -> io::Result<T> {
    match budget {
        Some(budget) => timeout(budget, operation)
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "SOCKS relay I/O timed out"))?,
        None => operation.await,
    }
}

async fn write_reply(
    stream: &mut TcpStream,
    reply: SocksReplyCode,
    bound: Option<SocketAddr>,
) -> io::Result<()> {
    let bound = bound.unwrap_or_else(|| "0.0.0.0:0".parse().expect("literal address"));
    let mut bytes = vec![SOCKS_VERSION, reply as u8, 0];
    match bound {
        SocketAddr::V4(address) => {
            bytes.push(1);
            bytes.extend_from_slice(&address.ip().octets());
            bytes.extend_from_slice(&address.port().to_be_bytes());
        }
        SocketAddr::V6(address) => {
            bytes.push(4);
            bytes.extend_from_slice(&address.ip().octets());
            bytes.extend_from_slice(&address.port().to_be_bytes());
        }
    }
    stream.write_all(&bytes).await
}

async fn classify_prefix(stream: &TcpStream, budget: Duration) -> SocksClassification {
    let mut bytes = [0_u8; 8];
    let read = timeout(budget, stream.peek(&mut bytes))
        .await
        .ok()
        .and_then(Result::ok)
        .unwrap_or(0);
    let prefix = &bytes[..read];
    if [
        b"GET ".as_slice(),
        b"POST ",
        b"PUT ",
        b"HEAD ",
        b"PATCH ",
        b"DELETE ",
        b"OPTIONS ",
        b"CONNECT ",
    ]
    .iter()
    .any(|method| prefix.len() >= method.len() && prefix.starts_with(method))
    {
        SocksClassification::Http
    } else if prefix.len() >= 3 && prefix[0] == 0x16 && prefix[1] == 0x03 {
        SocksClassification::Tls
    } else {
        SocksClassification::OpaqueTcp
    }
}

fn emit_connect(
    sink: &crate::application::SharedEventSink,
    session_id: &str,
    connection_id: u64,
    authority: &str,
    address_type: SocksAddressType,
    outcome: &'static str,
    classification: Option<SocksClassification>,
) {
    crate::application::emit(
        sink,
        ApplicationEvent::now(
            session_id,
            connection_id,
            None,
            None,
            ApplicationEventKind::SocksConnect(SocksConnectEvent {
                authority: authority.to_string(),
                address_type: address_type.as_str(),
                dns_owner: if address_type == SocksAddressType::Domain {
                    "proxy"
                } else {
                    "not-required"
                },
                outcome,
                classification,
            }),
        ),
    );
}

fn observation(
    session_id: &str,
    connection_id: u64,
    peer: SocketAddr,
    local: SocketAddr,
    authority: &str,
    classification: &str,
) -> ProxyObservation {
    ProxyObservation {
        session_id: session_id.to_string(),
        connection_id,
        request_ordinal: 1,
        client_peer: peer,
        proxy_local: local,
        timestamp_ns: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .try_into()
            .unwrap_or(u64::MAX),
        connection_opened_at_ns: 0,
        connection_closed_at_ns: 0,
        protocol: "socks5".to_string(),
        method: Some("CONNECT".to_string()),
        url: Some(format!("tcp://{authority}")),
        status: Some(0),
        inspectability: "metadata-only",
        reason: Some(classification.to_string()),
        tls: None,
        transformations: Vec::new(),
    }
}

#[derive(Debug)]
struct SocksFailure {
    code: &'static str,
    detail: String,
    reply: SocksReplyCode,
    authentication_refused: bool,
    policy_refused: bool,
    timed_out: bool,
}

impl SocksFailure {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
            reply: SocksReplyCode::GeneralFailure,
            authentication_refused: false,
            policy_refused: false,
            timed_out: false,
        }
    }

    fn auth(code: &'static str) -> Self {
        let mut failure = Self::new(code, code);
        failure.authentication_refused = true;
        failure
    }

    fn reply(code: &'static str, reply: SocksReplyCode) -> Self {
        let mut failure = Self::new(code, code);
        failure.reply = reply;
        failure
    }

    fn timeout(code: &'static str, reply: SocksReplyCode) -> Self {
        let mut failure = Self::reply(code, reply);
        failure.timed_out = true;
        failure
    }

    fn cancelled() -> Self {
        Self::new("connection-cancelled", "runtime is stopping")
    }

    fn upstream(error: UpstreamError) -> Self {
        let reply = match error.code {
            "connection-refused" => SocksReplyCode::ConnectionRefused,
            "network-unreachable" => SocksReplyCode::NetworkUnreachable,
            "host-unreachable" => SocksReplyCode::HostUnreachable,
            "dns-failed" | "dns-empty" => SocksReplyCode::HostUnreachable,
            "connect-timeout" | "dns-timeout" => SocksReplyCode::TtlExpired,
            _ if error.stage == UpstreamStage::Policy => SocksReplyCode::ConnectionRefused,
            _ => SocksReplyCode::GeneralFailure,
        };
        let mut failure = Self::reply(error.code, reply);
        failure.detail = error.detail;
        failure.policy_refused = error.stage == UpstreamStage::Policy;
        failure.timed_out = matches!(error.code, "connect-timeout" | "dns-timeout");
        failure
    }
}

fn failed(accounting: ProtocolAccounting, failure: SocksFailure) -> HttpRun {
    failed_with_observation(accounting, failure, Option::<ProxyObservation>::None)
}

fn failed_with_observation(
    accounting: ProtocolAccounting,
    failure: SocksFailure,
    observation: impl Into<Option<ProxyObservation>>,
) -> HttpRun {
    let mut error = ProtocolError::new(failure.code, failure.detail);
    error.authentication_refused = failure.authentication_refused;
    error.policy_refused = failure.policy_refused;
    error.timed_out = failure.timed_out;
    HttpRun {
        observations: observation.into().into_iter().collect(),
        accounting,
        failure: Some(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn relay_honors_upstream_read_and_write_budgets() {
        let (_idle_source, source) = tokio::io::duplex(1);
        let (destination, _idle_destination) = tokio::io::duplex(1);
        let transferred = AtomicU64::new(0);
        let activity = Notify::new();
        let read_timeout = relay_direction(
            source,
            destination,
            8,
            Some(Duration::from_millis(20)),
            None,
            &transferred,
            &activity,
        )
        .await
        .unwrap_err();
        assert_eq!(read_timeout.kind(), io::ErrorKind::TimedOut);

        let (mut source, reader) = tokio::io::duplex(8);
        let (writer, _blocked_reader) = tokio::io::duplex(1);
        source.write_all(b"bounded").await.unwrap();
        source.shutdown().await.unwrap();
        let write_timeout = relay_direction(
            reader,
            writer,
            8,
            None,
            Some(Duration::from_millis(20)),
            &transferred,
            &activity,
        )
        .await
        .unwrap_err();
        assert_eq!(write_timeout.kind(), io::ErrorKind::TimedOut);
        assert_eq!(transferred.load(Ordering::Relaxed), 1);
    }
}
