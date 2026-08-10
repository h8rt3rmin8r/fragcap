// SPDX-License-Identifier: Apache-2.0

//! Transports and streaming sinks (specification sections 14.1 to 14.4).
//!
//! The format writers ([`crate::pcapng`], [`crate::json`]) say what bytes a
//! capture becomes; a transport says where those bytes go. The two are
//! orthogonal by construction here: a [`SinkFactory`] builds a fresh format
//! encoder over any [`Write`], so any format writes to any transport.
//!
//! Three shapes live under this module.
//!
//! [`file::RotatingFileSink`] writes to a path and, when a rotation policy is
//! set, closes the current segment at a clean section boundary and opens the
//! next numbered one. With no policy it is a single segment, byte identical to
//! the file sink S06 and S07 produced.
//!
//! [`stream::StreamSink`] serves any number of consumers. Each connected
//! consumer gets its own factory-built encoder (its own Section Header Block
//! and Interface Description Blocks, replayed on connect, per specification
//! 14.3) and its own bounded queue. A consumer that stops reading has packets
//! dropped on its own connection only, counted per consumer; it never stalls
//! the capture or any other sink (specification 14.4, constitution P-4). The
//! streaming sink's [`Sink::write`] always returns success, so the pipeline's
//! conservation invariant is preserved and the sink is never retired for a slow
//! downstream reader.
//!
//! The three connection acceptors feed that one streaming sink:
//! [`tcp::TcpAcceptor`] everywhere, [`pipe::NamedPipeAcceptor`] on Windows, and
//! [`unix::UnixAcceptor`] on Unix. Each yields a [`Connection`]; the streaming
//! machinery is transport-agnostic above that seam.

pub mod file;
pub mod stream;
pub mod tcp;

#[cfg(windows)]
pub mod pipe;

#[cfg(unix)]
pub mod unix;

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use fragcap_core::traits::Sink;
use fragcap_core::LinkType;

use crate::error::WriteError;
use crate::json::{JsonLinesWriter, PayloadMode};
use crate::pcapng::interface::InterfaceDeclaration;
use crate::pcapng::PcapngWriter;

/// Which output format an encoder produces. Orthogonal to the transport it
/// writes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// pcapng, carrying attribution in packet comments.
    Pcapng,
    /// JSON Lines, one record per packet.
    JsonLines(PayloadMode),
}

/// One declared capture interface: the name and link type a new encoder
/// replays into its header, plus the snap length the file writers already use.
#[derive(Clone, Debug)]
pub struct InterfaceSpec {
    pub name: Arc<str>,
    pub link_type: LinkType,
    pub snap_len: u32,
}

impl InterfaceSpec {
    pub fn new(name: impl AsRef<str>, link_type: LinkType, snap_len: u32) -> Self {
        InterfaceSpec {
            name: Arc::from(name.as_ref()),
            link_type,
            snap_len,
        }
    }
}

/// Constructs a fresh format encoder over a supplied connection, writing the
/// format's header preamble before returning.
///
/// This is the seam that keeps format orthogonal to transport. A streaming
/// consumer's encoder and a file segment's encoder are both produced here, so a
/// mid-capture joiner and a fresh rotation segment each begin with their own
/// valid header (constitution P-5).
#[derive(Clone, Debug)]
pub struct SinkFactory {
    format: Format,
    interfaces: Vec<InterfaceSpec>,
}

impl SinkFactory {
    pub fn new(format: Format, interfaces: Vec<InterfaceSpec>) -> Self {
        SinkFactory { format, interfaces }
    }

    /// The format this factory builds.
    pub fn format(&self) -> Format {
        self.format
    }

    /// Build a fresh encoder over `conn`, writing its header preamble.
    ///
    /// For pcapng that is the Section Header Block (written by
    /// [`PcapngWriter::new`]) followed by one Interface Description Block per
    /// declared interface; for JSON Lines it is the header record. The returned
    /// value is fed packets with [`Sink::write`] and finalized with
    /// [`Sink::finish`] exactly as any sink.
    pub fn build(&self, conn: Box<dyn Write + Send>) -> Result<Box<dyn Sink>, WriteError> {
        match self.format {
            Format::Pcapng => {
                let mut writer = PcapngWriter::new(conn)?;
                for iface in &self.interfaces {
                    writer.declare_interface(&InterfaceDeclaration::new(
                        iface.link_type,
                        iface.snap_len,
                        iface.name.as_ref(),
                    ))?;
                }
                Ok(Box::new(writer))
            }
            Format::JsonLines(mode) => {
                let names: Vec<&str> = self.interfaces.iter().map(|i| i.name.as_ref()).collect();
                let writer = JsonLinesWriter::new(conn, &names, mode)?;
                Ok(Box::new(writer))
            }
        }
    }
}

/// Unblocks a consumer's connection from outside its writer thread.
///
/// A consumer whose reader has stopped leaves its writer thread parked in a
/// blocking socket or pipe write. When the streaming sink disconnects that
/// consumer on the backpressure timeout (specification 14.4) or at capture end,
/// it calls [`ConnShutdown::shutdown`] to unblock the write so the thread can
/// exit and be reaped. The connection is closed by the writer thread's `Drop`;
/// shutdown only unblocks, so the two do not double-close.
pub trait ConnShutdown: Send + Sync {
    fn shutdown(&self);
}

/// How long a socket write blocks before returning so the stop flag can be
/// checked. This is an internal poll interval, not the disconnect timeout: the
/// disconnect decision lives in the streaming sink, and this only bounds how
/// quickly a stalled writer notices it has been told to stop.
pub(crate) const WRITE_POLL_INTERVAL: Duration = Duration::from_millis(200);

