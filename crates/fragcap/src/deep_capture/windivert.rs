// SPDX-License-Identifier: Apache-2.0

//! Exact WinDivert 2.2.2-A x64 ABI, loaded only by an authorized route lease.

use std::{
    ffi::{c_void, CStr, CString},
    io, mem,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    os::windows::ffi::OsStrExt,
    path::Path,
    ptr,
    sync::Arc,
};

use windows_sys::Win32::{
    Foundation::{FILETIME, INVALID_HANDLE_VALUE},
    System::{
        LibraryLoader::{
            FreeLibrary, GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
        },
        Performance::{QueryPerformanceCounter, QueryPerformanceFrequency},
        SystemInformation::GetSystemTimePreciseAsFileTime,
    },
};

use fragcap_core::Proto;

use super::{
    direct_route::{DirectTuple, QpcAnchor},
    verify_windivert_component,
};

const FILETIME_UNIX_OFFSET_NS: i128 = 11_644_473_600_000_000_000;

pub(super) fn sample_qpc_anchor() -> io::Result<QpcAnchor> {
    let mut frequency = 0;
    let mut before = 0;
    let mut after = 0;
    if unsafe { QueryPerformanceFrequency(&mut frequency) } == 0
        || unsafe { QueryPerformanceCounter(&mut before) } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let mut filetime = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    unsafe { GetSystemTimePreciseAsFileTime(&mut filetime) };
    if unsafe { QueryPerformanceCounter(&mut after) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let ticks = (u64::from(filetime.dwHighDateTime) << 32) | u64::from(filetime.dwLowDateTime);
    let unix_ns = i64::try_from(i128::from(ticks) * 100 - FILETIME_UNIX_OFFSET_NS)
        .map_err(|_| io::Error::other("system time outside supported direct route range"))?;
    QpcAnchor::from_bracket(before, after, unix_ns, frequency)
        .ok_or_else(|| io::Error::other("QPC and wall-clock sample exceeded uncertainty bound"))
}

pub(super) const NETWORK_LAYER: i32 = 0;
pub(super) const FLOW_LAYER: i32 = 2;
pub(super) const SOCKET_LAYER: i32 = 3;
pub(super) const FLAG_SNIFF: u64 = 0x0001;
pub(super) const FLAG_RECV_ONLY: u64 = 0x0004;
const SHUTDOWN_BOTH: i32 = 3;
const SHUTDOWN_RECV: i32 = 1;

/// Matches the exact `WINDIVERT_ADDRESS` from the pinned vendor header.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(super) struct Address {
    pub timestamp_qpc: i64,
    pub flags: u32,
    pub reserved: u32,
    pub data: [u8; 64],
}

impl Default for Address {
    fn default() -> Self {
        Self {
            timestamp_qpc: 0,
            flags: 0,
            reserved: 0,
            data: [0; 64],
        }
    }
}

impl Address {
    pub fn layer(self) -> u8 {
        self.flags as u8
    }

    pub fn event(self) -> u8 {
        (self.flags >> 8) as u8
    }

    pub fn outbound(self) -> bool {
        self.flags & (1 << 17) != 0
    }

    pub fn loopback(self) -> bool {
        self.flags & (1 << 18) != 0
    }

    pub fn ipv6(self) -> bool {
        self.flags & (1 << 20) != 0
    }

    pub fn ip_checksum_valid(self) -> bool {
        self.flags & (1 << 21) != 0
    }

    pub fn socket(self) -> Option<SocketData> {
        (self.layer() == SOCKET_LAYER as u8 || self.layer() == FLOW_LAYER as u8)
            .then(|| unsafe { ptr::read_unaligned(self.data.as_ptr().cast()) })
    }
}

/// Vendor SOCKET and FLOW union member; only read for those layers.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(super) struct SocketData {
    pub endpoint_id: u64,
    pub parent_endpoint_id: u64,
    pub pid: u32,
    pub local_addr: [u32; 4],
    pub remote_addr: [u32; 4],
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: u8,
}

