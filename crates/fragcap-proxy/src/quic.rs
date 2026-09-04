// SPDX-License-Identifier: Apache-2.0

//! Scoped QUIC configuration, identity, and bounded application observation.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use std::time::SystemTime;

use bytes::{Buf, Bytes};
use quinn::crypto::rustls::{QuicClientConfig, QuicServerConfig};
use quinn::Endpoint;
use tokio::net::UdpSocket;
use tokio::task::JoinHandle;

use crate::{
    application, ApplicationEvent, ApplicationEventKind, BodyDirection, BodyOutcome,
    BodyRepresentation, BodySegment, CertificateIdentity, DestinationAuthority, EventDisposition,
    GenericStreamOutcome, GenericUdpOutcome, HttpTiming, LeafCache, MetadataBlock, MetadataField,
    MetadataKind, ProtocolAccounting, ProtocolLimits, ProtocolVersion, QuicConnectionEvent,
    QuicDatagramEvent, QuicDirection, QuicHalf, QuicRefusalEvent, QuicStreamEvent,
    SessionCertificateAuthority, SessionKeyLog,
};

static NEXT_PAIR_ID: AtomicU64 = AtomicU64::new(1);

struct AbortOnDrop<T>(JoinHandle<T>);

impl<T> AbortOnDrop<T> {
    fn new(task: JoinHandle<T>) -> Self {
        Self(task)
    }

    fn abort(&self) {
        self.0.abort();
    }
}

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuicRefusalCode {
    RouteUnscoped,
    OriginChanged,
    IdentityUnavailable,
    ClientTrustRejected,
    UpstreamValidationFailed,
    ClientCertificateRequired,
    PinningSuspected,
    ZeroRttRefused,
    MigrationRefused,
    AlpnUnsupported,
    CapacityExhausted,
    TransportFailed,
    Http3ProtocolFailed,
}

/// Owns the loopback datagram seam and the two independently authenticated QUIC halves.
///
/// The SOCKS association remains the authority for the real client and origin endpoints.
/// Quinn sees only this private bridge on its client-facing half, which prevents a QUIC
/// connection from widening the association's immutable route.
pub(crate) struct QuicAssociationGateway {
    plan: QuicInspectionPlan,
    bridge: UdpSocket,
    server_address: SocketAddr,
    task: Option<JoinHandle<()>>,
    observer: Arc<tokio::sync::Mutex<QuicEvidenceObserver>>,
    terminal: tokio::sync::watch::Receiver<Option<QuicGatewayTerminal>>,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QuicGatewayTerminal {
    Complete,
    Refused(QuicRefusalCode),
}

pub(crate) enum QuicGatewayIoError {
    Terminal(QuicGatewayTerminal),
    Io(io::Error),
}

pub(crate) struct QuicGatewayReport {
    pub accounting: ProtocolAccounting,
    pub terminal: Option<QuicGatewayTerminal>,
}

impl QuicAssociationGateway {
    #[expect(
        clippy::too_many_arguments,
        reason = "the gateway receives one immutable session authority bundle"
    )]
    pub(crate) async fn start(
        plan: QuicInspectionPlan,
        authority: DestinationAuthority,
        ca: Arc<SessionCertificateAuthority>,
        cache: Arc<tokio::sync::Mutex<LeafCache>>,
        upstream_tls: Arc<rustls::ClientConfig>,
        key_log: Option<Arc<SessionKeyLog>>,
        limits: ProtocolLimits,
        sink: application::SharedEventSink,
        session_retained: Arc<AtomicU64>,
        slots: Arc<tokio::sync::Semaphore>,
    ) -> Result<Self, QuicRefusalCode> {
        let permit = slots
            .try_acquire_owned()
            .map_err(|_| QuicRefusalCode::CapacityExhausted)?;
        let server_config = {
            let mut cache = cache.lock().await;
            build_quic_server_config(&authority, &ca, &mut cache, key_log, &limits)?
        };
        let loopback = loopback_for(plan.client_endpoint().ip(), 0);
        let server = Endpoint::server(server_config, loopback)
            .map_err(|_| QuicRefusalCode::TransportFailed)?;
        let server_address = server
            .local_addr()
            .map_err(|_| QuicRefusalCode::TransportFailed)?;
        let bridge = UdpSocket::bind(loopback_for(server_address.ip(), 0))
            .await
            .map_err(|_| QuicRefusalCode::TransportFailed)?;
        let task_plan = plan.clone();
        let observer = Arc::new(tokio::sync::Mutex::new(
            QuicEvidenceObserver::new(plan.clone()).with_session_retention(session_retained),
        ));
        let task_observer = Arc::clone(&observer);
        let (terminal_tx, terminal) = tokio::sync::watch::channel(None);
        let task = tokio::spawn(async move {
            let run_observer = Arc::clone(&task_observer);
            let result = run_pair(
                &server,
                task_plan.clone(),
                upstream_tls,
                limits,
                sink.clone(),
                run_observer,
            )
            .await;
            let outcome = result.map_or_else(QuicGatewayTerminal::Refused, |_| {
                QuicGatewayTerminal::Complete
            });
            terminal_tx.send_replace(Some(outcome));
            if let QuicGatewayTerminal::Refused(code) = outcome {
                let event = task_observer.lock().await.record_refusal(code);
                application::emit(&sink, event);
            }
        });
        Ok(Self {
            plan,
            bridge,
            server_address,
            task: Some(task),
            observer,
            terminal,
            _permit: permit,
        })
    }

    pub(crate) fn admits(
        &self,
        client: SocketAddr,
        origin: SocketAddr,
        authority: &DestinationAuthority,
    ) -> Result<(), QuicRefusalCode> {
        self.plan.admits(client, origin, authority)
    }

    pub(crate) async fn send(&self, payload: &[u8]) -> Result<usize, QuicGatewayIoError> {
        let mut terminal = self.terminal.clone();
        if let Some(outcome) = *terminal.borrow() {
            return Err(QuicGatewayIoError::Terminal(outcome));
        }
        tokio::select! {
            biased;
            changed = terminal.changed() => match changed {
                Ok(()) => Err(QuicGatewayIoError::Terminal(
                    (*terminal.borrow()).unwrap_or(QuicGatewayTerminal::Refused(
                        QuicRefusalCode::TransportFailed,
                    )),
                )),
                Err(_) => Err(QuicGatewayIoError::Terminal(
                    QuicGatewayTerminal::Refused(QuicRefusalCode::TransportFailed),
                )),
            },
            result = self.bridge.send_to(payload, self.server_address) => {
                result.map_err(QuicGatewayIoError::Io)
            }
        }
    }

    pub(crate) async fn recv(&self, payload: &mut [u8]) -> Result<usize, QuicGatewayIoError> {
        let mut terminal = self.terminal.clone();
        let result = tokio::select! {
            biased;
            changed = terminal.changed() => match changed {
                Ok(()) => return Err(QuicGatewayIoError::Terminal(
                    (*terminal.borrow()).unwrap_or(QuicGatewayTerminal::Refused(
                        QuicRefusalCode::TransportFailed,
                    )),
                )),
                Err(_) => return Err(QuicGatewayIoError::Terminal(
                    QuicGatewayTerminal::Refused(QuicRefusalCode::TransportFailed),
                )),
            },
            result = self.bridge.recv_from(payload) => result,
        };
        let (read, source) = result.map_err(QuicGatewayIoError::Io)?;
        if source != self.server_address {
            return Err(QuicGatewayIoError::Io(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "unexpected QUIC bridge peer",
            )));
        }
        Ok(read)
    }

    pub(crate) fn plan(&self) -> &QuicInspectionPlan {
        &self.plan
    }

    pub(crate) async fn close(mut self) -> QuicGatewayReport {
        if let Some(task) = self.task.take() {
            task.abort();
            let _ = task.await;
        }
        QuicGatewayReport {
            accounting: self.observer.lock().await.accounting,
            terminal: *self.terminal.borrow(),
        }
    }
}

