// SPDX-License-Identifier: Apache-2.0

//! Bounded, wire-faithful HTTP/1.1 proxy handling.

use std::future::Future;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::time::timeout;

use crate::{
    ApplicationEvent, ApplicationEventKind, BodyDirection, BodyOutcome, BodyRepresentation,
    BodySegment, DestinationAuthority, MetadataBlock, MetadataKind, ProtocolAccounting,
    ProtocolLimits, ProtocolVersion, ProxyAuthorizationError, ProxyObservation, SessionCapability,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolError {
    pub code: &'static str,
    pub detail: String,
    pub authentication_refused: bool,
    pub policy_refused: bool,
    pub timed_out: bool,
}

impl ProtocolError {
    pub(crate) fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
            authentication_refused: false,
            policy_refused: false,
            timed_out: false,
        }
    }

    pub(crate) fn authentication(error: ProxyAuthorizationError) -> Self {
        let code = match error {
            ProxyAuthorizationError::Missing => "proxy-auth-required",
            ProxyAuthorizationError::Duplicate => "proxy-auth-duplicate",
            ProxyAuthorizationError::Malformed => "proxy-auth-malformed",
            ProxyAuthorizationError::Refused => "proxy-auth-refused",
        };
        let mut value = Self::new(code, code);
        value.authentication_refused = true;
        value
    }

    pub(crate) fn timeout(code: &'static str) -> Self {
        let mut value = Self::new(code, code);
        value.timed_out = true;
        value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Framing {
    None,
    Fixed(u64),
    Chunked,
    CloseDelimited,
}

impl Framing {
    fn has_body(self) -> bool {
        !matches!(self, Self::None | Self::Fixed(0))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RequestHead {
    method: String,
    version: u8,
    headers: Vec<(String, Vec<u8>)>,
    framing: Framing,
    authority: DestinationAuthority,
    origin_target: String,
    url: String,
    close: bool,
    upgrade: bool,
    expects_continue: bool,
    transformations: Vec<&'static str>,
}

impl RequestHead {
    pub(crate) fn is_connect(&self) -> bool {
        self.method == "CONNECT"
    }

    pub(crate) fn authority(&self) -> &DestinationAuthority {
        &self.authority
    }

    pub(crate) fn method(&self) -> &str {
        &self.method
    }

    pub(crate) fn url(&self) -> &str {
        &self.url
    }
}

#[derive(Debug)]
struct ResponseHead {
    raw: Vec<u8>,
    status: u16,
    reason: Option<String>,
    headers: Vec<(String, Vec<u8>)>,
    framing: Framing,
    close: bool,
    upgrade: bool,
}

#[derive(Clone, Copy, Debug)]
struct WebSocketMode {
    compression: bool,
    client_no_context_takeover: bool,
    server_no_context_takeover: bool,
}

#[derive(Debug)]
pub(crate) struct HttpRun {
    pub observations: Vec<ProxyObservation>,
    pub accounting: ProtocolAccounting,
    pub failure: Option<ProtocolError>,
}

pub(crate) struct ObservationContext<'a> {
    pub session_id: &'a str,
    pub connection_id: u64,
    pub client_peer: SocketAddr,
    pub proxy_local: SocketAddr,
    pub protocol: &'static str,
    pub application_sink: crate::application::SharedEventSink,
    pub body_resources: crate::body::SessionBodyResources,
}

struct BodyEmitter {
    sink: crate::application::SharedEventSink,
    session_id: String,
    connection_id: u64,
    stream_id: u64,
    direction: BodyDirection,
    offset: u64,
    retained: u64,
    transfer_offset: u64,
    transfer_retained: u64,
    limit: u64,
    content_input: Vec<u8>,
    content_encoding: Option<String>,
    capture_payloads: bool,
    body_resources: crate::body::SessionBodyResources,
    session_retention_limit: u64,
    sse: Option<crate::SseObserver>,
}

impl BodyEmitter {
    fn emit_content(&mut self, bytes: &[u8], transfer_decoded: bool) {
        let raw_retained = self.emit_raw(bytes);
        if let Some(events) = self.sse.as_mut().map(|observer| observer.feed(bytes)) {
            self.emit_streaming(events);
        }
        if !self.capture_payloads {
            return;
        }
        if transfer_decoded {
            let remaining = self.limit.saturating_sub(self.transfer_retained);
            let requested = (bytes.len() as u64).min(remaining) as usize;
            let retained_len = crate::body::claim_retention(
                &self.body_resources.retained,
                self.session_retention_limit,
                requested,
            );
            let offset = self.transfer_offset;
            self.content_input.extend_from_slice(&bytes[..retained_len]);
            crate::application::emit(
                &self.sink,
                ApplicationEvent::now(
                    &self.session_id,
                    self.connection_id,
                    Some(self.stream_id),
                    Some(ProtocolVersion::Http11),
                    ApplicationEventKind::Body(BodySegment {
                        direction: self.direction,
                        representation: BodyRepresentation::TransferDecoded,
                        offset,
                        observed_len: bytes.len() as u64,
                        bytes: bytes::Bytes::copy_from_slice(&bytes[..retained_len]),
                        outcome: if retained_len == bytes.len() {
                            BodyOutcome::Complete
                        } else {
                            BodyOutcome::RetentionLimit
                        },
                    }),
                ),
            );
            self.transfer_offset = self.transfer_offset.saturating_add(bytes.len() as u64);
            self.transfer_retained = self.transfer_retained.saturating_add(retained_len as u64);
        } else {
            self.content_input.extend_from_slice(&bytes[..raw_retained]);
        }
    }

    fn emit_raw(&mut self, bytes: &[u8]) -> usize {
        let observed_len = bytes.len() as u64;
        let remaining = self.limit.saturating_sub(self.retained);
        let retained_len = if self.capture_payloads {
            crate::body::claim_retention(
                &self.body_resources.retained,
                self.session_retention_limit,
                observed_len.min(remaining) as usize,
            )
        } else {
            0
        };
        let outcome = if !self.capture_payloads {
            BodyOutcome::IntentionallyOmitted
        } else if retained_len == bytes.len() {
            BodyOutcome::Complete
        } else {
            BodyOutcome::RetentionLimit
        };
        let event = ApplicationEvent::now(
            &self.session_id,
            self.connection_id,
            Some(self.stream_id),
            Some(ProtocolVersion::Http11),
            ApplicationEventKind::Body(BodySegment {
                direction: self.direction,
                representation: BodyRepresentation::Raw,
                offset: self.offset,
                observed_len,
                bytes: bytes::Bytes::copy_from_slice(&bytes[..retained_len]),
                outcome,
            }),
        );
        crate::application::emit(&self.sink, event);
        self.offset = self.offset.saturating_add(observed_len);
        self.retained = self.retained.saturating_add(retained_len as u64);
        retained_len
    }

    async fn finish(&mut self, limits: &ProtocolLimits) {
        if let Some(events) = self
            .sse
            .as_mut()
            .map(|observer| observer.finish(crate::StreamingOutcome::Complete))
        {
            self.emit_streaming(events);
        }
        let Some(encoding) = &self.content_encoding else {
            return;
        };
        if !self.capture_payloads {
            return;
        }
        let (decoded, transformation) = match timeout(
            limits.decode_timeout,
            self.body_resources.decoder_slots.clone().acquire_owned(),
        )
        .await
        {
            Ok(Ok(permit)) => {
                let result = crate::decode_content(
                    encoding,
                    &self.content_input,
                    limits.max_decoded_body_bytes,
                    limits.max_decode_ratio,
                    limits.decode_timeout,
                )
                .await;
                drop(permit);
                result
            }
            Ok(Err(_)) | Err(_) => (
                Vec::new(),
                crate::Transformation {
                    encoding: encoding.clone(),
                    input_bytes: self.content_input.len() as u64,
                    output_bytes: 0,
                    outcome: BodyOutcome::TimeLimit,
                },
            ),
        };
        let outcome = transformation.outcome;
        crate::application::emit(
            &self.sink,
            ApplicationEvent::now(
                &self.session_id,
                self.connection_id,
                Some(self.stream_id),
                Some(ProtocolVersion::Http11),
                ApplicationEventKind::Transformation(transformation),
            ),
        );
        if !decoded.is_empty() {
            let retained_len = crate::body::claim_retention(
                &self.body_resources.retained,
                self.session_retention_limit,
                decoded.len(),
            );
            crate::application::emit(
                &self.sink,
                ApplicationEvent::now(
                    &self.session_id,
                    self.connection_id,
                    Some(self.stream_id),
                    Some(ProtocolVersion::Http11),
                    ApplicationEventKind::Body(BodySegment {
                        direction: self.direction,
                        representation: BodyRepresentation::ContentDecoded,
                        offset: 0,
                        observed_len: decoded.len() as u64,
                        bytes: bytes::Bytes::copy_from_slice(&decoded[..retained_len]),
                        outcome: if retained_len == decoded.len() {
                            outcome
                        } else {
                            BodyOutcome::RetentionLimit
                        },
                    }),
                ),
            );
        }
    }

    fn emit_streaming(&self, mut events: Vec<crate::StreamingEvent>) {
        for event in &mut events {
            if !self.capture_payloads {
                match event {
                    crate::StreamingEvent::SseField(value) => {
                        value.name.clear();
                        value.value.clear();
                        value.outcome = crate::StreamingOutcome::IntentionallyOmitted;
                    }
                    crate::StreamingEvent::SseEvent(value) => {
                        value.event_type.clear();
                        value.data.clear();
                        value.last_event_id.clear();
                        value.outcome = crate::StreamingOutcome::IntentionallyOmitted;
                    }
                    _ => {}
                }
            }
        }
        for event in events {
            crate::application::emit(
                &self.sink,
                ApplicationEvent::now(
                    &self.session_id,
                    self.connection_id,
                    Some(self.stream_id),
                    Some(ProtocolVersion::Http11),
                    ApplicationEventKind::Streaming(event),
                ),
            );
        }
    }

    fn emit_failure(&self, error: &ProtocolError) {
        let outcome = if error.timed_out {
            BodyOutcome::TimeLimit
        } else if error.code.contains("cancel") {
            BodyOutcome::Cancelled
        } else {
            BodyOutcome::Partial
        };
        crate::application::emit(
            &self.sink,
            ApplicationEvent::now(
                &self.session_id,
                self.connection_id,
                Some(self.stream_id),
                Some(ProtocolVersion::Http11),
                ApplicationEventKind::Body(BodySegment {
                    direction: self.direction,
                    representation: BodyRepresentation::Raw,
                    offset: self.offset,
                    observed_len: 0,
                    bytes: bytes::Bytes::new(),
                    outcome,
                }),
            ),
        );
    }

    fn emit_trailer(&self, line: &[u8]) {
        let line = line.strip_suffix(b"\r\n").unwrap_or(line);
        let Some(split) = line.iter().position(|byte| *byte == b':') else {
            return;
        };
        let field = (
            String::from_utf8_lossy(&line[..split]).to_string(),
            line[split + 1..]
                .iter()
                .copied()
                .skip_while(u8::is_ascii_whitespace)
                .collect(),
        );
        crate::application::emit(
            &self.sink,
            ApplicationEvent::now(
                &self.session_id,
                self.connection_id,
                Some(self.stream_id),
                Some(ProtocolVersion::Http11),
                ApplicationEventKind::Metadata(MetadataBlock::http1(
                    MetadataKind::Trailers,
                    &[field],
                )),
            ),
        );
    }
}

pub(crate) async fn read_authenticated_request<S>(
    stream: &mut S,
    capability: &SessionCapability,
    limits: &ProtocolLimits,
) -> Result<Option<RequestHead>, ProtocolError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let raw = match read_head(stream, limits.max_header_bytes, limits.header_timeout).await? {
        Some(raw) => raw,
        None => return Ok(None),
    };
    let parsed = parse_request(&raw, limits)?;
    let authorization: Vec<_> = parsed
        .headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("proxy-authorization"))
        .map(|(_, value)| value.as_slice())
        .collect();
    let authorization = match authorization.as_slice() {
        [] => None,
        [value] => Some(*value),
        _ => {
            return Err(ProtocolError::authentication(
                ProxyAuthorizationError::Duplicate,
            ))
        }
    };
    if let Err(error) = capability.authenticates_proxy_authorization(authorization) {
        let response = b"HTTP/1.1 407 Proxy Authentication Required\r\nProxy-Authenticate: Basic realm=\"fragcap-session\"\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        let _ = stream.write_all(response).await;
        return Err(ProtocolError::authentication(error));
    }
    Ok(Some(parsed))
}

