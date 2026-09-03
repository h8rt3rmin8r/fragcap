// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fmt;
use std::future::Future;
use std::net::{IpAddr, SocketAddr, SocketAddrV6};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer, ServerName};
use rustls::{ClientConfig, RootCertStore};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio::task::JoinSet;
use tokio::time::{timeout, Instant};

const MAX_RESOLVED_CANDIDATES: usize = 32;
const CONNECTION_ATTEMPT_DELAY: Duration = Duration::from_millis(250);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AuthorityHost {
    Dns(String),
    Ip(IpAddr),
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DestinationAuthority {
    host: AuthorityHost,
    port: u16,
    scope_id: Option<u32>,
}

impl DestinationAuthority {
    pub fn parse(value: &str) -> Result<Self, UpstreamError> {
        Self::parse_with_scope_syntax(value, ScopeSyntax::Raw)
    }

    /// Parse an authority taken from a URI, where RFC zone delimiters are
    /// percent encoded. Keeping this separate from raw authority-form parsing
    /// avoids confusing a raw numeric scope such as `%251` with scope `1`.
    pub fn parse_uri(value: &str) -> Result<Self, UpstreamError> {
        Self::parse_with_scope_syntax(value, ScopeSyntax::UriEncoded)
    }

    fn parse_with_scope_syntax(
        value: &str,
        scope_syntax: ScopeSyntax,
    ) -> Result<Self, UpstreamError> {
        if value.contains('@') {
            return Err(UpstreamError::new(
                UpstreamStage::Authority,
                "invalid-authority",
            ));
        }
        if value.starts_with('[') {
            return Self::parse_bracketed_ipv6(value, scope_syntax);
        }
        if value.contains('%') {
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
        Ok(Self {
            host,
            port,
            scope_id: None,
        })
    }

    fn parse_bracketed_ipv6(value: &str, scope_syntax: ScopeSyntax) -> Result<Self, UpstreamError> {
        let close = value
            .find(']')
            .ok_or_else(|| UpstreamError::new(UpstreamStage::Authority, "invalid-authority"))?;
        let port = value
            .get(close + 1..)
            .and_then(|tail| tail.strip_prefix(':'))
            .and_then(|port| port.parse::<u16>().ok())
            .filter(|port| *port != 0)
            .ok_or_else(|| UpstreamError::new(UpstreamStage::Authority, "invalid-port"))?;
        let literal = &value[1..close];
        let scoped = match scope_syntax {
            ScopeSyntax::Raw => literal.split_once('%'),
            ScopeSyntax::UriEncoded => literal.split_once("%25"),
        };
        let (address, scope_id) = match scoped {
            Some((address, scope)) => {
                let ip = address
                    .parse::<std::net::Ipv6Addr>()
                    .map_err(|_| UpstreamError::new(UpstreamStage::Authority, "invalid-host"))?;
                let scope_id = scope
                    .parse::<u32>()
                    .ok()
                    .filter(|value| *value != 0)
                    .ok_or_else(|| {
                        UpstreamError::new(UpstreamStage::Authority, "invalid-scope-id")
                    })?;
                if !(ip.is_unicast_link_local() || ip.is_multicast()) {
                    return Err(UpstreamError::new(
                        UpstreamStage::Authority,
                        "scope-not-applicable",
                    ));
                }
                (ip, Some(scope_id))
            }
            None => (
                literal
                    .parse::<std::net::Ipv6Addr>()
                    .map_err(|_| UpstreamError::new(UpstreamStage::Authority, "invalid-host"))?,
                None,
            ),
        };
        Ok(Self {
            host: AuthorityHost::Ip(IpAddr::V6(address)),
            port,
            scope_id,
        })
    }

    pub fn host(&self) -> &AuthorityHost {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn scope_id(&self) -> Option<u32> {
        self.scope_id
    }

    fn literal_address(&self) -> Option<SocketAddr> {
        match self.host {
            AuthorityHost::Ip(IpAddr::V4(ip)) => Some(SocketAddr::new(ip.into(), self.port)),
            AuthorityHost::Ip(IpAddr::V6(ip)) => Some(SocketAddr::V6(SocketAddrV6::new(
                ip,
                self.port,
                0,
                self.scope_id.unwrap_or(0),
            ))),
            AuthorityHost::Dns(_) => None,
        }
    }

    pub fn lookup_host(&self) -> String {
        match &self.host {
            AuthorityHost::Dns(name) => format!("{name}:{}", self.port),
            AuthorityHost::Ip(IpAddr::V4(ip)) => format!("{ip}:{}", self.port),
            AuthorityHost::Ip(IpAddr::V6(ip)) => format!("[{ip}]:{}", self.port),
        }
    }
}

#[derive(Clone, Copy)]
enum ScopeSyntax {
    Raw,
    UriEncoded,
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
            listener: canonical_address(listener),
            exact_grants: BTreeSet::new(),
        }
    }

    pub fn grant_for_test(&mut self, address: SocketAddr) {
        let address = canonical_address(address);
        if address != self.listener {
            self.exact_grants.insert(address);
        }
    }

    pub fn evaluate(&self, address: SocketAddr) -> DestinationDecision {
        let normalized = canonical_address(address);
        let (allowed, reason) = if normalized == self.listener {
            (false, "proxy-listener")
        } else if self.exact_grants.contains(&normalized) {
            (true, "controlled-origin-grant")
        } else if public_address(normalized.ip()) {
            (true, "public-destination")
        } else {
            (false, "local-destination-refused")
        };
        DestinationDecision {
            address,
            allowed,
            reason,
        }
    }
}

pub(crate) fn canonical_address(address: SocketAddr) -> SocketAddr {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TlsRefusalClass {
    ClientCertificateRequired,
    ClientCertificateRejected,
    CertificateValidation,
    ProtocolMismatch,
    ClientTrustRejection,
    CertificatePinned,
    Unknown,
}

impl TlsRefusalClass {
    pub fn code(self) -> &'static str {
        match self {
            Self::ClientCertificateRequired => "client-certificate-required",
            Self::ClientCertificateRejected => "client-certificate-rejected",
            Self::CertificateValidation => "certificate-validation",
            Self::ProtocolMismatch => "protocol-mismatch",
            Self::ClientTrustRejection => "client-trust-rejection",
            Self::CertificatePinned => "certificate-pinned",
            Self::Unknown => "tls-refusal-unknown",
        }
    }
}

pub fn classify_tls_io_error(
    error: &std::io::Error,
    client_facing: bool,
    identity_configured: bool,
) -> TlsRefusalClass {
    use rustls::{AlertDescription as Alert, Error};
    let Some(error) = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<Error>())
    else {
        return TlsRefusalClass::Unknown;
    };
    match error {
        Error::AlertReceived(Alert::BadCertificateHashValue) if client_facing => {
            TlsRefusalClass::CertificatePinned
        }
        Error::AlertReceived(Alert::CertificateRequired | Alert::NoCertificate)
            if identity_configured =>
        {
            TlsRefusalClass::ClientCertificateRejected
        }
        Error::AlertReceived(Alert::CertificateRequired | Alert::NoCertificate) => {
            TlsRefusalClass::ClientCertificateRequired
        }
        Error::AlertReceived(
            Alert::BadCertificate
            | Alert::UnsupportedCertificate
            | Alert::CertificateRevoked
            | Alert::CertificateExpired
            | Alert::CertificateUnknown
            | Alert::UnknownCA
            | Alert::AccessDenied,
        ) if client_facing => TlsRefusalClass::ClientTrustRejection,
        Error::AlertReceived(
            Alert::BadCertificate
            | Alert::UnsupportedCertificate
            | Alert::CertificateRevoked
            | Alert::CertificateExpired
            | Alert::CertificateUnknown
            | Alert::UnknownCA
            | Alert::AccessDenied,
        ) if identity_configured => TlsRefusalClass::ClientCertificateRejected,
        Error::InvalidCertificate(_) => TlsRefusalClass::CertificateValidation,
        Error::PeerIncompatible(_)
        | Error::NoApplicationProtocol
        | Error::AlertReceived(
            Alert::ProtocolVersion | Alert::NoApplicationProtocol | Alert::InsufficientSecurity,
        ) => TlsRefusalClass::ProtocolMismatch,
        _ => TlsRefusalClass::Unknown,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpstreamError {
    pub stage: UpstreamStage,
    pub code: &'static str,
    pub detail: String,
}

/// Explicit operator-owned client identity for one upstream TLS configuration.
pub struct ClientIdentity {
    chain: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
}

impl ClientIdentity {
    pub fn from_bytes(
        certificate_bytes: &[u8],
        private_key_bytes: &[u8],
    ) -> Result<Self, UpstreamError> {
        let chain = if certificate_bytes.starts_with(b"-----BEGIN") {
            CertificateDer::pem_slice_iter(certificate_bytes)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| {
                    UpstreamError::with_detail(
                        UpstreamStage::Tls,
                        "client-certificate-invalid",
                        error.to_string(),
                    )
                })?
        } else {
            vec![CertificateDer::from(certificate_bytes.to_vec())]
        };
        if chain.is_empty() {
            return Err(UpstreamError::new(
                UpstreamStage::Tls,
                "client-certificate-empty",
            ));
        }
        let key = if private_key_bytes.starts_with(b"-----BEGIN") {
            PrivateKeyDer::from_pem_slice(private_key_bytes).map_err(|error| {
                UpstreamError::with_detail(
                    UpstreamStage::Tls,
                    "client-private-key-invalid",
                    error.to_string(),
                )
            })?
        } else {
            PrivateKeyDer::try_from(private_key_bytes.to_vec()).map_err(|error| {
                UpstreamError::with_detail(UpstreamStage::Tls, "client-private-key-invalid", error)
            })?
        };
        Ok(Self { chain, key })
    }
}

impl fmt::Debug for ClientIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientIdentity")
            .field("certificates", &self.chain.len())
            .field("private_key", &"[REDACTED]")
            .finish()
    }
}

