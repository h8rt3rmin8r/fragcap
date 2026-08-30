// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, RootCertStore};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio::time::{timeout, Instant};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AuthorityHost {
    Dns(String),
    Ip(IpAddr),
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DestinationAuthority {
    host: AuthorityHost,
    port: u16,
}

impl DestinationAuthority {
    pub fn parse(value: &str) -> Result<Self, UpstreamError> {
        if value.contains('@') || value.contains("%25") || value.contains('%') {
            return Err(UpstreamError::new(
                UpstreamStage::Authority,
                "invalid-authority",
            ));
        }
        let authority = hyper::http::uri::Authority::from_str(value)
            .map_err(|_| UpstreamError::new(UpstreamStage::Authority, "invalid-authority"))?;
        let port = authority
            .port_u16()
            .ok_or_else(|| UpstreamError::new(UpstreamStage::Authority, "missing-port"))?;
        if port == 0 {
            return Err(UpstreamError::new(UpstreamStage::Authority, "invalid-port"));
        }
        let raw_host = authority.host();
        let host = match IpAddr::from_str(raw_host.trim_matches(['[', ']'])) {
            Ok(ip) => AuthorityHost::Ip(ip),
            Err(_) if valid_dns_name(raw_host) => AuthorityHost::Dns(raw_host.to_ascii_lowercase()),
            Err(_) => return Err(UpstreamError::new(UpstreamStage::Authority, "invalid-host")),
        };
        Ok(Self { host, port })
    }

    pub fn host(&self) -> &AuthorityHost {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn lookup_host(&self) -> String {
        match &self.host {
            AuthorityHost::Dns(name) => format!("{name}:{}", self.port),
            AuthorityHost::Ip(IpAddr::V4(ip)) => format!("{ip}:{}", self.port),
            AuthorityHost::Ip(IpAddr::V6(ip)) => format!("[{ip}]:{}", self.port),
        }
    }
}

fn valid_dns_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && value.is_ascii()
        && !value.starts_with('.')
        && !value.ends_with('.')
        && !value.contains("..")
        && value.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
}

#[derive(Clone, Debug)]
pub struct DestinationPolicy {
    listener: SocketAddr,
    exact_grants: BTreeSet<SocketAddr>,
}

impl DestinationPolicy {
    pub fn new(listener: SocketAddr) -> Self {
        Self {
            listener: normalize_address(listener),
            exact_grants: BTreeSet::new(),
        }
    }

    pub fn grant_for_test(&mut self, address: SocketAddr) {
        let address = normalize_address(address);
        if address != self.listener {
            self.exact_grants.insert(address);
        }
    }

    pub fn evaluate(&self, address: SocketAddr) -> DestinationDecision {
        let normalized = normalize_address(address);
        let allowed = normalized != self.listener
            && (self.exact_grants.contains(&normalized) || public_address(normalized.ip()));
        DestinationDecision {
            address,
            allowed,
            reason: if allowed {
                "allowed"
            } else {
                "destination-refused"
            },
        }
    }
}

fn normalize_address(address: SocketAddr) -> SocketAddr {
    match address {
        SocketAddr::V6(address) => address
            .ip()
            .to_ipv4_mapped()
            .map(|ip| SocketAddr::new(IpAddr::V4(ip), address.port()))
            .unwrap_or(SocketAddr::V6(address)),
        address => address,
    }
}