pub(crate) async fn read_request<S>(
    stream: &mut S,
    limits: &ProtocolLimits,
) -> Result<Option<RequestHead>, ProtocolError>
where
    S: AsyncRead + Unpin,
{
    let raw = match read_head(stream, limits.max_header_bytes, limits.header_timeout).await? {
        Some(raw) => raw,
        None => return Ok(None),
    };
    parse_request(&raw, limits).map(Some)
}

pub(crate) async fn serve_http<S, C, F, U>(
    client: S,
    first: RequestHead,
    capability: &SessionCapability,
    limits: &ProtocolLimits,
    context: ObservationContext<'_>,
    authenticate_subsequent: bool,
    mut connect: C,
) -> HttpRun
where
    S: AsyncRead + AsyncWrite + Unpin,
    U: AsyncRead + AsyncWrite + Unpin,
    C: FnMut(DestinationAuthority) -> F,
    F: Future<Output = Result<U, ProtocolError>>,
{
    let mut client = BufReader::new(client);
    let mut observations = Vec::new();
    let mut accounting = ProtocolAccounting::default();
    let mut next = Some(first);
    let mut ordinal = 0_u64;
    let failure = loop {
        let request = match next.take() {
            Some(request) => request,
            None => match if authenticate_subsequent {
                read_authenticated_request(&mut client, capability, limits).await
            } else {
                read_request(&mut client, limits).await
            } {
                Ok(Some(request)) => request,
                Ok(None) => break None,
                Err(error) => break Some(error),
            },
        };
        ordinal = ordinal.saturating_add(1);
        if ordinal as usize > limits.max_requests_per_connection {
            break Some(ProtocolError::new(
                "request-count-limit-exceeded",
                "client connection exceeded its request limit",
            ));
        }
        if request.is_connect() {
            break Some(ProtocolError::new(
                "connect-after-request-unsupported",
                "CONNECT must be the first request on a client connection",
            ));
        }
        accounting.requests = accounting.requests.saturating_add(1);
        crate::application::emit(
            &context.application_sink,
            ApplicationEvent::now(
                context.session_id,
                context.connection_id,
                Some(ordinal),
                Some(ProtocolVersion::Http11),
                ApplicationEventKind::HttpStreamOpen,
            ),
        );
        crate::application::emit(
            &context.application_sink,
            ApplicationEvent::now(
                context.session_id,
                context.connection_id,
                Some(ordinal),
                Some(ProtocolVersion::Http11),
                ApplicationEventKind::Metadata(
                    MetadataBlock::http1(MetadataKind::Request, &request.headers)
                        .with_http1_request(&request.method, &request.origin_target),
                ),
            ),
        );
        let mut observation = request_observation(&context, ordinal, &request);
        let mut request_body = BodyEmitter {
            sink: context.application_sink.clone(),
            session_id: context.session_id.to_string(),
            connection_id: context.connection_id,
            stream_id: ordinal,
            direction: BodyDirection::Request,
            offset: 0,
            retained: 0,
            transfer_offset: 0,
            transfer_retained: 0,
            limit: limits.max_body_bytes,
            content_input: Vec::new(),
            content_encoding: header(&request.headers, "content-encoding")
                .and_then(|value| std::str::from_utf8(value).ok())
                .map(ToOwned::to_owned),
            capture_payloads: limits.capture_payloads,
            body_resources: context.body_resources.clone(),
            session_retention_limit: limits.max_session_body_bytes,
            sse: None,
        };
        let exchange = async {
            let mut upstream = BufReader::new(connect(request.authority.clone()).await?);
            let head = encode_request(&request);
            write_bounded(&mut upstream, &head, limits.idle_timeout).await?;

            let mut early_final = None;
            let mut force_close = false;
            if request.expects_continue && request.framing.has_body() {
                loop {
                    let ready = timeout(limits.idle_timeout, async {
                        tokio::select! {
                            biased;
                            value = upstream.fill_buf() => value.map(|_| false),
                            value = client.fill_buf() => value.map(|_| true),
                        }
                    })
                    .await
                    .map_err(|_| ProtocolError::timeout("http-expect-idle-timeout"))?
                    .map_err(|error| {
                        ProtocolError::new("http-expect-read-failed", error.to_string())
                    })?;
                    if ready {
                        relay_body(
                            &mut client,
                            &mut upstream,
                            request.framing,
                            limits,
                            &mut request_body,
                        )
                        .await?;
                        break;
                    }
                    let response =
                        read_forward_response(&mut upstream, &mut client, limits, &request.method)
                            .await?;
                    if (100..200).contains(&response.status) && response.status != 101 {
                        emit_response_metadata(
                            &context,
                            ordinal,
                            &response,
                            MetadataKind::InformationalResponse,
                        );
                        accounting.informational_responses =
                            accounting.informational_responses.saturating_add(1);
                        if response.status == 100 {
                            relay_body(
                                &mut client,
                                &mut upstream,
                                request.framing,
                                limits,
                                &mut request_body,
                            )
                            .await?;
                            break;
                        }
                        continue;
                    }
                    force_close = true;
                    early_final = Some(response);
                    break;
                }
            } else {
                relay_body(
                    &mut client,
                    &mut upstream,
                    request.framing,
                    limits,
                    &mut request_body,
                )
                .await?;
            }

            let response = match early_final {
                Some(response) => response,
                None => loop {
                    let response =
                        read_forward_response(&mut upstream, &mut client, limits, &request.method)
                            .await?;
                    if (100..200).contains(&response.status) && response.status != 101 {
                        emit_response_metadata(
                            &context,
                            ordinal,
                            &response,
                            MetadataKind::InformationalResponse,
                        );
                        accounting.informational_responses =
                            accounting.informational_responses.saturating_add(1);
                        continue;
                    }
                    break response;
                },
            };
            Ok::<_, ProtocolError>((upstream, response, force_close))
        }
        .await;
        request_body.finish(limits).await;
        let (mut upstream, response, force_close) = match exchange {
            Ok(value) => value,
            Err(error) => {
                request_body.emit_failure(&error);
                emit_http1_terminal(&context, ordinal, terminal_for_error(&error));
                observation.reason = Some(error.code.to_string());
                observations.push(observation);
                break Some(error);
            }
        };
        accounting.responses = accounting.responses.saturating_add(1);
        crate::application::emit(
            &context.application_sink,
            ApplicationEvent::now(
                context.session_id,
                context.connection_id,
                Some(ordinal),
                Some(ProtocolVersion::Http11),
                ApplicationEventKind::Metadata(
                    MetadataBlock::http1(MetadataKind::Response, &response.headers)
                        .with_http1_response(response.status, response.reason.as_deref()),
                ),
            ),
        );
        observation.status = Some(response.status);
        observation.inspectability = "full";
        if response.upgrade && request.upgrade {
            let websocket = websocket_mode(&request, &response);
            let result = match websocket {
                Ok(Some(mode)) => {
                    relay_websocket(&mut client, &mut upstream, limits, &context, ordinal, mode)
                        .await
                        .err()
                }
                Ok(None) => match timeout(
                    limits.idle_timeout,
                    tokio::io::copy_bidirectional(&mut client, &mut upstream),
                )
                .await
                {
                    Ok(Ok(_)) => None,
                    Ok(Err(error)) => {
                        Some(ProtocolError::new("upgrade-io-failed", error.to_string()))
                    }
                    Err(_) => Some(ProtocolError::timeout("upgrade-idle-timeout")),
                },
                Err(error) => Some(error),
            };
            if let Some(error) = &result {
                observation.reason = Some(error.code.to_string());
            }
            observations.push(observation);
            emit_http1_terminal(
                &context,
                ordinal,
                result
                    .as_ref()
                    .map_or(crate::StreamTerminal::Complete, terminal_for_error),
            );
            break result;
        }
        let response_is_sse = is_sse(&response.headers);
        let response_is_encoded = header(&response.headers, "content-encoding").is_some();
        if response_is_sse && response_is_encoded {
            crate::application::emit(
                &context.application_sink,
                ApplicationEvent::now(
                    context.session_id,
                    context.connection_id,
                    Some(ordinal),
                    Some(ProtocolVersion::Http11),
                    ApplicationEventKind::Streaming(crate::StreamingEvent::SseTerminal {
                        outcome: crate::StreamingOutcome::UnsupportedCompression,
                    }),
                ),
            );
        }
        let mut response_body = BodyEmitter {
            sink: context.application_sink.clone(),
            session_id: context.session_id.to_string(),
            connection_id: context.connection_id,
            stream_id: ordinal,
            direction: BodyDirection::Response,
            offset: 0,
            retained: 0,
            transfer_offset: 0,
            transfer_retained: 0,
            limit: limits.max_body_bytes,
            content_input: Vec::new(),
            content_encoding: header(&response.headers, "content-encoding")
                .and_then(|value| std::str::from_utf8(value).ok())
                .map(ToOwned::to_owned),
            capture_payloads: limits.capture_payloads,
            body_resources: context.body_resources.clone(),
            session_retention_limit: limits.max_session_body_bytes,
            sse: if !response_is_encoded && response_is_sse {
                Some(crate::SseObserver::new(
                    limits.max_sse_line_bytes,
                    limits.max_sse_event_bytes,
                ))
            } else {
                None
            },
        };
        let body_result = relay_body(
            &mut upstream,
            &mut client,
            response.framing,
            limits,
            &mut response_body,
        )
        .await;
        response_body.finish(limits).await;
        if let Err(error) = body_result {
            response_body.emit_failure(&error);
            emit_http1_terminal(&context, ordinal, terminal_for_error(&error));
            observation.reason = Some(error.code.to_string());
            observations.push(observation);
            break Some(error);
        }
        observations.push(observation);
        emit_http1_terminal(&context, ordinal, crate::StreamTerminal::Complete);
        if force_close
            || request.close
            || response.close
            || response.framing == Framing::CloseDelimited
        {
            break None;
        }
    };
    if let Some(error) = &failure {
        if error.authentication_refused || error.code.starts_with("http-") {
            accounting.parse_refused = accounting.parse_refused.saturating_add(1);
        }
        if error.policy_refused {
            accounting.policy_refused = accounting.policy_refused.saturating_add(1);
        }
        if error.timed_out {
            accounting.timed_out = accounting.timed_out.saturating_add(1);
        }
    }
    let _ = timeout(limits.idle_timeout, client.shutdown()).await;
    HttpRun {
        observations,
        accounting,
        failure,
    }
}

