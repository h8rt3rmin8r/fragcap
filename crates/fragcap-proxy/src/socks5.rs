// SPDX-License-Identifier: Apache-2.0

//! Bounded SOCKS5 TCP negotiation and byte-transparent relay.

use std::collections::BTreeSet;
use std::future::pending;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::watch;
use tokio::time::{timeout, Instant};

use crate::http1::{HttpRun, ProtocolError};
use crate::{
    connect_upstream, ApplicationEvent, ApplicationEventKind, DestinationAuthority,
    DestinationPolicy, ProtocolAccounting, ProtocolLimits, ProxyObservation, SessionCapability,
    SocksClassification, SocksConnectEvent, SocksNegotiationEvent, SocksTransferEvent,
    SocksUdpEvent, UpstreamError, UpstreamStage,
};

const SOCKS_VERSION: u8 = 5;
const AUTH_VERSION: u8 = 1;
const USERNAME_PASSWORD: u8 = 2;
const NO_ACCEPTABLE_METHODS: u8 = 0xff;
const CONNECT: u8 = 1;
const UDP_ASSOCIATE: u8 = 3;

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

#[derive(Debug)]
struct UdpAssociateRequest {
    client_endpoint: Option<SocketAddr>,
    address_type: SocksAddressType,
}

#[derive(Debug)]
enum SocksRequest {
    Connect(ConnectRequest),
    UdpAssociate(UdpAssociateRequest),
}