fn public_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            !(ip.is_loopback()
                || ip.is_private()
                || ip.is_link_local()
                || ip.is_broadcast()
                || ip.is_documentation()
                || ip.is_unspecified()
                || ip.is_multicast())
        }
        IpAddr::V6(ip) => {
            !(ip.is_loopback()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
                || ip.is_unspecified()
                || ip.is_multicast())
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DestinationDecision {
    pub address: SocketAddr,
    pub allowed: bool,
    pub reason: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UpstreamBudgets {
    pub dns: Duration,
    pub connect: Duration,
    pub read: Duration,
    pub write: Duration,
}

#[derive(Clone, Debug, Default)]
pub struct UpstreamCancellation {
    inner: Arc<CancellationState>,
}

#[derive(Debug, Default)]
struct CancellationState {
    cancelled: AtomicBool,
    changed: Notify,
}

impl UpstreamCancellation {
    pub fn cancel(&self) {
        if !self.inner.cancelled.swap(true, Ordering::AcqRel) {
            self.inner.changed.notify_waiters();
        }
    }

    async fn cancelled(&self) {
        loop {
            let changed = self.inner.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if self.inner.cancelled.load(Ordering::Acquire) {
                return;
            }
            changed.await;
        }
    }

    fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::Acquire)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpstreamStage {
    Authority,
    Dns,
    Policy,
    Tcp,
    Tls,
    Read,
    Write,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpstreamError {
    pub stage: UpstreamStage,
    pub code: &'static str,
    pub detail: String,
}

impl UpstreamError {
    fn new(stage: UpstreamStage, code: &'static str) -> Self {
        Self {
            stage,
            code,
            detail: code.to_string(),
        }
    }

    fn with_detail(stage: UpstreamStage, code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            stage,
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for UpstreamError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for UpstreamError {}

#[derive(Debug)]
pub struct BoundedUpstreamStream {
    stream: TcpStream,
    read_budget: Duration,
    write_budget: Duration,
}

#[derive(Debug)]
pub struct BoundedTlsUpstreamStream {
    stream: tokio_rustls::client::TlsStream<TcpStream>,
    read_budget: Duration,
    write_budget: Duration,
}

impl AsyncRead for BoundedTlsUpstreamStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buffer)
    }
}

impl AsyncWrite for BoundedTlsUpstreamStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, bytes)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

impl BoundedTlsUpstreamStream {
    pub(crate) fn protocol_version(&self) -> Option<rustls::ProtocolVersion> {
        self.stream.get_ref().1.protocol_version()
    }

    pub(crate) fn alpn_protocol(&self) -> Option<Vec<u8>> {
        self.stream.get_ref().1.alpn_protocol().map(<[u8]>::to_vec)
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, UpstreamError> {
        timeout(self.read_budget, self.stream.read(buffer))
            .await
            .map_err(|_| UpstreamError::new(UpstreamStage::Read, "read-timeout"))?
            .map_err(|error| {
                UpstreamError::with_detail(UpstreamStage::Read, "read-failed", error.to_string())
            })
    }

    pub async fn write_all(&mut self, bytes: &[u8]) -> Result<(), UpstreamError> {
        timeout(self.write_budget, self.stream.write_all(bytes))
            .await
            .map_err(|_| UpstreamError::new(UpstreamStage::Write, "write-timeout"))?
            .map_err(|error| {
                UpstreamError::with_detail(UpstreamStage::Write, "write-failed", error.to_string())
            })
    }
}

impl BoundedUpstreamStream {
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, UpstreamError> {
        timeout(self.read_budget, self.stream.read(buffer))
            .await
            .map_err(|_| UpstreamError::new(UpstreamStage::Read, "read-timeout"))?
            .map_err(|error| {
                UpstreamError::with_detail(UpstreamStage::Read, "read-failed", error.to_string())
            })
    }

    pub async fn write_all(&mut self, bytes: &[u8]) -> Result<(), UpstreamError> {
        timeout(self.write_budget, self.stream.write_all(bytes))
            .await
            .map_err(|_| UpstreamError::new(UpstreamStage::Write, "write-timeout"))?
            .map_err(|error| {
                UpstreamError::with_detail(UpstreamStage::Write, "write-failed", error.to_string())
            })
    }
}

impl AsyncRead for BoundedUpstreamStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buffer)
    }
}

impl AsyncWrite for BoundedUpstreamStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, bytes)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

pub async fn connect_upstream(
    authority: &DestinationAuthority,
    policy: &DestinationPolicy,
    budgets: UpstreamBudgets,
) -> Result<BoundedUpstreamStream, UpstreamError> {
    connect_upstream_cancellable(authority, policy, budgets, &UpstreamCancellation::default()).await
}

