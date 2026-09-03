// SPDX-License-Identifier: Apache-2.0

//! Bounded generic TCP and protocol-unknown TLS stream observation.

use std::future::Future;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::{watch, Notify};
use tokio::time::{timeout, Instant};

use crate::application::SharedEventSink;
use crate::body::SessionBodyResources;
use crate::{
    ApplicationEvent, ApplicationEventKind, GenericStreamChunk, GenericStreamDirection,
    GenericStreamOutcome, GenericStreamProvenance, ProtocolLimits,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GenericRelayReport {
    pub client_to_upstream_bytes: u64,
    pub upstream_to_client_bytes: u64,
    pub observed_bytes: u64,
    pub retained_bytes: u64,
    pub omitted_bytes: u64,
}

pub(crate) struct GenericRelayContext<'a> {
    pub limits: &'a ProtocolLimits,
    pub shutdown: &'a mut watch::Receiver<bool>,
    pub session_id: &'a str,
    pub connection_id: u64,
    pub sink: &'a SharedEventSink,
    pub body_resources: SessionBodyResources,
    pub buffer_bytes: usize,
    pub provenance: GenericStreamProvenance,
}

#[derive(Debug)]
pub(crate) struct GenericRelayRun {
    pub report: GenericRelayReport,
    pub error: Option<GenericRelayFailure>,
}

#[derive(Debug)]
pub(crate) enum GenericRelayFailure {
    Cancelled,
    IdleTimeout,
    OperationTimeout,
    Transport(io::Error),
}

impl std::fmt::Display for GenericRelayFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => formatter.write_str("generic stream cancelled"),
            Self::IdleTimeout => formatter.write_str("generic stream idle timeout"),
            Self::OperationTimeout => formatter.write_str("generic stream I/O timed out"),
            Self::Transport(error) => error.fmt(formatter),
        }
    }
}

pub(crate) async fn relay_generic<C, U>(
    client: &mut C,
    upstream: &mut U,
    context: GenericRelayContext<'_>,
) -> GenericRelayRun
where
    C: AsyncRead + AsyncWrite + Unpin,
    U: AsyncRead + AsyncWrite + Unpin,
{
    let GenericRelayContext {
        limits,
        shutdown,
        session_id,
        connection_id,
        sink,
        body_resources,
        buffer_bytes,
        provenance,
    } = context;
    let client_bytes = AtomicU64::new(0);
    let upstream_bytes = AtomicU64::new(0);
    let observed = AtomicU64::new(0);
    let retained = AtomicU64::new(0);
    let connection_retained = Arc::new(AtomicU64::new(0));
    let activity = Notify::new();
    let client_emitter = ChunkEmitter {
        direction: GenericStreamDirection::ClientToUpstream,
        provenance,
        offset: AtomicU64::new(0),
        observed: &observed,
        retained: &retained,
        connection_retained: Arc::clone(&connection_retained),
        session_resources: body_resources.clone(),
        limits,
        session_id,
        connection_id,
        sink,
    };
    let upstream_emitter = ChunkEmitter {
        direction: GenericStreamDirection::UpstreamToClient,
        provenance,
        offset: AtomicU64::new(0),
        observed: &observed,
        retained: &retained,
        connection_retained,
        session_resources: body_resources,
        limits,
        session_id,
        connection_id,
        sink,
    };
    let relay = relay_bidirectional(
        client,
        upstream,
        buffer_bytes,
        limits.upstream.read,
        limits.upstream.write,
        &client_bytes,
        &upstream_bytes,
        &activity,
        &client_emitter,
        &upstream_emitter,
    );
    tokio::pin!(relay);
    let idle = tokio::time::sleep(limits.idle_timeout);
    tokio::pin!(idle);
    let result = loop {
        tokio::select! {
            biased;
            _ = shutdown.changed() => break Err(GenericRelayFailure::Cancelled),
            _ = activity.notified() => idle.as_mut().reset(Instant::now() + limits.idle_timeout),
            result = &mut relay => break result,
            _ = &mut idle => break Err(GenericRelayFailure::IdleTimeout),
        }
    };
    let observed_bytes = observed.load(Ordering::Relaxed);
    let retained_bytes = retained.load(Ordering::Relaxed);
    let report = GenericRelayReport {
        client_to_upstream_bytes: client_bytes.load(Ordering::Relaxed),
        upstream_to_client_bytes: upstream_bytes.load(Ordering::Relaxed),
        observed_bytes,
        retained_bytes,
        omitted_bytes: observed_bytes.saturating_sub(retained_bytes),
    };
    GenericRelayRun {
        report,
        error: result.err(),
    }
}