impl SocketData {
    fn address(words: [u32; 4], port: u16) -> SocketAddr {
        let mut octets = [0; 16];
        for (index, word) in words.into_iter().rev().enumerate() {
            octets[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
        }
        let ipv6 = Ipv6Addr::from(octets);
        let ip = ipv6
            .to_ipv4_mapped()
            .map(IpAddr::V4)
            .unwrap_or(IpAddr::V6(ipv6));
        SocketAddr::new(ip, port)
    }

    pub fn tuple(self) -> Option<DirectTuple> {
        let transport = match self.protocol {
            6 => Proto::Tcp,
            17 => Proto::Udp,
            _ => return None,
        };
        (self.local_port != 0 && self.remote_port != 0).then_some(DirectTuple {
            transport,
            local: Self::address(self.local_addr, self.local_port),
            remote: Self::address(self.remote_addr, self.remote_port),
        })
    }
}

type OpenFn = unsafe extern "system" fn(*const i8, i32, i16, u64) -> isize;
type RecvFn = unsafe extern "system" fn(isize, *mut c_void, u32, *mut u32, *mut Address) -> i32;
type SendFn = unsafe extern "system" fn(isize, *const c_void, u32, *mut u32, *const Address) -> i32;
type ShutdownFn = unsafe extern "system" fn(isize, i32) -> i32;
type CloseFn = unsafe extern "system" fn(isize) -> i32;
type SetParamFn = unsafe extern "system" fn(isize, i32, u64) -> i32;

pub(super) struct Api {
    module: isize,
    open: OpenFn,
    recv: RecvFn,
    send: SendFn,
    shutdown: ShutdownFn,
    close: CloseFn,
    set_param: SetParamFn,
}

impl Api {
    /// The caller must already hold the exact approved direct-route session plan.
    pub fn load_verified(directory: &Path) -> io::Result<Arc<Self>> {
        let path = verify_windivert_component(directory).map_err(io::Error::other)?;
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let module = unsafe { LoadLibraryExW(wide.as_ptr(), 0, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR) };
        if module == 0 {
            return Err(io::Error::last_os_error());
        }
        let loaded = unsafe { Self::from_module(module) };
        if loaded.is_err() {
            unsafe { FreeLibrary(module) };
        }
        loaded.map(Arc::new)
    }

    unsafe fn from_module(module: isize) -> io::Result<Self> {
        unsafe fn symbol<T: Copy>(module: isize, name: &str) -> io::Result<T> {
            let name = CString::new(name).map_err(io::Error::other)?;
            let proc = unsafe { GetProcAddress(module, name.as_ptr().cast()) };
            if proc.is_none() {
                return Err(io::Error::last_os_error());
            }
            Ok(unsafe { mem::transmute_copy(&proc) })
        }
        Ok(Self {
            module,
            open: unsafe { symbol(module, "WinDivertOpen")? },
            recv: unsafe { symbol(module, "WinDivertRecv")? },
            send: unsafe { symbol(module, "WinDivertSend")? },
            shutdown: unsafe { symbol(module, "WinDivertShutdown")? },
            close: unsafe { symbol(module, "WinDivertClose")? },
            set_param: unsafe { symbol(module, "WinDivertSetParam")? },
        })
    }

    pub fn open(self: &Arc<Self>, filter: &str, layer: i32, flags: u64) -> io::Result<Handle> {
        let filter = CString::new(filter).map_err(io::Error::other)?;
        self.open_cstr(&filter, layer, flags)
    }

    fn open_cstr(self: &Arc<Self>, filter: &CStr, layer: i32, flags: u64) -> io::Result<Handle> {
        let raw = unsafe { (self.open)(filter.as_ptr(), layer, 0, flags) };
        if raw == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        Ok(Handle {
            api: Arc::clone(self),
            raw,
        })
    }
}

impl Drop for Api {
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.module) };
    }
}

pub(super) struct Handle {
    api: Arc<Api>,
    raw: isize,
}