impl Drop for QuicAssociationGateway {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}

fn loopback_for(ip: IpAddr, port: u16) -> SocketAddr {
    match ip {
        IpAddr::V4(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
        IpAddr::V6(_) => SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), port),
    }
}

fn client_endpoint_for(ip: IpAddr) -> Result<Endpoint, QuicRefusalCode> {
    let socket = std::net::UdpSocket::bind(loopback_for(ip, 0))
        .map_err(|_| QuicRefusalCode::TransportFailed)?;
    Endpoint::new(
        quinn::EndpointConfig::default(),
        None,
        socket,
        quinn::default_runtime().ok_or(QuicRefusalCode::TransportFailed)?,
    )
    .map_err(|_| QuicRefusalCode::TransportFailed)
}

async fn run_pair(
    server: &Endpoint,
    plan: QuicInspectionPlan,
    upstream_tls: Arc<rustls::ClientConfig>,
    limits: ProtocolLimits,
    sink: application::SharedEventSink,
    observer: Arc<tokio::sync::Mutex<QuicEvidenceObserver>>,
) -> Result<(), QuicRefusalCode> {
    let incoming = tokio::time::timeout(limits.tls_handshake_timeout, server.accept())
        .await
        .map_err(|_| QuicRefusalCode::TransportFailed)?
        .ok_or(QuicRefusalCode::TransportFailed)?;
    let client = tokio::time::timeout(limits.tls_handshake_timeout, incoming)
        .await
        .map_err(|_| QuicRefusalCode::TransportFailed)?
        .map_err(|_| QuicRefusalCode::ClientTrustRejected)?;
    let client_alpn = negotiated_alpn(&client);
    emit_observed(
        &sink,
        observer
            .lock()
            .await
            .connection(QuicHalf::Client, client_alpn.clone(), "established"),
    );
    if client_alpn.as_deref() != Some(b"h3".as_slice()) {
        return Err(QuicRefusalCode::AlpnUnsupported);
    }

    let mut upstream = client_endpoint_for(plan.origin_endpoint().ip())?;
    upstream.set_default_client_config(build_quic_client_config(&upstream_tls, &limits)?);
    let connecting = upstream
        .connect(plan.origin_endpoint(), plan.server_name())
        .map_err(|_| QuicRefusalCode::UpstreamValidationFailed)?;
    let origin = tokio::time::timeout(limits.tls_handshake_timeout, connecting)
        .await
        .map_err(|_| QuicRefusalCode::TransportFailed)?
        .map_err(|_| QuicRefusalCode::UpstreamValidationFailed)?;
    let origin_alpn = negotiated_alpn(&origin);
    emit_observed(
        &sink,
        observer
            .lock()
            .await
            .connection(QuicHalf::Upstream, origin_alpn.clone(), "established"),
    );
    if origin_alpn.as_deref() != Some(b"h3".as_slice()) {
        return Err(QuicRefusalCode::AlpnUnsupported);
    }

    proxy_http3(client.clone(), origin.clone(), observer, limits, sink).await?;
    client.close(0_u32.into(), b"complete");
    origin.close(0_u32.into(), b"complete");
    server.wait_idle().await;
    upstream.wait_idle().await;
    Ok(())
}

fn negotiated_alpn(connection: &quinn::Connection) -> Option<Vec<u8>> {
    connection
        .handshake_data()?
        .downcast_ref::<quinn::crypto::rustls::HandshakeData>()?
        .protocol
        .clone()
}

