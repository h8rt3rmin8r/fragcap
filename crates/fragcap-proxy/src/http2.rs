// SPDX-License-Identifier: Apache-2.0

//! Bounded HTTP/2 bridge.

use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use bytes::Bytes;
use h2::client::SendRequest;
use h2::{RecvStream, SendStream};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::timeout;

use crate::application::{emit, ApplicationEvent, ApplicationEventKind, SharedEventSink};
use crate::metadata::{fields_from_header_map, MetadataBlock, MetadataField, MetadataKind};
use crate::{
    BodyDirection, BodyOutcome, BodyRepresentation, BodySegment, DestinationAuthority,
    DestinationPolicy, ProtocolAccounting, ProtocolError, ProtocolLimits, ProtocolVersion,
    SessionCapability, StreamTerminal,
};

#[derive(Debug)]
pub(crate) struct Http2Run {
    pub accounting: ProtocolAccounting,
    pub failure: Option<ProtocolError>,
}

pub(crate) struct Http2ConnectionContext {
    pub limits: ProtocolLimits,
    pub session_id: String,
    pub connection_id: u64,
    pub sink: SharedEventSink,
    pub body_resources: crate::body::SessionBodyResources,
}

struct AbortOnDrop<T>(tokio::task::JoinHandle<T>);

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[derive(Clone)]
struct StreamContext {
    limits: ProtocolLimits,
    session_id: String,
    connection_id: u64,
    stream_id: u64,
    sink: SharedEventSink,
    session_retained: Arc<AtomicU64>,
    decoder_slots: Arc<Semaphore>,
}

struct BodyPumpContext {
    budget: std::time::Duration,
    sink: SharedEventSink,
    session_id: String,
    connection_id: u64,
    stream_id: u64,
    direction: BodyDirection,
    retention_limit: u64,
    content_encoding: Option<String>,
    max_decoded_body_bytes: usize,
    max_decode_ratio: usize,
    decode_timeout: std::time::Duration,
    capture_payloads: bool,
    session_retained: Arc<AtomicU64>,
    session_retention_limit: u64,
    decoder_slots: Arc<Semaphore>,
    streaming: Option<ProtocolObserver>,
}

#[derive(Clone, Copy)]
struct WebSocketMode {
    compression: bool,
    client_no_context_takeover: bool,
    server_no_context_takeover: bool,
}

enum ProtocolObserver {
    WebSocket(crate::WebSocketObserver),
    Sse(crate::SseObserver),
    Grpc(crate::GrpcObserver),
}

impl ProtocolObserver {
    fn feed(&mut self, bytes: &[u8]) -> Vec<crate::StreamingEvent> {
        match self {
            Self::WebSocket(value) => value.feed(bytes),
            Self::Sse(value) => value.feed(bytes),
            Self::Grpc(value) => value.feed(bytes),
        }
    }

    fn finish(&mut self, trailers: Option<&hyper::HeaderMap>) -> Vec<crate::StreamingEvent> {
        self.finish_with_outcome(trailers, crate::StreamingOutcome::Complete)
    }

    fn finish_with_outcome(
        &mut self,
        trailers: Option<&hyper::HeaderMap>,
        outcome: crate::StreamingOutcome,
    ) -> Vec<crate::StreamingEvent> {
        match self {
            Self::WebSocket(value) => vec![value.finish(outcome)],
            Self::Sse(value) => value.finish(outcome),
            Self::Grpc(value) => vec![value.finish(
                trailers
                    .and_then(|map| map.get("grpc-status"))
                    .map(|value| value.as_bytes().to_vec()),
                trailers
                    .and_then(|map| map.get("grpc-message"))
                    .map(|value| value.as_bytes().to_vec()),
                trailers
                    .and_then(|map| map.get("grpc-status-details-bin"))
                    .map(|value| value.as_bytes().to_vec()),
                outcome,
            )],
        }
    }
}

struct StreamingPumpObserver {
    observer: Option<ProtocolObserver>,
    sink: SharedEventSink,
    session_id: String,
    connection_id: u64,
    stream_id: u64,
    capture_payloads: bool,
}

impl StreamingPumpObserver {
    fn feed(&mut self, bytes: &[u8]) {
        let Some(observer) = &mut self.observer else {
            return;
        };
        let events = observer.feed(bytes);
        let terminal = events.iter().any(|event| {
            matches!(
                event,
                crate::StreamingEvent::WebSocketTerminal { .. }
                    | crate::StreamingEvent::SseTerminal { .. }
                    | crate::StreamingEvent::GrpcTerminal { .. }
            )
        });
        for event in events {
            emit_streaming(
                &self.sink,
                &self.session_id,
                self.connection_id,
                self.stream_id,
                event,
                self.capture_payloads,
            );
        }
        if terminal {
            self.observer = None;
        }
    }

    fn finish(&mut self, trailers: Option<&hyper::HeaderMap>) {
        let Some(mut observer) = self.observer.take() else {
            return;
        };
        for event in observer.finish(trailers) {
            emit_streaming(
                &self.sink,
                &self.session_id,
                self.connection_id,
                self.stream_id,
                event,
                self.capture_payloads,
            );
        }
    }
}

impl Drop for StreamingPumpObserver {
    fn drop(&mut self) {
        let Some(mut observer) = self.observer.take() else {
            return;
        };
        for event in observer.finish_with_outcome(None, crate::StreamingOutcome::Cancelled) {
            emit_streaming(
                &self.sink,
                &self.session_id,
                self.connection_id,
                self.stream_id,
                event,
                self.capture_payloads,
            );
        }
    }
}

