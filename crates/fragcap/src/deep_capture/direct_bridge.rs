// SPDX-License-Identifier: Apache-2.0

//! Session-authenticated SOCKS5 bridge for an admitted original TCP flow.

use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use fragcap_proxy::PROXY_USERNAME;
use zeroize::Zeroizing;

use super::{
    direct_route::DirectTuple,
    direct_stack::{StackError, TcpFlowStack},
};

const MAX_BRIDGE_BYTES: usize = 65_536;

#[derive(Debug)]
pub enum DirectBridgeError {
    Stack(StackError),
    Io(io::Error),
}

impl std::fmt::Display for DirectBridgeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stack(error) => write!(formatter, "packet stack: {error:?}"),
            Self::Io(error) => write!(formatter, "proxy helper: {error}"),
        }
    }
}

impl std::error::Error for DirectBridgeError {}

impl From<StackError> for DirectBridgeError {
    fn from(error: StackError) -> Self {
        Self::Stack(error)
    }
}

impl From<io::Error> for DirectBridgeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// One admitted TCP flow, with bounded pending bytes in each direction.
/// The caller owns WinDivert packet disposition and original-flow receipts.
pub struct DirectTcpBridge {
    stack: TcpFlowStack,
    helper: TcpStream,
    to_proxy: VecDeque<u8>,
    to_client: VecDeque<u8>,
    client_send_closed: bool,
    proxy_send_closed: bool,
    stack_send_closed: bool,
}

impl DirectTcpBridge {
    pub fn new(tuple: DirectTuple, helper: TcpStream) -> Result<Self, DirectBridgeError> {
        helper.set_read_timeout(Some(Duration::from_millis(1)))?;
        helper.set_write_timeout(Some(Duration::from_millis(1)))?;
        Ok(Self {
            stack: TcpFlowStack::new(tuple)?,
            helper,
            to_proxy: VecDeque::new(),
            to_client: VecDeque::new(),
            client_send_closed: false,
            proxy_send_closed: false,
            stack_send_closed: false,
        })
    }

    pub fn ingest_diverted(
        &mut self,
        packet: &[u8],
        kernel_validated_ipv4: bool,
        now_ms: i64,
    ) -> Result<Vec<Vec<u8>>, DirectBridgeError> {
        self.stack
            .ingest_diverted(packet, now_ms, kernel_validated_ipv4)?;
        self.poll(now_ms)
    }

    pub fn poll(&mut self, now_ms: i64) -> Result<Vec<Vec<u8>>, DirectBridgeError> {
        self.stack.poll(now_ms);
        let mut bytes = [0u8; 8192];
        loop {
            let available = (MAX_BRIDGE_BYTES - self.to_proxy.len()).min(bytes.len());
            if available == 0 {
                break;
            }
            let count = self.stack.receive_bytes(&mut bytes[..available]);
            if count == 0 {
                break;
            }
            self.to_proxy.extend(&bytes[..count]);
        }
        while !self.to_proxy.is_empty() {
            let front = self.to_proxy.as_slices().0;
            match self.helper.write(front) {
                Ok(0) => return Err(io::Error::from(io::ErrorKind::WriteZero).into()),
                Ok(count) => {
                    self.to_proxy.drain(..count);
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    break;
                }
                Err(error) => return Err(error.into()),
            }
        }
        if !self.client_send_closed && self.to_proxy.is_empty() && self.stack.receive_closed() {
            self.helper.shutdown(std::net::Shutdown::Write)?;
            self.client_send_closed = true;
        }
        if !self.proxy_send_closed {
            let available = (MAX_BRIDGE_BYTES - self.to_client.len()).min(bytes.len());
            match (available != 0).then(|| self.helper.read(&mut bytes[..available])) {
                None => {}
                Some(Ok(0)) => self.proxy_send_closed = true,
                Some(Ok(count)) => self.to_client.extend(&bytes[..count]),
                Some(Err(error))
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) => {}
                Some(Err(error)) => return Err(error.into()),
            }
        }
        while !self.to_client.is_empty() {
            let front = self.to_client.as_slices().0;
            let sent = self.stack.send_bytes(front, now_ms);
            if sent == 0 {
                break;
            }
            self.to_client.drain(..sent);
        }
        if self.proxy_send_closed && self.to_client.is_empty() && !self.stack_send_closed {
            self.stack.close_send(now_ms);
            self.stack_send_closed = true;
        }
        let mut egress = Vec::new();
        while let Some(packet) = self.stack.pop_egress() {
            egress.push(packet);
        }
        Ok(egress)
    }
}