impl Handle {
    pub fn recv(&self, packet: &mut [u8], address: &mut Address) -> io::Result<usize> {
        let length = u32::try_from(packet.len()).map_err(io::Error::other)?;
        let mut received = 0;
        let ok = unsafe {
            (self.api.recv)(
                self.raw,
                packet.as_mut_ptr().cast(),
                length,
                &mut received,
                address,
            )
        };
        if ok == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(received as usize)
        }
    }

    pub fn send(&self, packet: &[u8], address: &Address) -> io::Result<usize> {
        let length = u32::try_from(packet.len()).map_err(io::Error::other)?;
        let mut sent = 0;
        let ok = unsafe {
            (self.api.send)(self.raw, packet.as_ptr().cast(), length, &mut sent, address)
        };
        if ok == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(sent as usize)
        }
    }

    pub fn set_queue(&self, packets: u64, milliseconds: u64, bytes: u64) -> io::Result<()> {
        for (parameter, value) in [(0, packets), (1, milliseconds), (2, bytes)] {
            if unsafe { (self.api.set_param)(self.raw, parameter, value) } == 0 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    pub fn stop(&self) -> io::Result<()> {
        if unsafe { (self.api.shutdown)(self.raw, SHUTDOWN_BOTH) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn stop_receiving(&self) -> io::Result<()> {
        if unsafe { (self.api.shutdown)(self.raw, SHUTDOWN_RECV) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn close(mut self) -> io::Result<()> {
        let raw = mem::replace(&mut self.raw, INVALID_HANDLE_VALUE);
        if unsafe { (self.api.close)(raw) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        if self.raw != INVALID_HANDLE_VALUE {
            unsafe { (self.api.shutdown)(self.raw, SHUTDOWN_BOTH) };
            unsafe { (self.api.close)(self.raw) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An all-TCP/UDP handle is prearmed before the sentinels run. Compare
    /// sequential loopback UDP exchanges against a paired same-run baseline.
    /// This disposable probe measures the cost of immediate pass-through; it
    /// does not establish a production loss or latency guarantee.
    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_broad_pass_through_latency() {
        use std::{
            net::{Ipv4Addr, UdpSocket},
            sync::atomic::{AtomicU64, Ordering},
            thread,
            time::{Duration, Instant},
        };

        const EXCHANGES: usize = 200;
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let api = Api::load_verified(Path::new(&root)).expect("load verified component");
        let server = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let client = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        client.connect(server.local_addr().unwrap()).unwrap();
        server.connect(client.local_addr().unwrap()).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        server
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();

        let server_thread = thread::spawn(move || {
            let mut payload = [0u8; 8];
            for _ in 0..EXCHANGES * 3 {
                let received = server.recv(&mut payload).expect("server receives sentinel");
                assert_eq!(received, payload.len());
                server.send(&payload).expect("server replies to sentinel");
            }
        });
        let run_phase = |phase: &str, offset: usize| -> Vec<Duration> {
            let mut samples = Vec::with_capacity(EXCHANGES);
            let mut reply = [0u8; 8];
            for index in offset..offset + EXCHANGES {
                let payload = (index as u64).to_be_bytes();
                let start = Instant::now();
                client.send(&payload).expect("client sends sentinel");
                client
                    .recv(&mut reply)
                    .expect("client receives sentinel reply");
                assert_eq!(reply, payload);
                samples.push(start.elapsed());
            }
            samples.sort_unstable();
            let percentile =
                |percent: usize| samples[(samples.len() * percent / 100).min(samples.len() - 1)];
            eprintln!(
                "broad pass-through {phase}: median_us={}, p99_us={}, max_us={}",
                percentile(50).as_micros(),
                percentile(99).as_micros(),
                samples.last().unwrap().as_micros()
            );
            samples
        };
        let before = run_phase("before", 0);

        let handle = Arc::new(
            api.open("tcp or udp", NETWORK_LAYER, 0)
                .expect("open broad handle"),
        );
        let forwarded = Arc::new(AtomicU64::new(0));
        let worker_handle = Arc::clone(&handle);
        let worker_count = Arc::clone(&forwarded);
        let worker = thread::spawn(move || -> io::Result<()> {
            let mut packet = [0u8; 65_535];
            loop {
                let mut address = Address::default();
                match worker_handle.recv(&mut packet, &mut address) {
                    Ok(received) => {
                        let sent = worker_handle.send(&packet[..received], &address)?;
                        if sent != received {
                            return Err(io::Error::other("short packet reinjection"));
                        }
                        worker_count.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(error) if error.raw_os_error() == Some(232) => return Ok(()),
                    Err(error) => return Err(error),
                }
            }
        });
        let during = run_phase("during", EXCHANGES);
        let parallel = (0..4)
            .map(|_| {
                thread::spawn(|| {
                    let server = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
                    let client = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
                    client.connect(server.local_addr().unwrap()).unwrap();
                    server.connect(client.local_addr().unwrap()).unwrap();
                    client
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    server
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let echo = thread::spawn(move || {
                        let mut payload = [0u8; 8];
                        for _ in 0..500 {
                            server.recv(&mut payload).expect("parallel server receive");
                            server.send(&payload).expect("parallel server reply");
                        }
                    });
                    let mut samples = Vec::with_capacity(500);
                    let mut reply = [0u8; 8];
                    for sequence in 0..500u64 {
                        let payload = sequence.to_be_bytes();
                        let start = Instant::now();
                        client.send(&payload).expect("parallel client send");
                        client.recv(&mut reply).expect("parallel client receive");
                        assert_eq!(reply, payload);
                        samples.push(start.elapsed());
                    }
                    echo.join().unwrap();
                    samples
                })
            })
            .collect::<Vec<_>>();
        let mut loaded_samples = Vec::with_capacity(2000);
        for thread in parallel {
            loaded_samples.extend(thread.join().unwrap());
        }
        loaded_samples.sort_unstable();
        eprintln!(
            "broad pass-through parallel: exchanges={}, p99_us={}, max_us={}",
            loaded_samples.len(),
            loaded_samples[loaded_samples.len() * 99 / 100].as_micros(),
            loaded_samples.last().unwrap().as_micros(),
        );
        handle.stop_receiving().expect("stop new packet diversion");
        worker
            .join()
            .unwrap()
            .expect("drain and reinject queued packets");
        let after = run_phase("after", EXCHANGES * 2);
        server_thread.join().unwrap();
        eprintln!(
            "broad pass-through: forwarded={}, before_p99_us={}, during_p99_us={}, after_p99_us={}",
            forwarded.load(Ordering::Relaxed),
            before[EXCHANGES * 99 / 100].as_micros(),
            during[EXCHANGES * 99 / 100].as_micros(),
            after[EXCHANGES * 99 / 100].as_micros(),
        );
        assert!(forwarded.load(Ordering::Relaxed) >= (EXCHANGES * 2 + 4000) as u64);
    }

    /// The packet rule is installed before connection creation, so its first
    /// SYN can be paired with a PID-bearing socket event after both arrive.
    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_broad_handle_catches_first_syn() {
        use std::{
            io::{Read, Write},
            net::{Ipv4Addr, TcpListener, TcpStream},
            sync::mpsc,
            thread,
            time::Duration,
        };

        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let api = Api::load_verified(Path::new(&root)).expect("load verified component");
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let destination = listener.local_addr().unwrap();
        let socket_filter = format!("processId == {} and event == CONNECT", std::process::id());
        let socket = Arc::new(
            api.open(&socket_filter, SOCKET_LAYER, FLAG_SNIFF | FLAG_RECV_ONLY)
                .expect("open socket observer"),
        );
        let packet = Arc::new(
            api.open("tcp or udp", NETWORK_LAYER, 0)
                .expect("prearm broad rule"),
        );
        let (event_sender, event_receiver) = mpsc::channel();
        let socket_reader = Arc::clone(&socket);
        let event_worker = thread::spawn(move || {
            let mut data = [];
            let mut address = Address::default();
            let result = socket_reader.recv(&mut data, &mut address).map(|_| address);
            event_sender.send(result).unwrap();
        });
        let (syn_sender, syn_receiver) = mpsc::channel();
        let packet_reader = Arc::clone(&packet);
        let packet_worker = thread::spawn(move || -> io::Result<()> {
            let mut bytes = [0u8; 65_535];
            let mut sent_syn = false;
            loop {
                let mut address = Address::default();
                match packet_reader.recv(&mut bytes, &mut address) {
                    Ok(length) => {
                        if !sent_syn && ipv4_tcp_syn_to(&bytes[..length], destination.port()) {
                            syn_sender
                                .send((address, ipv4_tcp_endpoints(&bytes[..length])))
                                .unwrap();
                            sent_syn = true;
                        }
                        let sent = packet_reader.send(&bytes[..length], &address)?;
                        if sent != length {
                            return Err(io::Error::other("short reinjection"));
                        }
                    }
                    Err(error) if error.raw_os_error() == Some(232) => return Ok(()),
                    Err(error) => return Err(error),
                }
            }
        });

        let mut client = TcpStream::connect_timeout(&destination, Duration::from_secs(2)).unwrap();
        let (mut server, _) = listener.accept().unwrap();
        client.write_all(b"broad-syn-probe").unwrap();
        let mut body = [0u8; 15];
        server.read_exact(&mut body).unwrap();
        assert_eq!(&body, b"broad-syn-probe");
        let (syn, syn_endpoints) = syn_receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("prearmed broad handle receives initial SYN");
        let event = event_receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("socket event arrives")
            .expect("socket observer receives event");
        let socket_data = event.socket().expect("PID-bearing socket data");
        eprintln!(
            "socket endpoint words: local={:?}, remote={:?}, local_port={}, remote_port={}, protocol={}, endpoint_id={}",
            socket_data.local_addr,
            socket_data.remote_addr,
            socket_data.local_port,
            socket_data.remote_port,
            socket_data.protocol,
            socket_data.endpoint_id,
        );
        assert_eq!(socket_data.pid, std::process::id());
        assert_eq!(socket_data.remote_port, destination.port());
        assert_eq!(
            socket_data.tuple().map(|tuple| (tuple.local, tuple.remote)),
            syn_endpoints
        );
        assert!(event.timestamp_qpc <= syn.timestamp_qpc);
        let anchor = sample_qpc_anchor().expect("bounded QPC clock sample");
        let event_time = anchor
            .convert(event.timestamp_qpc)
            .expect("socket event is inside clock anchor window");
        let syn_time = anchor
            .convert(syn.timestamp_qpc)
            .expect("SYN is inside clock anchor window");
        assert!(event_time.0 <= syn_time.0);
        eprintln!(
            "broad first SYN: socket_qpc={}, syn_qpc={}, delta_qpc={}, clock_error_ns={}",
            event.timestamp_qpc,
            syn.timestamp_qpc,
            syn.timestamp_qpc - event.timestamp_qpc,
            event_time.1,
        );
        packet.stop_receiving().unwrap();
        packet_worker.join().unwrap().unwrap();
        socket.stop().unwrap();
        event_worker.join().unwrap();
    }

    fn ipv4_tcp_syn_to(bytes: &[u8], port: u16) -> bool {
        if bytes.len() < 40 || bytes[0] >> 4 != 4 || bytes[9] != 6 {
            return false;
        }
        let transport = usize::from(bytes[0] & 0x0f) * 4;
        transport >= 20
            && bytes.len() >= transport + 14
            && u16::from_be_bytes([bytes[transport + 2], bytes[transport + 3]]) == port
            && bytes[transport + 13] & 0x02 != 0
    }

    /// Prove that the local stack can terminate the first diverted SYN and
    /// return bytes through WinDivert without involving the bound OS listener.
    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_broad_smoltcp_terminates_tcp() {
        use std::{
            io::{Read, Write},
            net::{Ipv4Addr, TcpListener, TcpStream},
            thread,
            time::{Duration, Instant as WallInstant},
        };

        use crate::deep_capture::{direct_route::DirectTuple, direct_stack::TcpFlowStack};
        use fragcap_core::Proto;

        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let api = Api::load_verified(Path::new(&root)).expect("load verified component");
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let destination = listener.local_addr().unwrap();
        let handle = Arc::new(
            api.open("tcp or udp", NETWORK_LAYER, 0)
                .expect("prearm broad rule"),
        );
        let worker_handle = Arc::clone(&handle);
        let worker = thread::spawn(move || -> io::Result<Vec<u8>> {
            let start = WallInstant::now();
            let mut packet = [0u8; 65_535];
            let mut selected = None::<DirectTuple>;
            let mut stack = None::<TcpFlowStack>;
            let mut client_bytes = Vec::new();
            let mut replied = false;
            loop {
                let mut address = Address::default();
                let length = match worker_handle.recv(&mut packet, &mut address) {
                    Ok(length) => length,
                    Err(error) if error.raw_os_error() == Some(232) => return Ok(client_bytes),
                    Err(error) => return Err(error),
                };
                let endpoints = ipv4_tcp_endpoints(&packet[..length]);
                if selected.is_none() && ipv4_tcp_syn_to(&packet[..length], destination.port()) {
                    let (local, remote) =
                        endpoints.ok_or_else(|| io::Error::other("missing SYN endpoints"))?;
                    let tuple = DirectTuple {
                        transport: Proto::Tcp,
                        local,
                        remote,
                    };
                    stack = Some(
                        TcpFlowStack::new(tuple)
                            .map_err(|error| io::Error::other(format!("stack: {error:?}")))?,
                    );
                    selected = Some(tuple);
                }
                let selected_packet = selected.is_some_and(|tuple| {
                    endpoints == Some((tuple.local, tuple.remote))
                        || endpoints == Some((tuple.remote, tuple.local))
                });
                if selected_packet {
                    let now = start.elapsed().as_millis() as i64;
                    let stack = stack.as_mut().unwrap();
                    stack
                        .ingest_diverted(&packet[..length], now, address.ip_checksum_valid())
                        .map_err(|error| io::Error::other(format!("ingest: {error:?}")))?;
                    let mut body = [0u8; 32];
                    let count = stack.receive_bytes(&mut body);
                    client_bytes.extend_from_slice(&body[..count]);
                    if !replied && client_bytes == b"stack-in" {
                        let sent = stack.send_bytes(b"stack-out", now);
                        assert_eq!(sent, 9);
                        replied = true;
                    }
                    while let Some(response) = stack.pop_egress() {
                        let sent = worker_handle.send(&response, &address)?;
                        if sent != response.len() {
                            return Err(io::Error::other("short stack reinjection"));
                        }
                    }
                } else {
                    let sent = worker_handle.send(&packet[..length], &address)?;
                    if sent != length {
                        return Err(io::Error::other("short unrelated reinjection"));
                    }
                }
            }
        });

        let mut client = TcpStream::connect_timeout(&destination, Duration::from_secs(2)).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        client.write_all(b"stack-in").unwrap();
        let mut reply = [0u8; 9];
        client.read_exact(&mut reply).unwrap();
        assert_eq!(&reply, b"stack-out");
        handle.stop_receiving().unwrap();
        assert_eq!(worker.join().unwrap().unwrap(), b"stack-in");
    }

    fn ipv4_tcp_endpoints(bytes: &[u8]) -> Option<(std::net::SocketAddr, std::net::SocketAddr)> {
        use std::net::{Ipv4Addr, SocketAddr};
        if bytes.len() < 40 || bytes[0] >> 4 != 4 || bytes[9] != 6 {
            return None;
        }
        let offset = usize::from(bytes[0] & 0x0f) * 4;
        if offset < 20 || bytes.len() < offset + 20 {
            return None;
        }
        let source = SocketAddr::from((
            Ipv4Addr::new(bytes[12], bytes[13], bytes[14], bytes[15]),
            u16::from_be_bytes([bytes[offset], bytes[offset + 1]]),
        ));
        let destination = SocketAddr::from((
            Ipv4Addr::new(bytes[16], bytes[17], bytes[18], bytes[19]),
            u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]),
        ));
        Some((source, destination))
    }

    #[test]
    fn pinned_abi_layout_and_flag_decode() {
        assert_eq!(mem::size_of::<Address>(), 80);
        assert_eq!(mem::align_of::<Address>(), 8);
        assert_eq!(mem::size_of::<SocketData>(), 64);
        let address = Address {
            flags: 3 | (4 << 8) | (1 << 17) | (1 << 20),
            ..Address::default()
        };
        assert_eq!(address.layer(), 3);
        assert_eq!(address.event(), 4);
        assert!(address.outbound());
        assert!(address.ipv6());
        assert!(!address.loopback());
        let socket = SocketData {
            endpoint_id: 1,
            parent_endpoint_id: 0,
            pid: 2,
            local_addr: [0x7f00_0001, 0x0000_ffff, 0, 0],
            remote_addr: [0xc000_0201, 0x0000_ffff, 0, 0],
            local_port: 40000,
            remote_port: 443,
            protocol: 6,
        };
        assert_eq!(
            socket.tuple(),
            Some(DirectTuple {
                transport: Proto::Tcp,
                local: "127.0.0.1:40000".parse().unwrap(),
                remote: "192.0.2.1:443".parse().unwrap(),
            })
        );
    }

    /// Disposable hosted-runner probe. Both observers sniff without diverting.
    /// It is intentionally ignored by ordinary tests because loading the
    /// signed kernel component requires Administrator rights.
    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_socket_event_precedes_loopback_syn() {
        use std::{
            io::{Read, Write},
            net::{Ipv4Addr, TcpListener, TcpStream},
            sync::mpsc,
            thread,
            time::Duration,
        };

        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let api = Api::load_verified(Path::new(&root)).expect("load verified component");
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let socket_filter = format!("processId == {} and event == CONNECT", std::process::id());
        let socket = Arc::new(
            api.open(&socket_filter, SOCKET_LAYER, FLAG_SNIFF | FLAG_RECV_ONLY)
                .expect("open nonblocking socket observer"),
        );
        let packet_filter = format!("loopback and tcp.Syn and tcp.DstPort == {port}");
        let packet = Arc::new(
            api.open(&packet_filter, NETWORK_LAYER, FLAG_SNIFF | FLAG_RECV_ONLY)
                .expect("open nonblocking packet observer"),
        );
        let (sender, receiver) = mpsc::channel();
        let socket_reader = Arc::clone(&socket);
        let socket_sender = sender.clone();
        let socket_thread = thread::spawn(move || {
            let mut data = [];
            let mut address = Address::default();
            let result = socket_reader.recv(&mut data, &mut address).map(|_| address);
            socket_sender.send(("socket", result)).unwrap();
        });
        let packet_reader = Arc::clone(&packet);
        let packet_thread = thread::spawn(move || {
            let mut data = [0u8; 65535];
            let mut address = Address::default();
            let result = packet_reader.recv(&mut data, &mut address).map(|_| address);
            sender.send(("packet", result)).unwrap();
        });

        let mut client = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
        let (mut server, _) = listener.accept().unwrap();
        client.write_all(b"direct-route-timing").unwrap();
        let mut received = [0u8; 19];
        server.read_exact(&mut received).unwrap();
        assert_eq!(&received, b"direct-route-timing");

        let first = receiver.recv_timeout(Duration::from_secs(5));
        let second = receiver.recv_timeout(Duration::from_secs(5));
        let _ = socket.stop();
        let _ = packet.stop();
        socket_thread.join().unwrap();
        packet_thread.join().unwrap();
        let mut socket_qpc = None;
        let mut packet_qpc = None;
        for (kind, result) in [
            first.expect("first observer"),
            second.expect("second observer"),
        ] {
            let address = result.expect("observer received an event");
            if kind == "socket" {
                assert_eq!(address.event(), 4);
                let event = address.socket().expect("socket data");
                assert_eq!(event.pid, std::process::id());
                socket_qpc = Some(address.timestamp_qpc);
            } else {
                assert_eq!(address.layer(), NETWORK_LAYER as u8);
                packet_qpc = Some(address.timestamp_qpc);
            }
        }
        let socket_qpc = socket_qpc.unwrap();
        let packet_qpc = packet_qpc.unwrap();
        eprintln!(
            "direct-route timing: socket_qpc={socket_qpc}, syn_qpc={packet_qpc}, delta_qpc={}",
            packet_qpc - socket_qpc
        );
        assert!(
            socket_qpc <= packet_qpc,
            "socket event followed the first SYN"
        );
    }

    /// The network handle opens only after the PID-bearing SOCKET event.
    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn diagnostic_narrow_handle_misses_initial_syn() {
        use std::{
            io::{Read, Write},
            net::{Ipv4Addr, TcpListener, TcpStream},
            sync::Mutex,
            thread,
            time::Duration,
        };

        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let api = Api::load_verified(Path::new(&root)).expect("load verified component");
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let destination = listener.local_addr().unwrap();
        let socket_filter = format!("processId == {} and event == CONNECT", std::process::id());
        let socket = Arc::new(
            api.open(&socket_filter, SOCKET_LAYER, FLAG_SNIFF | FLAG_RECV_ONLY)
                .expect("open nonblocking socket observer"),
        );
        let filter = CString::new(format!(
            "loopback and tcp.Syn and tcp.DstPort == {}",
            destination.port()
        ))
        .unwrap();
        let narrow = Arc::new(Mutex::new(None::<Arc<Handle>>));
        let narrow_for_thread = Arc::clone(&narrow);
        let socket_for_thread = Arc::clone(&socket);
        let worker = thread::spawn(move || -> io::Result<(i64, i64)> {
            let mut empty = [];
            let mut event = Address::default();
            socket_for_thread.recv(&mut empty, &mut event)?;
            let handle = Arc::new(api.open_cstr(&filter, NETWORK_LAYER, 0)?);
            *narrow_for_thread.lock().unwrap() = Some(Arc::clone(&handle));
            let details = event
                .socket()
                .ok_or_else(|| io::Error::other("missing socket data"))?;
            eprintln!(
                "direct-route narrow event: qpc={}, local_port={}, remote_port={}, desired_remote={}, handle_opened=true",
                event.timestamp_qpc,
                details.local_port,
                details.remote_port,
                destination.port()
            );
            let mut packet = [0u8; 65_535];
            let mut address = Address::default();
            let received = handle.recv(&mut packet, &mut address)?;
            let sent = handle.send(&packet[..received], &address)?;
            assert_eq!(sent, received);
            Ok((event.timestamp_qpc, address.timestamp_qpc))
        });

        let mut client = TcpStream::connect_timeout(&destination, Duration::from_secs(5))
            .expect("the selected SYN must be reinjected or bypass cleanly");
        let (mut server, _) = listener.accept().unwrap();
        client.write_all(b"narrow-route-probe").unwrap();
        let mut received = [0u8; 18];
        server.read_exact(&mut received).unwrap();
        assert_eq!(&received, b"narrow-route-probe");
        thread::sleep(Duration::from_millis(500));
        let _ = socket.stop();
        let opened = narrow.lock().unwrap().is_some();
        if let Some(handle) = narrow.lock().unwrap().as_ref() {
            let _ = handle.stop();
        }
        assert!(
            opened,
            "the PID-bearing socket event did not open a narrow handle"
        );
        let (socket_qpc, syn_qpc) = worker
            .join()
            .unwrap()
            .expect("narrow first SYN was received");
        eprintln!("direct-route narrow timing: socket_qpc={socket_qpc}, syn_qpc={syn_qpc}");
        assert!(socket_qpc <= syn_qpc);
    }
}