impl Drop for ClientIdentity {
    fn drop(&mut self) {
        zeroize::Zeroize::zeroize(&mut self.key);
    }
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
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.stream.get_ref().0.local_addr()
    }

    pub fn peer_addr(&self) -> std::io::Result<SocketAddr> {
        self.stream.get_ref().0.peer_addr()
    }

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
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.stream.local_addr()
    }

    pub fn peer_addr(&self) -> std::io::Result<SocketAddr> {
        self.stream.peer_addr()
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

pub async fn resolve_allowed_udp(
    authority: &DestinationAuthority,
    policy: &DestinationPolicy,
    budget: Duration,
) -> Result<Vec<SocketAddr>, UpstreamError> {
    resolve_allowed_candidates(authority, policy, budget).await
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
    let allowed = tokio::select! {
        () = cancellation.cancelled() => {
            return Err(UpstreamError::new(UpstreamStage::Cancelled, "cancelled"));
        }
        result = resolve_allowed_candidates(authority, policy, budgets.dns) => result,
    }?;
    let addresses = interleave_families(allowed);
    race_candidates(
        addresses,
        budgets.connect,
        CONNECTION_ATTEMPT_DELAY,
        cancellation,
        TcpStream::connect,
    )
    .await
    .map(|stream| BoundedUpstreamStream {
        stream,
        read_budget: budgets.read,
        write_budget: budgets.write,
    })
}

async fn race_candidates<T, Connect, Attempt>(
    addresses: Vec<SocketAddr>,
    budget: Duration,
    attempt_delay: Duration,
    cancellation: &UpstreamCancellation,
    connect: Connect,
) -> Result<T, UpstreamError>
where
    T: Send + 'static,
    Connect: Fn(SocketAddr) -> Attempt + Clone + Send + 'static,
    Attempt: Future<Output = std::io::Result<T>> + Send + 'static,
{
    let mut attempts = JoinSet::new();
    let mut failures = Vec::new();
    let mut last = None;
    for (index, address) in addresses.into_iter().enumerate() {
        let connect = connect.clone();
        attempts.spawn(async move {
            tokio::time::sleep(attempt_delay.saturating_mul(index as u32)).await;
            (index, connect(address).await)
        });
    }
    let deadline = Instant::now() + budget;
    loop {
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            attempts.abort_all();
            while attempts.join_next().await.is_some() {}
            return Err(UpstreamError::new(UpstreamStage::Tcp, "connect-timeout"));
        };
        let joined = tokio::select! {
            () = cancellation.cancelled() => {
                attempts.abort_all();
                while attempts.join_next().await.is_some() {}
                return Err(UpstreamError::new(UpstreamStage::Cancelled, "cancelled"));
            }
            result = timeout(remaining, attempts.join_next()) => result,
        };
        match joined {
            Ok(Some(Ok((_index, Ok(stream))))) => {
                attempts.abort_all();
                while attempts.join_next().await.is_some() {}
                return Ok(stream);
            }
            Ok(Some(Ok((index, Err(error))))) => {
                failures.push((index, connect_error(error)));
            }
            Ok(Some(Err(error))) => {
                last = Some(UpstreamError::with_detail(
                    UpstreamStage::Tcp,
                    "connect-task-failed",
                    error.to_string(),
                ));
            }
            Ok(None) => {
                failures.sort_by_key(|(index, _)| *index);
                return Err(failures
                    .into_iter()
                    .next()
                    .map(|(_, error)| error)
                    .or(last)
                    .unwrap_or_else(|| UpstreamError::new(UpstreamStage::Tcp, "connect-failed")));
            }
            Err(_) => {
                attempts.abort_all();
                while attempts.join_next().await.is_some() {}
                return Err(UpstreamError::new(UpstreamStage::Tcp, "connect-timeout"));
            }
        }
    }
}