fn refused(reason: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, reason)
}

/// Connect one already-admitted original destination through the native proxy.
/// The caller retains the one-use route receipt and owns the returned helper
/// socket; this function never infers game ownership from that helper's PID.
pub fn connect_authenticated_socks5(
    proxy: SocketAddr,
    original: SocketAddr,
    password: &[u8],
    timeout: Duration,
) -> io::Result<TcpStream> {
    if password.is_empty()
        || password.len() > u8::MAX as usize
        || PROXY_USERNAME.len() > u8::MAX as usize
        || timeout.is_zero()
        || !proxy.ip().is_loopback()
        || original.ip().is_unspecified()
        || original.port() == 0
    {
        return Err(refused("invalid direct bridge scope"));
    }
    let mut stream = TcpStream::connect_timeout(&proxy, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;

    stream.write_all(&[5, 1, 2])?;
    let mut method = [0; 2];
    stream.read_exact(&mut method)?;
    if method != [5, 2] {
        return Err(refused("native proxy refused SOCKS5 authentication method"));
    }

    let mut authentication = Zeroizing::new(Vec::with_capacity(
        3 + PROXY_USERNAME.len() + password.len(),
    ));
    authentication.extend_from_slice(&[1, PROXY_USERNAME.len() as u8]);
    authentication.extend_from_slice(PROXY_USERNAME.as_bytes());
    authentication.push(password.len() as u8);
    authentication.extend_from_slice(password);
    stream.write_all(&authentication)?;
    let mut authenticated = [0; 2];
    stream.read_exact(&mut authenticated)?;
    if authenticated != [1, 0] {
        return Err(refused("native proxy refused direct bridge capability"));
    }

    let mut request = Vec::with_capacity(22);
    match original {
        SocketAddr::V4(address) => {
            request.extend_from_slice(&[5, 1, 0, 1]);
            request.extend_from_slice(&address.ip().octets());
        }
        SocketAddr::V6(address) => {
            request.extend_from_slice(&[5, 1, 0, 4]);
            request.extend_from_slice(&address.ip().octets());
        }
    }
    request.extend_from_slice(&original.port().to_be_bytes());
    stream.write_all(&request)?;
    let mut reply = [0; 4];
    stream.read_exact(&mut reply)?;
    if reply[0] != 5 || reply[1] != 0 || reply[2] != 0 {
        return Err(refused("native proxy refused original destination"));
    }
    let remaining = match reply[3] {
        1 => 6,
        4 => 18,
        _ => return Err(refused("native proxy returned invalid address family")),
    };
    let mut address = [0; 18];
    stream.read_exact(&mut address[..remaining])?;
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{net::TcpListener, thread};

    use crate::deep_capture::direct_route::DirectTuple;
    use crate::deep_capture::direct_stack::{ip_address, PacketDevice};
    use fragcap_core::Proto;
    use fragcap_proxy::{DestinationPolicy, NativeProxyBackend, NativeProxyConfig};
    use smoltcp::{
        iface::{Config, Interface, SocketSet},
        socket::tcp,
        time::Instant,
        wire::{HardwareAddress, IpCidr},
    };

    #[test]
    fn native_proxy_bridge_preserves_raw_bytes_and_half_close() {
        let origin = TcpListener::bind("127.0.0.1:0").unwrap();
        let original = origin.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = origin.accept().unwrap();
            let mut request = Vec::new();
            stream.read_to_end(&mut request).unwrap();
            assert_eq!(request, b"direct raw bytes");
            stream.write_all(b"origin reply").unwrap();
        });
        let config = NativeProxyConfig::new(
            "127.0.0.1:0".parse().unwrap(),
            4,
            1024,
            Duration::from_secs(2),
        )
        .unwrap()
        .with_session_id("s200-direct-bridge")
        .unwrap();
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(original);
        let mut proxy = NativeProxyBackend::new(config).with_destination_policy(policy);
        let mut lease = proxy.start(Duration::from_secs(2)).unwrap();
        let password = lease.capability_proof().proxy_password();
        let mut helper = connect_authenticated_socks5(
            lease.endpoint(),
            original,
            password.as_bytes(),
            Duration::from_secs(2),
        )
        .unwrap();
        helper.write_all(b"direct raw bytes").unwrap();
        helper.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = Vec::new();
        helper.read_to_end(&mut response).unwrap();
        assert_eq!(response, b"origin reply");
        server.join().unwrap();
        let report = lease.cleanup(Duration::from_secs(2));
        assert_eq!(report.observation.protocol.socks_connect_succeeded, 1);
        assert_eq!(report.observation.protocol.socks_client_bytes, 16);
        assert_eq!(report.observation.protocol.socks_upstream_bytes, 12);
    }

    #[test]
    fn packet_stack_to_native_proxy_preserves_original_destination_and_reply() {
        let origin = TcpListener::bind("127.0.0.1:0").unwrap();
        let original = origin.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = origin.accept().unwrap();
            let mut request = Vec::new();
            stream.read_to_end(&mut request).unwrap();
            assert_eq!(request, b"packet-to-proxy");
            stream.write_all(b"proxy-to-packet").unwrap();
        });
        let config = NativeProxyConfig::new(
            "127.0.0.1:0".parse().unwrap(),
            4,
            1024,
            Duration::from_secs(2),
        )
        .unwrap()
        .with_session_id("s200-packet-proxy")
        .unwrap();
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(original);
        let mut proxy = NativeProxyBackend::new(config).with_destination_policy(policy);
        let mut lease = proxy.start(Duration::from_secs(2)).unwrap();
        let password = lease.capability_proof().proxy_password();
        let helper = connect_authenticated_socks5(
            lease.endpoint(),
            original,
            password.as_bytes(),
            Duration::from_secs(2),
        )
        .unwrap();

        let tuple = DirectTuple {
            transport: Proto::Tcp,
            local: SocketAddr::from(([127, 0, 0, 2], 40120)),
            remote: original,
        };
        let mut bridge = DirectTcpBridge::new(tuple, helper).unwrap();
        let mut device = PacketDevice::new(16).unwrap();
        let mut iface = Interface::new(
            Config::new(HardwareAddress::Ip),
            &mut device,
            Instant::from_millis(0),
        );
        iface.update_ip_addrs(|addrs| {
            addrs
                .push(IpCidr::new(ip_address(tuple.local.ip()), 0))
                .unwrap();
        });
        let mut sockets = SocketSet::new(vec![]);
        let socket = sockets.add(tcp::Socket::new(
            tcp::SocketBuffer::new(vec![0; 65_536]),
            tcp::SocketBuffer::new(vec![0; 65_536]),
        ));
        sockets
            .get_mut::<tcp::Socket>(socket)
            .connect(
                iface.context(),
                (ip_address(original.ip()), original.port()),
                tuple.local.port(),
            )
            .unwrap();
        let mut client_sent = false;
        let mut response = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        for tick in 0..2_000 {
            assert!(std::time::Instant::now() < deadline, "bridge stalled");
            iface.poll(Instant::from_millis(tick), &mut device, &mut sockets);
            while let Some(packet) = device.pop_egress() {
                for reply in bridge.ingest_diverted(&packet, false, tick).unwrap() {
                    device.push(&reply).unwrap();
                }
            }
            let client = sockets.get_mut::<tcp::Socket>(socket);
            if !client_sent && client.can_send() {
                assert_eq!(client.send_slice(b"packet-to-proxy").unwrap(), 15);
                client.close();
                client_sent = true;
            }
            let mut buffer = [0; 64];
            if client.can_recv() {
                let count = client.recv_slice(&mut buffer).unwrap();
                response.extend_from_slice(&buffer[..count]);
            }
            for reply in bridge.poll(tick).unwrap() {
                device.push(&reply).unwrap();
            }
            if response == b"proxy-to-packet" {
                break;
            }
            thread::yield_now();
        }
        assert!(client_sent && bridge.client_send_closed);
        assert_eq!(response, b"proxy-to-packet");
        server.join().unwrap();
        let report = lease.cleanup(Duration::from_secs(2));
        assert_eq!(report.observation.protocol.socks_connect_succeeded, 1);
        assert_eq!(report.observation.protocol.socks_client_bytes, 15);
        assert_eq!(report.observation.protocol.socks_upstream_bytes, 15);
    }

    #[test]
    fn bridge_refuses_missing_capability_or_non_loopback_proxy() {
        let original = SocketAddr::from(([198, 51, 100, 10], 443));
        assert!(connect_authenticated_socks5(
            SocketAddr::from(([127, 0, 0, 1], 2000)),
            original,
            b"",
            Duration::from_secs(1),
        )
        .is_err());
        assert!(connect_authenticated_socks5(
            SocketAddr::from(([198, 51, 100, 11], 2000)),
            original,
            b"secret",
            Duration::from_secs(1),
        )
        .is_err());
    }
}