fn terminal_for_error(error: &ProtocolError) -> crate::StreamTerminal {
    if error.timed_out {
        crate::StreamTerminal::IdleTimeout
    } else if error.code.contains("cancel") {
        crate::StreamTerminal::Cancelled
    } else if ["read", "write", "eof", "closed", "io"]
        .into_iter()
        .any(|part| error.code.contains(part))
    {
        crate::StreamTerminal::TransportError
    } else {
        crate::StreamTerminal::ProtocolError
    }
}

fn is_sse(headers: &[(String, Vec<u8>)]) -> bool {
    header(headers, "content-type")
        .and_then(|value| value.split(|byte| *byte == b';').next())
        .is_some_and(|value| {
            value
                .trim_ascii()
                .eq_ignore_ascii_case(b"text/event-stream")
        })
}

fn websocket_mode(
    request: &RequestHead,
    response: &ResponseHead,
) -> Result<Option<WebSocketMode>, ProtocolError> {
    if !header(request.headers.as_slice(), "upgrade")
        .is_some_and(|value| value.eq_ignore_ascii_case(b"websocket"))
    {
        return Ok(None);
    }
    if request.method != "GET" || request.version != 1 {
        return Err(ProtocolError::new(
            "websocket-handshake-invalid",
            "WebSocket upgrade requires GET over HTTP/1.1",
        ));
    }
    let versions = header_values(&request.headers, "sec-websocket-version");
    let keys = header_values(&request.headers, "sec-websocket-key");
    let accepts = header_values(&response.headers, "sec-websocket-accept");
    if versions.as_slice() != [b"13".as_slice()] || keys.len() != 1 || accepts.len() != 1 {
        return Err(ProtocolError::new(
            "websocket-handshake-invalid",
            "WebSocket version, key, and accept fields must be singular and valid",
        ));
    }
    if !header(&response.headers, "upgrade")
        .is_some_and(|value| value.eq_ignore_ascii_case(b"websocket"))
    {
        return Err(ProtocolError::new(
            "websocket-handshake-invalid",
            "origin did not select the WebSocket upgrade",
        ));
    }
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(keys[0])
        .map_err(|_| ProtocolError::new("websocket-key-invalid", "WebSocket key is not base64"))?;
    if decoded.len() != 16 {
        return Err(ProtocolError::new(
            "websocket-key-invalid",
            "WebSocket key must decode to sixteen bytes",
        ));
    }
    let mut challenge = keys[0].to_vec();
    challenge.extend_from_slice(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let expected = base64::engine::general_purpose::STANDARD.encode(ring::digest::digest(
        &ring::digest::SHA1_FOR_LEGACY_USE_ONLY,
        &challenge,
    ));
    if accepts[0] != expected.as_bytes() {
        return Err(ProtocolError::new(
            "websocket-accept-invalid",
            "origin WebSocket accept proof does not match the client key",
        ));
    }
    let offered = header_values(&request.headers, "sec-websocket-extensions")
        .into_iter()
        .any(extension_has_permessage_deflate);
    let selected_values = header_values(&response.headers, "sec-websocket-extensions");
    let selected = selected_values
        .iter()
        .copied()
        .find(|value| extension_has_permessage_deflate(value));
    if selected.is_some() && !offered {
        return Err(ProtocolError::new(
            "websocket-extension-invalid",
            "origin selected permessage-deflate without a client offer",
        ));
    }
    let selected = selected.unwrap_or_default();
    Ok(Some(WebSocketMode {
        compression: !selected.is_empty(),
        client_no_context_takeover: extension_parameter(selected, b"client_no_context_takeover"),
        server_no_context_takeover: extension_parameter(selected, b"server_no_context_takeover"),
    }))
}

fn header_values<'a>(headers: &'a [(String, Vec<u8>)], name: &str) -> Vec<&'a [u8]> {
    headers
        .iter()
        .filter(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_slice())
        .collect()
}

