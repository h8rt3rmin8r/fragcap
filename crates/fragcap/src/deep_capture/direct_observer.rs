// SPDX-License-Identifier: Apache-2.0

//! Bounded passive socket-event observer for direct route candidate joins.

use std::{
    collections::VecDeque,
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use super::{
    direct_route::{DirectSocketEvent, SocketEventKind},
    windivert::{
        sample_qpc_anchor, Address, Api, Handle, FLAG_RECV_ONLY, FLAG_SNIFF, SOCKET_LAYER,
    },
};

const SOCKET_FILTER: &str = "(event == CONNECT or event == CLOSE) and (tcp or udp)";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SocketObserverStats {
    pub received: u64,
    pub retained: u64,
    pub capacity_lost: u64,
    pub clock_lost: u64,
}

#[derive(Default)]
struct Counters {
    received: AtomicU64,
    retained: AtomicU64,
    capacity_lost: AtomicU64,
    clock_lost: AtomicU64,
}

/// The observer opens before the managed root, keeps no payload, and publishes
/// only a finite, caller-drained queue of connect events.
pub struct DirectSocketObserver {
    handle: Arc<Handle>,
    events: Arc<Mutex<VecDeque<DirectSocketEvent>>>,
    counters: Arc<Counters>,
    worker: Option<JoinHandle<io::Result<()>>>,
}

impl DirectSocketObserver {
    pub fn start(api: &Arc<Api>, capacity: usize) -> io::Result<Self> {
        if capacity == 0 || capacity > 4096 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "socket-event capacity outside finite route bound",
            ));
        }
        let handle =
            Arc::new(api.open(SOCKET_FILTER, SOCKET_LAYER, FLAG_SNIFF | FLAG_RECV_ONLY)?);
        let events = Arc::new(Mutex::new(VecDeque::with_capacity(capacity)));
        let counters = Arc::new(Counters::default());
        let worker_handle = Arc::clone(&handle);
        let worker_events = Arc::clone(&events);
        let worker_counters = Arc::clone(&counters);
        let worker = thread::Builder::new()
            .name("fragcap-direct-socket-events".into())
            .spawn(move || {
                let mut empty = [];
                loop {
                    let mut address = Address::default();
                    match worker_handle.recv(&mut empty, &mut address) {
                        Ok(_) => {
                            worker_counters.received.fetch_add(1, Ordering::Relaxed);
                            let Some(socket) = address.socket() else {
                                continue;
                            };
                            let Some(tuple) = socket.tuple() else {
                                continue;
                            };
                            let Some((at_unix_ns, clock_error_ns)) = sample_qpc_anchor()
                                .ok()
                                .and_then(|anchor| anchor.convert(address.timestamp_qpc))
                            else {
                                worker_counters.clock_lost.fetch_add(1, Ordering::Relaxed);
                                continue;
                            };
                            let kind = match address.event() {
                                4 => SocketEventKind::Open,
                                7 => SocketEventKind::Close,
                                _ => continue,
                            };
                            let event = DirectSocketEvent {
                                tuple,
                                endpoint_id: socket.endpoint_id,
                                pid: socket.pid,
                                at_qpc: address.timestamp_qpc,
                                at_unix_ns,
                                clock_error_ns,
                                kind,
                            };
                            let mut queue = worker_events.lock().expect("socket event queue lock");
                            if queue.len() < capacity {
                                queue.push_back(event);
                                worker_counters.retained.fetch_add(1, Ordering::Relaxed);
                            } else {
                                worker_counters
                                    .capacity_lost
                                    .fetch_add(1, Ordering::Relaxed);
                            }
                        }
                        Err(error) if error.raw_os_error() == Some(232) => return Ok(()),
                        Err(error) => return Err(error),
                    }
                }
            })?;
        Ok(Self {
            handle,
            events,
            counters,
            worker: Some(worker),
        })
    }

    pub fn take_events(&self) -> Vec<DirectSocketEvent> {
        self.events
            .lock()
            .expect("socket event queue lock")
            .drain(..)
            .collect()
    }

    pub fn stats(&self) -> SocketObserverStats {
        SocketObserverStats {
            received: self.counters.received.load(Ordering::Relaxed),
            retained: self.counters.retained.load(Ordering::Relaxed),
            capacity_lost: self.counters.capacity_lost.load(Ordering::Relaxed),
            clock_lost: self.counters.clock_lost.load(Ordering::Relaxed),
        }
    }

    pub fn stop(&mut self) -> io::Result<()> {
        if let Some(worker) = self.worker.take() {
            self.handle.stop_receiving()?;
            worker
                .join()
                .map_err(|_| io::Error::other("socket observer worker panicked"))??;
        }
        Ok(())
    }
}

impl Drop for DirectSocketObserver {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::{Ipv4Addr, TcpListener, TcpStream},
        path::Path,
        time::{Duration, Instant},
    };

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_socket_observer_retains_exact_connect() {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let api = Api::load_verified(Path::new(&root)).expect("verified component");
        let mut observer = DirectSocketObserver::start(&api, 256).expect("start observer");
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let destination = listener.local_addr().unwrap();
        let mut client = TcpStream::connect_timeout(&destination, Duration::from_secs(2)).unwrap();
        let client_addr = client.local_addr().unwrap();
        let (mut server, _) = listener.accept().unwrap();
        client.write_all(b"observer").unwrap();
        let mut body = [0; 8];
        server.read_exact(&mut body).unwrap();
        assert_eq!(&body, b"observer");
        let deadline = Instant::now() + Duration::from_millis(100);
        let mut found = None;
        while Instant::now() < deadline {
            for event in observer.take_events() {
                if event.tuple.local == client_addr && event.tuple.remote == destination {
                    found = Some(event);
                }
            }
            if found.is_some() {
                break;
            }
            std::thread::yield_now();
        }
        let event = found.expect("exact socket connect event");
        assert_eq!(event.pid, std::process::id());
        assert!(event.at_unix_ns > 0);
        assert!(event.clock_error_ns <= 5_000_000);
        observer.stop().unwrap();
        assert_eq!(observer.stats().capacity_lost, 0);
        eprintln!(
            "socket observer: pid={}, endpoint_id={}, clock_error_ns={}",
            event.pid, event.endpoint_id, event.clock_error_ns,
        );
    }
}
