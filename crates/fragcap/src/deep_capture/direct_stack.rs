// SPDX-License-Identifier: Apache-2.0

//! Finite raw-IP device and one-flow TCP termination for the direct bridge.

use std::{
    collections::VecDeque,
    net::{IpAddr, SocketAddr},
};

use fragcap_core::{HeaderParser, InterfaceAddrs, LinkType, ParseOutcome, Proto};
use smoltcp::{
    iface::{Config, Interface, SocketHandle, SocketSet},
    phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken},
    socket::tcp,
    time::Instant,
    wire::{HardwareAddress, IpAddress, IpCidr, Ipv4Packet},
};

use super::direct_route::DirectTuple;

const MAX_RAW_PACKET: usize = 65_535;
const TCP_BUFFER: usize = 65_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StackError {
    InvalidScope,
    PacketTooLarge,
    QueueFull,
    FlowMismatch,
    UntrustedChecksum,
    ListenFailed,
}

/// No device read or write may grow a queue beyond its session-owned limit.
pub struct PacketDevice {
    ingress: VecDeque<Vec<u8>>,
    egress: VecDeque<Vec<u8>>,
    capacity: usize,
}

impl PacketDevice {
    pub fn new(capacity: usize) -> Result<Self, StackError> {
        if capacity == 0 || capacity > 256 {
            return Err(StackError::InvalidScope);
        }
        Ok(Self {
            ingress: VecDeque::new(),
            egress: VecDeque::new(),
            capacity,
        })
    }

    pub fn push(&mut self, packet: &[u8]) -> Result<(), StackError> {
        if packet.len() > MAX_RAW_PACKET {
            return Err(StackError::PacketTooLarge);
        }
        if self.ingress.len() >= self.capacity {
            return Err(StackError::QueueFull);
        }
        self.ingress.push_back(packet.to_vec());
        Ok(())
    }

    pub fn pop_egress(&mut self) -> Option<Vec<u8>> {
        self.egress.pop_front()
    }
}

pub struct PacketRx(Vec<u8>);

impl RxToken for PacketRx {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.0)
    }
}

pub struct PacketTx<'a>(&'a mut VecDeque<Vec<u8>>);

impl TxToken for PacketTx<'_> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut packet = vec![0; len];
        let result = f(&mut packet);
        self.0.push_back(packet);
        result
    }
}

impl Device for PacketDevice {
    type RxToken<'a> = PacketRx;
    type TxToken<'a> = PacketTx<'a>;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        if self.egress.len() >= self.capacity {
            return None;
        }
        let packet = self.ingress.pop_front()?;
        Some((PacketRx(packet), PacketTx(&mut self.egress)))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        (self.egress.len() < self.capacity).then(|| PacketTx(&mut self.egress))
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut capabilities = DeviceCapabilities::default();
        capabilities.medium = Medium::Ip;
        capabilities.max_transmission_unit = MAX_RAW_PACKET;
        capabilities
    }
}

pub(super) fn ip_address(ip: IpAddr) -> IpAddress {
    match ip {
        IpAddr::V4(value) => {
            let [a, b, c, d] = value.octets();
            IpAddress::v4(a, b, c, d)
        }
        IpAddr::V6(value) => {
            let [a, b, c, d, e, f, g, h] = value.segments();
            IpAddress::v6(a, b, c, d, e, f, g, h)
        }
    }
}

/// Terminates exactly one client-side TCP flow; the native proxy owns upstream work.
pub struct TcpFlowStack {
    tuple: DirectTuple,
    parser: HeaderParser,
    device: PacketDevice,
    iface: Interface,
    sockets: SocketSet<'static>,
    socket: SocketHandle,
    established: bool,
}