fn extension_has_permessage_deflate(value: &[u8]) -> bool {
    value.split(|byte| *byte == b',').any(|extension| {
        extension
            .split(|byte| *byte == b';')
            .next()
            .is_some_and(|name| {
                name.trim_ascii()
                    .eq_ignore_ascii_case(b"permessage-deflate")
            })
    })
}

fn extension_parameter(value: &[u8], expected: &[u8]) -> bool {
    value
        .split(|byte| matches!(byte, b';' | b','))
        .any(|part| part.trim_ascii().eq_ignore_ascii_case(expected))
}

async fn relay_websocket<C, U>(
    client: &mut C,
    upstream: &mut U,
    limits: &ProtocolLimits,
    context: &ObservationContext<'_>,
    stream_id: u64,
    mode: WebSocketMode,
) -> Result<(), ProtocolError>
where
    C: AsyncRead + AsyncWrite + Unpin,
    U: AsyncRead + AsyncWrite + Unpin,
{
    let mut request = crate::WebSocketObserver::new(
        BodyDirection::Request,
        true,
        mode.compression,
        limits.max_websocket_frame_bytes,
        limits.max_websocket_message_bytes,
    )
    .with_no_context_takeover(mode.client_no_context_takeover);
    let mut response = crate::WebSocketObserver::new(
        BodyDirection::Response,
        false,
        mode.compression,
        limits.max_websocket_frame_bytes,
        limits.max_websocket_message_bytes,
    )
    .with_no_context_takeover(mode.server_no_context_takeover);
    let mut client_open = true;
    let mut upstream_open = true;
    let mut client_buffer = vec![0; limits.max_event_chunk_bytes];
    let mut upstream_buffer = vec![0; limits.max_event_chunk_bytes];
    while client_open || upstream_open {
        let step = timeout(limits.idle_timeout, async {
            tokio::select! {
                value = client.read(&mut client_buffer), if client_open => (true, value),
                value = upstream.read(&mut upstream_buffer), if upstream_open => (false, value),
            }
        })
        .await
        .map_err(|_| ProtocolError::timeout("websocket-idle-timeout"))?;
        let (from_client, read) = step;
        let read =
            read.map_err(|error| ProtocolError::new("websocket-read-failed", error.to_string()))?;
        if read == 0 {
            if from_client {
                client_open = false;
                upstream.shutdown().await.map_err(|error| {
                    ProtocolError::new("websocket-shutdown-failed", error.to_string())
                })?;
            } else {
                upstream_open = false;
                client.shutdown().await.map_err(|error| {
                    ProtocolError::new("websocket-shutdown-failed", error.to_string())
                })?;
            }
            continue;
        }
        if from_client {
            let bytes = &client_buffer[..read];
            write_bounded(&mut *upstream, bytes, limits.idle_timeout).await?;
            emit_http1_streaming(
                context,
                stream_id,
                request.feed(bytes),
                limits.capture_payloads,
            );
        } else {
            let bytes = &upstream_buffer[..read];
            write_bounded(&mut *client, bytes, limits.idle_timeout).await?;
            emit_http1_streaming(
                context,
                stream_id,
                response.feed(bytes),
                limits.capture_payloads,
            );
        }
    }
    emit_http1_streaming(
        context,
        stream_id,
        vec![request.finish(crate::StreamingOutcome::Complete)],
        limits.capture_payloads,
    );
    emit_http1_streaming(
        context,
        stream_id,
        vec![response.finish(crate::StreamingOutcome::Complete)],
        limits.capture_payloads,
    );
    Ok(())
}

