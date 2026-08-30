// SPDX-License-Identifier: Apache-2.0

//! Bounded, wire-faithful HTTP/1.1 proxy handling.

use std::future::Future;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::time::timeout;

use crate::{
    DestinationAuthority, ProtocolAccounting, ProtocolLimits, ProxyAuthorizationError,
    ProxyObservation, SessionCapability,
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
    framing: Framing,
    close: bool,
    upgrade: bool,
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
    mut client: S,
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
        let mut observation = request_observation(&context, ordinal, &request);
        let mut upstream = match connect(request.authority.clone()).await {
            Ok(stream) => stream,
            Err(error) => {
                observation.reason = Some(error.code.to_string());
                observations.push(observation);
                break Some(error);
            }
        };
        let head = encode_request(&request);
        if let Err(error) = write_bounded(&mut upstream, &head, limits.idle_timeout).await {
            break Some(error);
        }
        if let Err(error) = relay_body(&mut client, &mut upstream, request.framing, limits).await {
            break Some(error);
        }
        let response = loop {
            let raw = match read_head(&mut upstream, limits.max_header_bytes, limits.idle_timeout)
                .await
            {
                Ok(Some(raw)) => raw,
                Ok(None) => {
                    break Err(ProtocolError::new(
                        "upstream-response-eof",
                        "upstream closed before a response head",
                    ))
                }
                Err(error) => break Err(error),
            };
            let response = match parse_response(&raw, limits, &request.method) {
                Ok(response) => response,
                Err(error) => break Err(error),
            };
            if let Err(error) = write_bounded(&mut client, &response.raw, limits.idle_timeout).await
            {
                break Err(error);
            }
            if (100..200).contains(&response.status) && response.status != 101 {
                accounting.informational_responses =
                    accounting.informational_responses.saturating_add(1);
                continue;
            }
            break Ok(response);
        };
        let response = match response {
            Ok(response) => response,
            Err(error) => break Some(error),
        };
        accounting.responses = accounting.responses.saturating_add(1);
        observation.status = Some(response.status);
        observation.inspectability = "full";
        observations.push(observation);
        if response.upgrade && request.upgrade {
            let result = timeout(
                limits.idle_timeout,
                tokio::io::copy_bidirectional(&mut client, &mut upstream),
            )
            .await;
            break match result {
                Ok(Ok(_)) => None,
                Ok(Err(error)) => Some(ProtocolError::new("upgrade-io-failed", error.to_string())),
                Err(_) => Some(ProtocolError::timeout("upgrade-idle-timeout")),
            };
        }
        if let Err(error) = relay_body(&mut upstream, &mut client, response.framing, limits).await {
            break Some(error);
        }
        if request.close || response.close || response.framing == Framing::CloseDelimited {
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
    let framing = framing(&headers, false, None, limits.max_body_bytes)?;
    let close = connection_token(&headers, "close");
    let upgrade = connection_token(&headers, "upgrade") && header(&headers, "upgrade").is_some();
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
        framing(&headers, true, Some(code), limits.max_body_bytes)?
    };
    Ok(ResponseHead {
        raw: raw.to_vec(),
        status: code,
        framing,
        close: connection_token(&headers, "close"),
        upgrade: code == 101
            && connection_token(&headers, "upgrade")
            && header(&headers, "upgrade").is_some(),
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
    max_body_bytes: u64,
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
        if length > max_body_bytes {
            return Err(ProtocolError::new(
                "http-body-limit-exceeded",
                "declared message body exceeds the configured limit",
            ));
        }
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
    headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("connection"))
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

async fn relay_body<R, W>(
    reader: &mut R,
    writer: &mut W,
    framing: Framing,
    limits: &ProtocolLimits,
) -> Result<(), ProtocolError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    match framing {
        Framing::None => Ok(()),
        Framing::Fixed(length) => relay_exact(reader, writer, length, limits.idle_timeout).await,
        Framing::Chunked => relay_chunked(reader, writer, limits).await,
        Framing::CloseDelimited => {
            let mut total = 0_u64;
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
                total = total.saturating_add(read as u64);
                if total > limits.max_body_bytes {
                    return Err(ProtocolError::new(
                        "http-body-limit-exceeded",
                        "close-delimited body exceeds the configured limit",
                    ));
                }
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
        remaining -= read as u64;
    }
    Ok(())
}

async fn relay_chunked<R, W>(
    reader: &mut R,
    writer: &mut W,
    limits: &ProtocolLimits,
) -> Result<(), ProtocolError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut total = 0_u64;
    loop {
        let line = read_line(reader, 8 * 1024, limits.idle_timeout).await?;
        write_bounded(writer, &line, limits.idle_timeout).await?;
        let text = std::str::from_utf8(&line[..line.len().saturating_sub(2)]).map_err(|_| {
            ProtocolError::new("http-chunk-size-invalid", "chunk size is not ASCII")
        })?;
        let length = u64::from_str_radix(text.split(';').next().unwrap_or_default().trim(), 16)
            .map_err(|_| ProtocolError::new("http-chunk-size-invalid", "chunk size is invalid"))?;
        total = total.saturating_add(length);
        if total > limits.max_body_bytes {
            return Err(ProtocolError::new(
                "http-body-limit-exceeded",
                "chunked body exceeds the configured limit",
            ));
        }
        if length == 0 {
            loop {
                let trailer =
                    read_line(reader, limits.max_header_bytes, limits.idle_timeout).await?;
                write_bounded(writer, &trailer, limits.idle_timeout).await?;
                if trailer == b"\r\n" {
                    return Ok(());
                }
            }
        }
        relay_exact(reader, writer, length, limits.idle_timeout).await?;
        let end = read_line(reader, 2, limits.idle_timeout).await?;
        if end != b"\r\n" {
            return Err(ProtocolError::new(
                "http-chunk-terminator-invalid",
                "chunk data has no CRLF terminator",
            ));
        }
        write_bounded(writer, &end, limits.idle_timeout).await?;
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
