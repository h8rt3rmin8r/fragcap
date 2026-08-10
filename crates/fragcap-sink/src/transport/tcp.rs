// SPDX-License-Identifier: Apache-2.0

//! The TCP streaming transport (specification section 14.2).
//!
//! Listens on an address and port and hands each accepted connection to the
//! streaming sink as a consumer. Cross-platform: this is the transport a
//! consumer that cannot reach a local pipe uses, including one in a container
//! or on another host.

use std::net::{SocketAddr, TcpListener};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use super::{Acceptor, Connection, PollingShutdown, PollingWriter, Stopper, WRITE_POLL_INTERVAL};

/// How long the acceptor parks between non-blocking accept polls.
const ACCEPT_POLL: Duration = Duration::from_millis(20);

/// A TCP listener that yields each connection as a consumer.
pub struct TcpAcceptor {
    listener: TcpListener,
    local_addr: SocketAddr,
    stop: Arc<AtomicBool>,
    next_ordinal: AtomicU64,
}

impl TcpAcceptor {
    /// Bind the listener. A bind failure (address in use, permission) is
    /// returned here, before capture starts.
    pub fn bind(addr: &str) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        let local_addr = listener.local_addr()?;
        Ok(TcpAcceptor {
            listener,
            local_addr,
            stop: Arc::new(AtomicBool::new(false)),
            next_ordinal: AtomicU64::new(0),
        })
    }

    /// The bound address, useful after binding to port 0.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}

impl Acceptor for TcpAcceptor {
    fn accept(&mut self) -> Option<Connection> {
        loop {
            if self.stop.load(Ordering::Acquire) {
                return None;
            }
            match self.listener.accept() {
                Ok((stream, peer)) => {
                    // The connection is blocking for its writer thread, with a
                    // short write timeout so a stalled write returns for the
                    // stop check; only the listener is non-blocking.
                    if stream.set_nonblocking(false).is_err() {
                        continue;
                    }
                    if stream.set_write_timeout(Some(WRITE_POLL_INTERVAL)).is_err() {
                        continue;
                    }
                    let stop = Arc::new(AtomicBool::new(false));
                    let ordinal = self.next_ordinal.fetch_add(1, Ordering::Relaxed);
                    let id = format!("tcp:{peer}#{ordinal}");
                    return Some(Connection {
                        id,
                        writer: Box::new(PollingWriter::new(stream, Arc::clone(&stop))),
                        shutdown: Box::new(PollingShutdown(stop)),
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(ACCEPT_POLL);
                }
                // A transient accept error is not fatal to the listener; keep
                // serving. The stop flag is the only way out.
                Err(_) => std::thread::sleep(ACCEPT_POLL),
            }
        }
    }

    fn stopper(&self) -> Stopper {
        let stop = Arc::clone(&self.stop);
        Arc::new(move || stop.store(true, Ordering::Release))
    }

    fn describe(&self) -> String {
        format!("tcp://{}", self.local_addr)
    }
}