pub(crate) async fn serve_http2<C, U>(
    client: C,
    upstream: U,
    authority: DestinationAuthority,
    context: Http2ConnectionContext,
) -> Http2Run
where
    C: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    U: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let Http2ConnectionContext {
        limits,
        session_id,
        connection_id,
        sink,
        body_resources,
    } = context;
    let session_retained = body_resources.retained;
    let decoder_slots = body_resources.decoder_slots;
    let mut server_builder = h2::server::Builder::new();
    server_builder
        .enable_connect_protocol()
        .max_concurrent_streams(limits.max_concurrent_streams as u32)
        .max_header_list_size(limits.max_header_bytes as u32)
        .initial_window_size(limits.http2_stream_window_bytes as u32)
        .initial_connection_window_size(limits.http2_connection_window_bytes as u32)
        .max_send_buffer_size(limits.http2_send_buffer_bytes)
        .max_concurrent_reset_streams(limits.max_reset_streams)
        .max_local_error_reset_streams(Some(limits.max_reset_streams))
        .max_pending_accept_reset_streams(limits.max_pending_reset_streams);
    let mut client_connection =
        match timeout(limits.header_timeout, server_builder.handshake(client)).await {
            Ok(Ok(connection)) => connection,
            Ok(Err(error)) => return failed("http2-client-handshake-failed", error.to_string()),
            Err(_) => {
                return failed(
                    "http2-client-handshake-timeout",
                    "HTTP/2 client handshake timed out",
                )
            }
        };

    let mut origin_builder = h2::client::Builder::new();
    origin_builder
        .initial_window_size(limits.http2_stream_window_bytes as u32)
        .initial_connection_window_size(limits.http2_connection_window_bytes as u32)
        .max_header_list_size(limits.max_header_bytes as u32)
        .max_send_buffer_size(limits.http2_send_buffer_bytes)
        .max_concurrent_streams(limits.max_concurrent_streams as u32)
        .max_concurrent_reset_streams(limits.max_reset_streams)
        .max_local_error_reset_streams(Some(limits.max_reset_streams))
        .max_pending_accept_reset_streams(limits.max_pending_reset_streams)
        .enable_push(false);
    let (origin_sender, origin_connection) =
        match timeout(limits.header_timeout, origin_builder.handshake(upstream)).await {
            Ok(Ok(value)) => value,
            Ok(Err(error)) => return failed("http2-origin-handshake-failed", error.to_string()),
            Err(_) => {
                return failed(
                    "http2-origin-handshake-timeout",
                    "HTTP/2 origin handshake timed out",
                )
            }
        };
    let origin_driver = AbortOnDrop(tokio::spawn(origin_connection));
    let mut tasks = JoinSet::new();
    let mut accounting = ProtocolAccounting::default();
    loop {
        let accepted = if tasks.is_empty() {
            match timeout(limits.idle_timeout, client_connection.accept()).await {
                Ok(value) => value,
                Err(_) => {
                    return Http2Run {
                        accounting,
                        failure: Some(ProtocolError::timeout("http2-connection-idle-timeout")),
                    }
                }
            }
        } else {
            tokio::select! {
                biased;
                joined = tasks.join_next() => {
                    if let Some(joined) = joined {
                        account_stream_result(&mut accounting, joined);
                    }
                    continue;
                }
                accepted = client_connection.accept() => accepted,
            }
        };
        let Some(accepted) = accepted else { break };
        let (request, mut respond) = match accepted {
            Ok(value) => value,
            Err(error) => {
                return Http2Run {
                    accounting,
                    failure: Some(ProtocolError::new("http2-accept-failed", error.to_string())),
                }
            }
        };
        let stream_id = request.body().stream_id().as_u32() as u64;
        if let Err(error) = validate_tunnel_authority(&request, &authority) {
            respond.send_reset(h2::Reason::REFUSED_STREAM);
            accounting.parse_refused = accounting.parse_refused.saturating_add(1);
            emit(
                &sink,
                ApplicationEvent::now(
                    &session_id,
                    connection_id,
                    Some(stream_id),
                    Some(ProtocolVersion::Http2),
                    ApplicationEventKind::Error { code: error.code },
                ),
            );
            emit(
                &sink,
                ApplicationEvent::now(
                    &session_id,
                    connection_id,
                    Some(stream_id),
                    Some(ProtocolVersion::Http2),
                    ApplicationEventKind::HttpStreamTerminal(StreamTerminal::Refused),
                ),
            );
            continue;
        }
        accounting.requests = accounting.requests.saturating_add(1);
        accounting.http2_streams = accounting.http2_streams.saturating_add(1);
        let sender = origin_sender.clone();
        let task_limits = limits.clone();
        let task_sink = sink.clone();
        let task_session = session_id.clone();
        let task_session_retained = Arc::clone(&session_retained);
        let task_decoder_slots = Arc::clone(&decoder_slots);
        tasks.spawn(async move {
            bridge_stream(
                request,
                respond,
                sender,
                StreamContext {
                    limits: task_limits,
                    session_id: task_session,
                    connection_id,
                    stream_id,
                    sink: task_sink,
                    session_retained: task_session_retained,
                    decoder_slots: task_decoder_slots,
                },
            )
            .await
        });
        while let Some(joined) = tasks.try_join_next() {
            account_stream_result(&mut accounting, joined);
        }
    }
    while let Some(joined) = tasks.join_next().await {
        account_stream_result(&mut accounting, joined);
    }
    origin_driver.0.abort();
    Http2Run {
        accounting,
        failure: None,
    }
}

