// SPDX-License-Identifier: Apache-2.0

//! Session-authenticated SOCKS5 bridge for an admitted original TCP flow.

use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use fragcap_proxy::PROXY_USERNAME;
use zeroize::Zeroizing;

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
    use crate::deep_capture::direct_stack::{ip_address, PacketDevice, TcpFlowStack};
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
        let mut helper = connect_authenticated_socks5(
            lease.endpoint(),
            original,
            password.as_bytes(),
            Duration::from_secs(2),
        )
        .unwrap();
        helper
            .set_read_timeout(Some(Duration::from_millis(1)))
            .unwrap();

        let tuple = DirectTuple {
            transport: Proto::Tcp,
            local: SocketAddr::from(([127, 0, 0, 2], 40120)),
            remote: original,
        };
        let mut stack = TcpFlowStack::new(tuple).unwrap();
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
        let mut helper_write_closed = false;
        let mut helper_read_closed = false;
        let mut received_request = Vec::new();
        let mut response = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        for tick in 0..2_000 {
            assert!(std::time::Instant::now() < deadline, "bridge stalled");
            iface.poll(Instant::from_millis(tick), &mut device, &mut sockets);
            while let Some(packet) = device.pop_egress() {
                stack.ingest(&packet, tick).unwrap();
            }
            stack.poll(tick);
            while let Some(packet) = stack.pop_egress() {
                device.push(&packet).unwrap();
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
            let count = stack.receive_bytes(&mut buffer);
            if count != 0 {
                received_request.extend_from_slice(&buffer[..count]);
                helper.write_all(&buffer[..count]).unwrap();
            }
            if !helper_write_closed
                && received_request == b"packet-to-proxy"
                && stack.receive_closed()
            {
                helper.shutdown(std::net::Shutdown::Write).unwrap();
                helper_write_closed = true;
            }
            if helper_write_closed && !helper_read_closed {
                match helper.read(&mut buffer) {
                    Ok(0) => {
                        stack.close_send(tick);
                        helper_read_closed = true;
                    }
                    Ok(count) => {
                        assert_eq!(stack.send_bytes(&buffer[..count], tick), count);
                    }
                    Err(error)
                        if matches!(
                            error.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                        ) => {}
                    Err(error) => panic!("proxy helper read: {error}"),
                }
            }
            if response == b"proxy-to-packet" {
                break;
            }
            thread::yield_now();
        }
        assert!(client_sent && helper_write_closed);
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