async fn proxy_http3(
    client: quinn::Connection,
    origin: quinn::Connection,
    observer: Arc<tokio::sync::Mutex<QuicEvidenceObserver>>,
    limits: ProtocolLimits,
    sink: application::SharedEventSink,
) -> Result<(), QuicRefusalCode> {
    let datagram_count = Arc::new(AtomicU64::new(0));
    let client_datagrams = client.clone();
    let origin_datagrams = origin.clone();
    let request_datagrams = spawn_datagram_pump(
        client_datagrams,
        origin_datagrams.clone(),
        QuicDirection::ClientToUpstream,
        Arc::clone(&observer),
        limits.clone(),
        sink.clone(),
        Arc::clone(&datagram_count),
    );
    let response_datagrams = spawn_datagram_pump(
        origin_datagrams,
        client.clone(),
        QuicDirection::UpstreamToClient,
        Arc::clone(&observer),
        limits.clone(),
        sink.clone(),
        datagram_count,
    );
    let mut server = h3::server::Connection::new(h3_quinn::Connection::new(client))
        .await
        .map_err(|_| QuicRefusalCode::Http3ProtocolFailed)?;
    let (mut driver, sender) = h3::client::new(h3_quinn::Connection::new(origin))
        .await
        .map_err(|_| QuicRefusalCode::Http3ProtocolFailed)?;
    let driver_task = AbortOnDrop::new(tokio::spawn(async move {
        std::future::poll_fn(|cx| driver.poll_close(cx)).await
    }));
    let mut requests = tokio::task::JoinSet::new();
    let mut admitted = 0_usize;
    loop {
        let resolver = match server.accept().await {
            Ok(Some(value)) => value,
            Ok(None) => break,
            Err(error) if error.is_h3_no_error() => break,
            Err(_) => return Err(QuicRefusalCode::Http3ProtocolFailed),
        };
        if admitted >= limits.max_requests_per_connection {
            return Err(QuicRefusalCode::CapacityExhausted);
        }
        admitted += 1;
        let mut sender = sender.clone();
        let stream_observer = Arc::clone(&observer);
        let stream_limits = limits.clone();
        let stream_sink = sink.clone();
        requests.spawn(async move {
            let started_at = Instant::now();
            let (request, client_stream) = resolver
                .resolve_request()
                .await
                .map_err(|_| QuicRefusalCode::Http3ProtocolFailed)?;
            let stream_id = client_stream.id().index();
            let request_event = {
                let mut observer = stream_observer.lock().await;
                observer.http3_stream_open(stream_id);
                ApplicationEvent::now(
                    observer.plan.session_id(),
                    observer.plan.association_id(),
                    Some(stream_id),
                    Some(ProtocolVersion::Http3),
                    ApplicationEventKind::Metadata(request_metadata(&request)),
                )
            };
            emit_observed(&stream_sink, request_event);
            emit_observed(
                &stream_sink,
                http3_event(
                    &stream_observer,
                    stream_id,
                    ApplicationEventKind::HttpStreamOpen,
                )
                .await,
            );
            let origin_stream = match sender.send_request(request).await {
                Ok(stream) => stream,
                Err(error) => {
                    let terminal = http3_stream_terminal(error);
                    emit_http3_terminal(&stream_observer, &stream_sink, stream_id, terminal).await;
                    return Err(QuicRefusalCode::Http3ProtocolFailed);
                }
            };
            let (mut client_send, mut client_recv) = client_stream.split();
            let (mut origin_send, mut origin_recv) = origin_stream.split();

            let request_observer = Arc::clone(&stream_observer);
            let request_sink = stream_sink.clone();
            let request_limits = stream_limits.clone();
            let request_forward = async move {
                let mut evidence = Vec::with_capacity(request_limits.max_event_chunk_bytes);
                loop {
                    let next = match client_recv.recv_data().await {
                        Ok(value) => value,
                        Err(error) => {
                            emit_coalesced_stream(
                                &request_observer,
                                &request_sink,
                                QuicDirection::ClientToUpstream,
                                stream_id,
                                "http3-request",
                                &mut evidence,
                                &request_limits,
                                true,
                            )
                            .await;
                            return Err(http3_stream_terminal(error));
                        }
                    };
                    let Some(mut data) = next else {
                        break;
                    };
                    let bytes = copy_buf(&mut data);
                    let sent = origin_send.send_data(bytes.clone()).await;
                    if sent.is_ok() {
                        evidence.extend_from_slice(&bytes);
                        emit_coalesced_stream(
                            &request_observer,
                            &request_sink,
                            QuicDirection::ClientToUpstream,
                            stream_id,
                            "http3-request",
                            &mut evidence,
                            &request_limits,
                            false,
                        )
                        .await;
                    } else {
                        emit_coalesced_stream(
                            &request_observer,
                            &request_sink,
                            QuicDirection::ClientToUpstream,
                            stream_id,
                            "http3-request",
                            &mut evidence,
                            &request_limits,
                            true,
                        )
                        .await;
                        let event = request_observer.lock().await.stream(
                            QuicDirection::ClientToUpstream,
                            stream_id,
                            "http3-request",
                            &bytes,
                            &request_limits,
                            "transport-failed",
                        );
                        emit_stream_observed(&request_sink, event);
                    }
                    sent.map_err(http3_stream_terminal)?;
                }
                emit_coalesced_stream(
                    &request_observer,
                    &request_sink,
                    QuicDirection::ClientToUpstream,
                    stream_id,
                    "http3-request",
                    &mut evidence,
                    &request_limits,
                    true,
                )
                .await;
                if let Some(trailers) = client_recv
                    .recv_trailers()
                    .await
                    .map_err(http3_stream_terminal)?
                {
                    emit_observed(
                        &request_sink,
                        trailer_event(&request_observer, stream_id, &trailers).await,
                    );
                    origin_send
                        .send_trailers(trailers)
                        .await
                        .map_err(http3_stream_terminal)?;
                }
                origin_send.finish().await.map_err(http3_stream_terminal)?;
                Ok::<Instant, crate::StreamTerminal>(Instant::now())
            };

            let response_observer = Arc::clone(&stream_observer);
            let response_sink = stream_sink.clone();
            let response_limits = stream_limits.clone();
            let response_forward = async move {
                let mut evidence = Vec::with_capacity(response_limits.max_event_chunk_bytes);
                let response = origin_recv
                    .recv_response()
                    .await
                    .map_err(http3_stream_terminal)?;
                let response_event = {
                    let mut observer = response_observer.lock().await;
                    observer.http3_metadata();
                    ApplicationEvent::now(
                        observer.plan.session_id(),
                        observer.plan.association_id(),
                        Some(stream_id),
                        Some(ProtocolVersion::Http3),
                        ApplicationEventKind::Metadata(response_metadata(&response)),
                    )
                };
                emit_observed(&response_sink, response_event);
                let response_head_at = Instant::now();
                client_send
                    .send_response(response)
                    .await
                    .map_err(http3_stream_terminal)?;
                loop {
                    let next = match origin_recv.recv_data().await {
                        Ok(value) => value,
                        Err(error) => {
                            emit_coalesced_stream(
                                &response_observer,
                                &response_sink,
                                QuicDirection::UpstreamToClient,
                                stream_id,
                                "http3-response",
                                &mut evidence,
                                &response_limits,
                                true,
                            )
                            .await;
                            return Err(http3_stream_terminal(error));
                        }
                    };
                    let Some(mut data) = next else {
                        break;
                    };
                    let bytes = copy_buf(&mut data);
                    let sent = client_send.send_data(bytes.clone()).await;
                    if sent.is_ok() {
                        evidence.extend_from_slice(&bytes);
                        emit_coalesced_stream(
                            &response_observer,
                            &response_sink,
                            QuicDirection::UpstreamToClient,
                            stream_id,
                            "http3-response",
                            &mut evidence,
                            &response_limits,
                            false,
                        )
                        .await;
                    } else {
                        emit_coalesced_stream(
                            &response_observer,
                            &response_sink,
                            QuicDirection::UpstreamToClient,
                            stream_id,
                            "http3-response",
                            &mut evidence,
                            &response_limits,
                            true,
                        )
                        .await;
                        let event = response_observer.lock().await.stream(
                            QuicDirection::UpstreamToClient,
                            stream_id,
                            "http3-response",
                            &bytes,
                            &response_limits,
                            "transport-failed",
                        );
                        emit_stream_observed(&response_sink, event);
                    }
                    sent.map_err(http3_stream_terminal)?;
                }
                emit_coalesced_stream(
                    &response_observer,
                    &response_sink,
                    QuicDirection::UpstreamToClient,
                    stream_id,
                    "http3-response",
                    &mut evidence,
                    &response_limits,
                    true,
                )
                .await;
                if let Some(trailers) = origin_recv
                    .recv_trailers()
                    .await
                    .map_err(http3_stream_terminal)?
                {
                    emit_observed(
                        &response_sink,
                        trailer_event(&response_observer, stream_id, &trailers).await,
                    );
                    client_send
                        .send_trailers(trailers)
                        .await
                        .map_err(http3_stream_terminal)?;
                }
                client_send.finish().await.map_err(http3_stream_terminal)?;
                Ok::<(Instant, Instant), crate::StreamTerminal>((response_head_at, Instant::now()))
            };

            let (request_sent_at, (response_head_at, completed_at)) =
                match tokio::try_join!(request_forward, response_forward) {
                    Ok(value) => value,
                    Err(terminal) => {
                        emit_http3_terminal(&stream_observer, &stream_sink, stream_id, terminal)
                            .await;
                        return Err(QuicRefusalCode::Http3ProtocolFailed);
                    }
                };
            stream_observer.lock().await.http3_complete();
            if response_head_at >= request_sent_at {
                emit_observed(
                    &stream_sink,
                    http3_event(
                        &stream_observer,
                        stream_id,
                        ApplicationEventKind::HttpTiming(HttpTiming {
                            send_ns: duration_ns(
                                request_sent_at.saturating_duration_since(started_at),
                            ),
                            wait_ns: duration_ns(
                                response_head_at.saturating_duration_since(request_sent_at),
                            ),
                            receive_ns: duration_ns(
                                completed_at.saturating_duration_since(response_head_at),
                            ),
                        }),
                    )
                    .await,
                );
            }
            emit_observed(
                &stream_sink,
                http3_event(
                    &stream_observer,
                    stream_id,
                    ApplicationEventKind::HttpStreamTerminal(crate::StreamTerminal::Complete),
                )
                .await,
            );
            Ok::<(), QuicRefusalCode>(())
        });
    }
    while let Some(result) = requests.join_next().await {
        result.map_err(|_| QuicRefusalCode::TransportFailed)??;
    }
    driver_task.abort();
    request_datagrams.abort();
    response_datagrams.abort();
    Ok(())
}