pub(crate) async fn serve_cleartext_http2<C>(
    client: C,
    capability: SessionCapability,
    policy: DestinationPolicy,
    context: Http2ConnectionContext,
) -> Http2Run
where
    C: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let Http2ConnectionContext {
        limits,
        session_id,
        connection_id,
        sink,
        body_resources,
    } = context;
    let session_retained = body_resources.retained;
    let decoder_slots = body_resources.decoder_slots;
    let mut server_builder = h2::server::Builder::new();
    configure_server(&mut server_builder, &limits);
    let mut client_connection =
        match timeout(limits.header_timeout, server_builder.handshake(client)).await {
            Ok(Ok(connection)) => connection,
            Ok(Err(error)) => return failed("http2-client-handshake-failed", error.to_string()),
            Err(_) => {
                return failed(
                    "http2-client-handshake-timeout",
                    "HTTP/2 client handshake timed out",
                )
            }
        };
    let (first, first_response) =
        match timeout(limits.header_timeout, client_connection.accept()).await {
            Ok(Some(Ok(value))) => value,
            Ok(Some(Err(error))) => return failed("http2-accept-failed", error.to_string()),
            Ok(None) => {
                return failed(
                    "proxy-auth-required",
                    "HTTP/2 client closed before authentication",
                )
            }
            Err(_) => return failed("http2-auth-timeout", "HTTP/2 authentication timed out"),
        };
    let (first, authority) = match authenticate_cleartext_request(first, &capability, None) {
        Ok(value) => value,
        Err(error) => {
            return Http2Run {
                accounting: ProtocolAccounting::default(),
                failure: Some(error),
            }
        }
    };
    let upstream = match crate::connect_upstream(&authority, &policy, limits.upstream).await {
        Ok(upstream) => upstream,
        Err(error) => {
            let mut failure = ProtocolError::new(error.code, error.detail);
            failure.policy_refused = matches!(error.stage, crate::UpstreamStage::Policy);
            return Http2Run {
                accounting: ProtocolAccounting::default(),
                failure: Some(failure),
            };
        }
    };
    let mut origin_builder = h2::client::Builder::new();
    configure_client(&mut origin_builder, &limits);
    let (origin_sender, origin_connection) =
        match timeout(limits.header_timeout, origin_builder.handshake(upstream)).await {
            Ok(Ok(value)) => value,
            Ok(Err(error)) => return failed("http2-origin-handshake-failed", error.to_string()),
            Err(_) => {
                return failed(
                    "http2-origin-handshake-timeout",
                    "HTTP/2 origin handshake timed out",
                )
            }
        };
    emit(
        &sink,
        ApplicationEvent::now(
            &session_id,
            connection_id,
            None,
            Some(ProtocolVersion::Http2),
            ApplicationEventKind::ConnectionOpen,
        ),
    );
    let origin_driver = AbortOnDrop(tokio::spawn(origin_connection));
    let mut tasks = JoinSet::new();
    let mut accounting = ProtocolAccounting::default();
    let mut accepted = Some((first, first_response));
    loop {
        let next = match accepted.take() {
            Some(value) => Some(Ok(value)),
            None if tasks.is_empty() => {
                match timeout(limits.idle_timeout, client_connection.accept()).await {
                    Ok(value) => value,
                    Err(_) => {
                        return Http2Run {
                            accounting,
                            failure: Some(ProtocolError::timeout("http2-connection-idle-timeout")),
                        };
                    }
                }
            }
            None => {
                tokio::select! {
                    biased;
                    joined = tasks.join_next() => {
                        if let Some(joined) = joined {
                            account_stream_result(&mut accounting, joined);
                        }
                        continue;
                    }
                    accepted = client_connection.accept() => accepted,
                }
            }
        };
        let Some(next) = next else { break };
        let (request, mut respond) = match next {
            Ok(value) => value,
            Err(error) => {
                return Http2Run {
                    accounting,
                    failure: Some(h2_error("http2-accept-failed", error)),
                }
            }
        };
        let (request, _) =
            match authenticate_cleartext_request(request, &capability, Some(&authority)) {
                Ok(value) => value,
                Err(error) => {
                    respond.send_reset(h2::Reason::REFUSED_STREAM);
                    accounting.parse_refused = accounting.parse_refused.saturating_add(1);
                    emit(
                        &sink,
                        ApplicationEvent::now(
                            &session_id,
                            connection_id,
                            None,
                            Some(ProtocolVersion::Http2),
                            ApplicationEventKind::Error { code: error.code },
                        ),
                    );
                    continue;
                }
            };
        accounting.requests = accounting.requests.saturating_add(1);
        accounting.http2_streams = accounting.http2_streams.saturating_add(1);
        let stream_id = request.body().stream_id().as_u32() as u64;
        let context = StreamContext {
            limits: limits.clone(),
            session_id: session_id.clone(),
            connection_id,
            stream_id,
            sink: sink.clone(),
            session_retained: Arc::clone(&session_retained),
            decoder_slots: Arc::clone(&decoder_slots),
        };
        let sender = origin_sender.clone();
        tasks.spawn(async move { bridge_stream(request, respond, sender, context).await });
        while let Some(joined) = tasks.try_join_next() {
            account_stream_result(&mut accounting, joined);
        }
    }
    while let Some(joined) = tasks.join_next().await {
        account_stream_result(&mut accounting, joined);
    }
    origin_driver.0.abort();
    Http2Run {
        accounting,
        failure: None,
    }
}