pub async fn connect_upstream_cancellable(
    authority: &DestinationAuthority,
    policy: &DestinationPolicy,
    budgets: UpstreamBudgets,
    cancellation: &UpstreamCancellation,
) -> Result<BoundedUpstreamStream, UpstreamError> {
    if cancellation.is_cancelled() {
        return Err(UpstreamError::new(UpstreamStage::Cancelled, "cancelled"));
    }
    let resolved = tokio::select! {
        () = cancellation.cancelled() => {
            return Err(UpstreamError::new(UpstreamStage::Cancelled, "cancelled"));
        }
        result = timeout(budgets.dns, tokio::net::lookup_host(authority.lookup_host())) => result,
    }
    .map_err(|_| UpstreamError::new(UpstreamStage::Dns, "dns-timeout"))?
    .map_err(|error| {
        UpstreamError::with_detail(UpstreamStage::Dns, "dns-failed", error.to_string())
    })?;
    let addresses: Vec<_> = resolved.collect();
    if addresses.is_empty() {
        return Err(UpstreamError::new(UpstreamStage::Dns, "dns-empty"));
    }
    let mut last = None;
    let deadline = Instant::now() + budgets.connect;
    for address in addresses {
        if cancellation.is_cancelled() {
            return Err(UpstreamError::new(UpstreamStage::Cancelled, "cancelled"));
        }
        let decision = policy.evaluate(address);
        if !decision.allowed {
            last = Some(UpstreamError::new(UpstreamStage::Policy, decision.reason));
            continue;
        }
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            return Err(UpstreamError::new(UpstreamStage::Tcp, "connect-timeout"));
        };
        let attempt = tokio::select! {
            () = cancellation.cancelled() => {
                return Err(UpstreamError::new(UpstreamStage::Cancelled, "cancelled"));
            }
            result = timeout(remaining, TcpStream::connect(address)) => result,
        };
        match attempt {
            Ok(Ok(stream)) => {
                return Ok(BoundedUpstreamStream {
                    stream,
                    read_budget: budgets.read,
                    write_budget: budgets.write,
                })
            }
            Ok(Err(error)) => {
                last = Some(UpstreamError::with_detail(
                    UpstreamStage::Tcp,
                    "connect-failed",
                    error.to_string(),
                ))
            }
            Err(_) => last = Some(UpstreamError::new(UpstreamStage::Tcp, "connect-timeout")),
        }
    }
    Err(last.unwrap_or_else(|| UpstreamError::new(UpstreamStage::Policy, "destination-refused")))
}

pub fn native_tls_client_config() -> Result<Arc<ClientConfig>, UpstreamError> {
    let loaded = rustls_native_certs::load_native_certs();
    let mut roots = RootCertStore::empty();
    let mut rejected = 0_usize;
    for certificate in loaded.certs {
        if roots.add(certificate).is_err() {
            rejected += 1;
        }
    }
    if roots.is_empty() {
        return Err(UpstreamError::with_detail(
            UpstreamStage::Tls,
            "empty-native-root-store",
            format!(
                "no usable native roots; {} loader error(s), {rejected} rejected certificate(s)",
                loaded.errors.len()
            ),
        ));
    }
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| {
            UpstreamError::with_detail(UpstreamStage::Tls, "tls-config-failed", error.to_string())
        })?
        .with_root_certificates(roots)
        .with_no_client_auth();
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    Ok(Arc::new(config))
}

pub async fn connect_tls_upstream(
    authority: &DestinationAuthority,
    policy: &DestinationPolicy,
    budgets: UpstreamBudgets,
    config: Arc<ClientConfig>,
) -> Result<BoundedTlsUpstreamStream, UpstreamError> {
    connect_tls_upstream_cancellable(
        authority,
        policy,
        budgets,
        config,
        &UpstreamCancellation::default(),
    )
    .await
}

pub async fn connect_tls_upstream_cancellable(
    authority: &DestinationAuthority,
    policy: &DestinationPolicy,
    budgets: UpstreamBudgets,
    config: Arc<ClientConfig>,
    cancellation: &UpstreamCancellation,
) -> Result<BoundedTlsUpstreamStream, UpstreamError> {
    let stream = connect_upstream_cancellable(authority, policy, budgets, cancellation).await?;
    let server_name = match authority.host() {
        AuthorityHost::Dns(name) => ServerName::try_from(name.clone()),
        AuthorityHost::Ip(ip) => Ok(ServerName::IpAddress((*ip).into())),
    }
    .map_err(|error| {
        UpstreamError::with_detail(UpstreamStage::Tls, "invalid-server-name", error.to_string())
    })?;
    let connector = tokio_rustls::TlsConnector::from(config);
    let tls = tokio::select! {
        () = cancellation.cancelled() => {
            return Err(UpstreamError::new(UpstreamStage::Cancelled, "cancelled"));
        }
        result = timeout(budgets.connect, connector.connect(server_name, stream.stream)) => result,
    }
    .map_err(|_| UpstreamError::new(UpstreamStage::Tls, "tls-timeout"))?
    .map_err(|error| {
        UpstreamError::with_detail(
            UpstreamStage::Tls,
            "tls-verification-failed",
            error.to_string(),
        )
    })?;
    Ok(BoundedTlsUpstreamStream {
        stream: tls,
        read_budget: budgets.read,
        write_budget: budgets.write,
    })
}

pub fn tls_client_config_with_roots(
    roots: RootCertStore,
) -> Result<Arc<ClientConfig>, UpstreamError> {
    if roots.is_empty() {
        return Err(UpstreamError::new(UpstreamStage::Tls, "empty-root-store"));
    }
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| {
            UpstreamError::with_detail(UpstreamStage::Tls, "tls-config-failed", error.to_string())
        })?
        .with_root_certificates(roots)
        .with_no_client_auth();
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    Ok(Arc::new(config))
}
