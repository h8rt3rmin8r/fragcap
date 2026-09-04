// SPDX-License-Identifier: Apache-2.0

//! Bounded byte entry points shared by stable corpus replay and libFuzzer.

use std::io;
use std::pin::Pin;
use std::sync::OnceLock;
use std::task::{Context, Poll};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::{
    BodyDirection, CertificateIdentity, DestinationAuthority, GrpcObserver, ProtocolLimits,
    QuicDirection, QuicEvidenceObserver, QuicInspectionPlan, SessionCapability, SseObserver,
    StreamingOutcome, WebSocketObserver,
};

pub const MAX_FUZZ_INPUT_BYTES: usize = 64 * 1024;
const RETENTION_LIMIT: usize = 4096;

fn bounded(data: &[u8]) -> Option<&[u8]> {
    (data.len() <= MAX_FUZZ_INPUT_BYTES).then_some(data)
}

fn chunks(control: u8, payload: &[u8]) -> impl Iterator<Item = &[u8]> {
    let width = usize::from(control % 31).saturating_add(1);
    payload.chunks(width)
}

pub fn http1(data: &[u8]) {
    let Some(data) = bounded(data) else { return };
    let limits = ProtocolLimits {
        max_header_bytes: MAX_FUZZ_INPUT_BYTES,
        max_headers: 32,
        ..ProtocolLimits::default()
    };
    let payload = data.get(1..).unwrap_or_default();
    let mut normalized;
    let payload = if payload.contains(&b'\n') && !payload.contains(&b'\r') {
        normalized = payload.iter().fold(
            Vec::with_capacity(payload.len().saturating_mul(2)),
            |mut normalized, byte| {
                if *byte == b'\n' {
                    normalized.extend_from_slice(b"\r\n");
                } else {
                    normalized.push(*byte);
                }
                normalized
            },
        );
        if data.first().is_some_and(|control| control % 3 != 2)
            && !normalized.ends_with(b"\r\n\r\n")
        {
            normalized.extend_from_slice(b"\r\n");
        }
        normalized.as_slice()
    } else {
        payload
    };
    match data.first().copied().unwrap_or_default() % 3 {
        0 => {
            let _ = crate::http1::parse_request(payload, &limits);
        }
        1 => {
            let _ = crate::http1::parse_response(payload, &limits, "GET");
        }
        _ => {
            let _ = crate::http1::parse_chunk_size(payload);
        }
    }
}

pub fn proxy_auth(data: &[u8]) {
    let Some(data) = bounded(data) else { return };
    let capability = SessionCapability::from_test_bytes([0x5a; crate::CAPABILITY_BYTES]);
    let value = (!data.is_empty()).then_some(data);
    let _ = capability.authenticates_proxy_authorization(value);
    if data.first().is_some_and(|value| value & 1 == 1) {
        let authorization = capability.proof().proxy_authorization();
        let _ = capability.authenticates_proxy_authorization(Some(authorization.as_bytes()));
    }
    let split = data
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(data.len());
    let (username, password) = data.split_at(split);
    let password = password.get(1..).unwrap_or_default();
    let _ = capability.authenticates_socks_credentials(username, password);
}

pub fn socks5(data: &[u8]) {
    let Some(data) = bounded(data) else { return };
    let control = data.first().copied().unwrap_or_default();
    let payload = data.get(1..).unwrap_or_default();
    let capability = SessionCapability::from_test_bytes([0x5a; crate::CAPABILITY_BYTES]);
    if control % 3 == 1 {
        let _ = crate::socks5::parse_udp_datagram(payload);
        return;
    }
    let input = if control % 3 == 2 {
        let password = capability.proof().proxy_password();
        let mut framed = vec![5, 1, 2, 1, crate::PROXY_USERNAME.len() as u8];
        framed.extend_from_slice(crate::PROXY_USERNAME.as_bytes());
        framed.push(password.len() as u8);
        framed.extend_from_slice(password.as_bytes());
        framed.extend_from_slice(payload);
        framed
    } else {
        payload.to_vec()
    };
    let mut stream = MemoryStream::new(&input, usize::from(control % 7).saturating_add(1));
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    let runtime = RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("bounded fuzz runtime builds")
    });
    runtime.block_on(async {
        if crate::socks5::negotiate(&mut stream, &capability, Duration::from_millis(1))
            .await
            .is_ok()
        {
            let _ = crate::socks5::read_request(&mut stream, Duration::from_millis(1)).await;
        }
    });
}