async fn http3_event(
    observer: &Arc<tokio::sync::Mutex<QuicEvidenceObserver>>,
    stream_id: u64,
    kind: ApplicationEventKind,
) -> ApplicationEvent {
    let observer = observer.lock().await;
    ApplicationEvent::now(
        observer.plan.session_id(),
        observer.plan.association_id(),
        Some(stream_id),
        Some(ProtocolVersion::Http3),
        kind,
    )
}

fn http3_stream_terminal(error: h3::error::StreamError) -> crate::StreamTerminal {
    match error {
        h3::error::StreamError::RemoteTerminate { .. } => crate::StreamTerminal::Reset,
        _ => crate::StreamTerminal::ProtocolError,
    }
}

async fn emit_http3_terminal(
    observer: &Arc<tokio::sync::Mutex<QuicEvidenceObserver>>,
    sink: &application::SharedEventSink,
    stream_id: u64,
    terminal: crate::StreamTerminal,
) {
    observer.lock().await.http3_terminal(terminal);
    emit_observed(
        sink,
        http3_event(
            observer,
            stream_id,
            ApplicationEventKind::HttpStreamTerminal(terminal),
        )
        .await,
    );
}

async fn trailer_event(
    observer: &Arc<tokio::sync::Mutex<QuicEvidenceObserver>>,
    stream_id: u64,
    trailers: &hyper::HeaderMap,
) -> ApplicationEvent {
    let mut observer = observer.lock().await;
    observer.http3_metadata();
    ApplicationEvent::now(
        observer.plan.session_id(),
        observer.plan.association_id(),
        Some(stream_id),
        Some(ProtocolVersion::Http3),
        ApplicationEventKind::Metadata(MetadataBlock::http3(
            MetadataKind::Trailers,
            Vec::new(),
            crate::metadata::fields_from_header_map(trailers),
        )),
    )
}

fn emit_stream_observed(sink: &application::SharedEventSink, event: ApplicationEvent) {
    let ApplicationEventKind::QuicStream(value) = &event.kind else {
        emit_observed(sink, event);
        return;
    };
    let direction = match value.direction {
        QuicDirection::ClientToUpstream => BodyDirection::Request,
        QuicDirection::UpstreamToClient => BodyDirection::Response,
    };
    let outcome = match value.outcome {
        GenericStreamOutcome::Complete => BodyOutcome::Complete,
        GenericStreamOutcome::IntentionallyOmitted => BodyOutcome::IntentionallyOmitted,
        GenericStreamOutcome::RetentionLimit => BodyOutcome::RetentionLimit,
    };
    let body = ApplicationEvent {
        kind: ApplicationEventKind::Body(BodySegment {
            direction,
            representation: BodyRepresentation::Raw,
            offset: value.offset,
            observed_len: value.observed_len,
            bytes: value.bytes.clone(),
            outcome,
        }),
        ..event.clone()
    };
    emit_observed(sink, event);
    emit_observed(sink, body);
}

#[expect(
    clippy::too_many_arguments,
    reason = "the coalescer preserves the complete QUIC evidence identity"
)]
async fn emit_coalesced_stream(
    observer: &Arc<tokio::sync::Mutex<QuicEvidenceObserver>>,
    sink: &application::SharedEventSink,
    direction: QuicDirection,
    stream_id: u64,
    kind: &'static str,
    pending: &mut Vec<u8>,
    limits: &ProtocolLimits,
    flush: bool,
) {
    let chunk_limit = limits.max_event_chunk_bytes;
    while pending.len() >= chunk_limit || (flush && !pending.is_empty()) {
        let length = pending.len().min(chunk_limit);
        let chunk = pending.drain(..length).collect::<Vec<_>>();
        let event =
            observer
                .lock()
                .await
                .stream(direction, stream_id, kind, &chunk, limits, "forwarded");
        emit_stream_observed(sink, event);
    }
}

fn duration_ns(value: std::time::Duration) -> u64 {
    value.as_nanos().try_into().unwrap_or(u64::MAX)
}

