// SPDX-License-Identifier: Apache-2.0

//! Bounded body evidence and derived content decoding.

use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_compression::tokio::bufread::{BrotliDecoder, GzipDecoder, ZlibDecoder};
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};
use tokio::sync::Semaphore;
use tokio::time::timeout;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BodyDirection {
    Request,
    Response,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BodyRepresentation {
    Raw,
    TransferDecoded,
    ContentDecoded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BodyOutcome {
    Complete,
    Partial,
    IntentionallyOmitted,
    RetentionLimit,
    QueueDropped,
    StorageFailed,
    UnsupportedEncoding,
    MalformedEncoding,
    ExpansionLimit,
    TimeLimit,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BodySegment {
    pub direction: BodyDirection,
    pub representation: BodyRepresentation,
    pub offset: u64,
    pub observed_len: u64,
    pub bytes: bytes::Bytes,
    pub outcome: BodyOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transformation {
    pub encoding: String,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub outcome: BodyOutcome,
}

#[derive(Clone)]
pub(crate) struct SessionBodyResources {
    pub retained: Arc<AtomicU64>,
    pub decoder_slots: Arc<Semaphore>,
}

impl SessionBodyResources {
    pub fn new(max_concurrent_decoders: usize) -> Self {
        Self {
            retained: Arc::new(AtomicU64::new(0)),
            decoder_slots: Arc::new(Semaphore::new(max_concurrent_decoders)),
        }
    }
}

pub(crate) fn claim_retention(counter: &AtomicU64, limit: u64, requested: usize) -> usize {
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

pub async fn decode_content(
    encoding: &str,
    input: &[u8],
    max_output: usize,
    max_ratio: usize,
    budget: Duration,
) -> (Vec<u8>, Transformation) {
    let normalized = encoding.trim().to_ascii_lowercase();
    let reader: Box<dyn AsyncRead + Unpin + Send + '_> = match normalized.as_str() {
        "gzip" => Box::new(GzipDecoder::new(BufReader::new(input))),
        "deflate" => Box::new(ZlibDecoder::new(BufReader::new(input))),
        "br" => Box::new(BrotliDecoder::new(BufReader::new(input))),
        _ => {
            return (
                Vec::new(),
                Transformation {
                    encoding: normalized,
                    input_bytes: input.len() as u64,
                    output_bytes: 0,
                    outcome: BodyOutcome::UnsupportedEncoding,
                },
            )
        }
    };
    let expansion_limit = input.len().saturating_mul(max_ratio).max(1);
    let limit = max_output.min(expansion_limit);
    let result = timeout(budget, read_limited(reader, limit)).await;
    let (output, outcome) = match result {
        Err(_) => (Vec::new(), BodyOutcome::TimeLimit),
        Ok(Err(error)) if error.kind() == io::ErrorKind::OutOfMemory => {
            (Vec::new(), BodyOutcome::ExpansionLimit)
        }
        Ok(Err(_)) => (Vec::new(), BodyOutcome::MalformedEncoding),
        Ok(Ok(output)) => (output, BodyOutcome::Complete),
    };
    let transformation = Transformation {
        encoding: normalized,
        input_bytes: input.len() as u64,
        output_bytes: output.len() as u64,
        outcome,
    };
    (output, transformation)
}

async fn read_limited(
    mut reader: Box<dyn AsyncRead + Unpin + Send + '_>,
    limit: usize,
) -> io::Result<Vec<u8>> {
    let mut output = Vec::with_capacity(limit.min(16 * 1024));
    let mut chunk = [0_u8; 8 * 1024];
    loop {
        let read = reader.read(&mut chunk).await?;
        if read == 0 {
            return Ok(output);
        }
        if output.len().saturating_add(read) > limit {
            return Err(io::Error::new(
                io::ErrorKind::OutOfMemory,
                "decoded body exceeds its configured bound",
            ));
        }
        output.extend_from_slice(&chunk[..read]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_compression::tokio::write::{BrotliEncoder, GzipEncoder, ZlibEncoder};
    use tokio::io::AsyncWriteExt;

    async fn encode(kind: &str, input: &[u8]) -> Vec<u8> {
        match kind {
            "gzip" => {
                let mut encoder = GzipEncoder::new(Vec::new());
                encoder.write_all(input).await.unwrap();
                encoder.shutdown().await.unwrap();
                encoder.into_inner()
            }
            "deflate" => {
                let mut encoder = ZlibEncoder::new(Vec::new());
                encoder.write_all(input).await.unwrap();
                encoder.shutdown().await.unwrap();
                encoder.into_inner()
            }
            "br" => {
                let mut encoder = BrotliEncoder::new(Vec::new());
                encoder.write_all(input).await.unwrap();
                encoder.shutdown().await.unwrap();
                encoder.into_inner()
            }
            _ => unreachable!(),
        }
    }

    #[tokio::test]
    async fn supported_decoders_preserve_raw_authority_and_bound_output() {
        let source = b"fragcap-body".repeat(128);
        for encoding in ["gzip", "deflate", "br"] {
            let compressed = encode(encoding, &source).await;
            let (decoded, record) = decode_content(
                encoding,
                &compressed,
                source.len() * 2,
                128,
                Duration::from_secs(1),
            )
            .await;
            assert_eq!(decoded, source);
            assert_eq!(record.outcome, BodyOutcome::Complete);
            assert_eq!(record.input_bytes, compressed.len() as u64);
        }

        let compressed = encode("gzip", &source).await;
        let (decoded, record) =
            decode_content("gzip", &compressed, 8, 128, Duration::from_secs(1)).await;
        assert!(decoded.is_empty());
        assert_eq!(record.outcome, BodyOutcome::ExpansionLimit);
    }

    #[tokio::test]
    async fn unsupported_and_malformed_encodings_are_distinct() {
        assert_eq!(
            decode_content("zstd", b"raw", 1024, 16, Duration::from_secs(1))
                .await
                .1
                .outcome,
            BodyOutcome::UnsupportedEncoding
        );
        assert_eq!(
            decode_content("gzip", b"not-gzip", 1024, 16, Duration::from_secs(1))
                .await
                .1
                .outcome,
            BodyOutcome::MalformedEncoding
        );
    }
}
