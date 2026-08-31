// SPDX-License-Identifier: Apache-2.0

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;

use fragcap_proxy::{
    tls_client_config_with_roots, DestinationPolicy, NativeProxyBackend, NativeProxyConfig,
};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName};

fn read_head(stream: &mut impl Read) -> Vec<u8> {
    let mut head = Vec::new();
    let mut byte = [0_u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        stream.read_exact(&mut byte).unwrap();
        head.push(byte[0]);
    }
    head
}

fn tls_origin(
    version: &'static rustls::SupportedProtocolVersion,
) -> (
    SocketAddr,
    CertificateDer<'static>,
    std::thread::JoinHandle<()>,
) {
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let certificate = CertificateDer::from(generated.cert);
    let private = PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der());
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut config = rustls::ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[version])
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![certificate.clone()], private.into())
        .unwrap();
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let task = std::thread::spawn(move || {
        let (tcp, _) = listener.accept().unwrap();
        let connection = rustls::ServerConnection::new(Arc::new(config)).unwrap();
        let mut tls = rustls::StreamOwned::new(connection, tcp);
        let request = String::from_utf8(read_head(&mut tls)).unwrap();
        assert!(request.starts_with("GET /secure HTTP/1.1\r\n"));
        tls.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\nSECURE")
            .unwrap();
        tls.flush().unwrap();
    });
    (address, certificate, task)
}

fn failing_tls_origin() -> (
    SocketAddr,
    CertificateDer<'static>,
    std::thread::JoinHandle<()>,
) {
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let certificate = CertificateDer::from(generated.cert);
    let private = PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der());
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![certificate.clone()], private.into())
        .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let task = std::thread::spawn(move || {
        let (tcp, _) = listener.accept().unwrap();
        let connection = rustls::ServerConnection::new(Arc::new(config)).unwrap();
        let mut tls = rustls::StreamOwned::new(connection, tcp);
        let mut byte = [0_u8; 1];
        let _ = tls.read(&mut byte);
    });
    (address, certificate, task)
}

fn round_trip(version: &'static rustls::SupportedProtocolVersion, session: &str) {
    let (origin, origin_certificate, server) = tls_origin(version);
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        8,
        16 * 1024,
        Duration::from_secs(2),
    )
    .unwrap()
    .with_session_id(session)
    .unwrap();
    let mut policy = DestinationPolicy::new(config.listen());
    policy.grant_for_test(origin);
    let mut upstream_roots = rustls::RootCertStore::empty();
    upstream_roots.add(origin_certificate).unwrap();
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_tls_client_config(tls_client_config_with_roots(upstream_roots).unwrap())
        .start(Duration::from_secs(2))
        .unwrap();

    let endpoint = lease.observation(Duration::from_secs(1)).unwrap().endpoint;
    let authorization = lease.capability_proof().proxy_authorization();
    let mut tcp = TcpStream::connect(endpoint).unwrap();
    write!(
        tcp,
        "CONNECT localhost:{} HTTP/1.1\r\nHost: localhost:{}\r\nProxy-Authorization: {}\r\n\r\n",
        origin.port(),
        origin.port(),
        authorization.as_str(),
    )
    .unwrap();
    assert_eq!(
        read_head(&mut tcp),
        b"HTTP/1.1 200 Connection Established\r\n\r\n"
    );

    let mut client_roots = rustls::RootCertStore::empty();
    client_roots
        .add(CertificateDer::from(lease.ca_der().to_vec()))
        .unwrap();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut client_config = rustls::ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[version])
        .unwrap()
        .with_root_certificates(client_roots)
        .with_no_client_auth();
    client_config.alpn_protocols = vec![b"http/1.1".to_vec()];
    let connection = rustls::ClientConnection::new(
        Arc::new(client_config),
        ServerName::try_from("localhost").unwrap().to_owned(),
    )
    .unwrap();
    let mut tls = rustls::StreamOwned::new(connection, tcp);
    write!(
        tls,
        "GET /secure HTTP/1.1\r\nHost: localhost:{}\r\nConnection: close\r\n\r\n",
        origin.port(),
    )
    .unwrap();
    let mut response = String::new();
    tls.read_to_string(&mut response).unwrap();
    assert!(response.ends_with("SECURE"), "{response}");
    server.join().unwrap();

    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.connect_requests, 1);
    assert_eq!(report.observation.protocol.client_tls_completed, 1);
    assert_eq!(report.observation.protocol.upstream_tls_completed, 1);
    assert_eq!(report.observation.protocol.requests, 1);
    assert_eq!(report.observation.protocol.responses, 1);
    assert_eq!(report.observation.application.len(), 4);
    let expected_version = if std::ptr::eq(version, &rustls::version::TLS12) {
        "TLS1.2"
    } else {
        "TLS1.3"
    };
    let boundaries: Vec<_> = report
        .observation
        .application
        .iter()
        .filter_map(|item| item.tls.as_ref())
        .collect();
    assert_eq!(boundaries.len(), 2);
    assert!(boundaries.iter().all(|facts| {
        facts.requested_identity == format!("localhost:{}", origin.port())
            && facts.version.as_deref() == Some(expected_version)
            && facts.alpn.as_deref() == Some(b"http/1.1".as_slice())
    }));
}

