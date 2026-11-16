// SPDX-License-Identifier: Apache-2.0

#![windows_subsystem = "windows"]

use std::{
    io::{self, Read, Write},
    net::{Shutdown, SocketAddr, TcpStream},
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

fn run() -> io::Result<()> {
    let peer: SocketAddr = std::env::var("S205_CONTROLLED_PEER")
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "missing controlled peer"))?
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid controlled peer"))?;
    let gate =
        PathBuf::from(std::env::var_os("S205_CONTROLLED_GATE").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "missing controlled gate")
        })?);
    let mut stream = TcpStream::connect_timeout(&peer, Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    stream.write_all(b"runtime-client-request")?;
    let mut reply = [0; 20];
    stream.read_exact(&mut reply)?;
    if &reply != b"runtime-origin-reply" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected reply",
        ));
    }
    std::fs::write(&gate, b"reply-received")?;
    let release = gate.with_extension("release");
    let deadline = Instant::now() + Duration::from_secs(30);
    while !release.is_file() {
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "controlled release timed out",
            ));
        }
        thread::sleep(Duration::from_millis(20));
    }
    stream.write_all(b"runtime-client-finish")?;
    stream.shutdown(Shutdown::Write)?;
    let mut tail = Vec::new();
    stream.read_to_end(&mut tail)?;
    if !tail.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected response tail",
        ));
    }
    Ok(())
}

fn main() {
    if run().is_err() {
        std::process::exit(1);
    }
}