impl TcpFlowStack {
    pub fn new(tuple: DirectTuple) -> Result<Self, StackError> {
        if tuple.transport != Proto::Tcp
            || tuple.local.ip().is_unspecified()
            || tuple.remote.ip().is_unspecified()
            || tuple.local.ip().is_ipv4() != tuple.remote.ip().is_ipv4()
            || tuple.local.port() == 0
            || tuple.remote.port() == 0
        {
            return Err(StackError::InvalidScope);
        }
        let mut device = PacketDevice::new(16)?;
        let mut iface = Interface::new(
            Config::new(HardwareAddress::Ip),
            &mut device,
            Instant::from_millis(0),
        );
        let remote = ip_address(tuple.remote.ip());
        iface.update_ip_addrs(|addrs| {
            addrs
                .push(IpCidr::new(remote, 0))
                .expect("one address slot");
        });
        let mut sockets = SocketSet::new(vec![]);
        let tcp = tcp::Socket::new(
            tcp::SocketBuffer::new(vec![0; TCP_BUFFER]),
            tcp::SocketBuffer::new(vec![0; TCP_BUFFER]),
        );
        let socket = sockets.add(tcp);
        sockets
            .get_mut::<tcp::Socket>(socket)
            .listen((remote, tuple.remote.port()))
            .map_err(|_| StackError::ListenFailed)?;
        Ok(Self {
            tuple,
            parser: HeaderParser::new(InterfaceAddrs::new([tuple.local.ip()])),
            device,
            iface,
            sockets,
            socket,
            established: false,
        })
    }

    pub fn ingest(&mut self, packet: &[u8], now_ms: i64) -> Result<(), StackError> {
        let ParseOutcome::Parsed { flow, .. } = self.parser.parse(LinkType::RAW, packet) else {
            return Err(StackError::FlowMismatch);
        };
        let exact = flow.local == self.tuple.local && flow.remote == self.tuple.remote;
        let loopback_canonical = self.tuple.local.ip() == self.tuple.remote.ip()
            && flow.local == self.tuple.remote
            && flow.remote == self.tuple.local;
        if flow.proto != self.tuple.transport || !(exact || loopback_canonical) {
            return Err(StackError::FlowMismatch);
        }
        self.device.push(packet)?;
        self.poll(now_ms);
        Ok(())
    }

    /// WinDivert marks some loopback IPv4 headers as kernel-validated even
    /// though their wire checksum is unfinished. Normalize only that header
    /// for the private stack; never modify the packet used for reinjection.
    pub fn ingest_diverted(
        &mut self,
        packet: &[u8],
        now_ms: i64,
        kernel_validated_ipv4: bool,
    ) -> Result<(), StackError> {
        if packet.first().is_some_and(|version| version >> 4 == 4) {
            let header = Ipv4Packet::new_checked(packet).map_err(|_| StackError::FlowMismatch)?;
            if !header.verify_checksum() {
                if !kernel_validated_ipv4 {
                    return Err(StackError::UntrustedChecksum);
                }
                let mut normalized = packet.to_vec();
                Ipv4Packet::new_unchecked(&mut normalized).fill_checksum();
                return self.ingest(&normalized, now_ms);
            }
        }
        self.ingest(packet, now_ms)
    }

    pub fn poll(&mut self, now_ms: i64) {
        self.iface.poll(
            Instant::from_millis(now_ms),
            &mut self.device,
            &mut self.sockets,
        );
        let socket = self.sockets.get_mut::<tcp::Socket>(self.socket);
        self.established |= socket.may_recv() || socket.may_send();
    }

    pub fn pop_egress(&mut self) -> Option<Vec<u8>> {
        self.device.pop_egress()
    }

    pub fn receive_bytes(&mut self, output: &mut [u8]) -> usize {
        let socket = self.sockets.get_mut::<tcp::Socket>(self.socket);
        if socket.can_recv() {
            socket.recv_slice(output).unwrap_or(0)
        } else {
            0
        }
    }

    pub fn receive_closed(&mut self) -> bool {
        self.established && !self.sockets.get_mut::<tcp::Socket>(self.socket).may_recv()
    }

    pub fn send_bytes(&mut self, input: &[u8], now_ms: i64) -> usize {
        let socket = self.sockets.get_mut::<tcp::Socket>(self.socket);
        let sent = if socket.can_send() {
            socket.send_slice(input).unwrap_or(0)
        } else {
            0
        };
        self.poll(now_ms);
        sent
    }