fn spawn_datagram_pump(
    source: quinn::Connection,
    destination: quinn::Connection,
    direction: QuicDirection,
    observer: Arc<tokio::sync::Mutex<QuicEvidenceObserver>>,
    limits: ProtocolLimits,
    sink: application::SharedEventSink,
    count: Arc<AtomicU64>,
) -> AbortOnDrop<()> {
    AbortOnDrop::new(tokio::spawn(async move {
        loop {
            let bytes = match source.read_datagram().await {
                Ok(value) => value,
                Err(_) => break,
            };
            let sequence = count.fetch_add(1, Ordering::Relaxed);
            if sequence >= limits.max_quic_datagrams as u64 {
                let event = observer
                    .lock()
                    .await
                    .datagram_capacity_drop(direction, sequence, &bytes);
                emit_observed(&sink, event);
                break;
            }
            let sent = destination.send_datagram_wait(bytes.clone()).await;
            let event = {
                let mut observer = observer.lock().await;
                if sent.is_err() {
                    observer.accounting.quic_transport_losses =
                        observer.accounting.quic_transport_losses.saturating_add(1);
                }
                observer.datagram(
                    direction,
                    &bytes,
                    &limits,
                    if sent.is_ok() {
                        "forwarded"
                    } else {
                        "transport-failed"
                    },
                )
            };
            emit_observed(&sink, event);
            if sent.is_err() {
                break;
            }
        }
    }))
}

fn copy_buf<B: Buf>(buffer: &mut B) -> Bytes {
    let mut bytes = Vec::with_capacity(buffer.remaining());
    while buffer.has_remaining() {
        let chunk = buffer.chunk();
        bytes.extend_from_slice(chunk);
        let length = chunk.len();
        buffer.advance(length);
    }
    Bytes::from(bytes)
}

fn request_metadata(request: &hyper::Request<()>) -> MetadataBlock {
    let values = [
        (b":method".as_slice(), request.method().as_str().as_bytes()),
        (
            b":scheme".as_slice(),
            request.uri().scheme_str().unwrap_or_default().as_bytes(),
        ),
        (
            b":authority".as_slice(),
            request
                .uri()
                .authority()
                .map_or("", |value| value.as_str())
                .as_bytes(),
        ),
        (
            b":path".as_slice(),
            request
                .uri()
                .path_and_query()
                .map_or("/", |value| value.as_str())
                .as_bytes(),
        ),
    ];
    let pseudo = values
        .into_iter()
        .enumerate()
        .map(|(index, (name, value))| MetadataField {
            name: name.to_vec(),
            value: value.to_vec(),
            original_index: index as u32,
            sensitive: false,
        })
        .collect();
    MetadataBlock::http3(
        MetadataKind::Request,
        pseudo,
        crate::metadata::fields_from_header_map(request.headers()),
    )
}

fn response_metadata(response: &hyper::Response<()>) -> MetadataBlock {
    MetadataBlock::http3(
        MetadataKind::Response,
        vec![MetadataField {
            name: b":status".to_vec(),
            value: response.status().as_str().as_bytes().to_vec(),
            original_index: 0,
            sensitive: false,
        }],
        crate::metadata::fields_from_header_map(response.headers()),
    )
}

fn emit_observed(sink: &application::SharedEventSink, event: ApplicationEvent) {
    let _ = match application::emit(sink, event) {
        EventDisposition::Accepted => true,
        EventDisposition::QueueFull | EventDisposition::Retired => false,
    };
}

impl QuicRefusalCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RouteUnscoped => "quic-route-unscoped",
            Self::OriginChanged => "quic-origin-changed",
            Self::IdentityUnavailable => "quic-identity-unavailable",
            Self::ClientTrustRejected => "quic-client-trust-rejected",
            Self::UpstreamValidationFailed => "quic-upstream-validation-failed",
            Self::ClientCertificateRequired => "quic-client-certificate-required",
            Self::PinningSuspected => "quic-pinning-suspected",
            Self::ZeroRttRefused => "quic-zero-rtt-refused",
            Self::MigrationRefused => "quic-migration-refused",
            Self::AlpnUnsupported => "quic-alpn-unsupported",
            Self::CapacityExhausted => "quic-capacity-exhausted",
            Self::TransportFailed => "quic-transport-failed",
            Self::Http3ProtocolFailed => "http3-protocol-failed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuicInspectionPlan {
    pair_id: u64,
    session_id: String,
    association_id: u64,
    client_endpoint: SocketAddr,
    origin_endpoint: SocketAddr,
    authority: DestinationAuthority,
    server_name: String,
}

impl QuicInspectionPlan {
    pub fn new(
        session_id: impl Into<String>,
        association_id: u64,
        client_endpoint: SocketAddr,
        origin_endpoint: SocketAddr,
        authority: &DestinationAuthority,
        authenticated: bool,
    ) -> Result<Self, QuicRefusalCode> {
        let session_id = session_id.into();
        if !authenticated || session_id.trim().is_empty() || association_id == 0 {
            return Err(QuicRefusalCode::RouteUnscoped);
        }
        let server_name = match authority.host() {
            crate::AuthorityHost::Dns(value) => value.clone(),
            crate::AuthorityHost::Ip(value) => value.to_string(),
        };
        Ok(Self {
            pair_id: NEXT_PAIR_ID.fetch_add(1, Ordering::Relaxed),
            session_id,
            association_id,
            client_endpoint,
            origin_endpoint,
            authority: authority.clone(),
            server_name: server_name.to_string(),
        })
    }

    pub fn pair_id(&self) -> u64 {
        self.pair_id
    }
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
    pub fn association_id(&self) -> u64 {
        self.association_id
    }
    pub fn client_endpoint(&self) -> SocketAddr {
        self.client_endpoint
    }
    pub fn origin_endpoint(&self) -> SocketAddr {
        self.origin_endpoint
    }
    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    pub fn admits(
        &self,
        client: SocketAddr,
        origin: SocketAddr,
        authority: &DestinationAuthority,
    ) -> Result<(), QuicRefusalCode> {
        if crate::upstream::canonical_address(client)
            != crate::upstream::canonical_address(self.client_endpoint)
        {
            return Err(QuicRefusalCode::MigrationRefused);
        }
        if crate::upstream::canonical_address(origin)
            != crate::upstream::canonical_address(self.origin_endpoint)
            || authority != &self.authority
        {
            return Err(QuicRefusalCode::OriginChanged);
        }
        Ok(())
    }
}

