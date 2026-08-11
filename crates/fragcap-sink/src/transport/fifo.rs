// SPDX-License-Identifier: Apache-2.0

//! Opening the analyzer's FIFO for the extcap capture (specification section
//! 14.5).
//!
//! extcap hands fragcap a path to stream pcapng to. On Windows it is a named
//! pipe the analyzer created, so fragcap connects to it as a client; on other
//! targets it is a FIFO the analyzer created, opened for writing. A path that is
//! neither (a plain file) is opened for writing and truncated, which is what lets
//! the extcap capture be driven at tier 1 against a regular temp file with no
//! named-pipe server and no blocking reader.
//!
//! This module only opens a writer. The pcapng bytes come from the unchanged
//! [`crate::pcapng`] writer built over that writer through a
//! [`super::SinkFactory`], so the FIFO stream is byte-identical to a file capture
//! (constitution P-5). It never reads and never transmits on a socket.

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

/// Open the analyzer-supplied FIFO or named-pipe path for writing.
///
/// On Windows a path under `\\.\pipe\` (or `\\?\pipe\`) is opened as a named-pipe
/// client: the analyzer owns the pipe, so nothing is created, and a momentarily
/// busy pipe is retried for a bounded time. Any other path is opened for writing,
/// created and truncated, which covers a Unix FIFO the analyzer created and a
/// regular file a test streams to.
pub fn open_fifo(path: &Path) -> io::Result<Box<dyn Write + Send>> {
    #[cfg(windows)]
    {
        if is_named_pipe_path(path) {
            return open_named_pipe_client(path);
        }
    }
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    Ok(Box::new(file))
}

/// Whether the path names a Windows named pipe, which is opened as a client
/// rather than created.
#[cfg(windows)]
fn is_named_pipe_path(path: &Path) -> bool {
    let text = path.to_string_lossy().replace('/', "\\");
    let lower = text.to_ascii_lowercase();
    lower.starts_with("\\\\.\\pipe\\") || lower.starts_with("\\\\?\\pipe\\")
}

/// Connect to an existing named pipe as a client, retrying while it is busy.
///
/// The analyzer creates and owns the pipe, so this opens it for writing without
/// creating it. A pipe with no free instance yet returns `ERROR_PIPE_BUSY`; that
/// is retried for a bounded time rather than failing the capture on a startup
/// race with the analyzer.
#[cfg(windows)]
fn open_named_pipe_client(path: &Path) -> io::Result<Box<dyn Write + Send>> {
    // ERROR_PIPE_BUSY: all pipe instances are busy. The analyzer will free one.
    const ERROR_PIPE_BUSY: i32 = 231;
    const MAX_ATTEMPTS: u32 = 40;
    const RETRY: std::time::Duration = std::time::Duration::from_millis(50);

    let mut attempt: u32 = 0;
    loop {
        match OpenOptions::new().write(true).open(path) {
            Ok(file) => return Ok(Box::new(file)),
            Err(error) if error.raw_os_error() == Some(ERROR_PIPE_BUSY) => {
                attempt += 1;
                if attempt >= MAX_ATTEMPTS {
                    return Err(error);
                }
                std::thread::sleep(RETRY);
            }
            Err(error) => return Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{Format, InterfaceSpec, SinkFactory};
    use fragcap_core::stats::CaptureStats;
    use fragcap_core::LinkType;

    #[test]
    fn open_fifo_over_a_regular_path_streams_a_pcapng_preamble() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("fifo.fcapng");

        let writer = open_fifo(&path).expect("open the fifo path");
        let factory = SinkFactory::new(
            Format::Pcapng,
            vec![InterfaceSpec::new("capture", LinkType::ETHERNET, 65_535)],
        );
        let sink = factory
            .build(writer)
            .expect("build a pcapng encoder over the fifo");
        sink.finish(&CaptureStats::default()).expect("finish");

        let bytes = std::fs::read(&path).expect("read the fifo file back");
        // The Section Header Block type is 0x0A0D0D0A at offset 0 (little-endian),
        // so an unmodified analyzer opens the stream (constitution P-5).
        assert_eq!(&bytes[0..4], &0x0A0D_0D0Au32.to_le_bytes());
    }
}