pub(crate) struct SocksConnectionContext<'a> {
    pub limits: &'a ProtocolLimits,
    pub shutdown: watch::Receiver<bool>,
    pub session_id: &'a str,
    pub connection_id: u64,
    pub peer: SocketAddr,
    pub local: SocketAddr,
    pub sink: &'a crate::application::SharedEventSink,
    pub body_resources: crate::body::SessionBodyResources,
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
        body_resources,
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
        result = read_request(&mut client, limits.header_timeout) => result,
    };
    let request = match request {
        Ok(request) => request,
        Err(failure) => {
            accounting.parse_refused = 1;
            if failure.request_command == Some(UDP_ASSOCIATE) {
                accounting.socks_udp_associate_requested = 1;
                accounting.socks_udp_associate_refused = 1;
            } else {
                accounting.socks_connect_refused = 1;
            }
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
    let request = match request {
        SocksRequest::Connect(request) => request,
        SocksRequest::UdpAssociate(request) => {
            accounting.socks_udp_associate_requested = 1;
            return serve_udp_association(
                client,
                request,
                policy,
                UdpAssociationContext {
                    limits,
                    shutdown,
                    session_id,
                    connection_id,
                    peer,
                    local,
                    sink,
                },
                accounting,
            )
            .await;
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
    let provenance = match classification {
        SocksClassification::Tls => crate::GenericStreamProvenance::TlsEncrypted,
        SocksClassification::Http | SocksClassification::OpaqueTcp => {
            crate::GenericStreamProvenance::TcpPlaintext
        }
    };
    match classification {
        SocksClassification::Tls => accounting.generic_streams_tls_opaque = 1,
        SocksClassification::Http | SocksClassification::OpaqueTcp => {
            accounting.generic_streams_plain = 1;
        }
    }
    let forwarding = crate::relay_generic(
        &mut client,
        &mut upstream,
        crate::GenericRelayContext {
            limits,
            shutdown: &mut shutdown,
            session_id,
            connection_id,
            sink,
            body_resources,
            buffer_bytes,
            provenance,
        },
    )
    .await;
    let report = forwarding.report;
    let client_bytes = report.client_to_upstream_bytes;
    let upstream_bytes = report.upstream_to_client_bytes;
    accounting.socks_client_bytes = client_bytes;
    accounting.socks_upstream_bytes = upstream_bytes;
    accounting.generic_stream_bytes_observed = report.observed_bytes;
    accounting.generic_stream_bytes_retained = report.retained_bytes;
    accounting.generic_stream_bytes_omitted = report.omitted_bytes;
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
    match forwarding.error {
        None => HttpRun {
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
        Some(error) => {
            let failure = if error.kind() == io::ErrorKind::TimedOut {
                SocksFailure::timeout("socks-forward-timeout", SocksReplyCode::TtlExpired)
            } else if error.kind() == io::ErrorKind::Interrupted {
                SocksFailure::cancelled()
            } else {
                SocksFailure::new("socks-forward-failed", error.to_string())
            };
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

async fn read_request(
    stream: &mut TcpStream,
    budget: Duration,
) -> Result<SocksRequest, SocksFailure> {
    let deadline = Instant::now() + budget;
    let mut prefix = [0_u8; 2];
    read_exact_until(stream, &mut prefix, deadline, "socks-request-prefix").await?;
    let command = prefix[1];
    let mut suffix = [0_u8; 2];
    read_exact_until(stream, &mut suffix, deadline, "socks-request-head")
        .await
        .map_err(|failure| failure.for_request_command(command))?;
    let head = [prefix[0], prefix[1], suffix[0], suffix[1]];
    let result = async {
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
        let port = u16::from_be_bytes(port);
        if head[1] == UDP_ASSOCIATE {
            let client_endpoint = match address_type {
                SocksAddressType::Ipv4 | SocksAddressType::Ipv6 => {
                    let ip = host
                        .trim_matches(['[', ']'])
                        .parse::<IpAddr>()
                        .map_err(|_| {
                            SocksFailure::reply(
                                "socks-udp-client-address-invalid",
                                SocksReplyCode::AddressTypeNotSupported,
                            )
                        })?;
                    if ip.is_unspecified() {
                        (port != 0).then(|| SocketAddr::new(ip, port))
                    } else {
                        Some(SocketAddr::new(ip, port))
                    }
                }
                SocksAddressType::Domain => {
                    return Err(SocksFailure::reply(
                        "socks-udp-client-domain-unsupported",
                        SocksReplyCode::AddressTypeNotSupported,
                    ));
                }
            };
            return Ok(SocksRequest::UdpAssociate(UdpAssociateRequest {
                client_endpoint,
                address_type,
            }));
        }
        if head[1] != CONNECT {
            return Err(SocksFailure::reply(
                "socks-command-unsupported",
                SocksReplyCode::CommandNotSupported,
            ));
        }
        let authority =
            DestinationAuthority::parse(&format!("{host}:{port}")).map_err(|error| {
                SocksFailure::reply(error.code, SocksReplyCode::AddressTypeNotSupported)
            })?;
        Ok(SocksRequest::Connect(ConnectRequest {
            authority,
            address_type,
        }))
    }
    .await;
    result.map_err(|failure| failure.for_request_command(command))
}

struct UdpAssociationContext<'a> {
    limits: &'a ProtocolLimits,
    shutdown: watch::Receiver<bool>,
    session_id: &'a str,
    connection_id: u64,
    peer: SocketAddr,
    local: SocketAddr,
    sink: &'a crate::application::SharedEventSink,
}

async fn serve_udp_association(
    mut control: TcpStream,
    request: UdpAssociateRequest,
    policy: &DestinationPolicy,
    context: UdpAssociationContext<'_>,
    mut accounting: ProtocolAccounting,
) -> HttpRun {
    let UdpAssociationContext {
        limits,
        mut shutdown,
        session_id,
        connection_id,
        peer,
        local,
        sink,
    } = context;
    let claimed = request.client_endpoint.map(|address| {
        if address.ip().is_unspecified() {
            SocketAddr::new(peer.ip(), address.port())
        } else {
            normalize_socket(address)
        }
    });
    if claimed.is_some_and(|address| normalize_ip(address.ip()) != normalize_ip(peer.ip())) {
        accounting.socks_udp_associate_refused = 1;
        let failure = SocksFailure::reply(
            "socks-udp-client-address-refused",
            SocksReplyCode::ConnectionRefused,
        );
        let _ = write_reply(&mut control, failure.reply, None).await;
        emit_udp(
            sink,
            session_id,
            connection_id,
            "association",
            "client-address-refused",
            Some(request.address_type),
            claimed,
            0,
            0,
        );
        return failed(accounting, failure);
    }

    let relay_bind = SocketAddr::new(local.ip(), 0);
    let relay = match UdpSocket::bind(relay_bind).await {
        Ok(socket) => socket,
        Err(error) => {
            accounting.socks_udp_associate_refused = 1;
            let failure = SocksFailure::new("socks-udp-bind-failed", error.to_string());
            let _ = write_reply(&mut control, failure.reply, None).await;
            return failed(accounting, failure);
        }
    };
    let upstream_v4 = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(socket) => socket,
        Err(error) => {
            accounting.socks_udp_associate_refused = 1;
            let failure = SocksFailure::new("socks-udp-ipv4-bind-failed", error.to_string());
            let _ = write_reply(&mut control, failure.reply, None).await;
            return failed(accounting, failure);
        }
    };
    let upstream_v6 = UdpSocket::bind("[::]:0").await.ok();
    let relay_address = match relay.local_addr() {
        Ok(address) => address,
        Err(error) => {
            accounting.socks_udp_associate_refused = 1;
            return failed(
                accounting,
                SocksFailure::new("socks-udp-local-address-failed", error.to_string()),
            );
        }
    };
    if let Err(error) =
        write_reply(&mut control, SocksReplyCode::Succeeded, Some(relay_address)).await
    {
        accounting.socks_udp_associate_refused = 1;
        return failed(
            accounting,
            SocksFailure::new("socks-udp-reply-write-failed", error.to_string()),
        );
    }
    accounting.socks_udp_associate_succeeded = 1;
    emit_udp(
        sink,
        session_id,
        connection_id,
        "association",
        if claimed.is_some_and(|value| value.port() != 0) {
            "declared-client"
        } else {
            "awaiting-client"
        },
        Some(request.address_type),
        Some(relay_address),
        0,
        0,
    );

    let capacity = limits.max_socks_udp_datagram_bytes.saturating_add(1);
    let upstream_capacity = limits
        .max_socks_udp_datagram_bytes
        .saturating_sub(22)
        .saturating_add(1);
    let mut client_buffer = vec![0_u8; capacity];
    let mut upstream_v4_buffer = vec![0_u8; upstream_capacity];
    let mut upstream_v6_buffer = vec![0_u8; upstream_capacity];
    let mut client_endpoint = claimed.filter(|value| value.port() != 0);
    let mut peers = BTreeSet::new();
    let idle = tokio::time::sleep(limits.idle_timeout);
    tokio::pin!(idle);
    let terminal = loop {
        let mut control_byte = [0_u8; 1];
        tokio::select! {
            biased;
            _ = shutdown.changed() => break Err(SocksFailure::cancelled()),
            result = control.read(&mut control_byte) => match result {
                Ok(0) => break Ok("control-closed"),
                Ok(_) => break Err(SocksFailure::new(
                    "socks-udp-control-data-unexpected",
                    "UDP association control connection carried unexpected data",
                )),
                Err(error) => break Err(SocksFailure::new(
                    "socks-udp-control-read-failed",
                    error.to_string(),
                )),
            },
            result = relay.recv_from(&mut client_buffer) => {
                let (read, source) = match result {
                    Ok(value) => value,
                    Err(error) => break Err(SocksFailure::new("socks-udp-client-read-failed", error.to_string())),
                };
                accounting.socks_udp_client_datagrams = accounting.socks_udp_client_datagrams.saturating_add(1);
                if normalize_ip(source.ip()) != normalize_ip(peer.ip())
                    || client_endpoint.is_some_and(|expected| normalize_socket(expected) != normalize_socket(source))
                {
                    accounting.socks_udp_source_dropped = accounting.socks_udp_source_dropped.saturating_add(1);
                    emit_udp(sink, session_id, connection_id, "drop", "client-source-refused", None, Some(source), 0, peers.len());
                    continue;
                }
                if read > limits.max_socks_udp_datagram_bytes {
                    accounting.socks_udp_oversized_dropped = accounting.socks_udp_oversized_dropped.saturating_add(1);
                    emit_udp(sink, session_id, connection_id, "drop", "oversized", None, Some(source), read as u64, peers.len());
                    continue;
                }
                let datagram = match parse_udp_datagram(&client_buffer[..read]) {
                    Ok(value) => value,
                    Err(UdpFrameError::Fragmented) => {
                        accounting.socks_udp_fragment_dropped = accounting.socks_udp_fragment_dropped.saturating_add(1);
                        emit_udp(sink, session_id, connection_id, "drop", "fragmentation-unsupported", None, Some(source), 0, peers.len());
                        continue;
                    }
                    Err(UdpFrameError::Malformed) => {
                        accounting.socks_udp_malformed_dropped = accounting.socks_udp_malformed_dropped.saturating_add(1);
                        emit_udp(sink, session_id, connection_id, "drop", "malformed", None, Some(source), 0, peers.len());
                        continue;
                    }
                };
                if client_endpoint.is_none() {
                    client_endpoint = Some(source);
                }
                let address_type = datagram.address_type;
                let payload_len = datagram.payload.len();
                let forwarding = tokio::select! {
                    biased;
                    _ = shutdown.changed() => OwnerBound::Shutdown,
                    result = control.read(&mut control_byte) => OwnerBound::Control(result),
                    _ = &mut idle => OwnerBound::Idle,
                    result = forward_client_datagram(
                        &datagram.authority,
                        datagram.payload,
                        policy,
                        limits,
                        &upstream_v4,
                        upstream_v6.as_ref(),
                        &peers,
                    ) => OwnerBound::Completed(result),
                };
                let (selected, written) = match forwarding {
                    OwnerBound::Shutdown => {
                        record_udp_owner_drop(
                            &mut accounting, sink, session_id, connection_id,
                            "cancellation", address_type, payload_len, peers.len(),
                        );
                        break Err(SocksFailure::cancelled());
                    }
                    OwnerBound::Idle => {
                        record_udp_owner_drop(
                            &mut accounting, sink, session_id, connection_id,
                            "timeout", address_type, payload_len, peers.len(),
                        );
                        break Err(SocksFailure::timeout(
                            "socks-udp-idle-timeout",
                            SocksReplyCode::TtlExpired,
                        ));
                    }
                    OwnerBound::Control(result) => {
                        record_udp_owner_drop(
                            &mut accounting, sink, session_id, connection_id,
                            "control-revoked", address_type, payload_len, peers.len(),
                        );
                        match result {
                            Ok(0) => break Ok("control-closed"),
                            Ok(_) => break Err(SocksFailure::new(
                                "socks-udp-control-data-unexpected",
                                "UDP association control connection carried unexpected data",
                            )),
                            Err(error) => break Err(SocksFailure::new(
                                "socks-udp-control-read-failed",
                                error.to_string(),
                            )),
                        }
                    }
                    OwnerBound::Completed(Ok(value)) => value,
                    OwnerBound::Completed(Err(UdpForwardFailure::Policy)) => {
                        accounting.socks_udp_policy_dropped = accounting.socks_udp_policy_dropped.saturating_add(1);
                        emit_udp(sink, session_id, connection_id, "drop", "destination-refused", Some(address_type), None, payload_len as u64, peers.len());
                        continue;
                    }
                    OwnerBound::Completed(Err(UdpForwardFailure::Resolution)) => {
                        accounting.socks_udp_resolution_dropped = accounting.socks_udp_resolution_dropped.saturating_add(1);
                        emit_udp(sink, session_id, connection_id, "drop", "resolution-failed", Some(address_type), None, payload_len as u64, peers.len());
                        continue;
                    }
                    OwnerBound::Completed(Err(UdpForwardFailure::DnsTimeout)) => {
                        accounting.socks_udp_resolution_dropped = accounting.socks_udp_resolution_dropped.saturating_add(1);
                        accounting.timed_out = accounting.timed_out.saturating_add(1);
                        emit_udp(sink, session_id, connection_id, "drop", "timeout", Some(address_type), None, payload_len as u64, peers.len());
                        continue;
                    }
                    OwnerBound::Completed(Err(UdpForwardFailure::Ipv6Unavailable(selected))) => {
                        accounting.socks_udp_resolution_dropped = accounting.socks_udp_resolution_dropped.saturating_add(1);
                        emit_udp(sink, session_id, connection_id, "drop", "ipv6-unavailable", Some(address_type), Some(selected), payload_len as u64, peers.len());
                        continue;
                    }
                    OwnerBound::Completed(Err(UdpForwardFailure::PeerLimit(selected))) => {
                        accounting.socks_udp_peer_limit_dropped = accounting.socks_udp_peer_limit_dropped.saturating_add(1);
                        emit_udp(sink, session_id, connection_id, "drop", "peer-limit", Some(address_type), Some(selected), payload_len as u64, peers.len());
                        continue;
                    }
                    OwnerBound::Completed(Err(UdpForwardFailure::Transport(selected))) => {
                        accounting.socks_udp_transport_dropped = accounting.socks_udp_transport_dropped.saturating_add(1);
                        emit_udp(sink, session_id, connection_id, "drop", "transport-failed", Some(address_type), Some(selected), payload_len as u64, peers.len());
                        continue;
                    }
                };
                peers.insert(selected);
                accounting.socks_udp_peak_peers = accounting.socks_udp_peak_peers.max(peers.len() as u64);
                accounting.socks_udp_client_forwarded = accounting.socks_udp_client_forwarded.saturating_add(1);
                accounting.socks_udp_client_bytes = accounting.socks_udp_client_bytes.saturating_add(written as u64);
                emit_udp(sink, session_id, connection_id, "datagram", "forwarded", Some(address_type), Some(selected), written as u64, peers.len());
                idle.as_mut().reset(Instant::now() + limits.idle_timeout);
            },
            result = upstream_v4.recv_from(&mut upstream_v4_buffer) => {
                let (read, source) = match result {
                    Ok(value) => value,
                    Err(error) => break Err(SocksFailure::new("socks-udp-ipv4-read-failed", error.to_string())),
                };
                if handle_upstream_datagram(
                    &upstream_v4_buffer[..read],
                    source,
                    UdpReplyContext {
                        relay: &relay,
                        client_endpoint,
                        peers: &peers,
                        limits,
                        sink,
                        session_id,
                        connection_id,
                    },
                    &mut accounting,
                ).await {
                    idle.as_mut().reset(Instant::now() + limits.idle_timeout);
                }
            },
            result = recv_optional(upstream_v6.as_ref(), &mut upstream_v6_buffer) => {
                let (read, source) = match result {
                    Ok(value) => value,
                    Err(error) => break Err(SocksFailure::new("socks-udp-ipv6-read-failed", error.to_string())),
                };
                if handle_upstream_datagram(
                    &upstream_v6_buffer[..read],
                    source,
                    UdpReplyContext {
                        relay: &relay,
                        client_endpoint,
                        peers: &peers,
                        limits,
                        sink,
                        session_id,
                        connection_id,
                    },
                    &mut accounting,
                ).await {
                    idle.as_mut().reset(Instant::now() + limits.idle_timeout);
                }
            },
            _ = &mut idle => break Err(SocksFailure::timeout("socks-udp-idle-timeout", SocksReplyCode::TtlExpired)),
        }
    };
    peers.clear();
    drop(upstream_v6);
    drop(upstream_v4);
    drop(relay);
    emit_udp(
        sink,
        session_id,
        connection_id,
        "terminal",
        udp_terminal_outcome(&terminal),
        None,
        None,
        0,
        0,
    );
    let observation = udp_observation(session_id, connection_id, peer, local);
    match terminal {
        Ok(_) => HttpRun {
            observations: vec![observation],
            accounting,
            failure: None,
        },
        Err(failure) => {
            accounting.timed_out = accounting
                .timed_out
                .saturating_add(u64::from(failure.timed_out));
            failed_with_observation(accounting, failure, observation)
        }
    }
}

enum OwnerBound<T> {
    Completed(T),
    Shutdown,
    Idle,
    Control(io::Result<usize>),
}

enum UdpForwardFailure {
    Policy,
    Resolution,
    DnsTimeout,
    Ipv6Unavailable(SocketAddr),
    PeerLimit(SocketAddr),
    Transport(SocketAddr),
}

async fn forward_client_datagram(
    authority: &DestinationAuthority,
    payload: &[u8],
    policy: &DestinationPolicy,
    limits: &ProtocolLimits,
    upstream_v4: &UdpSocket,
    upstream_v6: Option<&UdpSocket>,
    peers: &BTreeSet<SocketAddr>,
) -> Result<(SocketAddr, usize), UdpForwardFailure> {
    let candidates = crate::resolve_allowed_udp(authority, policy, limits.upstream.dns)
        .await
        .map_err(|error| match (error.stage, error.code) {
            (UpstreamStage::Policy, _) => UdpForwardFailure::Policy,
            (_, "dns-timeout") => UdpForwardFailure::DnsTimeout,
            _ => UdpForwardFailure::Resolution,
        })?;
    let selected = candidates
        .iter()
        .copied()
        .find(|address| peers.contains(address))
        .or_else(|| candidates.first().copied())
        .ok_or(UdpForwardFailure::Resolution)?;
    if !peers.contains(&selected) && peers.len() >= limits.max_socks_udp_peers {
        return Err(UdpForwardFailure::PeerLimit(selected));
    }
    let send = match selected {
        SocketAddr::V4(_) => {
            timeout(
                limits.upstream.write,
                upstream_v4.send_to(payload, selected),
            )
            .await
        }
        SocketAddr::V6(_) => {
            let socket = upstream_v6.ok_or(UdpForwardFailure::Ipv6Unavailable(selected))?;
            timeout(limits.upstream.write, socket.send_to(payload, selected)).await
        }
    };
    match send {
        Ok(Ok(written)) if written == payload.len() => Ok((selected, written)),
        _ => Err(UdpForwardFailure::Transport(selected)),
    }
}

fn udp_terminal_outcome(terminal: &Result<&'static str, SocksFailure>) -> &'static str {
    match terminal {
        Ok(outcome) => outcome,
        Err(failure) if failure.timed_out => "timed-out",
        Err(failure) if failure.code == "connection-cancelled" => "cancelled",
        Err(failure) => failure.code,
    }
}

async fn recv_optional(
    socket: Option<&UdpSocket>,
    buffer: &mut [u8],
) -> io::Result<(usize, SocketAddr)> {
    match socket {
        Some(socket) => socket.recv_from(buffer).await,
        None => pending().await,
    }
}

struct UdpReplyContext<'a> {
    relay: &'a UdpSocket,
    client_endpoint: Option<SocketAddr>,
    peers: &'a BTreeSet<SocketAddr>,
    limits: &'a ProtocolLimits,
    sink: &'a crate::application::SharedEventSink,
    session_id: &'a str,
    connection_id: u64,
}

async fn handle_upstream_datagram(
    payload: &[u8],
    source: SocketAddr,
    context: UdpReplyContext<'_>,
    accounting: &mut ProtocolAccounting,
) -> bool {
    let UdpReplyContext {
        relay,
        client_endpoint,
        peers,
        limits,
        sink,
        session_id,
        connection_id,
    } = context;
    accounting.socks_udp_upstream_datagrams =
        accounting.socks_udp_upstream_datagrams.saturating_add(1);
    if payload.len() > limits.max_socks_udp_datagram_bytes.saturating_sub(22) {
        accounting.socks_udp_oversized_dropped =
            accounting.socks_udp_oversized_dropped.saturating_add(1);
        emit_udp(
            sink,
            session_id,
            connection_id,
            "drop",
            "oversized",
            None,
            Some(source),
            payload.len() as u64,
            peers.len(),
        );
        return false;
    }
    if !peers.contains(&source) || client_endpoint.is_none() {
        accounting.socks_udp_unsolicited_dropped =
            accounting.socks_udp_unsolicited_dropped.saturating_add(1);
        emit_udp(
            sink,
            session_id,
            connection_id,
            "drop",
            "unsolicited-peer",
            None,
            Some(source),
            payload.len() as u64,
            peers.len(),
        );
        return false;
    }
    let mut framed = encode_udp_response(source, payload);
    let client_endpoint = client_endpoint.expect("checked above");
    match timeout(
        limits.upstream.write,
        relay.send_to(&framed, client_endpoint),
    )
    .await
    {
        Ok(Ok(written)) if written == framed.len() => {
            accounting.socks_udp_upstream_forwarded =
                accounting.socks_udp_upstream_forwarded.saturating_add(1);
            accounting.socks_udp_upstream_bytes = accounting
                .socks_udp_upstream_bytes
                .saturating_add(payload.len() as u64);
            emit_udp(
                sink,
                session_id,
                connection_id,
                "datagram",
                "forwarded",
                Some(address_type(source)),
                Some(source),
                payload.len() as u64,
                peers.len(),
            );
            framed.fill(0);
            return true;
        }
        _ => {
            accounting.socks_udp_transport_dropped =
                accounting.socks_udp_transport_dropped.saturating_add(1);
            emit_udp(
                sink,
                session_id,
                connection_id,
                "drop",
                "client-send-failed",
                Some(address_type(source)),
                Some(source),
                payload.len() as u64,
                peers.len(),
            );
        }
    }
    framed.fill(0);
    false
}

#[derive(Debug)]
struct ParsedUdpDatagram<'a> {
    authority: DestinationAuthority,
    address_type: SocksAddressType,
    payload: &'a [u8],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UdpFrameError {
    Fragmented,
    Malformed,
}

fn parse_udp_datagram(bytes: &[u8]) -> Result<ParsedUdpDatagram<'_>, UdpFrameError> {
    if bytes.get(2).is_some_and(|fragment| *fragment != 0) {
        return Err(UdpFrameError::Fragmented);
    }
    if bytes.len() < 7 || bytes[0..2] != [0, 0] {
        return Err(UdpFrameError::Malformed);
    }
    let mut cursor = 4;
    let (host, address_type) = match bytes[3] {
        1 if bytes.len() >= cursor + 4 + 2 => {
            let ip = Ipv4Addr::new(
                bytes[cursor],
                bytes[cursor + 1],
                bytes[cursor + 2],
                bytes[cursor + 3],
            );
            cursor += 4;
            (ip.to_string(), SocksAddressType::Ipv4)
        }
        4 if bytes.len() >= cursor + 16 + 2 => {
            let octets: [u8; 16] = bytes[cursor..cursor + 16]
                .try_into()
                .map_err(|_| UdpFrameError::Malformed)?;
            cursor += 16;
            (
                format!("[{}]", Ipv6Addr::from(octets)),
                SocksAddressType::Ipv6,
            )
        }
        3 if bytes.len() > cursor => {
            let length = usize::from(bytes[cursor]);
            cursor += 1;
            if length == 0 || bytes.len() < cursor + length + 2 {
                return Err(UdpFrameError::Malformed);
            }
            let domain = std::str::from_utf8(&bytes[cursor..cursor + length])
                .map_err(|_| UdpFrameError::Malformed)?;
            cursor += length;
            (domain.to_string(), SocksAddressType::Domain)
        }
        _ => return Err(UdpFrameError::Malformed),
    };
    if bytes.len() < cursor + 2 {
        return Err(UdpFrameError::Malformed);
    }
    let port = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]);
    cursor += 2;
    let authority = DestinationAuthority::parse(&format!("{host}:{port}"))
        .map_err(|_| UdpFrameError::Malformed)?;
    Ok(ParsedUdpDatagram {
        authority,
        address_type,
        payload: &bytes[cursor..],
    })
}

fn encode_udp_response(source: SocketAddr, payload: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0, 0, 0];
    match source {
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
    bytes.extend_from_slice(payload);
    bytes
}

fn normalize_ip(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(ip) => ip
            .to_ipv4_mapped()
            .map(IpAddr::V4)
            .unwrap_or(IpAddr::V6(ip)),
        ip => ip,
    }
}