async fn resolve_allowed_candidates(
    authority: &DestinationAuthority,
    policy: &DestinationPolicy,
    budget: Duration,
) -> Result<Vec<SocketAddr>, UpstreamError> {
    if let Some(address) = authority.literal_address() {
        return retain_allowed_candidates([address], policy);
    }
    let resolved = timeout(budget, tokio::net::lookup_host(authority.lookup_host()))
        .await
        .map_err(|_| UpstreamError::new(UpstreamStage::Dns, "dns-timeout"))?
        .map_err(|error| {
            UpstreamError::with_detail(UpstreamStage::Dns, "dns-failed", error.to_string())
        })?;
    retain_allowed_candidates(resolved, policy)
}

fn retain_allowed_candidates(
    resolved: impl IntoIterator<Item = SocketAddr>,
    policy: &DestinationPolicy,
) -> Result<Vec<SocketAddr>, UpstreamError> {
    let mut allowed = Vec::new();
    let mut saw_address = false;
    for address in resolved {
        saw_address = true;
        let decision = policy.evaluate(address);
        if !decision.allowed
            || allowed
                .iter()
                .any(|existing| canonical_address(*existing) == canonical_address(decision.address))
        {
            continue;
        }
        if allowed.len() < MAX_RESOLVED_CANDIDATES {
            allowed.push(decision.address);
            continue;
        }
        let family_present = allowed
            .iter()
            .any(|existing| existing.is_ipv6() == decision.address.is_ipv6());
        if !family_present {
            allowed.pop();
            allowed.push(decision.address);
        }
    }
    if !saw_address {
        Err(UpstreamError::new(UpstreamStage::Dns, "dns-empty"))
    } else if allowed.is_empty() {
        Err(UpstreamError::new(
            UpstreamStage::Policy,
            "destination-refused",
        ))
    } else {
        Ok(allowed)
    }
}

