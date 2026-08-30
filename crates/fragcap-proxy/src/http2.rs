// SPDX-License-Identifier: Apache-2.0

//! Bounded HTTP/2 bridge.

use std::sync::atomic::{AtomicU64, Ordering};
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
}

pub(crate) async fn serve_http2<C, U>(
    client: C,
    upstream: U,
    limits: ProtocolLimits,
    session_id: String,
    connection_id: u64,
    sink: SharedEventSink,
) -> Http2Run
where
    C: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    U: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let mut server_builder = h2::server::Builder::new();
    server_builder
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
    let session_retained = Arc::new(AtomicU64::new(0));
    let decoder_slots = Arc::new(Semaphore::new(limits.max_concurrent_decoders));
    while let Some(accepted) = match timeout(limits.idle_timeout, client_connection.accept()).await
    {
        Ok(value) => value,
        Err(_) => {
            return Http2Run {
                accounting,
                failure: Some(ProtocolError::timeout("http2-connection-idle-timeout")),
            }
        }
    } {
        let (request, respond) = match accepted {
            Ok(value) => value,
            Err(error) => {
                return Http2Run {
                    accounting,
                    failure: Some(ProtocolError::new("http2-accept-failed", error.to_string())),
                }
            }
        };
        accounting.requests = accounting.requests.saturating_add(1);
        accounting.http2_streams = accounting.http2_streams.saturating_add(1);
        let stream_id = request.body().stream_id().as_u32() as u64;
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
    limits: ProtocolLimits,
    session_id: String,
    connection_id: u64,
    sink: SharedEventSink,
) -> Http2Run
where
    C: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
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
    let session_retained = Arc::new(AtomicU64::new(0));
    let decoder_slots = Arc::new(Semaphore::new(limits.max_concurrent_decoders));
    let mut tasks = JoinSet::new();
    let mut accounting = ProtocolAccounting::default();
    let mut accepted = Some((first, first_response));
    loop {
        let next = match accepted.take() {
            Some(value) => Some(Ok(value)),
            None => match timeout(limits.idle_timeout, client_connection.accept()).await {
                Ok(value) => value,
                Err(_) => {
                    return Http2Run {
                        accounting,
                        failure: Some(ProtocolError::timeout("http2-connection-idle-timeout")),
                    };
                }
            },
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
    let (parts, request_body) = request.into_parts();
    let pseudo = request_pseudo_fields(&parts);
    let request_encoding = content_encoding(&parts.headers);
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
    let outbound = hyper::Request::from_parts(parts, ());
    let mut origin = origin
        .ready()
        .await
        .map_err(|error| ProtocolError::new("http2-origin-not-ready", error.to_string()))?;
    let end_stream = request_body.is_end_stream();
    let (response_future, origin_body) = origin
        .send_request(outbound, end_stream)
        .map_err(|error| ProtocolError::new("http2-request-send-failed", error.to_string()))?;
    let mut request_pump = AbortOnDrop(tokio::spawn(pump_body(
        request_body,
        origin_body,
        end_stream,
        BodyPumpContext {
            budget: limits.idle_timeout,
            sink: sink.clone(),
            session_id: session_id.clone(),
            connection_id,
            stream_id,
            direction: BodyDirection::Request,
            retention_limit: limits.max_body_bytes,
            content_encoding: request_encoding,
            max_decoded_body_bytes: limits.max_decoded_body_bytes,
            max_decode_ratio: limits.max_decode_ratio,
            decode_timeout: limits.decode_timeout,
            capture_payloads: limits.capture_payloads,
            session_retained: Arc::clone(&context.session_retained),
            session_retention_limit: limits.max_session_body_bytes,
            decoder_slots: Arc::clone(&context.decoder_slots),
        },
    )));
    let response = timeout(limits.idle_timeout, response_future)
        .await
        .map_err(|_| ProtocolError::timeout("http2-response-timeout"))?
        .map_err(|error| h2_error("http2-response-failed", error))?;
    let (parts, response_body) = response.into_parts();
    let status = parts.status;
    let response_encoding = content_encoding(&parts.headers);
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
            content_encoding: response_encoding,
            max_decoded_body_bytes: limits.max_decoded_body_bytes,
            max_decode_ratio: limits.max_decode_ratio,
            decode_timeout: limits.decode_timeout,
            capture_payloads: limits.capture_payloads,
            session_retained: Arc::clone(&context.session_retained),
            session_retention_limit: limits.max_session_body_bytes,
            decoder_slots: Arc::clone(&context.decoder_slots),
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
    } = context;
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
            claim_retention(
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
    if let Some(trailers) = receive
        .trailers()
        .await
        .map_err(|error| h2_error("http2-trailers-failed", error))?
    {
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
                        bytes: Bytes::from(decoded),
                        outcome,
                    }),
                ),
            );
        }
    }
    Ok(())
}

fn claim_retention(counter: &AtomicU64, limit: u64, requested: usize) -> usize {
    let mut current = counter.load(Ordering::Relaxed);
    loop {
        let granted = requested.min(limit.saturating_sub(current) as usize);
        match counter.compare_exchange_weak(
            current,
            current.saturating_add(granted as u64),
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return granted,
            Err(observed) => current = observed,
        }
    }
}

fn content_encoding(headers: &hyper::HeaderMap) -> Option<String> {
    headers
        .get(hyper::header::CONTENT_ENCODING)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
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
    values
        .into_iter()
        .enumerate()
        .map(|(index, (name, value))| MetadataField {
            name: name.to_vec(),
            value: value.to_vec(),
            original_index: index as u32,
            sensitive: false,
        })
        .collect()
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
