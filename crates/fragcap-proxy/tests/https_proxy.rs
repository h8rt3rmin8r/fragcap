// SPDX-License-Identifier: Apache-2.0

use std::io::{Read, Seek, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fragcap_proxy::{
    tls_client_config_with_roots, tls_client_config_with_roots_and_identity, ApplicationEvent,
    ApplicationEventKind, ApplicationEventSink, ClientIdentity, DestinationPolicy,
    EventDisposition, NativeProxyBackend, NativeProxyConfig, SessionKeyLog,
};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName};

#[derive(Default)]
struct Collector(Mutex<Vec<ApplicationEvent>>);

impl ApplicationEventSink for Collector {
    fn try_emit(&self, event: ApplicationEvent) -> EventDisposition {
        self.0.lock().unwrap().push(event);
        EventDisposition::Accepted
    }
}

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
    method: &'static str,
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
        assert!(request.starts_with(&format!("{method} /secure HTTP/1.1\r\n")));
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

fn round_trip(
    version: &'static rustls::SupportedProtocolVersion,
    session: &str,
    advertise_http_alpn: bool,
    method: &'static str,
) {
    let (origin, origin_certificate, server) = tls_origin(version, method);
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
    let mut key_log_file = tempfile::tempfile().unwrap();
    let key_log = Arc::new(SessionKeyLog::new(key_log_file.try_clone().unwrap()));
    let collector = Arc::new(Collector::default());
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_tls_client_config(tls_client_config_with_roots(upstream_roots).unwrap())
        .with_key_log(Arc::clone(&key_log))
        .with_application_event_sink(collector.clone())
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
    if advertise_http_alpn {
        client_config.alpn_protocols = vec![b"http/1.1".to_vec()];
    }
    let connection = rustls::ClientConnection::new(
        Arc::new(client_config),
        ServerName::try_from("localhost").unwrap().to_owned(),
    )
    .unwrap();
    let mut tls = rustls::StreamOwned::new(connection, tcp);
    write!(
        tls,
        "{} /secure HTTP/1.1\r\nHost: localhost:{}\r\nConnection: close\r\n\r\n",
        method,
        origin.port(),
    )
    .unwrap();
    let mut response = String::new();
    if let Err(error) = tls.read_to_string(&mut response) {
        // HTTP Content-Length proves this fixture's response is complete.
        // rustls can still report a transport EOF without close_notify after
        // returning every authenticated application byte.
        assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof, "{error}");
    }
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
            && facts.alpn.as_deref() == advertise_http_alpn.then_some(b"http/1.1".as_slice())
    }));
    let events = collector.0.lock().unwrap();
    assert!(events.iter().any(|event| {
        matches!(
            event.kind,
            ApplicationEventKind::UpstreamSocket(value)
                if value.protocol == "https"
                    && value.peer == origin
                    && value.local.is_ipv4()
        )
    }));
    drop(events);
    key_log_file.rewind().unwrap();
    let mut secrets = String::new();
    key_log_file.read_to_string(&mut secrets).unwrap();
    assert!(!secrets.is_empty());
    assert!(secrets
        .lines()
        .all(|line| line.split_ascii_whitespace().count() == 3));
    if std::ptr::eq(version, &rustls::version::TLS12) {
        assert!(secrets
            .lines()
            .any(|line| line.starts_with("CLIENT_RANDOM ")));
    } else {
        assert!(secrets
            .lines()
            .any(|line| line.starts_with("CLIENT_HANDSHAKE_TRAFFIC_SECRET ")));
        assert!(secrets
            .lines()
            .any(|line| line.starts_with("SERVER_TRAFFIC_SECRET_0 ")));
    }
    assert_eq!(key_log.status().records, secrets.lines().count() as u64);
}

#[test]
fn tls12_connect_performs_two_verified_boundaries_and_relays_http() {
    round_trip(&rustls::version::TLS12, "s104-https-tls12", true, "GET");
}

#[test]
fn tls13_connect_performs_two_verified_boundaries_and_relays_http() {
    round_trip(&rustls::version::TLS13, "s104-https-tls13", true, "GET");
}

#[test]
fn no_alpn_http_prefix_retains_the_http_engine() {
    round_trip(
        &rustls::version::TLS13,
        "s116-no-alpn-http",
        false,
        "PROPFIND",
    );
}