pub fn streaming(data: &[u8]) {
    let Some(data) = bounded(data) else { return };
    let control = data.first().copied().unwrap_or_default();
    let payload = data.get(1..).unwrap_or_default();
    let direction = if control & 0x80 == 0 {
        BodyDirection::Request
    } else {
        BodyDirection::Response
    };
    let terminal = streaming_outcome(control);
    match control % 3 {
        0 => {
            let mut value = WebSocketObserver::new(
                direction,
                control & 0x20 != 0,
                control & 0x40 != 0,
                RETENTION_LIMIT,
                RETENTION_LIMIT,
            );
            for chunk in chunks(control, payload) {
                let _ = value.feed(chunk);
            }
            let _ = value.finish(terminal);
        }
        1 => {
            let mut value = SseObserver::new(256, RETENTION_LIMIT);
            for chunk in chunks(control, payload) {
                let _ = value.feed(chunk);
            }
            let _ = value.finish(terminal);
        }
        _ => {
            let mut value = GrpcObserver::new(direction, RETENTION_LIMIT);
            for chunk in chunks(control, payload) {
                let _ = value.feed(chunk);
            }
            let _ = value.finish(None, None, None, terminal);
        }
    }
}

fn streaming_outcome(control: u8) -> StreamingOutcome {
    if control & 0x10 == 0 {
        StreamingOutcome::Cancelled
    } else {
        StreamingOutcome::Complete
    }
}

pub fn identities_quic(data: &[u8]) {
    let Some(data) = bounded(data) else { return };
    let control = data.first().copied().unwrap_or_default();
    let payload = data.get(1..).unwrap_or_default();
    let text = std::str::from_utf8(payload)
        .unwrap_or_default()
        .trim_end_matches(['\r', '\n']);
    match control % 2 {
        0 => {
            let _ = DestinationAuthority::parse(text);
            let _ = DestinationAuthority::parse_uri(text);
        }
        _ => {
            let _ = CertificateIdentity::parse(text);
        }
    }
    let _ = crate::is_quic_initial(data);

    let authority = DestinationAuthority::parse("example.invalid:443").expect("fixture authority");
    let plan = QuicInspectionPlan::new(
        "fuzz-session",
        1,
        "127.0.0.1:40000".parse().expect("fixture endpoint"),
        "127.0.0.1:443".parse().expect("fixture endpoint"),
        &authority,
        true,
    )
    .expect("scoped fixture plan");
    let limits = ProtocolLimits {
        max_body_bytes: RETENTION_LIMIT as u64,
        max_session_body_bytes: RETENTION_LIMIT as u64,
        ..ProtocolLimits::default()
    };
    let mut observer = QuicEvidenceObserver::new(plan);
    let direction = if data.first().is_some_and(|value| value & 1 == 1) {
        QuicDirection::UpstreamToClient
    } else {
        QuicDirection::ClientToUpstream
    };
    let _ = observer.stream(direction, 0, "fuzz", data, &limits, "cancelled");
    let _ = observer.datagram(direction, data, &limits, "cancelled");
}

struct MemoryStream {
    input: Vec<u8>,
    offset: usize,
    written: usize,
    max_read: usize,
    read_calls: usize,
}

impl MemoryStream {
    fn new(input: &[u8], max_read: usize) -> Self {
        Self {
            input: input.to_vec(),
            offset: 0,
            written: 0,
            max_read,
            read_calls: 0,
        }
    }
}

impl AsyncRead for MemoryStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        output: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let remaining = &self.input[self.offset..];
        let count = remaining.len().min(output.remaining()).min(self.max_read);
        output.put_slice(&remaining[..count]);
        self.offset += count;
        self.read_calls = self.read_calls.saturating_add(1);
        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for MemoryStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        input: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.written = self
            .written
            .saturating_add(input.len().min(RETENTION_LIMIT));
        Poll::Ready(Ok(input.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    #[test]
    fn memory_stream_exercises_fragmented_reads() {
        let mut stream = MemoryStream::new(b"fragmented", 2);
        let mut output = [0_u8; 10];
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(stream.read_exact(&mut output))
            .unwrap();
        assert_eq!(&output, b"fragmented");
        assert_eq!(stream.read_calls, 5);
    }

    #[test]
    fn streaming_terminal_selector_reaches_complete_and_cancelled() {
        assert_eq!(streaming_outcome(b'0'), StreamingOutcome::Complete);
        assert_eq!(streaming_outcome(b'@'), StreamingOutcome::Cancelled);
    }
}