pub fn build_quic_server_config(
    authority: &DestinationAuthority,
    ca: &SessionCertificateAuthority,
    cache: &mut LeafCache,
    key_log: Option<Arc<SessionKeyLog>>,
    limits: &ProtocolLimits,
) -> Result<quinn::ServerConfig, QuicRefusalCode> {
    let identity = match authority.host() {
        crate::AuthorityHost::Dns(value) => CertificateIdentity::Dns(value.clone()),
        crate::AuthorityHost::Ip(value) => CertificateIdentity::Ip(*value),
    };
    let leaf = cache
        .certificate_for(ca, identity, SystemTime::now())
        .map_err(|_| QuicRefusalCode::IdentityUnavailable)?;
    let tls = crate::tls::client_server_config_with_alpn(
        Arc::clone(&leaf.certified_key),
        vec![b"h3".to_vec()],
        key_log,
    )
    .map_err(|_| QuicRefusalCode::IdentityUnavailable)?;
    let crypto =
        QuicServerConfig::try_from((*tls).clone()).map_err(|_| QuicRefusalCode::TransportFailed)?;
    let mut config = quinn::ServerConfig::with_crypto(Arc::new(crypto));
    config.migration(false);
    let mut transport = quinn::TransportConfig::default();
    transport.max_concurrent_bidi_streams((limits.max_concurrent_streams as u32).into());
    transport.max_concurrent_uni_streams((limits.max_concurrent_streams as u32).into());
    transport.datagram_receive_buffer_size(Some(limits.max_socks_udp_datagram_bytes));
    transport.datagram_send_buffer_size(limits.max_socks_udp_datagram_bytes);
    transport.max_idle_timeout(Some(
        limits
            .idle_timeout
            .try_into()
            .map_err(|_| QuicRefusalCode::TransportFailed)?,
    ));
    config.transport_config(Arc::new(transport));
    Ok(config)
}

pub fn build_quic_client_config(
    base: &Arc<rustls::ClientConfig>,
    limits: &ProtocolLimits,
) -> Result<quinn::ClientConfig, QuicRefusalCode> {
    let mut tls = (**base).clone();
    tls.alpn_protocols = vec![b"h3".to_vec()];
    tls.enable_early_data = false;
    let crypto =
        QuicClientConfig::try_from(tls).map_err(|_| QuicRefusalCode::UpstreamValidationFailed)?;
    let mut config = quinn::ClientConfig::new(Arc::new(crypto));
    let mut transport = quinn::TransportConfig::default();
    transport.max_concurrent_bidi_streams((limits.max_concurrent_streams as u32).into());
    transport.max_concurrent_uni_streams((limits.max_concurrent_streams as u32).into());
    transport.datagram_receive_buffer_size(Some(limits.max_socks_udp_datagram_bytes));
    transport.datagram_send_buffer_size(limits.max_socks_udp_datagram_bytes);
    transport.max_idle_timeout(Some(
        limits
            .idle_timeout
            .try_into()
            .map_err(|_| QuicRefusalCode::TransportFailed)?,
    ));
    config.transport_config(Arc::new(transport));
    Ok(config)
}

pub fn is_quic_initial(payload: &[u8]) -> bool {
    if payload.len() < 1200 || payload[0] & 0xc0 != 0xc0 {
        return false;
    }
    let Some(version) = payload.get(1..5) else {
        return false;
    };
    match version {
        [0, 0, 0, 1] => payload[0] & 0x30 == 0,
        [0x6b, 0x33, 0x43, 0xcf] => payload[0] & 0x30 == 0x10,
        _ => false,
    }
}

pub struct QuicEvidenceObserver {
    plan: QuicInspectionPlan,
    client_sequence: u64,
    upstream_sequence: u64,
    stream_offsets: BTreeMap<u64, [u64; 2]>,
    retained: u64,
    session_retained: Option<Arc<AtomicU64>>,
    seen_streams: BTreeSet<u64>,
    accounting: ProtocolAccounting,
}

impl QuicEvidenceObserver {
    pub fn new(plan: QuicInspectionPlan) -> Self {
        Self {
            plan,
            client_sequence: 0,
            upstream_sequence: 0,
            stream_offsets: BTreeMap::new(),
            retained: 0,
            session_retained: None,
            seen_streams: BTreeSet::new(),
            accounting: ProtocolAccounting::default(),
        }
    }

    pub(crate) fn with_session_retention(mut self, retained: Arc<AtomicU64>) -> Self {
        self.session_retained = Some(retained);
        self
    }

    fn http3_stream_open(&mut self, stream_id: u64) {
        if self.seen_streams.insert(stream_id) {
            self.accounting.quic_streams = self.accounting.quic_streams.saturating_add(1);
            self.accounting.http3_streams = self.accounting.http3_streams.saturating_add(1);
            self.accounting.metadata_blocks = self.accounting.metadata_blocks.saturating_add(1);
        }
    }

    fn http3_metadata(&mut self) {
        self.accounting.metadata_blocks = self.accounting.metadata_blocks.saturating_add(1);
    }

    fn http3_complete(&mut self) {
        self.accounting.http3_streams_completed =
            self.accounting.http3_streams_completed.saturating_add(1);
    }

    fn http3_terminal(&mut self, terminal: crate::StreamTerminal) {
        self.accounting.http3_streams_reset = self
            .accounting
            .http3_streams_reset
            .saturating_add(u64::from(terminal == crate::StreamTerminal::Reset));
    }

    pub fn connection(
        &self,
        half: QuicHalf,
        alpn: Option<Vec<u8>>,
        outcome: &'static str,
    ) -> ApplicationEvent {
        ApplicationEvent::now(
            self.plan.session_id(),
            self.plan.association_id(),
            None,
            None,
            ApplicationEventKind::QuicConnection(QuicConnectionEvent {
                pair_id: self.plan.pair_id(),
                half,
                peer: self.plan.client_endpoint(),
                origin: self.plan.origin_endpoint(),
                server_name: self.plan.server_name().to_string(),
                alpn,
                zero_rtt: "refused",
                migration: "disabled",
                outcome,
            }),
        )
    }

    pub fn refusal(&self, code: QuicRefusalCode) -> ApplicationEvent {
        ApplicationEvent::now(
            self.plan.session_id(),
            self.plan.association_id(),
            None,
            None,
            ApplicationEventKind::QuicRefusal(QuicRefusalEvent {
                pair_id: Some(self.plan.pair_id()),
                code: code.as_str(),
            }),
        )
    }

    fn record_refusal(&mut self, code: QuicRefusalCode) -> ApplicationEvent {
        self.accounting.quic_pairs_refused = self.accounting.quic_pairs_refused.saturating_add(1);
        self.accounting.quic_transport_losses = self
            .accounting
            .quic_transport_losses
            .saturating_add(u64::from(matches!(
                code,
                QuicRefusalCode::TransportFailed | QuicRefusalCode::Http3ProtocolFailed
            )));
        self.accounting.quic_zero_rtt_refused = self
            .accounting
            .quic_zero_rtt_refused
            .saturating_add(u64::from(code == QuicRefusalCode::ZeroRttRefused));
        self.accounting.quic_migration_refused = self
            .accounting
            .quic_migration_refused
            .saturating_add(u64::from(code == QuicRefusalCode::MigrationRefused));
        self.refusal(code)
    }