fn validate_tunnel_authority(
    request: &hyper::Request<RecvStream>,
    expected: &DestinationAuthority,
) -> Result<(), ProtocolError> {
    let raw = request.uri().authority().ok_or_else(|| {
        ProtocolError::new(
            "http2-authority-required",
            "tunneled HTTP/2 request has no authority",
        )
    })?;
    let observed = DestinationAuthority::parse(raw.as_str())
        .map_err(|error| ProtocolError::new(error.code, error.detail))?;
    if &observed != expected {
        return Err(ProtocolError::new(
            "http2-tunnel-authority-mismatch",
            "HTTP/2 request authority differs from CONNECT authority",
        ));
    }
    Ok(())
}

fn authenticate_cleartext_request(
    mut request: hyper::Request<RecvStream>,
    capability: &SessionCapability,
    expected: Option<&DestinationAuthority>,
) -> Result<(hyper::Request<RecvStream>, DestinationAuthority), ProtocolError> {
    if expected.is_none() {
        let values: Vec<_> = request
            .headers()
            .get_all(hyper::header::PROXY_AUTHORIZATION)
            .iter()
            .map(|value| value.as_bytes())
            .collect();
        let value = match values.as_slice() {
            [] => None,
            [value] => Some(*value),
            _ => {
                return Err(ProtocolError::authentication(
                    crate::ProxyAuthorizationError::Duplicate,
                ))
            }
        };
        capability
            .authenticates_proxy_authorization(value)
            .map_err(ProtocolError::authentication)?;
    }
    request
        .headers_mut()
        .remove(hyper::header::PROXY_AUTHORIZATION);
    if request.uri().scheme_str() != Some("http") {
        return Err(ProtocolError::new(
            "http2-cleartext-scheme-refused",
            "cleartext HTTP/2 requires an http URI",
        ));
    }
    let raw = request.uri().authority().ok_or_else(|| {
        ProtocolError::new(
            "http2-authority-required",
            "HTTP/2 request has no authority",
        )
    })?;
    let authority = DestinationAuthority::parse(raw.as_str())
        .map_err(|error| ProtocolError::new(error.code, error.detail))?;
    if expected.is_some_and(|value| value != &authority) {
        return Err(ProtocolError::new(
            "http2-authority-mismatch",
            "HTTP/2 proxy connection is bound to one authority",
        ));
    }
    Ok((request, authority))
}

fn configure_server(builder: &mut h2::server::Builder, limits: &ProtocolLimits) {
    builder
        .enable_connect_protocol()
        .max_concurrent_streams(limits.max_concurrent_streams as u32)
        .max_header_list_size(limits.max_header_bytes as u32)
        .initial_window_size(limits.http2_stream_window_bytes as u32)
        .initial_connection_window_size(limits.http2_connection_window_bytes as u32)
        .max_send_buffer_size(limits.http2_send_buffer_bytes)
        .max_concurrent_reset_streams(limits.max_reset_streams)
        .max_local_error_reset_streams(Some(limits.max_reset_streams))
        .max_pending_accept_reset_streams(limits.max_pending_reset_streams);
}

fn configure_client(builder: &mut h2::client::Builder, limits: &ProtocolLimits) {
    builder
        .initial_window_size(limits.http2_stream_window_bytes as u32)
        .initial_connection_window_size(limits.http2_connection_window_bytes as u32)
        .max_header_list_size(limits.max_header_bytes as u32)
        .max_send_buffer_size(limits.http2_send_buffer_bytes)
        .max_concurrent_streams(limits.max_concurrent_streams as u32)
        .max_concurrent_reset_streams(limits.max_reset_streams)
        .max_local_error_reset_streams(Some(limits.max_reset_streams))
        .max_pending_accept_reset_streams(limits.max_pending_reset_streams)
        .enable_push(false);
}

async fn bridge_stream(
    request: hyper::Request<RecvStream>,
    respond: h2::server::SendResponse<Bytes>,
    origin: SendRequest<Bytes>,
    context: StreamContext,
) -> Result<(), ProtocolError> {
    let result = bridge_stream_inner(request, respond, origin, context.clone()).await;
    let terminal = match &result {
        Ok(()) => StreamTerminal::Complete,
        Err(error) if error.code.contains("timeout") => StreamTerminal::IdleTimeout,
        Err(error) if error.code.contains("reset") => StreamTerminal::Reset,
        Err(_) => StreamTerminal::ProtocolError,
    };
    emit(
        &context.sink,
        ApplicationEvent::now(
            &context.session_id,
            context.connection_id,
            Some(context.stream_id),
            Some(ProtocolVersion::Http2),
            ApplicationEventKind::HttpStreamTerminal(terminal),
        ),
    );
    result
}

fn request_pump_context(
    limits: &ProtocolLimits,
    context: &StreamContext,
    content_encoding: Option<String>,
    streaming: Option<ProtocolObserver>,
) -> BodyPumpContext {
    BodyPumpContext {
        budget: limits.idle_timeout,
        sink: context.sink.clone(),
        session_id: context.session_id.clone(),
        connection_id: context.connection_id,
        stream_id: context.stream_id,
        direction: BodyDirection::Request,
        retention_limit: limits.max_body_bytes,
        content_encoding,
        max_decoded_body_bytes: limits.max_decoded_body_bytes,
        max_decode_ratio: limits.max_decode_ratio,
        decode_timeout: limits.decode_timeout,
        capture_payloads: limits.capture_payloads,
        session_retained: Arc::clone(&context.session_retained),
        session_retention_limit: limits.max_session_body_bytes,
        decoder_slots: Arc::clone(&context.decoder_slots),
        streaming,
    }
}

