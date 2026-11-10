// SPDX-License-Identifier: Apache-2.0

//! Session-authenticated UDP association for one exact original peer.

use std::{
    io::{self, Write},
    net::{IpAddr, SocketAddr, TcpStream, UdpSocket},
    time::Duration,
};

use super::direct_bridge::{authenticate_socks5, read_socks5_reply, socks5_request};
use super::direct_route::DirectTuple;
use fragcap_core::Proto;

const MAX_SOCKS_UDP_FRAME: usize = 65_507;

/// The control connection owns the relay's lifetime; replies are accepted
/// only when the native proxy names the exact contacted original peer.
pub struct DirectUdpAssociation {
    control: TcpStream,
    relay: UdpSocket,
    original: SocketAddr,
}

impl DirectUdpAssociation {
    pub fn new(
        proxy: SocketAddr,
        original: SocketAddr,
        password: &[u8],
        timeout: Duration,
    ) -> io::Result<Self> {
        if original.ip().is_unspecified() || original.port() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid original UDP peer",
            ));
        }
        let bind = SocketAddr::new(proxy.ip(), 0);
        let relay = UdpSocket::bind(bind)?;
        relay.set_read_timeout(Some(Duration::from_millis(1)))?;
        relay.set_write_timeout(Some(Duration::from_millis(1)))?;
        let mut control = authenticate_socks5(proxy, password, timeout)?;
        control.write_all(&socks5_request(3, relay.local_addr()?))?;
        let relay_address = read_socks5_reply(&mut control)?;
        if !relay_address.ip().is_loopback()
            || relay_address.is_ipv4() != proxy.is_ipv4()
            || relay_address.port() == 0
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "invalid UDP relay scope",
            ));
        }
        relay.connect(relay_address)?;
        Ok(Self {
            control,
            relay,
            original,
        })
    }

    pub fn send(&self, payload: &[u8]) -> io::Result<()> {
        let mut frame = Vec::with_capacity(payload.len().saturating_add(22));
        frame.extend_from_slice(&[0, 0, 0]);
        match self.original.ip() {
            IpAddr::V4(ip) => {
                frame.push(1);
                frame.extend_from_slice(&ip.octets());
            }
            IpAddr::V6(ip) => {
                frame.push(4);
                frame.extend_from_slice(&ip.octets());
            }
        }
        frame.extend_from_slice(&self.original.port().to_be_bytes());
        frame.extend_from_slice(payload);
        if frame.len() > MAX_SOCKS_UDP_FRAME {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "UDP route datagram exceeds native bound",
            ));
        }
        if self.relay.send(&frame)? != frame.len() {
            return Err(io::Error::other("short UDP relay send"));
        }
        Ok(())
    }

    pub fn poll_reply(&self) -> io::Result<Option<Vec<u8>>> {
        let mut frame = [0u8; MAX_SOCKS_UDP_FRAME];
        let length = match self.relay.recv(&mut frame) {
            Ok(length) => length,
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                return Ok(None)
            }
            Err(error) => return Err(error),
        };
        let (source, payload) = parse_udp_reply(&frame[..length])?;
        if source != self.original {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "UDP relay returned a different peer",
            ));
        }
        Ok(Some(payload.to_vec()))
    }

    pub fn control_peer(&self) -> io::Result<SocketAddr> {
        self.control.peer_addr()
    }
}

fn parse_udp_reply(frame: &[u8]) -> io::Result<(SocketAddr, &[u8])> {
    if frame.len() < 10 || frame[..3] != [0, 0, 0] {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid SOCKS UDP frame",
        ));
    }
    let (ip, address_end) = match frame[3] {
        1 if frame.len() >= 10 => (
            IpAddr::V4(std::net::Ipv4Addr::new(
                frame[4], frame[5], frame[6], frame[7],
            )),
            8,
        ),
        4 if frame.len() >= 22 => {
            let octets: [u8; 16] = frame[4..20].try_into().expect("fixed IPv6 frame length");
            (IpAddr::V6(std::net::Ipv6Addr::from(octets)), 20)
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported SOCKS UDP address",
            ))
        }
    };
    let port = u16::from_be_bytes([frame[address_end], frame[address_end + 1]]);
    Ok((SocketAddr::new(ip, port), &frame[address_end + 2..]))
}