fn emit_http1_streaming(
    context: &ObservationContext<'_>,
    stream_id: u64,
    mut events: Vec<crate::StreamingEvent>,
    capture_payloads: bool,
) {
    if !capture_payloads {
        for event in &mut events {
            match event {
                crate::StreamingEvent::WebSocketFrame(value) => {
                    value.wire_payload.clear();
                    value.close_reason.clear();
                    value.outcome = crate::StreamingOutcome::IntentionallyOmitted;
                }
                crate::StreamingEvent::WebSocketMessage(value) => {
                    value.payload.clear();
                    value.outcome = crate::StreamingOutcome::IntentionallyOmitted;
                }
                _ => {}
            }
        }
    }
    for event in events {
        crate::application::emit(
            &context.application_sink,
            ApplicationEvent::now(
                context.session_id,
                context.connection_id,
                Some(stream_id),
                Some(ProtocolVersion::Http11),
                ApplicationEventKind::Streaming(event),
            ),
        );
    }
}

fn emit_http1_terminal(
    context: &ObservationContext<'_>,
    ordinal: u64,
    terminal: crate::StreamTerminal,
) {
    crate::application::emit(
        &context.application_sink,
        ApplicationEvent::now(
            context.session_id,
            context.connection_id,
            Some(ordinal),
            Some(ProtocolVersion::Http11),
            ApplicationEventKind::HttpStreamTerminal(terminal),
        ),
    );
}