fn normalize_socket(address: SocketAddr) -> SocketAddr {
    SocketAddr::new(normalize_ip(address.ip()), address.port())
}

fn address_type(address: SocketAddr) -> SocksAddressType {
    match address {
        SocketAddr::V4(_) => SocksAddressType::Ipv4,
        SocketAddr::V6(_) => SocksAddressType::Ipv6,
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "owner revocation must update accounting and emit the matching loss fact together"
)]
fn record_udp_owner_drop(
    accounting: &mut ProtocolAccounting,
    sink: &crate::application::SharedEventSink,
    session_id: &str,
    connection_id: u64,
    outcome: &'static str,
    address_type: SocksAddressType,
    payload_bytes: usize,
    active_peers: usize,
) {
    accounting.socks_udp_owner_dropped = accounting.socks_udp_owner_dropped.saturating_add(1);
    emit_udp(
        sink,
        session_id,
        connection_id,
        "drop",
        outcome,
        Some(address_type),
        None,
        payload_bytes as u64,
        active_peers,
    );
}

#[expect(
    clippy::too_many_arguments,
    reason = "the typed UDP evidence fields remain explicit at each security decision"
)]
fn emit_udp(
    sink: &crate::application::SharedEventSink,
    session_id: &str,
    connection_id: u64,
    action: &'static str,
    outcome: &'static str,
    address_type: Option<SocksAddressType>,
    remote: Option<SocketAddr>,
    payload_bytes: u64,
    active_peers: usize,
) {
    crate::application::emit(
        sink,
        ApplicationEvent::now(
            session_id,
            connection_id,
            None,
            None,
            ApplicationEventKind::SocksUdp(SocksUdpEvent {
                action,
                outcome,
                address_type: address_type.map(SocksAddressType::as_str),
                remote,
                payload_bytes,
                active_peers,
            }),
        ),
    );
}

