// SPDX-License-Identifier: Apache-2.0

//! The Unix domain socket streaming transport (specification section 14.2).
//!
//! Present for parity and for future platform support. Available only where the
//! platform provides `std::os::unix::net`; on Windows the `unix:` scheme is
//! refused at configuration time. Same shape as the TCP transport: one consumer
//! per accepted connection, with the same per-consumer backpressure.

use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use super::{Acceptor, Connection, PollingShutdown, PollingWriter, Stopper, WRITE_POLL_INTERVAL};

/// How long the acceptor parks between non-blocking accept polls.
const ACCEPT_POLL: Duration = Duration::from_millis(20);

/// A Unix domain socket listener that yields each connection as a consumer.
pub struct UnixAcceptor {
    listener: UnixListener,
    path: PathBuf,
    stop: Arc<AtomicBool>,
    next_ordinal: AtomicU64,
}

impl UnixAcceptor {
    /// Bind the socket at `path`. A bind failure is returned here, before
    /// capture starts.
    pub fn bind(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let listener = UnixListener::bind(&path)?;
        listener.set_nonblocking(true)?;
        Ok(UnixAcceptor {
            listener,
            path,
            stop: Arc::new(AtomicBool::new(false)),
            next_ordinal: AtomicU64::new(0),
        })
    }
}

impl Drop for UnixAcceptor {
    fn drop(&mut self) {
        // Remove the socket file so a later bind at the same path succeeds.
        let _ = std::fs::remove_file(&self.path);
    }
}

impl Acceptor for UnixAcceptor {
    fn accept(&mut self) -> Option<Connection> {
        loop {
            if self.stop.load(Ordering::Acquire) {
                return None;
            }
            match self.listener.accept() {
                Ok((stream, _addr)) => {
                    if stream.set_nonblocking(false).is_err() {
                        continue;
                    }
                    if stream.set_write_timeout(Some(WRITE_POLL_INTERVAL)).is_err() {
                        continue;
                    }
                    let stop = Arc::new(AtomicBool::new(false));
                    let ordinal = self.next_ordinal.fetch_add(1, Ordering::Relaxed);
                    let id = format!("unix:{}#{ordinal}", self.path.display());
                    return Some(Connection {
                        id,
                        writer: Box::new(PollingWriter::new(stream, Arc::clone(&stop))),
                        shutdown: Box::new(PollingShutdown(stop)),
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(ACCEPT_POLL);
                }
                Err(_) => std::thread::sleep(ACCEPT_POLL),
            }
        }
    }

    fn stopper(&self) -> Stopper {
        let stop = Arc::clone(&self.stop);
        Arc::new(move || stop.store(true, Ordering::Release))
    }

    fn describe(&self) -> String {
        format!("unix:{}", self.path.display())
    }
}