struct ChunkEmitter<'a> {
    direction: GenericStreamDirection,
    provenance: GenericStreamProvenance,
    offset: AtomicU64,
    observed: &'a AtomicU64,
    retained: &'a AtomicU64,
    connection_retained: Arc<AtomicU64>,
    session_resources: SessionBodyResources,
    limits: &'a ProtocolLimits,
    session_id: &'a str,
    connection_id: u64,
    sink: &'a SharedEventSink,
}

impl ChunkEmitter<'_> {
    fn observe(&self, bytes: &[u8]) {
        for source in bytes.chunks(self.limits.max_event_chunk_bytes) {
            let offset = self
                .offset
                .fetch_add(source.len() as u64, Ordering::Relaxed);
            self.observed
                .fetch_add(source.len() as u64, Ordering::Relaxed);
            let retained_len = if self.limits.capture_payloads {
                let connection_grant = crate::body::claim_retention(
                    &self.connection_retained,
                    self.limits.max_body_bytes,
                    source.len(),
                );
                let session_grant = crate::body::claim_retention(
                    &self.session_resources.retained,
                    self.limits.max_session_body_bytes,
                    connection_grant,
                );
                if session_grant < connection_grant {
                    self.connection_retained
                        .fetch_sub((connection_grant - session_grant) as u64, Ordering::Relaxed);
                }
                session_grant
            } else {
                0
            };
            self.retained
                .fetch_add(retained_len as u64, Ordering::Relaxed);
            let outcome = if !self.limits.capture_payloads {
                GenericStreamOutcome::IntentionallyOmitted
            } else if retained_len < source.len() {
                GenericStreamOutcome::RetentionLimit
            } else {
                GenericStreamOutcome::Complete
            };
            crate::application::emit(
                self.sink,
                ApplicationEvent::now(
                    self.session_id,
                    self.connection_id,
                    None,
                    None,
                    ApplicationEventKind::GenericStreamChunk(GenericStreamChunk {
                        direction: self.direction,
                        provenance: self.provenance,
                        offset,
                        observed_len: source.len() as u64,
                        bytes: bytes::Bytes::copy_from_slice(&source[..retained_len]),
                        outcome,
                    }),
                ),
            );
        }
    }
}

// Both directions share one connection budget and activity clock, while their
// readers, writers, counters, and emitters must remain distinct.
#[allow(clippy::too_many_arguments)]
async fn relay_bidirectional<C, U>(
    client: &mut C,
    upstream: &mut U,
    buffer_bytes: usize,
    upstream_read_timeout: Duration,
    upstream_write_timeout: Duration,
    client_bytes: &AtomicU64,
    upstream_bytes: &AtomicU64,
    activity: &Notify,
    client_emitter: &ChunkEmitter<'_>,
    upstream_emitter: &ChunkEmitter<'_>,
) -> Result<(), GenericRelayFailure>
where
    C: AsyncRead + AsyncWrite + Unpin,
    U: AsyncRead + AsyncWrite + Unpin,
{
    let (client_read, client_write) = tokio::io::split(client);
    let (upstream_read, upstream_write) = tokio::io::split(upstream);
    tokio::try_join!(
        relay_direction(
            client_read,
            upstream_write,
            buffer_bytes,
            None,
            Some(upstream_write_timeout),
            client_bytes,
            activity,
            client_emitter,
        ),
        relay_direction(
            upstream_read,
            client_write,
            buffer_bytes,
            Some(upstream_read_timeout),
            None,
            upstream_bytes,
            activity,
            upstream_emitter,
        ),
    )?;
    Ok(())
}