fn udp_observation(
    session_id: &str,
    connection_id: u64,
    peer: SocketAddr,
    local: SocketAddr,
) -> ProxyObservation {
    let mut value = observation(
        session_id,
        connection_id,
        peer,
        local,
        "",
        "udp-association",
    );
    value.method = Some("UDP_ASSOCIATE".to_string());
    value.url = None;
    value
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
    request_command: Option<u8>,
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
            request_command: None,
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

    fn for_request_command(mut self, command: u8) -> Self {
        self.request_command = Some(command);
        self
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

    #[test]
    fn udp_frames_cover_address_forms_and_refuse_fragmentation() {
        let ipv4 = [0, 0, 0, 1, 127, 0, 0, 1, 0, 53, 1, 2];
        let parsed = parse_udp_datagram(&ipv4).unwrap();
        assert_eq!(parsed.address_type, SocksAddressType::Ipv4);
        assert_eq!(parsed.authority.lookup_host(), "127.0.0.1:53");
        assert_eq!(parsed.payload, [1, 2]);

        let domain = [
            0, 0, 0, 3, 9, b'l', b'o', b'c', b'a', b'l', b'h', b'o', b's', b't', 0, 53,
        ];
        let parsed = parse_udp_datagram(&domain).unwrap();
        assert_eq!(parsed.address_type, SocksAddressType::Domain);
        assert_eq!(parsed.authority.lookup_host(), "localhost:53");

        let mut ipv6 = vec![0, 0, 0, 4];
        ipv6.extend_from_slice(&Ipv6Addr::LOCALHOST.octets());
        ipv6.extend_from_slice(&53_u16.to_be_bytes());
        ipv6.extend_from_slice(b"payload");
        let parsed = parse_udp_datagram(&ipv6).unwrap();
        assert_eq!(parsed.address_type, SocksAddressType::Ipv6);
        assert_eq!(parsed.payload, b"payload");

        let mut fragmented = ipv4;
        fragmented[2] = 1;
        assert_eq!(
            parse_udp_datagram(&fragmented).unwrap_err(),
            UdpFrameError::Fragmented
        );
        for malformed in [&[][..], &[0, 0, 0, 9][..], &[0, 0, 0, 3, 4, b'a'][..]] {
            assert_eq!(
                parse_udp_datagram(malformed).unwrap_err(),
                UdpFrameError::Malformed
            );
        }
    }

    #[test]
    fn udp_response_names_the_observed_source() {
        let source: SocketAddr = "[::1]:4242".parse().unwrap();
        let encoded = encode_udp_response(source, b"reply");
        assert_eq!(&encoded[..4], &[0, 0, 0, 4]);
        assert_eq!(&encoded[4..20], &Ipv6Addr::LOCALHOST.octets());
        assert_eq!(&encoded[20..22], &4242_u16.to_be_bytes());
        assert_eq!(&encoded[22..], b"reply");
    }
}