#[test]
fn no_alpn_connect_records_protocol_unknown_decrypted_chunks() {
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let origin_certificate = CertificateDer::from(generated.cert);
    let private = PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der());
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let origin_config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![origin_certificate.clone()], private.into())
        .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (tcp, _) = listener.accept().unwrap();
        let connection = rustls::ServerConnection::new(Arc::new(origin_config)).unwrap();
        let mut tls = rustls::StreamOwned::new(connection, tcp);
        let mut request = [0_u8; 5];
        tls.read_exact(&mut request).unwrap();
        assert_eq!(&request, b"\x01game");
        tls.write_all(b"\x02reply").unwrap();
        tls.flush().unwrap();
    });

    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        8,
        16 * 1024,
        Duration::from_secs(2),
    )
    .unwrap()
    .with_session_id("s116-generic-tls")
    .unwrap();
    let mut policy = DestinationPolicy::new(config.listen());
    policy.grant_for_test(origin);
    let mut roots = rustls::RootCertStore::empty();
    roots.add(origin_certificate).unwrap();
    let collector = Arc::new(Collector::default());
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_tls_client_config(tls_client_config_with_roots(roots).unwrap())
        .with_application_event_sink(collector.clone())
        .start(Duration::from_secs(2))
        .unwrap();
    let mut tcp = TcpStream::connect(lease.endpoint()).unwrap();
    let authorization = lease.capability_proof().proxy_authorization();
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
    let client_config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(client_roots)
        .with_no_client_auth();
    let connection = rustls::ClientConnection::new(
        Arc::new(client_config),
        ServerName::try_from("localhost").unwrap().to_owned(),
    )
    .unwrap();
    let mut tls = rustls::StreamOwned::new(connection, tcp);
    tls.write_all(b"\x01game").unwrap();
    tls.flush().unwrap();
    let mut response = [0_u8; 6];
    tls.read_exact(&mut response).unwrap();
    assert_eq!(&response, b"\x02reply");
    drop(tls);
    server.join().unwrap();

    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(
        report.observation.protocol.generic_streams_tls_intercepted,
        1
    );
    assert_eq!(
        report.observation.protocol.generic_stream_bytes_observed,
        11
    );
    assert!(report.observation.application.iter().any(|observation| {
        observation.protocol == "tls-protocol-unknown"
            && observation.inspectability == "protocol-unknown"
    }));
    let events = collector.0.lock().unwrap();
    assert!(events.iter().any(|event| {
        matches!(
            &event.kind,
            ApplicationEventKind::GenericStreamChunk(chunk)
                if chunk.provenance == fragcap_proxy::GenericStreamProvenance::TlsDecrypted
                    && chunk.bytes.as_ref() == b"\x01game"
        )
    }));
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
            .any(|failure| failure.code == "certificate-validation"));
        assert!(!report
            .observation
            .application
            .iter()
            .any(|item| { item.protocol == "https" && item.inspectability == "full" }));
    }
}

#[test]
fn explicit_operator_identity_authenticates_to_mtls_upstream() {
    let origin = rcgen::generate_simple_self_signed(vec!["127.0.0.1".to_string()]).unwrap();
    let origin_cert = CertificateDer::from(origin.cert);
    let origin_key = PrivatePkcs8KeyDer::from(origin.signing_key.serialize_der());
    let client_ca_key = rcgen::KeyPair::generate().unwrap();
    let mut client_ca_params = rcgen::CertificateParams::new(Vec::<String>::new()).unwrap();
    client_ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    client_ca_params.key_usages = vec![rcgen::KeyUsagePurpose::KeyCertSign];
    let client_ca_certificate = client_ca_params.self_signed(&client_ca_key).unwrap();
    let client_key_pair = rcgen::KeyPair::generate().unwrap();
    let mut client_params = rcgen::CertificateParams::new(Vec::<String>::new()).unwrap();
    let mut client_name = rcgen::DistinguishedName::new();
    client_name.push(rcgen::DnType::CommonName, "fragcap controlled client");
    client_params.distinguished_name = client_name;
    client_params.key_usages = vec![rcgen::KeyUsagePurpose::DigitalSignature];
    client_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
    let client_issuer = rcgen::Issuer::from_params(&client_ca_params, &client_ca_key);
    let client_certificate = client_params
        .signed_by(&client_key_pair, &client_issuer)
        .unwrap();
    let client_cert = client_certificate.der().clone();
    let client_key = client_key_pair.serialize_der();

    let mut client_roots = rustls::RootCertStore::empty();
    client_roots
        .add(client_ca_certificate.der().clone())
        .unwrap();
    let verifier = rustls::server::WebPkiClientVerifier::builder(Arc::new(client_roots))
        .build()
        .unwrap();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let server_config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_client_cert_verifier(verifier)
        .with_single_cert(vec![origin_cert.clone()], origin_key.into())
        .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (tcp, _) = listener.accept().unwrap();
        let connection = rustls::ServerConnection::new(Arc::new(server_config)).unwrap();
        let mut tls = rustls::StreamOwned::new(connection, tcp);
        let mut byte = [0_u8; 1];
        tls.read_exact(&mut byte).unwrap();
        assert_eq!(byte, [7]);
        tls.write_all(&[8]).unwrap();
        tls.flush().unwrap();
    });

    let identity = ClientIdentity::from_bytes(client_cert.as_ref(), &client_key).unwrap();
    let mut roots = rustls::RootCertStore::empty();
    roots.add(origin_cert).unwrap();
    let config = tls_client_config_with_roots_and_identity(roots, Some(&identity)).unwrap();
    let connection = rustls::ClientConnection::new(
        config,
        ServerName::IpAddress("127.0.0.1".parse::<std::net::IpAddr>().unwrap().into()),
    )
    .unwrap();
    let mut stream = rustls::StreamOwned::new(connection, TcpStream::connect(address).unwrap());
    stream.write_all(&[7]).unwrap();
    stream.flush().unwrap();
    let mut response = [0_u8; 1];
    stream.read_exact(&mut response).unwrap();
    assert_eq!(response, [8]);
    server.join().unwrap();
}

#[test]
fn mismatched_operator_identity_is_rejected_before_connection() {
    let certificate = rcgen::generate_simple_self_signed(vec!["client".to_string()]).unwrap();
    let wrong_key = rcgen::KeyPair::generate().unwrap().serialize_der();
    let identity = ClientIdentity::from_bytes(certificate.cert.der(), &wrong_key).unwrap();
    let root = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let mut roots = rustls::RootCertStore::empty();
    roots.add(CertificateDer::from(root.cert)).unwrap();
    let error = tls_client_config_with_roots_and_identity(roots, Some(&identity)).unwrap_err();
    assert_eq!(error.code, "client-identity-invalid");
    assert!(!format!("{identity:?}").contains("PRIVATE"));
}