async fn bridge_stream_inner(
    request: hyper::Request<RecvStream>,
    mut respond: h2::server::SendResponse<Bytes>,
    origin: SendRequest<Bytes>,
    context: StreamContext,
) -> Result<(), ProtocolError> {
    let limits = &context.limits;
    let session_id = &context.session_id;
    let connection_id = context.connection_id;
    let stream_id = context.stream_id;
    let sink = &context.sink;
    emit(
        sink,
        ApplicationEvent::now(
            session_id.as_str(),
            connection_id,
            Some(stream_id),
            Some(ProtocolVersion::Http2),
            ApplicationEventKind::HttpStreamOpen,
        ),
    );
    let websocket = request.method() == hyper::Method::CONNECT
        && request
            .extensions()
            .get::<h2::ext::Protocol>()
            .is_some_and(|value| value.as_str().eq_ignore_ascii_case("websocket"));
    let (parts, request_body) = request.into_parts();
    let pseudo = request_pseudo_fields(&parts);
    let request_encoding = content_encoding(&parts.headers);
    let websocket_offer = websocket.then(|| {
        crate::streaming::permessage_deflate(
            parts
                .headers
                .get_all("sec-websocket-extensions")
                .iter()
                .map(|value| value.as_bytes()),
        )
    });
    let grpc = grpc_content_type(&parts.headers);
    let request_method = parts.uri.path().as_bytes().to_vec();
    let grpc_encoding = parts
        .headers
        .get("grpc-encoding")
        .map(|value| value.as_bytes().to_vec());
    emit(
        sink,
        ApplicationEvent::now(
            session_id.as_str(),
            connection_id,
            Some(stream_id),
            Some(ProtocolVersion::Http2),
            ApplicationEventKind::Metadata(MetadataBlock::http2(
                MetadataKind::Request,
                pseudo,
                fields_from_header_map(&parts.headers),
            )),
        ),
    );
    if let Some(content_type) = &grpc {
        emit(
            sink,
            ApplicationEvent::now(
                session_id.as_str(),
                connection_id,
                Some(stream_id),
                Some(ProtocolVersion::Http2),
                ApplicationEventKind::Streaming(crate::StreamingEvent::GrpcCall {
                    method: request_method,
                    content_type: content_type.clone(),
                    encoding: grpc_encoding.clone(),
                }),
            ),
        );
    }
    let outbound = hyper::Request::from_parts(parts, ());
    let mut origin = origin
        .ready()
        .await
        .map_err(|error| ProtocolError::new("http2-origin-not-ready", error.to_string()))?;
    let end_stream = request_body.is_end_stream();
    let (response_future, origin_body) = origin
        .send_request(outbound, end_stream)
        .map_err(|error| ProtocolError::new("http2-request-send-failed", error.to_string()))?;
    let mut deferred_request = Some((request_body, origin_body));
    let mut request_pump = if websocket {
        None
    } else {
        let (request_body, origin_body) = deferred_request.take().expect("request streams exist");
        Some(AbortOnDrop(tokio::spawn(pump_body(
            request_body,
            origin_body,
            end_stream,
            request_pump_context(
                limits,
                &context,
                request_encoding.clone(),
                grpc.as_ref().map(|_| {
                    ProtocolObserver::Grpc(
                        crate::GrpcObserver::new(
                            BodyDirection::Request,
                            limits.max_grpc_message_bytes,
                        )
                        .with_encoding(grpc_encoding.clone()),
                    )
                }),
            ),
        ))))
    };
    let response = timeout(limits.idle_timeout, response_future)
        .await
        .map_err(|_| ProtocolError::timeout("http2-response-timeout"))?
        .map_err(|error| h2_error("http2-response-failed", error))?;
    let (parts, response_body) = response.into_parts();
    let status = parts.status;
    let websocket_mode = if websocket && status.is_success() {
        let selected = crate::streaming::permessage_deflate(
            parts
                .headers
                .get_all("sec-websocket-extensions")
                .iter()
                .map(|value| value.as_bytes()),
        );
        if selected.is_some() && websocket_offer.flatten().is_none() {
            return Err(ProtocolError::new(
                "websocket-extension-invalid",
                "origin selected permessage-deflate without a client offer",
            ));
        }
        let compression = selected.is_some();
        let selected = selected.unwrap_or_default();
        Some(WebSocketMode {
            compression,
            client_no_context_takeover: selected.client_no_context_takeover,
            server_no_context_takeover: selected.server_no_context_takeover,
        })
    } else {
        None
    };
    if request_pump.is_none() {
        let (request_body, origin_body) = deferred_request.take().expect("request streams exist");
        let streaming = websocket_mode.map(|mode| {
            ProtocolObserver::WebSocket(
                crate::WebSocketObserver::new(
                    BodyDirection::Request,
                    true,
                    mode.compression,
                    limits.max_websocket_frame_bytes,
                    limits.max_websocket_message_bytes,
                )
                .with_no_context_takeover(mode.client_no_context_takeover),
            )
        });
        request_pump = Some(AbortOnDrop(tokio::spawn(pump_body(
            request_body,
            origin_body,
            end_stream,
            request_pump_context(limits, &context, request_encoding.clone(), streaming),
        ))));
    }
    let mut request_pump = request_pump.expect("request pump is started");
    let response_encoding = content_encoding(&parts.headers);
    let response_grpc_encoding = parts
        .headers
        .get("grpc-encoding")
        .map(|value| value.as_bytes().to_vec());
    let response_grpc_status = parts
        .headers
        .get("grpc-status")
        .map(|value| value.as_bytes().to_vec());
    let response_grpc_message = parts
        .headers
        .get("grpc-message")
        .map(|value| value.as_bytes().to_vec());
    let response_grpc_status_details = parts
        .headers
        .get("grpc-status-details-bin")
        .map(|value| value.as_bytes().to_vec());
    let response_grpc = grpc.is_some() || grpc_content_type(&parts.headers).is_some();
    let response_sse = !response_grpc && sse_content_type(&parts.headers);
    emit(
        sink,
        ApplicationEvent::now(
            session_id.as_str(),
            connection_id,
            Some(stream_id),
            Some(ProtocolVersion::Http2),
            ApplicationEventKind::Metadata(MetadataBlock::http2(
                MetadataKind::Response,
                vec![MetadataField {
                    name: b":status".to_vec(),
                    value: status.as_str().as_bytes().to_vec(),
                    original_index: 0,
                    sensitive: false,
                }],
                fields_from_header_map(&parts.headers),
            )),
        ),
    );
    if response_sse && response_encoding.is_some() {
        emit(
            sink,
            ApplicationEvent::now(
                session_id.as_str(),
                connection_id,
                Some(stream_id),
                Some(ProtocolVersion::Http2),
                ApplicationEventKind::Streaming(crate::StreamingEvent::SseTerminal {
                    outcome: crate::StreamingOutcome::UnsupportedCompression,
                }),
            ),
        );
    }
    let response_end_stream = response_body.is_end_stream();
    let downstream = respond
        .send_response(hyper::Response::from_parts(parts, ()), response_end_stream)
        .map_err(|error| ProtocolError::new("http2-response-send-failed", error.to_string()))?;
    let response_pump = pump_body(
        response_body,
        downstream,
        response_end_stream,
        BodyPumpContext {
            budget: limits.idle_timeout,
            sink: sink.clone(),
            session_id: session_id.clone(),
            connection_id,
            stream_id,
            direction: BodyDirection::Response,
            retention_limit: limits.max_body_bytes,
            content_encoding: response_encoding.clone(),
            max_decoded_body_bytes: limits.max_decoded_body_bytes,
            max_decode_ratio: limits.max_decode_ratio,
            decode_timeout: limits.decode_timeout,
            capture_payloads: limits.capture_payloads,
            session_retained: Arc::clone(&context.session_retained),
            session_retention_limit: limits.max_session_body_bytes,
            decoder_slots: Arc::clone(&context.decoder_slots),
            streaming: if let Some(mode) = websocket_mode {
                Some(ProtocolObserver::WebSocket(
                    crate::WebSocketObserver::new(
                        BodyDirection::Response,
                        false,
                        mode.compression,
                        limits.max_websocket_frame_bytes,
                        limits.max_websocket_message_bytes,
                    )
                    .with_no_context_takeover(mode.server_no_context_takeover),
                ))
            } else if response_grpc {
                Some(ProtocolObserver::Grpc(
                    crate::GrpcObserver::new(
                        BodyDirection::Response,
                        limits.max_grpc_message_bytes,
                    )
                    .with_encoding(response_grpc_encoding)
                    .with_terminal_metadata(
                        response_grpc_status,
                        response_grpc_message,
                        response_grpc_status_details,
                    ),
                ))
            } else if response_sse && response_encoding.is_none() {
                Some(ProtocolObserver::Sse(crate::SseObserver::new(
                    limits.max_sse_line_bytes,
                    limits.max_sse_event_bytes,
                )))
            } else {
                None
            },
        },
    );
    let (request_result, response_result) = tokio::join!(&mut request_pump.0, response_pump);
    request_result.map_err(|error| {
        ProtocolError::new("http2-request-pump-join-failed", error.to_string())
    })??;
    response_result?;
    Ok(())
}