#[test]
fn tls12_connect_performs_two_verified_boundaries_and_relays_http() {
    round_trip(&rustls::version::TLS12, "s104-https-tls12");
}

#[test]
fn tls13_connect_performs_two_verified_boundaries_and_relays_http() {
    round_trip(&rustls::version::TLS13, "s104-https-tls13");
}

#[test]
fn connect_fails_closed_for_wrong_name_and_untrusted_upstream() {
    for wrong_name in [true, false] {
        let (origin, certificate, server) = failing_tls_origin();
        let config = NativeProxyConfig::new(
            "127.0.0.1:0".parse().unwrap(),
            8,
            16 * 1024,
            Duration::from_secs(2),
        )
        .unwrap()
        .with_session_id(if wrong_name {
            "s104-wrong-name"
        } else {
            "s104-untrusted"
        })
        .unwrap();
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(origin);
        let mut roots = rustls::RootCertStore::empty();
        if wrong_name {
            roots.add(certificate).unwrap();
        } else {
            let decoy = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
            roots.add(CertificateDer::from(decoy.cert)).unwrap();
        }
        let mut lease = NativeProxyBackend::new(config)
            .with_destination_policy(policy)
            .with_tls_client_config(tls_client_config_with_roots(roots).unwrap())
            .start(Duration::from_secs(2))
            .unwrap();
        let endpoint = lease.observation(Duration::from_secs(1)).unwrap().endpoint;
        let auth = lease.capability_proof().proxy_authorization();
        let authority = if wrong_name {
            origin.to_string()
        } else {
            format!("localhost:{}", origin.port())
        };
        let mut client = TcpStream::connect(endpoint).unwrap();
        write!(
            client,
            "CONNECT {authority} HTTP/1.1\r\nHost: {authority}\r\nProxy-Authorization: {}\r\n\r\n",
            auth.as_str()
        )
        .unwrap();
        assert_eq!(
            read_head(&mut client),
            b"HTTP/1.1 200 Connection Established\r\n\r\n"
        );
        let mut client_roots = rustls::RootCertStore::empty();
        client_roots
            .add(CertificateDer::from(lease.ca_der().to_vec()))
            .unwrap();
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let mut client_config = rustls::ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_root_certificates(client_roots)
            .with_no_client_auth();
        client_config.alpn_protocols = vec![b"http/1.1".to_vec()];
        let server_name = if wrong_name {
            ServerName::IpAddress("127.0.0.1".parse::<std::net::IpAddr>().unwrap().into())
        } else {
            ServerName::try_from("localhost").unwrap().to_owned()
        };
        let connection =
            rustls::ClientConnection::new(Arc::new(client_config), server_name).unwrap();
        let mut tls = rustls::StreamOwned::new(connection, client);
        let _ = tls.write_all(b"GET / HTTP/1.1\r\nHost: refused\r\n\r\n");
        let mut response = Vec::new();
        let _ = tls.read_to_end(&mut response);
        assert!(response.is_empty());
        server.join().unwrap();
        let report = lease.cleanup(Duration::from_secs(2));
        assert_eq!(report.observation.failed_connections, 1);
        assert!(report
            .observation
            .failures
            .iter()
            .any(|failure| failure.code == "tls-verification-failed"));
        assert!(!report
            .observation
            .application
            .iter()
            .any(|item| { item.protocol == "https" && item.inspectability == "full" }));
    }
}