fn word_sum(bytes: &[u8]) -> u32 {
    let mut sum = 0u32;
    let mut words = bytes.chunks_exact(2);
    for word in &mut words {
        sum += u32::from(u16::from_be_bytes([word[0], word[1]]));
    }
    if let [last] = words.remainder() {
        sum += u32::from(*last) << 8;
    }
    sum
}

fn finish_checksum(mut sum: u32) -> u16 {
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

/// Construct only a reply to this exact selected UDP tuple. Checksums cover
/// the final packet independently of any offload flags on the original.
pub fn build_udp_reply(tuple: DirectTuple, payload: &[u8]) -> io::Result<Vec<u8>> {
    if tuple.transport != Proto::Udp {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "non-UDP route tuple",
        ));
    }
    let udp_length = 8usize
        .checked_add(payload.len())
        .ok_or_else(|| io::Error::other("UDP length overflow"))?;
    let udp_length_u16 = u16::try_from(udp_length)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "UDP reply too long"))?;
    let (header_length, mut packet) = match (tuple.remote.ip(), tuple.local.ip()) {
        (IpAddr::V4(source), IpAddr::V4(destination)) => {
            let total = 20usize
                .checked_add(udp_length)
                .ok_or_else(|| io::Error::other("IPv4 length overflow"))?;
            let total_u16 = u16::try_from(total)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "IPv4 reply too long"))?;
            let mut packet = vec![0u8; total];
            packet[0] = 0x45;
            packet[2..4].copy_from_slice(&total_u16.to_be_bytes());
            packet[6..8].copy_from_slice(&0x4000u16.to_be_bytes());
            packet[8] = 64;
            packet[9] = 17;
            packet[12..16].copy_from_slice(&source.octets());
            packet[16..20].copy_from_slice(&destination.octets());
            let checksum = finish_checksum(word_sum(&packet[..20]));
            packet[10..12].copy_from_slice(&checksum.to_be_bytes());
            (20, packet)
        }
        (IpAddr::V6(source), IpAddr::V6(destination)) => {
            let mut packet = vec![0u8; 40 + udp_length];
            packet[0] = 0x60;
            packet[4..6].copy_from_slice(&udp_length_u16.to_be_bytes());
            packet[6] = 17;
            packet[7] = 64;
            packet[8..24].copy_from_slice(&source.octets());
            packet[24..40].copy_from_slice(&destination.octets());
            (40, packet)
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "UDP route address families differ",
            ))
        }
    };
    let udp = &mut packet[header_length..];
    udp[0..2].copy_from_slice(&tuple.remote.port().to_be_bytes());
    udp[2..4].copy_from_slice(&tuple.local.port().to_be_bytes());
    udp[4..6].copy_from_slice(&udp_length_u16.to_be_bytes());
    udp[8..].copy_from_slice(payload);
    let pseudo_sum = match (tuple.remote.ip(), tuple.local.ip()) {
        (IpAddr::V4(source), IpAddr::V4(destination)) => {
            word_sum(&source.octets())
                + word_sum(&destination.octets())
                + 17
                + u32::from(udp_length_u16)
        }
        (IpAddr::V6(source), IpAddr::V6(destination)) => {
            word_sum(&source.octets())
                + word_sum(&destination.octets())
                + word_sum(&(udp_length as u32).to_be_bytes())
                + 17
        }
        _ => unreachable!("validated address family"),
    };
    let checksum = finish_checksum(pseudo_sum + word_sum(udp));
    udp[6..8].copy_from_slice(&if checksum == 0 { 0xffff } else { checksum }.to_be_bytes());
    Ok(packet)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap_core::{parse::raw_udp_datagram, Proto};
    use fragcap_proxy::{DestinationPolicy, NativeProxyBackend, NativeProxyConfig};
    use smoltcp::wire::{IpAddress, Ipv4Packet, UdpPacket};
    use std::{thread, time::Instant};

    #[test]
    fn synthetic_udp_replies_have_exact_peer_and_valid_checksums() {
        let tuple = DirectTuple {
            transport: Proto::Udp,
            local: "127.0.0.2:40000".parse().unwrap(),
            remote: "127.0.0.1:5300".parse().unwrap(),
        };
        let reply = build_udp_reply(tuple, b"reply bytes").unwrap();
        let parsed = raw_udp_datagram(&reply).unwrap();
        assert_eq!(parsed.source, tuple.remote);
        assert_eq!(parsed.destination, tuple.local);
        assert_eq!(parsed.payload, b"reply bytes");
        assert!(Ipv4Packet::new_checked(&reply).unwrap().verify_checksum());
        assert!(UdpPacket::new_checked(&reply[20..])
            .unwrap()
            .verify_checksum(&IpAddress::v4(127, 0, 0, 1), &IpAddress::v4(127, 0, 0, 2)));
        let ipv6 = DirectTuple {
            local: "[::1]:40000".parse().unwrap(),
            remote: "[::1]:5300".parse().unwrap(),
            ..tuple
        };
        let reply_v6 = build_udp_reply(ipv6, b"v6").unwrap();
        assert_eq!(raw_udp_datagram(&reply_v6).unwrap().payload, b"v6");
        assert!(UdpPacket::new_checked(&reply_v6[40..])
            .unwrap()
            .verify_checksum(
                &IpAddress::v6(0, 0, 0, 0, 0, 0, 0, 1),
                &IpAddress::v6(0, 0, 0, 0, 0, 0, 0, 1),
            ));
    }

    #[test]
    fn native_udp_association_preserves_peer_and_datagram() {
        let origin = UdpSocket::bind("127.0.0.1:0").unwrap();
        let original = origin.local_addr().unwrap();
        origin
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let server = thread::spawn(move || {
            let mut bytes = [0u8; 32];
            let (count, client) = origin.recv_from(&mut bytes).unwrap();
            assert_eq!(&bytes[..count], b"udp-route");
            origin.send_to(b"udp-reply", client).unwrap();
        });
        let config = NativeProxyConfig::new(
            "127.0.0.1:0".parse().unwrap(),
            4,
            1024,
            Duration::from_secs(2),
        )
        .unwrap()
        .with_session_id("s200-direct-udp")
        .unwrap();
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(original);
        let mut proxy = NativeProxyBackend::new(config).with_destination_policy(policy);
        let mut lease = proxy.start(Duration::from_secs(2)).unwrap();
        let association = DirectUdpAssociation::new(
            lease.endpoint(),
            original,
            lease.capability_proof().proxy_password().as_bytes(),
            Duration::from_secs(2),
        )
        .unwrap();
        assert_eq!(association.control_peer().unwrap(), lease.endpoint());
        association.send(b"udp-route").unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let reply = loop {
            assert!(Instant::now() < deadline, "UDP association stalled");
            if let Some(reply) = association.poll_reply().unwrap() {
                break reply;
            }
        };
        assert_eq!(reply, b"udp-reply");
        drop(association);
        server.join().unwrap();
        let report = lease.cleanup(Duration::from_secs(2));
        assert_eq!(report.observation.protocol.socks_udp_associate_succeeded, 1);
        assert_eq!(report.observation.protocol.socks_udp_client_forwarded, 1);
        assert_eq!(report.observation.protocol.socks_udp_upstream_forwarded, 1);
    }

    #[test]
    fn malformed_or_wrong_peer_udp_replies_are_refused() {
        assert!(parse_udp_reply(&[0, 0, 1, 1, 127, 0, 0, 1, 0, 53]).is_err());
        assert!(parse_udp_reply(&[0, 0, 0, 4]).is_err());
        let (source, payload) = parse_udp_reply(&[0, 0, 0, 1, 127, 0, 0, 1, 0, 53, 7]).unwrap();
        assert_eq!(source, "127.0.0.1:53".parse().unwrap());
        assert_eq!(payload, [7]);
    }
}