async fn pump_body(
    mut receive: RecvStream,
    mut send: SendStream<Bytes>,
    send_already_ended: bool,
    context: BodyPumpContext,
) -> Result<(), ProtocolError> {
    let BodyPumpContext {
        budget,
        sink,
        session_id,
        connection_id,
        stream_id,
        direction,
        retention_limit,
        content_encoding,
        max_decoded_body_bytes,
        max_decode_ratio,
        decode_timeout,
        capture_payloads,
        session_retained,
        session_retention_limit,
        decoder_slots,
        streaming,
    } = context;
    let mut streaming = StreamingPumpObserver {
        observer: streaming,
        sink: sink.clone(),
        session_id: session_id.clone(),
        connection_id,
        stream_id,
        capture_payloads,
    };
    let mut offset = 0_u64;
    let mut retained = 0_u64;
    let mut retained_bytes = Vec::new();
    while let Some(chunk) = timeout(budget, receive.data())
        .await
        .map_err(|_| ProtocolError::timeout("http2-body-idle-timeout"))?
    {
        let chunk = chunk.map_err(|error| h2_error("http2-body-read-failed", error))?;
        let remaining = retention_limit.saturating_sub(retained);
        let keep = if capture_payloads {
            crate::body::claim_retention(
                &session_retained,
                session_retention_limit,
                (chunk.len() as u64).min(remaining) as usize,
            )
        } else {
            0
        };
        emit(
            &sink,
            ApplicationEvent::now(
                &session_id,
                connection_id,
                Some(stream_id),
                Some(ProtocolVersion::Http2),
                ApplicationEventKind::Body(BodySegment {
                    direction,
                    representation: BodyRepresentation::Raw,
                    offset,
                    observed_len: chunk.len() as u64,
                    bytes: chunk.slice(..keep),
                    outcome: if !capture_payloads {
                        BodyOutcome::IntentionallyOmitted
                    } else if keep == chunk.len() {
                        BodyOutcome::Complete
                    } else {
                        BodyOutcome::RetentionLimit
                    },
                }),
            ),
        );
        offset = offset.saturating_add(chunk.len() as u64);
        retained = retained.saturating_add(keep as u64);
        retained_bytes.extend_from_slice(&chunk[..keep]);
        streaming.feed(&chunk);
        let mut start = 0;
        while start < chunk.len() {
            send.reserve_capacity(chunk.len() - start);
            let capacity = loop {
                match timeout(budget, std::future::poll_fn(|cx| send.poll_capacity(cx))).await {
                    Ok(Some(Ok(capacity))) if capacity > 0 => break capacity,
                    Ok(Some(Ok(_))) => continue,
                    Ok(Some(Err(error))) => {
                        return Err(ProtocolError::new(
                            "http2-flow-control-failed",
                            error.to_string(),
                        ))
                    }
                    Ok(None) => {
                        return Err(ProtocolError::new(
                            "http2-stream-closed",
                            "HTTP/2 send stream closed",
                        ))
                    }
                    Err(_) => return Err(ProtocolError::timeout("http2-flow-control-timeout")),
                }
            };
            let sent = capacity.min(chunk.len() - start);
            send.send_data(chunk.slice(start..start + sent), false)
                .map_err(|error| ProtocolError::new("http2-body-send-failed", error.to_string()))?;
            receive
                .flow_control()
                .release_capacity(sent)
                .map_err(|error| {
                    ProtocolError::new("http2-flow-control-release-failed", error.to_string())
                })?;
            start += sent;
        }
    }
    let trailers = receive
        .trailers()
        .await
        .map_err(|error| h2_error("http2-trailers-failed", error))?;
    streaming.finish(trailers.as_ref());
    if let Some(trailers) = trailers {
        emit(
            &sink,
            ApplicationEvent::now(
                &session_id,
                connection_id,
                Some(stream_id),
                Some(ProtocolVersion::Http2),
                ApplicationEventKind::Metadata(MetadataBlock::http2(
                    MetadataKind::Trailers,
                    Vec::new(),
                    fields_from_header_map(&trailers),
                )),
            ),
        );
        send.send_trailers(trailers)
            .map_err(|error| ProtocolError::new("http2-trailers-send-failed", error.to_string()))?;
    } else if !send_already_ended {
        send.send_data(Bytes::new(), true)
            .map_err(|error| ProtocolError::new("http2-end-stream-failed", error.to_string()))?;
    }
    if let Some(encoding) = content_encoding.filter(|_| capture_payloads) {
        let (decoded, transformation) = match timeout(decode_timeout, decoder_slots.acquire()).await
        {
            Ok(Ok(permit)) => {
                let result = crate::decode_content(
                    &encoding,
                    &retained_bytes,
                    max_decoded_body_bytes,
                    max_decode_ratio,
                    decode_timeout,
                )
                .await;
                drop(permit);
                result
            }
            Ok(Err(_)) | Err(_) => (
                Vec::new(),
                crate::Transformation {
                    encoding: encoding.clone(),
                    input_bytes: retained_bytes.len() as u64,
                    output_bytes: 0,
                    outcome: BodyOutcome::TimeLimit,
                },
            ),
        };
        let outcome = transformation.outcome;
        emit(
            &sink,
            ApplicationEvent::now(
                &session_id,
                connection_id,
                Some(stream_id),
                Some(ProtocolVersion::Http2),
                ApplicationEventKind::Transformation(transformation),
            ),
        );
        if !decoded.is_empty() {
            let keep = crate::body::claim_retention(
                &session_retained,
                session_retention_limit,
                decoded.len(),
            );
            emit(
                &sink,
                ApplicationEvent::now(
                    &session_id,
                    connection_id,
                    Some(stream_id),
                    Some(ProtocolVersion::Http2),
                    ApplicationEventKind::Body(BodySegment {
                        direction,
                        representation: BodyRepresentation::ContentDecoded,
                        offset: 0,
                        observed_len: decoded.len() as u64,
                        bytes: Bytes::copy_from_slice(&decoded[..keep]),
                        outcome: if keep == decoded.len() {
                            outcome
                        } else {
                            BodyOutcome::RetentionLimit
                        },
                    }),
                ),
            );
        }
    }
    Ok(())
}