/// A stop flag shared between a [`PollingWriter`] and its [`PollingShutdown`].
pub(crate) type StopFlag = Arc<AtomicBool>;

/// Wraps a socket-backed writer so the streaming sink can unblock it portably.
///
/// The inner stream is given a modest write timeout ([`WRITE_POLL_INTERVAL`]),
/// so a write to a stalled reader returns periodically rather than blocking
/// forever. On each return this rechecks the stop flag: if the sink has tripped
/// it, the write aborts with `Interrupted`; otherwise it retries. This does not
/// depend on `shutdown()` unblocking a blocked send, which is not portable
/// across platforms, so a stalled consumer is always reaped within one poll
/// interval on every target.
pub(crate) struct PollingWriter<W: Write + Send> {
    inner: W,
    stop: StopFlag,
}

impl<W: Write + Send> PollingWriter<W> {
    pub(crate) fn new(inner: W, stop: StopFlag) -> Self {
        PollingWriter { inner, stop }
    }
}

impl<W: Write + Send> Write for PollingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        loop {
            if self.stop.load(Ordering::Acquire) {
                // Not `Interrupted`: `write_all` retries that kind, which would
                // loop forever here. `ConnectionAborted` propagates so the
                // consumer's write ends and its thread exits.
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "consumer stopped",
                ));
            }
            match self.inner.write(buf) {
                Ok(n) => return Ok(n),
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    // The write timed out with no progress; recheck the stop
                    // flag and retry.
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        loop {
            if self.stop.load(Ordering::Acquire) {
                return Ok(());
            }
            match self.inner.flush() {
                Ok(()) => return Ok(()),
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => return Err(e),
            }
        }
    }
}

/// Trips a [`PollingWriter`]'s stop flag so a stalled writer thread exits within
/// one poll interval.
pub(crate) struct PollingShutdown(pub(crate) StopFlag);

impl ConnShutdown for PollingShutdown {
    fn shutdown(&self) {
        self.0.store(true, Ordering::Release);
    }
}

/// A live consumer connection handed up by an [`Acceptor`].
pub struct Connection {
    /// A stable identity for logs and per-consumer accounting.
    pub id: String,
    /// The byte stream the consumer's encoder writes to.
    pub writer: Box<dyn Write + Send>,
    /// The handle used to unblock the writer on disconnect.
    pub shutdown: Box<dyn ConnShutdown>,
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection").field("id", &self.id).finish()
    }
}

/// A cloneable handle that signals an [`Acceptor`] to stop and unblocks any
/// pending `accept`.
pub type Stopper = Arc<dyn Fn() + Send + Sync>;

/// A bound transport that yields consumer connections until told to stop.
///
/// The streaming sink drives one acceptor on its own thread, registering each
/// [`Connection`] as a consumer. `accept` blocks until the next connection or
/// returns `None` once the [`Stopper`] has been invoked and any pending accept
/// has been unblocked. The stopper is read before the acceptor is moved onto
/// its thread, so the sink can stop it from outside.
pub trait Acceptor: Send {
    /// Block until the next connection, or return `None` when stopping.
    fn accept(&mut self) -> Option<Connection>;

    /// A handle that stops this acceptor when invoked. Must be safe to call
    /// more than once and from another thread.
    fn stopper(&self) -> Stopper;

    /// The bound address or name, for diagnostics.
    fn describe(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    use fragcap_core::interface::InterfaceId;
    use fragcap_core::packet::{CapturedPacket, Payload, RawPacket, Timestamp};

    /// A byte sink a test can hold one end of while the built sink writes the
    /// other. The factory takes a `Box<dyn Write + Send>`, so the shared buffer
    /// lives behind an `Arc<Mutex<..>>`.
    #[derive(Clone, Default)]
    struct SharedBuf(Arc<Mutex<Vec<u8>>>);

    impl SharedBuf {
        fn take(&self) -> Vec<u8> {
            std::mem::take(&mut *self.0.lock().unwrap())
        }
    }

    impl Write for SharedBuf {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn a_packet() -> CapturedPacket {
        let raw = RawPacket::new(
            Timestamp::from_nanos(1_000),
            Payload::from(vec![0x11u8; 8]),
            8,
        );
        CapturedPacket::from_raw(raw, InterfaceId::default())
    }

    #[test]
    fn pcapng_factory_writes_a_valid_preamble_then_a_packet() {
        let factory = SinkFactory::new(
            Format::Pcapng,
            vec![InterfaceSpec::new("eth0", LinkType::ETHERNET, 262_144)],
        );
        let buf = SharedBuf::default();
        let mut sink = factory.build(Box::new(buf.clone())).expect("build");
        sink.write(&a_packet()).expect("write");
        let out = buf.take();
        // Section Header Block type is 0x0A0D0D0A at offset 0 (little-endian).
        assert_eq!(&out[0..4], &0x0A0D_0D0Au32.to_le_bytes());
        // The SHB, one IDB, and one EPB were all written.
        assert!(out.len() > 60);
    }

    #[test]
    fn jsonl_factory_writes_a_header_then_a_record() {
        let factory = SinkFactory::new(
            Format::JsonLines(PayloadMode::MetadataOnly),
            vec![InterfaceSpec::new("eth0", LinkType::ETHERNET, 262_144)],
        );
        let buf = SharedBuf::default();
        let mut sink = factory.build(Box::new(buf.clone())).expect("build");
        sink.write(&a_packet()).expect("write");
        let text = String::from_utf8(buf.take()).expect("utf8");
        assert!(text.starts_with("{\"type\":\"header\""));
        assert!(text.lines().count() >= 2);
    }
}