fn parse_request(raw: &[u8], limits: &ProtocolLimits) -> Result<RequestHead, ProtocolError> {
    let mut storage = vec![httparse::EMPTY_HEADER; limits.max_headers];
    let mut request = httparse::Request::new(&mut storage);
    let status = request
        .parse(raw)
        .map_err(|error| ProtocolError::new("http-request-head-invalid", error.to_string()))?;
    if !status.is_complete() {
        return Err(ProtocolError::new(
            "http-request-head-incomplete",
            "request head is incomplete",
        ));
    }
    let method = request
        .method
        .ok_or_else(|| ProtocolError::new("http-method-missing", "request method is missing"))?
        .to_string();
    let target = request
        .path
        .ok_or_else(|| ProtocolError::new("http-target-missing", "request target is missing"))?
        .to_string();
    let version = request
        .version
        .ok_or_else(|| ProtocolError::new("http-version-missing", "request version is missing"))?;
    let headers: Vec<_> = request
        .headers
        .iter()
        .map(|header| (header.name.to_string(), header.value.to_vec()))
        .collect();
    if headers
        .iter()
        .any(|(_, value)| value.iter().any(|byte| matches!(byte, 0 | b'\r' | b'\n')))
    {
        return Err(ProtocolError::new(
            "http-header-control-byte",
            "header contains a prohibited control byte",
        ));
    }
    let (authority, origin_target, url) = request_destination(&method, &target, &headers)?;
    let framing = framing(&headers, false, None)?;
    let close = connection_token(&headers, "close");
    let upgrade = connection_token(&headers, "upgrade") && header(&headers, "upgrade").is_some();
    let expects_continue = header_token(&headers, "expect", "100-continue");
    let mut transformations = vec!["proxy-authorization-removed"];
    if origin_target != target {
        transformations.push("absolute-to-origin-form");
    }
    if header(&headers, "proxy-connection").is_some() {
        transformations.push("proxy-connection-removed");
    }
    Ok(RequestHead {
        method,
        version,
        headers,
        framing,
        authority,
        origin_target,
        url,
        close,
        upgrade,
        expects_continue,
        transformations,
    })
}

fn parse_response(
    raw: &[u8],
    limits: &ProtocolLimits,
    request_method: &str,
) -> Result<ResponseHead, ProtocolError> {
    let mut storage = vec![httparse::EMPTY_HEADER; limits.max_headers];
    let mut response = httparse::Response::new(&mut storage);
    let status = response
        .parse(raw)
        .map_err(|error| ProtocolError::new("http-response-head-invalid", error.to_string()))?;
    if !status.is_complete() {
        return Err(ProtocolError::new(
            "http-response-head-incomplete",
            "response head is incomplete",
        ));
    }
    let code = response
        .code
        .ok_or_else(|| ProtocolError::new("http-status-missing", "response status is missing"))?;
    let reason = response.reason.map(ToOwned::to_owned);
    let headers: Vec<_> = response
        .headers
        .iter()
        .map(|header| (header.name.to_string(), header.value.to_vec()))
        .collect();
    let no_body =
        request_method == "HEAD" || (100..200).contains(&code) || code == 204 || code == 304;
    let framing = if no_body {
        Framing::None
    } else {
        framing(&headers, true, Some(code))?
    };
    let close = connection_token(&headers, "close");
    let upgrade = code == 101
        && connection_token(&headers, "upgrade")
        && header(&headers, "upgrade").is_some();
    Ok(ResponseHead {
        raw: raw.to_vec(),
        status: code,
        reason,
        headers,
        framing,
        close,
        upgrade,
    })
}

fn request_destination(
    method: &str,
    target: &str,
    headers: &[(String, Vec<u8>)],
) -> Result<(DestinationAuthority, String, String), ProtocolError> {
    let hosts: Vec<_> = headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("host"))
        .collect();
    if hosts.len() > 1 {
        return Err(ProtocolError::new(
            "http-host-duplicate",
            "request contains more than one Host field",
        ));
    }
    if method == "CONNECT" {
        let authority = DestinationAuthority::parse(target)
            .map_err(|error| ProtocolError::new(error.code, error.detail))?;
        return Ok((authority, target.to_string(), format!("https://{target}/")));
    }
    if target.starts_with("http://") || target.starts_with("https://") {
        let uri: hyper::Uri = target
            .parse::<hyper::Uri>()
            .map_err(|error| ProtocolError::new("http-target-invalid", error.to_string()))?;
        let scheme = uri.scheme_str().unwrap_or("http");
        let authority_text = uri
            .authority()
            .ok_or_else(|| ProtocolError::new("http-authority-missing", "URI has no authority"))?
            .as_str();
        let authority_with_port = if uri.authority().and_then(|value| value.port_u16()).is_some() {
            authority_text.to_string()
        } else {
            format!(
                "{authority_text}:{}",
                if scheme == "https" { 443 } else { 80 }
            )
        };
        let authority = DestinationAuthority::parse(&authority_with_port)
            .map_err(|error| ProtocolError::new(error.code, error.detail))?;
        if let Some((_, host)) = hosts.first() {
            let host = std::str::from_utf8(host)
                .map_err(|_| ProtocolError::new("http-host-invalid", "Host is not ASCII"))?;
            let normalized_host =
                normalize_authority(host, if scheme == "https" { 443 } else { 80 });
            if normalized_host != authority.lookup_host() {
                return Err(ProtocolError::new(
                    "http-host-mismatch",
                    "absolute request target and Host field disagree",
                ));
            }
        }
        let origin = uri
            .path_and_query()
            .map_or_else(|| "/".to_string(), ToString::to_string);
        return Ok((authority, origin, target.to_string()));
    }
    if !(target.starts_with('/') || target == "*") {
        return Err(ProtocolError::new(
            "http-target-form-invalid",
            "request target is not a supported HTTP proxy form",
        ));
    }
    let (_, host) = hosts
        .first()
        .ok_or_else(|| ProtocolError::new("http-host-missing", "request has no Host field"))?;
    let host = std::str::from_utf8(host)
        .map_err(|_| ProtocolError::new("http-host-invalid", "Host is not ASCII"))?;
    let normalized = normalize_authority(host, 80);
    let authority = DestinationAuthority::parse(&normalized)
        .map_err(|error| ProtocolError::new(error.code, error.detail))?;
    Ok((
        authority,
        target.to_string(),
        format!("http://{host}{target}"),
    ))
}

fn normalize_authority(value: &str, default_port: u16) -> String {
    if hyper::http::uri::Authority::try_from(value)
        .ok()
        .and_then(|value| value.port_u16())
        .is_some()
    {
        value.to_ascii_lowercase()
    } else {
        format!("{}:{default_port}", value.to_ascii_lowercase())
    }
}

