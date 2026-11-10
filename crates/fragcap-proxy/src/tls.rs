// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use std::time::SystemTime;

use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use rustls::{ClientConfig, ServerConfig};
use tokio::net::TcpStream;
use tokio::time::{timeout, timeout_at, Instant};
use tokio_rustls::server::TlsStream;

use crate::{
    AuthorityHost, BoundedTlsUpstreamStream, CertificateIdentity, DestinationAuthority,
    DestinationPolicy, LeafCache, ProtocolError, ProtocolLimits, SessionCertificateAuthority,
    SessionKeyLog, UpstreamStage,
};

#[derive(Debug)]
struct FixedCertificate(Arc<CertifiedKey>);

impl ResolvesServerCert for FixedCertificate {
    fn resolve(&self, _client_hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        Some(Arc::clone(&self.0))
    }
}

pub(crate) fn client_server_config_with_alpn(
    leaf: Arc<CertifiedKey>,
    alpn_protocols: Vec<Vec<u8>>,
    key_log: Option<Arc<SessionKeyLog>>,
) -> Result<Arc<ServerConfig>, ProtocolError> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut config = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| ProtocolError::new("client-tls-config-failed", error.to_string()))?
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(FixedCertificate(leaf)));
    config.alpn_protocols = alpn_protocols;
    if let Some(key_log) = key_log {
        config.key_log = key_log;
    }
    Ok(Arc::new(config))
}

pub(crate) fn client_server_config(
    authority: &DestinationAuthority,
    certificate_authority: &SessionCertificateAuthority,
    leaf_cache: &mut LeafCache,
    key_log: Option<Arc<SessionKeyLog>>,
) -> Result<Arc<ServerConfig>, ProtocolError> {
    let identity = match authority.host() {
        AuthorityHost::Dns(name) => CertificateIdentity::Dns(name.clone()),
        AuthorityHost::Ip(ip) => CertificateIdentity::Ip(*ip),
    };
    let leaf = leaf_cache
        .certificate_for(certificate_authority, identity, SystemTime::now())
        .map_err(|error| ProtocolError::new(error.code, error.detail))?;
    client_server_config_with_alpn(
        Arc::clone(&leaf.certified_key),
        vec![b"h2".to_vec(), b"http/1.1".to_vec()],
        key_log,
    )
}

pub(crate) fn upstream_client_config_for_alpn(
    base: &Arc<ClientConfig>,
    alpn: Vec<u8>,
) -> Arc<ClientConfig> {
    let mut config = (**base).clone();
    config.alpn_protocols = vec![alpn];
    Arc::new(config)
}

pub(crate) fn upstream_client_config_without_alpn(base: &Arc<ClientConfig>) -> Arc<ClientConfig> {
    let mut config = (**base).clone();
    config.alpn_protocols.clear();
    Arc::new(config)
}

pub(crate) async fn accept_client_tls(
    stream: TcpStream,
    authority: &DestinationAuthority,
    config: Arc<ServerConfig>,
    limits: &ProtocolLimits,
) -> Result<TlsStream<TcpStream>, ProtocolError> {
    let acceptor = tokio_rustls::TlsAcceptor::from(config);
    let tls = timeout(limits.tls_handshake_timeout, acceptor.accept(stream))
        .await
        .map_err(|_| ProtocolError::timeout("client-tls-handshake-timeout"))?
        .map_err(client_tls_error)?;
    validate_client_tls(tls, authority)
}

pub(crate) async fn accept_direct_client_tls(
    stream: TcpStream,
    original_authority: &DestinationAuthority,
    certificate_authority: &SessionCertificateAuthority,
    leaf_cache: &tokio::sync::Mutex<LeafCache>,
    key_log: Option<Arc<SessionKeyLog>>,
    limits: &ProtocolLimits,
) -> Result<(TlsStream<TcpStream>, DestinationAuthority), ProtocolError> {
    let deadline = Instant::now() + limits.tls_handshake_timeout;
    let start = timeout_at(
        deadline,
        tokio_rustls::LazyConfigAcceptor::new(rustls::server::Acceptor::default(), stream),
    )
    .await
    .map_err(|_| ProtocolError::timeout("client-tls-handshake-timeout"))?
    .map_err(client_tls_error)?;
    let authority = match start.client_hello().server_name() {
        Some(server_name) => {
            if let AuthorityHost::Dns(expected) = original_authority.host() {
                if !expected.eq_ignore_ascii_case(server_name) {
                    return Err(ProtocolError::new(
                        "client-tls-sni-mismatch",
                        "client SNI does not match the direct route authority",
                    ));
                }
            }
            DestinationAuthority::parse(&format!("{server_name}:{}", original_authority.port()))
                .map_err(|error| ProtocolError::new("direct-tls-sni-invalid", error.to_string()))?
        }
        None => original_authority.clone(),
    };
    let server_config = {
        let mut cache = leaf_cache.lock().await;
        client_server_config(&authority, certificate_authority, &mut cache, key_log)?
    };
    let tls = timeout_at(deadline, start.into_stream(server_config))
        .await
        .map_err(|_| ProtocolError::timeout("client-tls-handshake-timeout"))?
        .map_err(client_tls_error)?;
    Ok((validate_client_tls(tls, &authority)?, authority))
}

fn client_tls_error(error: std::io::Error) -> ProtocolError {
    let class = crate::classify_tls_io_error(&error, true, false);
    let (tls_alert, transport_category, os_error_code) = crate::upstream::diagnostic_io(&error);
    let mut protocol = ProtocolError::new(class.code(), error.to_string());
    protocol.tls_alert = tls_alert;
    protocol.transport_category = Some(transport_category);
    protocol.os_error_code = os_error_code;
    protocol
}

fn validate_client_tls(
    tls: TlsStream<TcpStream>,
    authority: &DestinationAuthority,
) -> Result<TlsStream<TcpStream>, ProtocolError> {
    if let (AuthorityHost::Dns(expected), Some(observed)) =
        (authority.host(), tls.get_ref().1.server_name())
    {
        if !expected.eq_ignore_ascii_case(observed) {
            return Err(ProtocolError::new(
                "client-tls-sni-mismatch",
                "client SNI does not match the CONNECT authority",
            ));
        }
    }
    if !matches!(
        tls.get_ref().1.alpn_protocol(),
        Some(b"h2") | Some(b"http/1.1") | None
    ) {
        return Err(ProtocolError::new(
            "client-tls-alpn-unsupported",
            "client negotiated an unsupported application protocol",
        ));
    }
    Ok(tls)
}

pub(crate) async fn connect_verified_tls(
    authority: DestinationAuthority,
    policy: DestinationPolicy,
    limits: ProtocolLimits,
    config: Arc<ClientConfig>,
) -> Result<BoundedTlsUpstreamStream, ProtocolError> {
    crate::connect_tls_upstream(&authority, &policy, limits.upstream, config)
        .await
        .map_err(|error| {
            let mut protocol = ProtocolError::new(error.code, error.detail);
            protocol.policy_refused = matches!(error.stage, UpstreamStage::Policy);
            protocol
        })
}