fn interleave_families(addresses: Vec<SocketAddr>) -> Vec<SocketAddr> {
    let prefer_v6 = addresses.first().is_some_and(SocketAddr::is_ipv6);
    let (v6, v4): (Vec<_>, Vec<_>) = addresses.into_iter().partition(SocketAddr::is_ipv6);
    let mut v6 = v6.into_iter();
    let mut v4 = v4.into_iter();
    let mut ordered = Vec::new();
    loop {
        let pair = if prefer_v6 {
            (v6.next(), v4.next())
        } else {
            (v4.next(), v6.next())
        };
        if pair.0.is_none() && pair.1.is_none() {
            break;
        }
        ordered.extend(pair.0);
        ordered.extend(pair.1);
    }
    ordered
}

fn connect_error(error: std::io::Error) -> UpstreamError {
    let code = match error.kind() {
        std::io::ErrorKind::ConnectionRefused => "connection-refused",
        std::io::ErrorKind::NetworkUnreachable => "network-unreachable",
        std::io::ErrorKind::HostUnreachable => "host-unreachable",
        _ => "connect-failed",
    };
    UpstreamError::with_detail(UpstreamStage::Tcp, code, error.to_string())
}

#[cfg(test)]
mod address_tests {
    use std::future::pending;
    use std::net::SocketAddr;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use super::{
        canonical_address, interleave_families, race_candidates, retain_allowed_candidates,
        DestinationPolicy, UpstreamCancellation, UpstreamStage, MAX_RESOLVED_CANDIDATES,
    };