    pub fn stream(
        &mut self,
        direction: QuicDirection,
        stream_id: u64,
        kind: &'static str,
        payload: &[u8],
        limits: &ProtocolLimits,
        terminal: &'static str,
    ) -> ApplicationEvent {
        let sequence = self.next(direction);
        let offsets = self.stream_offsets.entry(stream_id).or_default();
        let offset = &mut offsets[match direction {
            QuicDirection::ClientToUpstream => 0,
            QuicDirection::UpstreamToClient => 1,
        }];
        let start_offset = *offset;
        *offset = offset.saturating_add(payload.len() as u64);
        let retained = self.claim(payload.len(), limits);
        self.accounting.quic_stream_bytes_observed = self
            .accounting
            .quic_stream_bytes_observed
            .saturating_add(payload.len() as u64);
        self.accounting.quic_stream_bytes_retained = self
            .accounting
            .quic_stream_bytes_retained
            .saturating_add(retained as u64);
        self.accounting.quic_stream_bytes_omitted = self
            .accounting
            .quic_stream_bytes_omitted
            .saturating_add(payload.len().saturating_sub(retained) as u64);
        ApplicationEvent::now(
            self.plan.session_id(),
            self.plan.association_id(),
            Some(stream_id),
            Some(ProtocolVersion::Http3),
            ApplicationEventKind::QuicStream(QuicStreamEvent {
                pair_id: self.plan.pair_id(),
                direction,
                stream_id,
                stream_kind: kind,
                sequence,
                offset: start_offset,
                observed_len: payload.len() as u64,
                bytes: Bytes::copy_from_slice(&payload[..retained]),
                outcome: stream_outcome(payload.len(), retained, limits.capture_payloads),
                terminal,
            }),
        )
    }

    pub fn datagram(
        &mut self,
        direction: QuicDirection,
        payload: &[u8],
        limits: &ProtocolLimits,
        terminal: &'static str,
    ) -> ApplicationEvent {
        let sequence = self.next(direction);
        let retained = self.claim(payload.len(), limits);
        self.accounting.quic_datagrams = self.accounting.quic_datagrams.saturating_add(1);
        self.accounting.quic_datagram_bytes_observed = self
            .accounting
            .quic_datagram_bytes_observed
            .saturating_add(payload.len() as u64);
        self.accounting.quic_datagram_bytes_retained = self
            .accounting
            .quic_datagram_bytes_retained
            .saturating_add(retained as u64);
        self.accounting.quic_datagram_bytes_omitted = self
            .accounting
            .quic_datagram_bytes_omitted
            .saturating_add(payload.len().saturating_sub(retained) as u64);
        ApplicationEvent::now(
            self.plan.session_id(),
            self.plan.association_id(),
            None,
            None,
            ApplicationEventKind::QuicDatagram(QuicDatagramEvent {
                pair_id: self.plan.pair_id(),
                direction,
                sequence,
                observed_len: payload.len() as u64,
                bytes: Bytes::copy_from_slice(&payload[..retained]),
                outcome: datagram_outcome(payload.len(), retained, limits.capture_payloads),
                terminal,
            }),
        )
    }

    fn datagram_capacity_drop(
        &mut self,
        direction: QuicDirection,
        sequence: u64,
        payload: &[u8],
    ) -> ApplicationEvent {
        self.accounting.quic_datagrams = self.accounting.quic_datagrams.saturating_add(1);
        self.accounting.quic_datagram_bytes_observed = self
            .accounting
            .quic_datagram_bytes_observed
            .saturating_add(payload.len() as u64);
        self.accounting.quic_datagram_bytes_omitted = self
            .accounting
            .quic_datagram_bytes_omitted
            .saturating_add(payload.len() as u64);
        self.accounting.quic_datagrams_capacity_dropped = self
            .accounting
            .quic_datagrams_capacity_dropped
            .saturating_add(1);
        ApplicationEvent::now(
            self.plan.session_id(),
            self.plan.association_id(),
            None,
            None,
            ApplicationEventKind::QuicDatagram(QuicDatagramEvent {
                pair_id: self.plan.pair_id(),
                direction,
                sequence,
                observed_len: payload.len() as u64,
                bytes: Bytes::new(),
                outcome: GenericUdpOutcome::RetentionLimit,
                terminal: "capacity-dropped",
            }),
        )
    }

    fn next(&mut self, direction: QuicDirection) -> u64 {
        let value = match direction {
            QuicDirection::ClientToUpstream => &mut self.client_sequence,
            QuicDirection::UpstreamToClient => &mut self.upstream_sequence,
        };
        let result = *value;
        *value = value.saturating_add(1);
        result
    }

    fn claim(&mut self, length: usize, limits: &ProtocolLimits) -> usize {
        if !limits.capture_payloads {
            return 0;
        }
        let remaining = limits.max_body_bytes.saturating_sub(self.retained) as usize;
        let requested = remaining.min(length);
        let claim = self.session_retained.as_ref().map_or(requested, |counter| {
            crate::body::claim_retention(counter, limits.max_session_body_bytes, requested)
        });
        self.retained = self.retained.saturating_add(claim as u64);
        claim
    }
}

fn stream_outcome(length: usize, retained: usize, enabled: bool) -> GenericStreamOutcome {
    if !enabled {
        GenericStreamOutcome::IntentionallyOmitted
    } else if retained < length {
        GenericStreamOutcome::RetentionLimit
    } else {
        GenericStreamOutcome::Complete
    }
}