// The relay loop keeps every bound and observation authority explicit at the
// call site so a new discard path cannot bypass accounting.
#[allow(clippy::too_many_arguments)]
async fn relay_direction<R, W>(
    mut reader: R,
    mut writer: W,
    buffer_bytes: usize,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
    transferred: &AtomicU64,
    activity: &Notify,
    emitter: &ChunkEmitter<'_>,
) -> Result<(), GenericRelayFailure>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buffer = vec![0_u8; buffer_bytes];
    loop {
        let read = timed_io(read_timeout, reader.read(&mut buffer)).await?;
        if read == 0 {
            timed_io(write_timeout, writer.shutdown()).await?;
            return Ok(());
        }
        emitter.observe(&buffer[..read]);
        activity.notify_one();
        let mut written = 0;
        while written < read {
            let count = timed_io(write_timeout, writer.write(&buffer[written..read])).await?;
            if count == 0 {
                return Err(GenericRelayFailure::Transport(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "generic relay write made no progress",
                )));
            }
            written += count;
            transferred.fetch_add(count as u64, Ordering::Relaxed);
            activity.notify_one();
        }
    }
}

async fn timed_io<T>(
    budget: Option<Duration>,
    operation: impl Future<Output = io::Result<T>>,
) -> Result<T, GenericRelayFailure> {
    match budget {
        Some(budget) => timeout(budget, operation)
            .await
            .map_err(|_| GenericRelayFailure::OperationTimeout)?
            .map_err(GenericRelayFailure::Transport),
        None => operation.await.map_err(GenericRelayFailure::Transport),
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

    #[tokio::test]
    async fn generic_stream_chunks_are_bounded_and_offsets_reconcile() {
        let (mut client, mut client_peer) = tokio::io::duplex(64);
        let (mut upstream, mut upstream_peer) = tokio::io::duplex(64);
        let sink = Arc::new(Sink::default());
        let shared: SharedEventSink = Some(sink.clone());
        let limits = ProtocolLimits {
            max_event_chunk_bytes: 3,
            max_body_bytes: 5,
            max_session_body_bytes: 5,
            ..ProtocolLimits::default()
        };
        let (_send, mut shutdown) = watch::channel(false);
        let task = tokio::spawn(async move {
            client_peer.write_all(b"abcdefgh").await.unwrap();
            client_peer.shutdown().await.unwrap();
            let mut echoed = Vec::new();
            client_peer.read_to_end(&mut echoed).await.unwrap();
            echoed
        });
        let origin = tokio::spawn(async move {
            let mut bytes = Vec::new();
            upstream_peer.read_to_end(&mut bytes).await.unwrap();
            upstream_peer.write_all(&bytes).await.unwrap();
            upstream_peer.shutdown().await.unwrap();
        });
        let run = relay_generic(
            &mut client,
            &mut upstream,
            GenericRelayContext {
                limits: &limits,
                shutdown: &mut shutdown,
                session_id: "generic-test",
                connection_id: 1,
                sink: &shared,
                body_resources: SessionBodyResources::new(1),
                buffer_bytes: 4,
                provenance: GenericStreamProvenance::TcpPlaintext,
            },
        )
        .await;
        assert!(run.error.is_none(), "{:?}", run.error);
        let report = run.report;
        origin.await.unwrap();
        assert_eq!(task.await.unwrap(), b"abcdefgh");
        assert_eq!(report.observed_bytes, 16);
        assert_eq!(report.retained_bytes, 5);
        assert_eq!(report.omitted_bytes, 11);
        let events = sink.0.lock().unwrap();
        let chunks = events
            .iter()
            .filter_map(|event| match &event.kind {
                ApplicationEventKind::GenericStreamChunk(chunk) => Some(chunk),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(chunks.iter().all(|chunk| chunk.observed_len <= 3));
        for direction in [
            GenericStreamDirection::ClientToUpstream,
            GenericStreamDirection::UpstreamToClient,
        ] {
            let mut expected = 0;
            for chunk in chunks.iter().filter(|chunk| chunk.direction == direction) {
                assert_eq!(chunk.offset, expected);
                expected += chunk.observed_len;
            }
            assert_eq!(expected, 8);
        }
    }

    #[tokio::test]
    async fn operation_timeout_is_distinct_from_the_idle_clock() {
        let error = timed_io(
            Some(Duration::ZERO),
            std::future::pending::<io::Result<()>>(),
        )
        .await
        .unwrap_err();
        assert!(matches!(error, GenericRelayFailure::OperationTimeout));
    }
}