    #[test]
    fn candidate_order_interleaves_families_from_the_resolver_preference() {
        let addresses = vec![
            "[2001:db8::1]:443".parse().unwrap(),
            "[2001:db8::2]:443".parse().unwrap(),
            "192.0.2.1:443".parse().unwrap(),
            "192.0.2.2:443".parse().unwrap(),
        ];
        assert_eq!(
            interleave_families(addresses),
            vec![
                "[2001:db8::1]:443".parse().unwrap(),
                "192.0.2.1:443".parse().unwrap(),
                "[2001:db8::2]:443".parse().unwrap(),
                "192.0.2.2:443".parse().unwrap(),
            ]
        );
    }

    #[test]
    fn mapped_and_native_ipv4_have_one_canonical_peer_identity() {
        assert_eq!(
            canonical_address("[::ffff:192.0.2.1]:443".parse().unwrap()),
            "192.0.2.1:443".parse().unwrap()
        );
    }

    #[test]
    fn candidate_cap_retains_a_late_second_family() {
        let mut resolved = (1..=MAX_RESOLVED_CANDIDATES)
            .map(|port| format!("[2001:4860:4860::8888]:{port}").parse().unwrap())
            .collect::<Vec<_>>();
        let ipv4 = "8.8.8.8:53".parse().unwrap();
        resolved.push(ipv4);
        let retained = retain_allowed_candidates(
            resolved,
            &DestinationPolicy::new("127.0.0.1:1".parse().unwrap()),
        )
        .unwrap();
        assert_eq!(retained.len(), MAX_RESOLVED_CANDIDATES);
        assert!(retained.contains(&ipv4));
        assert!(retained.iter().any(SocketAddr::is_ipv6));
    }

    #[test]
    fn every_mixed_dns_answer_is_rechecked_without_local_fallback() {
        let listener = "127.0.0.1:8080".parse().unwrap();
        let policy = DestinationPolicy::new(listener);
        let allowed = retain_allowed_candidates(
            vec![
                listener,
                "10.0.0.1:443".parse().unwrap(),
                "8.8.8.8:443".parse().unwrap(),
                "[::1]:443".parse().unwrap(),
                "[2606:4700:4700::1111]:443".parse().unwrap(),
            ],
            &policy,
        )
        .unwrap();
        assert_eq!(
            allowed,
            vec![
                "8.8.8.8:443".parse::<SocketAddr>().unwrap(),
                "[2606:4700:4700::1111]:443".parse::<SocketAddr>().unwrap(),
            ]
        );
        assert_eq!(policy.evaluate(listener).reason, "proxy-listener");
        assert_eq!(
            policy.evaluate("127.0.0.2:443".parse().unwrap()).reason,
            "local-destination-refused"
        );
    }