fn content_encoding(headers: &hyper::HeaderMap) -> Option<String> {
    headers
        .get(hyper::header::CONTENT_ENCODING)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

fn grpc_content_type(headers: &hyper::HeaderMap) -> Option<Vec<u8>> {
    let value = headers.get(hyper::header::CONTENT_TYPE)?.as_bytes();
    let prefix = b"application/grpc";
    let (kind, boundary) = value.split_at_checked(prefix.len())?;
    (kind.eq_ignore_ascii_case(prefix)
        && (boundary.is_empty() || matches!(boundary.first(), Some(b'+') | Some(b';'))))
    .then(|| value.to_vec())
}

fn sse_content_type(headers: &hyper::HeaderMap) -> bool {
    headers
        .get(hyper::header::CONTENT_TYPE)
        .map(|value| {
            value
                .as_bytes()
                .split(|byte| *byte == b';')
                .next()
                .is_some_and(|kind| kind.trim_ascii().eq_ignore_ascii_case(b"text/event-stream"))
        })
        .unwrap_or(false)
}

fn emit_streaming(
    sink: &SharedEventSink,
    session_id: &str,
    connection_id: u64,
    stream_id: u64,
    mut event: crate::StreamingEvent,
    capture_payloads: bool,
) {
    if !capture_payloads {
        match &mut event {
            crate::StreamingEvent::WebSocketFrame(value) => {
                value.wire_payload.clear();
                value.close_reason.clear();
                value.payload_omitted = true;
                if value.outcome == crate::StreamingOutcome::Complete {
                    value.outcome = crate::StreamingOutcome::IntentionallyOmitted;
                }
            }
            crate::StreamingEvent::WebSocketMessage(value) => {
                value.payload.clear();
                value.payload_omitted = true;
                if value.outcome == crate::StreamingOutcome::Complete {
                    value.outcome = crate::StreamingOutcome::IntentionallyOmitted;
                }
            }
            crate::StreamingEvent::SseField(value) => {
                value.name.clear();
                value.value.clear();
                value.payload_omitted = true;
                if value.outcome == crate::StreamingOutcome::Complete {
                    value.outcome = crate::StreamingOutcome::IntentionallyOmitted;
                }
            }
            crate::StreamingEvent::SseEvent(value) => {
                value.event_type.clear();
                value.data.clear();
                value.last_event_id.clear();
                value.payload_omitted = true;
                if value.outcome == crate::StreamingOutcome::Complete {
                    value.outcome = crate::StreamingOutcome::IntentionallyOmitted;
                }
            }
            crate::StreamingEvent::GrpcMessage(value) => {
                value.payload.clear();
                value.payload_omitted = true;
                if value.outcome == crate::StreamingOutcome::Complete {
                    value.outcome = crate::StreamingOutcome::IntentionallyOmitted;
                }
            }
            _ => {}
        }
    }
    emit(
        sink,
        ApplicationEvent::now(
            session_id,
            connection_id,
            Some(stream_id),
            Some(ProtocolVersion::Http2),
            ApplicationEventKind::Streaming(event),
        ),
    );
}

fn request_pseudo_fields(parts: &hyper::http::request::Parts) -> Vec<MetadataField> {
    let values = [
        (b":method".as_slice(), parts.method.as_str().as_bytes()),
        (
            b":scheme".as_slice(),
            parts.uri.scheme_str().unwrap_or_default().as_bytes(),
        ),
        (
            b":authority".as_slice(),
            parts
                .uri
                .authority()
                .map(|value| value.as_str())
                .unwrap_or_default()
                .as_bytes(),
        ),
        (
            b":path".as_slice(),
            parts
                .uri
                .path_and_query()
                .map(|value| value.as_str())
                .unwrap_or("/")
                .as_bytes(),
        ),
    ];
    let mut fields: Vec<_> = values
        .into_iter()
        .enumerate()
        .map(|(index, (name, value))| MetadataField {
            name: name.to_vec(),
            value: value.to_vec(),
            original_index: index as u32,
            sensitive: false,
        })
        .collect();
    if let Some(protocol) = parts.extensions.get::<h2::ext::Protocol>() {
        fields.push(MetadataField {
            name: b":protocol".to_vec(),
            value: protocol.as_ref().to_vec(),
            original_index: fields.len() as u32,
            sensitive: false,
        });
    }
    fields
}

fn failed(code: &'static str, detail: impl Into<String>) -> Http2Run {
    Http2Run {
        accounting: ProtocolAccounting::default(),
        failure: Some(ProtocolError::new(code, detail)),
    }
}

fn h2_error(code: &'static str, error: h2::Error) -> ProtocolError {
    if error.is_reset() {
        ProtocolError::new(
            "http2-stream-reset",
            format!("peer reset stream with {:?}", error.reason()),
        )
    } else {
        ProtocolError::new(code, error.to_string())
    }
}

fn account_stream_result(
    accounting: &mut ProtocolAccounting,
    joined: Result<Result<(), ProtocolError>, tokio::task::JoinError>,
) {
    match joined {
        Ok(Ok(())) => {
            accounting.responses = accounting.responses.saturating_add(1);
            accounting.http2_streams_completed =
                accounting.http2_streams_completed.saturating_add(1);
        }
        Ok(Err(error)) if error.code.contains("reset") => {
            accounting.http2_streams_reset = accounting.http2_streams_reset.saturating_add(1);
        }
        Ok(Err(error)) if error.code.contains("timeout") => {
            accounting.timed_out = accounting.timed_out.saturating_add(1);
        }
        Ok(Err(_)) | Err(_) => {
            accounting.parse_refused = accounting.parse_refused.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grpc_content_type_requires_a_native_media_type_boundary() {
        for accepted in [
            "application/grpc",
            "application/grpc+proto",
            "application/grpc; charset=utf-8",
            "Application/Grpc+Json",
        ] {
            let mut headers = hyper::HeaderMap::new();
            headers.insert(hyper::header::CONTENT_TYPE, accepted.parse().unwrap());
            assert!(grpc_content_type(&headers).is_some(), "{accepted}");
        }
        for rejected in ["application/grpc-web+proto", "application/grpcanything"] {
            let mut headers = hyper::HeaderMap::new();
            headers.insert(hyper::header::CONTENT_TYPE, rejected.parse().unwrap());
            assert!(grpc_content_type(&headers).is_none(), "{rejected}");
        }
    }
}