    pub fn close_send(&mut self, now_ms: i64) {
        self.sockets.get_mut::<tcp::Socket>(self.socket).close();
        self.poll(now_ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_device_queue_is_finite_and_receive_requires_reply_space() {
        let mut device = PacketDevice::new(1).unwrap();
        device.push(b"packet").unwrap();
        assert_eq!(device.push(b"second"), Err(StackError::QueueFull));
        let (rx, tx) = device.receive(Instant::from_millis(0)).unwrap();
        assert_eq!(rx.consume(|bytes| bytes.to_vec()), b"packet");
        tx.consume(3, |bytes| bytes.copy_from_slice(b"ack"));
        device.push(b"next").unwrap();
        assert!(device.receive(Instant::from_millis(1)).is_none());
        assert_eq!(device.pop_egress(), Some(b"ack".to_vec()));
        assert!(device.receive(Instant::from_millis(2)).is_some());
    }

    #[test]
    fn tcp_flow_stack_refuses_non_tcp_or_invalid_scopes() {
        let tuple = DirectTuple {
            transport: Proto::Udp,
            local: SocketAddr::from(([127, 0, 0, 1], 40000)),
            remote: SocketAddr::from(([127, 0, 0, 1], 23000)),
        };
        assert!(matches!(
            TcpFlowStack::new(tuple),
            Err(StackError::InvalidScope)
        ));
        let mut stack = TcpFlowStack::new(DirectTuple {
            transport: Proto::Tcp,
            ..tuple
        })
        .unwrap();
        assert!(!stack.receive_closed());
    }

    #[test]
    fn tcp_stack_conserves_raw_bytes_and_reply_across_packet_boundary() {
        let tuple = DirectTuple {
            transport: Proto::Tcp,
            local: SocketAddr::from(([127, 0, 0, 1], 40000)),
            remote: SocketAddr::from(([127, 0, 0, 1], 23000)),
        };
        let mut server = TcpFlowStack::new(tuple).unwrap();
        let mut client_device = PacketDevice::new(16).unwrap();
        let client_ip = ip_address(tuple.local.ip());
        let mut client_iface = Interface::new(
            Config::new(HardwareAddress::Ip),
            &mut client_device,
            Instant::from_millis(0),
        );
        client_iface.update_ip_addrs(|addrs| {
            addrs.push(IpCidr::new(client_ip, 0)).unwrap();
        });
        let mut client_sockets = SocketSet::new(vec![]);
        let handle = client_sockets.add(tcp::Socket::new(
            tcp::SocketBuffer::new(vec![0; TCP_BUFFER]),
            tcp::SocketBuffer::new(vec![0; TCP_BUFFER]),
        ));
        client_sockets
            .get_mut::<tcp::Socket>(handle)
            .connect(
                client_iface.context(),
                (ip_address(tuple.remote.ip()), tuple.remote.port()),
                tuple.local.port(),
            )
            .unwrap();

        let mut sent_client = false;
        let mut sent_server = false;
        let mut received_client = Vec::new();
        let mut received_server = Vec::new();
        let mut checked_kernel_checksum = false;
        for now in 0..2_000 {
            let at = Instant::from_millis(now);
            client_iface.poll(at, &mut client_device, &mut client_sockets);
            while let Some(packet) = client_device.pop_egress() {
                if !checked_kernel_checksum {
                    let mut unfinished = packet.clone();
                    unfinished[10..12].copy_from_slice(&[0, 0]);
                    assert_eq!(
                        server.ingest_diverted(&unfinished, now, false),
                        Err(StackError::UntrustedChecksum)
                    );
                    server.ingest_diverted(&unfinished, now, true).unwrap();
                    server.ingest_diverted(&unfinished, now, true).unwrap();
                    checked_kernel_checksum = true;
                } else {
                    server.ingest(&packet, now).unwrap();
                    // An exact duplicate must not duplicate bridge bytes.
                    server.ingest(&packet, now).unwrap();
                }
            }
            server.poll(now);
            while let Some(packet) = server.pop_egress() {
                client_device.push(&packet).unwrap();
            }
            let client = client_sockets.get_mut::<tcp::Socket>(handle);
            if !sent_client && client.can_send() {
                assert_eq!(client.send_slice(b"client raw bytes").unwrap(), 16);
                client.close();
                sent_client = true;
            }
            let mut bytes = [0; 32];
            if client.can_recv() {
                let count = client.recv_slice(&mut bytes).unwrap();
                received_client.extend_from_slice(&bytes[..count]);
            }
            let count = server.receive_bytes(&mut bytes);
            received_server.extend_from_slice(&bytes[..count]);
            if !sent_server && received_server == b"client raw bytes" {
                assert_eq!(server.send_bytes(b"server reply", now), 12);
                server.close_send(now);
                sent_server = true;
            }
            if received_client == b"server reply" {
                break;
            }
        }
        assert!(checked_kernel_checksum && sent_client && sent_server);
        assert_eq!(received_server, b"client raw bytes");
        assert_eq!(received_client, b"server reply");
    }
}
