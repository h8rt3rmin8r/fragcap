// SPDX-License-Identifier: Apache-2.0

//! Windows named-pipe streaming (specification 14.2, 14.3; user story US1).
//! Tier 2: runs on a Windows machine (a real pipe instance and a real client),
//! needing no capture driver, no elevation, and no game.

#![cfg(windows)]

mod common;

use std::ffi::c_void;
use std::time::{Duration, Instant};

use common::{assert_valid_pcapng_stream, epb_payloads, expected_payloads, packets, walk};

use fragcap_core::stats::CaptureStats;
use fragcap_core::traits::Sink;
use fragcap_core::LinkType;
use fragcap_sink::{Format, InterfaceSpec, NamedPipeAcceptor, SinkFactory, StreamSink};

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};

const GENERIC_READ: u32 = 0x8000_0000;
const SNAP: u32 = 262_144;

fn factory() -> SinkFactory {
    SinkFactory::new(
        Format::Pcapng,
        vec![InterfaceSpec::new("eth0", LinkType::ETHERNET, SNAP)],
    )
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// A named-pipe client that connects, then reads to end of stream.
struct PipeClient(HANDLE);

impl PipeClient {
    /// Connect, retrying while the server's instance is briefly absent.
    fn connect(name: &str) -> Self {
        let path = wide(&format!(r"\\.\pipe\{name}"));
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let handle = unsafe {
                CreateFileW(
                    path.as_ptr(),
                    GENERIC_READ,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    std::ptr::null(),
                    OPEN_EXISTING,
                    0,
                    0,
                )
            };
            if handle != INVALID_HANDLE_VALUE {
                return PipeClient(handle);
            }
            assert!(Instant::now() < deadline, "could not connect to the pipe");
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn read_to_end(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut buf = [0u8; 8192];
        loop {
            let mut read: u32 = 0;
            let ok = unsafe {
                ReadFile(
                    self.0,
                    buf.as_mut_ptr() as *mut c_void,
                    buf.len() as u32,
                    &mut read,
                    std::ptr::null_mut(),
                )
            };
            if ok == 0 || read == 0 {
                // Zero bytes or a broken-pipe error is end of stream.
                break;
            }
            out.extend_from_slice(&buf[..read as usize]);
        }
        out
    }
}

impl Drop for PipeClient {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

fn pipe_name(tag: &str) -> String {
    format!("fragcap-test-{}-{tag}", std::process::id())
}

fn wait_registered(sink: &StreamSink, n: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while sink.active_consumers() < n {
        assert!(Instant::now() < deadline, "consumer did not register");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn a_pipe_client_receives_a_valid_stream() {
    let name = pipe_name("single");
    let acceptor = NamedPipeAcceptor::bind(&name).expect("bind pipe");
    let mut sink = StreamSink::new(factory(), Box::new(acceptor));

    let client = PipeClient::connect(&name);
    wait_registered(&sink, 1);

    let pkts = packets(6, 64);
    for p in &pkts {
        sink.write(p).expect("write");
    }
    Box::new(sink)
        .finish(&CaptureStats::default())
        .expect("finish");

    let bytes = client.read_to_end();
    assert_valid_pcapng_stream(&bytes, 1);
    assert_eq!(epb_payloads(&walk(&bytes)), expected_payloads(&pkts));
}

#[test]
fn two_pipe_clients_each_receive_a_valid_stream() {
    let name = pipe_name("multi");
    let acceptor = NamedPipeAcceptor::bind(&name).expect("bind pipe");
    let mut sink = StreamSink::new(factory(), Box::new(acceptor));

    let a = PipeClient::connect(&name);
    wait_registered(&sink, 1);
    let b = PipeClient::connect(&name);
    wait_registered(&sink, 2);

    let pkts = packets(8, 64);
    for p in &pkts {
        sink.write(p).expect("write");
    }
    Box::new(sink)
        .finish(&CaptureStats::default())
        .expect("finish");

    for client in [a, b] {
        let bytes = client.read_to_end();
        assert_valid_pcapng_stream(&bytes, 1);
        assert_eq!(epb_payloads(&walk(&bytes)), expected_payloads(&pkts));
    }
}