fn framing(
    headers: &[(String, Vec<u8>)],
    response: bool,
    _status: Option<u16>,
) -> Result<Framing, ProtocolError> {
    let transfer: Vec<_> = headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("transfer-encoding"))
        .map(|(_, value)| value.as_slice())
        .collect();
    let lengths: Vec<_> = headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .flat_map(|(_, value)| value.split(|byte| *byte == b','))
        .map(|value| {
            value
                .iter()
                .copied()
                .skip_while(u8::is_ascii_whitespace)
                .collect::<Vec<_>>()
        })
        .collect();
    if !transfer.is_empty() && !lengths.is_empty() {
        return Err(ProtocolError::new(
            "http-framing-ambiguous",
            "message contains both Transfer-Encoding and Content-Length",
        ));
    }
    if !transfer.is_empty() {
        if transfer.len() != 1 || !transfer[0].eq_ignore_ascii_case(b"chunked") {
            return Err(ProtocolError::new(
                "http-transfer-coding-unsupported",
                "only one exact chunked transfer coding is supported",
            ));
        }
        return Ok(Framing::Chunked);
    }
    if !lengths.is_empty() {
        let values: Result<Vec<u64>, _> = lengths
            .iter()
            .map(|value| {
                std::str::from_utf8(value)
                    .map(str::trim)
                    .unwrap_or_default()
                    .parse::<u64>()
            })
            .collect();
        let values = values.map_err(|_| {
            ProtocolError::new("http-content-length-invalid", "Content-Length is invalid")
        })?;
        if values.windows(2).any(|pair| pair[0] != pair[1]) {
            return Err(ProtocolError::new(
                "http-framing-ambiguous",
                "Content-Length values disagree",
            ));
        }
        let length = values[0];
        return Ok(if length == 0 {
            Framing::None
        } else {
            Framing::Fixed(length)
        });
    }
    Ok(if response {
        Framing::CloseDelimited
    } else {
        Framing::None
    })
}

fn header<'a>(headers: &'a [(String, Vec<u8>)], name: &str) -> Option<&'a [u8]> {
    headers
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_slice())
}

fn connection_token(headers: &[(String, Vec<u8>)], token: &str) -> bool {
    header_token(headers, "connection", token)
}

fn header_token(headers: &[(String, Vec<u8>)], name: &str, token: &str) -> bool {
    headers
        .iter()
        .filter(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
        .flat_map(|(_, value)| value.split(|byte| *byte == b','))
        .filter_map(|value| std::str::from_utf8(value).ok())
        .any(|value| value.trim().eq_ignore_ascii_case(token))
}

fn encode_request(request: &RequestHead) -> Vec<u8> {
    let mut result = format!(
        "{} {} HTTP/1.{}\r\n",
        request.method, request.origin_target, request.version
    )
    .into_bytes();
    for (name, value) in &request.headers {
        if name.eq_ignore_ascii_case("proxy-authorization")
            || name.eq_ignore_ascii_case("proxy-connection")
        {
            continue;
        }
        result.extend_from_slice(name.as_bytes());
        result.extend_from_slice(b": ");
        result.extend_from_slice(value);
        result.extend_from_slice(b"\r\n");
    }
    result.extend_from_slice(b"\r\n");
    result
}

async fn read_head<S>(
    stream: &mut S,
    max_bytes: usize,
    budget: std::time::Duration,
) -> Result<Option<Vec<u8>>, ProtocolError>
where
    S: AsyncRead + Unpin,
{
    timeout(budget, async {
        let mut result = Vec::with_capacity(max_bytes.min(1024));
        let mut byte = [0_u8; 1];
        loop {
            let read = stream
                .read(&mut byte)
                .await
                .map_err(|error| ProtocolError::new("http-head-read-failed", error.to_string()))?;
            if read == 0 {
                return if result.is_empty() {
                    Ok(None)
                } else {
                    Err(ProtocolError::new(
                        "http-head-early-eof",
                        "peer closed during a message head",
                    ))
                };
            }
            result.push(byte[0]);
            if result.ends_with(b"\r\n\r\n") {
                return Ok(Some(result));
            }
            if result.len() >= max_bytes {
                return Err(ProtocolError::new(
                    "http-header-limit-exceeded",
                    "message head exceeds the configured byte limit",
                ));
            }
        }
    })
    .await
    .map_err(|_| ProtocolError::timeout("http-header-timeout"))?
}

async fn read_forward_response<R, W>(
    upstream: &mut R,
    client: &mut W,
    limits: &ProtocolLimits,
    request_method: &str,
) -> Result<ResponseHead, ProtocolError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let raw = read_head(upstream, limits.max_header_bytes, limits.idle_timeout)
        .await?
        .ok_or_else(|| {
            ProtocolError::new(
                "upstream-response-eof",
                "upstream closed before a response head",
            )
        })?;
    let response = parse_response(&raw, limits, request_method)?;
    write_bounded(client, &response.raw, limits.idle_timeout).await?;
    Ok(response)
}

async fn relay_body<R, W>(
    reader: &mut R,
    writer: &mut W,
    framing: Framing,
    limits: &ProtocolLimits,
    observer: &mut BodyEmitter,
) -> Result<(), ProtocolError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    match framing {
        Framing::None => Ok(()),
        Framing::Fixed(length) => {
            relay_exact(reader, writer, length, limits.idle_timeout, observer, false).await
        }
        Framing::Chunked => relay_chunked(reader, writer, limits, observer).await,
        Framing::CloseDelimited => {
            let mut buffer = vec![0_u8; 16 * 1024];
            loop {
                let read = timeout(limits.idle_timeout, reader.read(&mut buffer))
                    .await
                    .map_err(|_| ProtocolError::timeout("http-body-idle-timeout"))?
                    .map_err(|error| {
                        ProtocolError::new("http-body-read-failed", error.to_string())
                    })?;
                if read == 0 {
                    return Ok(());
                }
                observer.emit_content(&buffer[..read], false);
                write_bounded(writer, &buffer[..read], limits.idle_timeout).await?;
            }
        }
    }
}