    #[tokio::test]
    async fn one_hundred_staggered_races_return_one_winner_and_drain_losers() {
        let addresses = vec![
            "[2001:db8::1]:1".parse().unwrap(),
            "192.0.2.1:2".parse().unwrap(),
            "[2001:db8::2]:3".parse().unwrap(),
        ];
        for _ in 0..100 {
            let starts = Arc::new(Mutex::new(Vec::new()));
            let observed = Arc::clone(&starts);
            let winner = race_candidates(
                addresses.clone(),
                Duration::from_millis(500),
                Duration::from_millis(50),
                &UpstreamCancellation::default(),
                move |address| {
                    let observed = Arc::clone(&observed);
                    async move {
                        observed.lock().unwrap().push(address.port());
                        if address.port() == 2 {
                            Ok(address)
                        } else {
                            pending().await
                        }
                    }
                },
            )
            .await
            .unwrap();
            assert_eq!(winner.port(), 2);
            assert_eq!(*starts.lock().unwrap(), vec![1, 2]);
        }
    }

    #[tokio::test]
    async fn a_race_timeout_drains_every_pending_attempt() {
        let error = race_candidates(
            vec!["192.0.2.1:1".parse().unwrap()],
            Duration::from_millis(5),
            Duration::ZERO,
            &UpstreamCancellation::default(),
            |_address| pending::<std::io::Result<()>>(),
        )
        .await
        .unwrap_err();
        assert_eq!(error.stage, UpstreamStage::Tcp);
        assert_eq!(error.code, "connect-timeout");
    }
}

pub fn native_tls_client_config() -> Result<Arc<ClientConfig>, UpstreamError> {
    native_tls_client_config_with_identity(None)
}

pub fn native_tls_client_config_with_identity(
    identity: Option<&ClientIdentity>,
) -> Result<Arc<ClientConfig>, UpstreamError> {
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
    let builder = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| {
            UpstreamError::with_detail(UpstreamStage::Tls, "tls-config-failed", error.to_string())
        })?
        .with_root_certificates(roots);
    let mut config = match identity {
        Some(identity) => builder
            .with_client_auth_cert(identity.chain.clone(), identity.key.clone_key())
            .map_err(|error| {
                UpstreamError::with_detail(
                    UpstreamStage::Tls,
                    "client-identity-invalid",
                    error.to_string(),
                )
            })?,
        None => builder.with_no_client_auth(),
    };
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
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
    connect_tls_over_upstream_cancellable(authority, stream, budgets, config, cancellation).await
}

pub(crate) async fn connect_tls_over_upstream(
    authority: &DestinationAuthority,
    stream: BoundedUpstreamStream,
    budgets: UpstreamBudgets,
    config: Arc<ClientConfig>,
) -> Result<BoundedTlsUpstreamStream, UpstreamError> {
    connect_tls_over_upstream_cancellable(
        authority,
        stream,
        budgets,
        config,
        &UpstreamCancellation::default(),
    )
    .await
}

