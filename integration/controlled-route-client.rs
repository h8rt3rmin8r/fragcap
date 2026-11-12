// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(windows, windows_subsystem = "windows")]

//! Finite raw TCP client for a cold installed-product direct-route check.

use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::time::Duration;

const REQUEST: &[u8] = b"runtime-client-request";
const REPLY: &[u8] = b"runtime-origin-reply";
const FINISH: &[u8] = b"runtime-client-finish";

fn exchange(peer: SocketAddr) -> std::io::Result<()> {
    let mut stream = TcpStream::connect_timeout(&peer, Duration::from_secs(10))?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    stream.write_all(REQUEST)?;
    let mut reply = [0u8; REPLY.len()];
    stream.read_exact(&mut reply)?;
    if reply != REPLY {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "controlled origin reply differed",
        ));
    }
    stream.write_all(FINISH)?;
    stream.shutdown(Shutdown::Write)?;
    let mut trailing = [0u8; 1];
    if stream.read(&mut trailing)? != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "controlled origin sent trailing bytes",
        ));
    }
    Ok(())
}

fn main() {
    let result = std::env::var("S205_CONTROLLED_PEER")
        .ok()
        .and_then(|value| value.parse::<SocketAddr>().ok())
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing controlled peer")
        })
        .and_then(exchange);
    if result.is_err() {
        std::process::exit(2);
    }
}