async fn relay_exact<R, W>(
    reader: &mut R,
    writer: &mut W,
    mut remaining: u64,
    budget: std::time::Duration,
    observer: &mut BodyEmitter,
    transfer_decoded: bool,
) -> Result<(), ProtocolError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buffer = vec![0_u8; 16 * 1024];
    while remaining > 0 {
        let wanted = usize::try_from(remaining.min(buffer.len() as u64)).unwrap_or(buffer.len());
        let read = timeout(budget, reader.read(&mut buffer[..wanted]))
            .await
            .map_err(|_| ProtocolError::timeout("http-body-idle-timeout"))?
            .map_err(|error| ProtocolError::new("http-body-read-failed", error.to_string()))?;
        if read == 0 {
            return Err(ProtocolError::new(
                "http-body-early-eof",
                "peer closed before the declared message body ended",
            ));
        }
        write_bounded(writer, &buffer[..read], budget).await?;
        observer.emit_content(&buffer[..read], transfer_decoded);
        remaining -= read as u64;
    }
    Ok(())
}

async fn relay_chunked<R, W>(
    reader: &mut R,
    writer: &mut W,
    limits: &ProtocolLimits,
    observer: &mut BodyEmitter,
) -> Result<(), ProtocolError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    loop {
        let line = read_line(reader, 8 * 1024, limits.idle_timeout).await?;
        write_bounded(writer, &line, limits.idle_timeout).await?;
        let _ = observer.emit_raw(&line);
        let text = std::str::from_utf8(&line[..line.len().saturating_sub(2)]).map_err(|_| {
            ProtocolError::new("http-chunk-size-invalid", "chunk size is not ASCII")
        })?;
        let length = u64::from_str_radix(text.split(';').next().unwrap_or_default().trim(), 16)
            .map_err(|_| ProtocolError::new("http-chunk-size-invalid", "chunk size is invalid"))?;
        if length == 0 {
            loop {
                let trailer =
                    read_line(reader, limits.max_header_bytes, limits.idle_timeout).await?;
                write_bounded(writer, &trailer, limits.idle_timeout).await?;
                let _ = observer.emit_raw(&trailer);
                if trailer == b"\r\n" {
                    return Ok(());
                }
                observer.emit_trailer(&trailer);
            }
        }
        relay_exact(reader, writer, length, limits.idle_timeout, observer, true).await?;
        let end = read_line(reader, 2, limits.idle_timeout).await?;
        if end != b"\r\n" {
            return Err(ProtocolError::new(
                "http-chunk-terminator-invalid",
                "chunk data has no CRLF terminator",
            ));
        }
        write_bounded(writer, &end, limits.idle_timeout).await?;
        let _ = observer.emit_raw(&end);
    }
}

async fn read_line<R>(
    reader: &mut R,
    max_bytes: usize,
    budget: std::time::Duration,
) -> Result<Vec<u8>, ProtocolError>
where
    R: AsyncRead + Unpin,
{
    timeout(budget, async {
        let mut line = Vec::new();
        let mut byte = [0_u8; 1];
        loop {
            let read = reader
                .read(&mut byte)
                .await
                .map_err(|error| ProtocolError::new("http-line-read-failed", error.to_string()))?;
            if read == 0 {
                return Err(ProtocolError::new(
                    "http-line-early-eof",
                    "peer closed during a framed line",
                ));
            }
            line.push(byte[0]);
            if line.ends_with(b"\r\n") {
                return Ok(line);
            }
            if line.len() >= max_bytes {
                return Err(ProtocolError::new(
                    "http-line-limit-exceeded",
                    "framed line exceeds the configured limit",
                ));
            }
        }
    })
    .await
    .map_err(|_| ProtocolError::timeout("http-line-timeout"))?
}

async fn write_bounded<W>(
    writer: &mut W,
    bytes: &[u8],
    budget: std::time::Duration,
) -> Result<(), ProtocolError>
where
    W: AsyncWrite + Unpin,
{
    timeout(budget, writer.write_all(bytes))
        .await
        .map_err(|_| ProtocolError::timeout("http-write-timeout"))?
        .map_err(|error| ProtocolError::new("http-write-failed", error.to_string()))
}

fn request_observation(
    context: &ObservationContext<'_>,
    ordinal: u64,
    request: &RequestHead,
) -> ProxyObservation {
    ProxyObservation {
        session_id: context.session_id.to_string(),
        connection_id: context.connection_id,
        request_ordinal: ordinal,
        client_peer: context.client_peer,
        proxy_local: context.proxy_local,
        timestamp_ns: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .try_into()
            .unwrap_or(u64::MAX),
        protocol: context.protocol.to_string(),
        method: Some(request.method.clone()),
        url: Some(if context.protocol == "https" {
            request.url.replacen("http://", "https://", 1)
        } else {
            request.url.clone()
        }),
        status: None,
        inspectability: "metadata-only",
        reason: None,
        tls: None,
        transformations: request.transformations.clone(),
    }
}

fn emit_response_metadata(
    context: &ObservationContext<'_>,
    ordinal: u64,
    response: &ResponseHead,
    kind: MetadataKind,
) {
    crate::application::emit(
        &context.application_sink,
        ApplicationEvent::now(
            context.session_id,
            context.connection_id,
            Some(ordinal),
            Some(ProtocolVersion::Http11),
            ApplicationEventKind::Metadata(
                MetadataBlock::http1(kind, &response.headers)
                    .with_http1_response(response.status, response.reason.as_deref()),
            ),
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn websocket_handshake_requires_matching_accept_proof() {
        let limits = ProtocolLimits::default();
        let request = parse_request(
            b"GET http://example.test/chat HTTP/1.1\r\nHost: example.test\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Extensions: permessage-deflate; client_no_context_takeover\r\n\r\n",
            &limits,
        )
        .expect("valid request");
        let response = parse_response(
            b"HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\nSec-WebSocket-Extensions: permessage-deflate; client_no_context_takeover\r\n\r\n",
            &limits,
            "GET",
        )
        .expect("valid response");
        let mode = websocket_mode(&request, &response)
            .expect("verified handshake")
            .expect("websocket selected");
        assert!(mode.compression);
        assert!(mode.client_no_context_takeover);
    }

    #[test]
    fn websocket_handshake_rejects_wrong_accept_proof() {
        let limits = ProtocolLimits::default();
        let request = parse_request(
            b"GET http://example.test/chat HTTP/1.1\r\nHost: example.test\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n",
            &limits,
        )
        .expect("valid request");
        let response = parse_response(
            b"HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Accept: wrong\r\n\r\n",
            &limits,
            "GET",
        )
        .expect("parse response");
        assert_eq!(
            websocket_mode(&request, &response)
                .expect_err("accept proof must fail")
                .code,
            "websocket-accept-invalid"
        );
    }
}