async fn connect_tls_over_upstream_cancellable(
    authority: &DestinationAuthority,
    stream: BoundedUpstreamStream,
    budgets: UpstreamBudgets,
    config: Arc<ClientConfig>,
    cancellation: &UpstreamCancellation,
) -> Result<BoundedTlsUpstreamStream, UpstreamError> {
    let server_name = match authority.host() {
        AuthorityHost::Dns(name) => ServerName::try_from(name.clone()),
        AuthorityHost::Ip(ip) => Ok(ServerName::IpAddress((*ip).into())),
    }
    .map_err(|error| {
        UpstreamError::with_detail(UpstreamStage::Tls, "invalid-server-name", error.to_string())
    })?;
    let connector = tokio_rustls::TlsConnector::from(Arc::clone(&config));
    let tls = tokio::select! {
        () = cancellation.cancelled() => {
            return Err(UpstreamError::new(UpstreamStage::Cancelled, "cancelled"));
        }
        result = timeout(budgets.connect, connector.connect(server_name, stream.stream)) => result,
    }
    .map_err(|_| UpstreamError::new(UpstreamStage::Tls, "tls-timeout"))?
    .map_err(|error| {
        let class =
            classify_tls_io_error(&error, false, config.client_auth_cert_resolver.has_certs());
        UpstreamError::with_detail(UpstreamStage::Tls, class.code(), error.to_string())
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
    tls_client_config_with_roots_and_identity(roots, None)
}

pub fn tls_client_config_with_roots_and_identity(
    roots: RootCertStore,
    identity: Option<&ClientIdentity>,
) -> Result<Arc<ClientConfig>, UpstreamError> {
    if roots.is_empty() {
        return Err(UpstreamError::new(UpstreamStage::Tls, "empty-root-store"));
    }
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| {
            UpstreamError::with_detail(UpstreamStage::Tls, "tls-config-failed", error.to_string())
        })?
        .with_root_certificates(roots);
    let mut config = match identity {
        Some(identity) => builder
            .with_client_auth_cert(identity.chain.clone(), identity.key.clone_key())
            .map_err(|error| {
                UpstreamError::with_detail(
                    UpstreamStage::Tls,
                    "client-identity-invalid",
                    error.to_string(),
                )
            })?,
        None => builder.with_no_client_auth(),
    };
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(Arc::new(config))
}

#[cfg(test)]
mod tls_refusal_tests {
    use super::*;

    fn io(error: rustls::Error) -> std::io::Error {
        std::io::Error::other(error)
    }

    #[test]
    fn refusal_categories_use_structured_rustls_evidence() {
        use rustls::AlertDescription as Alert;
        assert_eq!(
            classify_tls_io_error(
                &io(rustls::Error::AlertReceived(Alert::CertificateRequired)),
                false,
                false,
            ),
            TlsRefusalClass::ClientCertificateRequired
        );
        assert_eq!(
            classify_tls_io_error(
                &io(rustls::Error::AlertReceived(Alert::CertificateRequired)),
                false,
                true,
            ),
            TlsRefusalClass::ClientCertificateRejected
        );
        assert_eq!(
            classify_tls_io_error(
                &io(rustls::Error::AlertReceived(Alert::BadCertificate)),
                false,
                true,
            ),
            TlsRefusalClass::ClientCertificateRejected
        );
        for alert in [Alert::CertificateExpired, Alert::CertificateRevoked] {
            assert_eq!(
                classify_tls_io_error(&io(rustls::Error::AlertReceived(alert)), false, true,),
                TlsRefusalClass::ClientCertificateRejected
            );
        }
        assert_eq!(
            classify_tls_io_error(
                &io(rustls::Error::AlertReceived(Alert::BadCertificateHashValue)),
                true,
                false,
            ),
            TlsRefusalClass::CertificatePinned
        );
        assert_eq!(
            classify_tls_io_error(
                &io(rustls::Error::AlertReceived(Alert::UnknownCA)),
                true,
                false,
            ),
            TlsRefusalClass::ClientTrustRejection
        );
        assert_eq!(
            classify_tls_io_error(
                &std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "closed"),
                true,
                false,
            ),
            TlsRefusalClass::Unknown
        );
        assert_eq!(
            classify_tls_io_error(&io(rustls::Error::NoApplicationProtocol), false, false,),
            TlsRefusalClass::ProtocolMismatch
        );
    }
}