fn datagram_outcome(length: usize, retained: usize, enabled: bool) -> GenericUdpOutcome {
    if !enabled {
        GenericUdpOutcome::IntentionallyOmitted
    } else if retained < length {
        GenericUdpOutcome::RetentionLimit
    } else {
        GenericUdpOutcome::Complete
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ApplicationEventSink, EventDisposition};
    use std::sync::Mutex;

    #[derive(Default)]
    struct Sink(Mutex<Vec<ApplicationEvent>>);

    impl ApplicationEventSink for Sink {
        fn try_emit(&self, event: ApplicationEvent) -> EventDisposition {
            self.0.lock().unwrap().push(event);
            EventDisposition::Accepted
        }
    }

    fn authority() -> DestinationAuthority {
        DestinationAuthority::parse("localhost:443").unwrap()
    }
    fn address(port: u16) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], port))
    }

    #[tokio::test]
    async fn coalesced_stream_evidence_is_bounded_and_exact() {
        let plan =
            QuicInspectionPlan::new("s", 1, address(1), address(2), &authority(), true).unwrap();
        let observer = Arc::new(tokio::sync::Mutex::new(QuicEvidenceObserver::new(plan)));
        let sink = Arc::new(Sink::default());
        let shared: application::SharedEventSink = Some(sink.clone());
        let limits = ProtocolLimits {
            max_event_chunk_bytes: 3,
            ..ProtocolLimits::default()
        };
        let mut pending = b"abcdefghij".to_vec();
        emit_coalesced_stream(
            &observer,
            &shared,
            QuicDirection::ClientToUpstream,
            4,
            "http3-request",
            &mut pending,
            &limits,
            true,
        )
        .await;
        assert!(pending.is_empty());
        let events = sink.0.lock().unwrap();
        let streams: Vec<_> = events
            .iter()
            .filter_map(|event| match &event.kind {
                ApplicationEventKind::QuicStream(value) => Some(value),
                _ => None,
            })
            .collect();
        assert_eq!(streams.len(), 4);
        assert_eq!(
            streams.iter().map(|value| value.offset).collect::<Vec<_>>(),
            vec![0, 3, 6, 9]
        );
        assert_eq!(
            streams
                .iter()
                .map(|value| value.sequence)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert!(streams.iter().all(|value| {
            value.observed_len <= 3 && value.bytes.len() <= 3 && value.terminal == "forwarded"
        }));
        assert_eq!(
            streams
                .iter()
                .flat_map(|value| value.bytes.iter().copied())
                .collect::<Vec<_>>(),
            b"abcdefghij"
        );
    }

    #[test]
    fn admission_is_authenticated_and_immutable() {
        assert_eq!(
            QuicInspectionPlan::new("s", 1, address(1), address(2), &authority(), false),
            Err(QuicRefusalCode::RouteUnscoped)
        );
        let plan =
            QuicInspectionPlan::new("s", 1, address(1), address(2), &authority(), true).unwrap();
        assert_eq!(
            plan.admits(address(3), address(2), &authority()),
            Err(QuicRefusalCode::MigrationRefused)
        );
        assert_eq!(
            plan.admits(address(1), address(4), &authority()),
            Err(QuicRefusalCode::OriginChanged)
        );
    }

    #[test]
    fn evidence_retention_is_bounded_and_directional() {
        let plan =
            QuicInspectionPlan::new("s", 1, address(1), address(2), &authority(), true).unwrap();
        let mut observer = QuicEvidenceObserver::new(plan);
        let limits = ProtocolLimits {
            max_body_bytes: 3,
            ..ProtocolLimits::default()
        };
        let first = observer.stream(
            QuicDirection::ClientToUpstream,
            0,
            "bidirectional",
            b"abcd",
            &limits,
            "finished",
        );
        let second =
            observer.datagram(QuicDirection::UpstreamToClient, b"ef", &limits, "forwarded");
        let ApplicationEventKind::QuicStream(first) = first.kind else {
            panic!()
        };
        let ApplicationEventKind::QuicDatagram(second) = second.kind else {
            panic!()
        };
        assert_eq!(first.sequence, 0);
        assert_eq!(first.bytes.as_ref(), b"abc");
        assert_eq!(first.outcome, GenericStreamOutcome::RetentionLimit);
        assert_eq!(second.sequence, 0);
        assert!(second.bytes.is_empty());
        assert_eq!(second.outcome, GenericUdpOutcome::RetentionLimit);
    }

    #[test]
    fn stream_offsets_are_independent_per_stream_and_direction() {
        let plan =
            QuicInspectionPlan::new("s", 1, address(1), address(2), &authority(), true).unwrap();
        let mut observer = QuicEvidenceObserver::new(plan);
        let limits = ProtocolLimits::default();
        let first = observer.stream(
            QuicDirection::ClientToUpstream,
            0,
            "http3-request",
            b"first",
            &limits,
            "forwarded",
        );
        let second = observer.stream(
            QuicDirection::ClientToUpstream,
            4,
            "http3-request",
            b"second",
            &limits,
            "forwarded",
        );
        let response = observer.stream(
            QuicDirection::UpstreamToClient,
            0,
            "http3-response",
            b"response",
            &limits,
            "forwarded",
        );
        let ApplicationEventKind::QuicStream(first) = first.kind else {
            panic!()
        };
        let ApplicationEventKind::QuicStream(second) = second.kind else {
            panic!()
        };
        let ApplicationEventKind::QuicStream(response) = response.kind else {
            panic!()
        };
        assert_eq!((first.offset, second.offset, response.offset), (0, 0, 0));
    }

    #[test]
    fn datagram_capacity_drop_is_explicit_and_accounted() {
        let plan =
            QuicInspectionPlan::new("s", 1, address(1), address(2), &authority(), true).unwrap();
        let mut observer = QuicEvidenceObserver::new(plan);
        let event =
            observer.datagram_capacity_drop(QuicDirection::ClientToUpstream, 7, b"discarded");
        let ApplicationEventKind::QuicDatagram(event) = event.kind else {
            panic!()
        };
        assert_eq!(event.sequence, 7);
        assert_eq!(event.observed_len, 9);
        assert!(event.bytes.is_empty());
        assert_eq!(event.terminal, "capacity-dropped");
        assert_eq!(observer.accounting.quic_datagrams_capacity_dropped, 1);
        assert_eq!(observer.accounting.quic_datagram_bytes_omitted, 9);
    }

    #[test]
    fn reset_terminal_advances_http3_reset_accounting() {
        let plan =
            QuicInspectionPlan::new("s", 1, address(1), address(2), &authority(), true).unwrap();
        let mut observer = QuicEvidenceObserver::new(plan);
        observer.http3_terminal(crate::StreamTerminal::Reset);
        observer.http3_terminal(crate::StreamTerminal::TransportError);
        assert_eq!(observer.accounting.http3_streams_reset, 1);
    }

    #[test]
    fn quic_initial_detection_is_conservative() {
        let mut packet = vec![0; 1200];
        packet[0] = 0xc0;
        packet[4] = 1;
        assert!(is_quic_initial(&packet));
        assert!(!is_quic_initial(&packet[..1199]));
        packet[0] = 0xd0;
        packet[1..5].copy_from_slice(&[0x6b, 0x33, 0x43, 0xcf]);
        assert!(is_quic_initial(&packet));
        packet[1..5].fill(0);
        assert!(!is_quic_initial(&packet));
    }
}
