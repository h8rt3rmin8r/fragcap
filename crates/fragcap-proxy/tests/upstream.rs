// SPDX-License-Identifier: Apache-2.0

use std::net::SocketAddr;
use std::time::Duration;

use fragcap_proxy::{
    connect_tls_upstream, connect_upstream, connect_upstream_cancellable,
    tls_client_config_with_roots, AuthorityHost, DestinationAuthority, DestinationPolicy,
    UpstreamBudgets, UpstreamCancellation, UpstreamStage,
};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use tokio::net::TcpListener;

#[test]
fn authority_parser_preserves_names_and_rejects_ambiguity() {
    let named = DestinationAuthority::parse("Example.Invalid:443").unwrap();
    assert_eq!(named.host(), &AuthorityHost::Dns("example.invalid".into()));
    assert_eq!(named.port(), 443);
    let ipv6 = DestinationAuthority::parse("[::1]:8443").unwrap();
    assert_eq!(ipv6.host(), &AuthorityHost::Ip("::1".parse().unwrap()));
    assert_eq!(ipv6.scope_id(), None);
    let scoped = DestinationAuthority::parse("[fe80::1%7]:8443").unwrap();
    assert_eq!(
        scoped.host(),
        &AuthorityHost::Ip("fe80::1".parse().unwrap())
    );
    assert_eq!(scoped.scope_id(), Some(7));
    assert_eq!(scoped.lookup_host(), "[fe80::1]:8443");
    let encoded_scope = DestinationAuthority::parse("[fe80::1%257]:8443").unwrap();
    assert_eq!(encoded_scope.scope_id(), Some(7));
    for bad in [
        "example.invalid",
        "user@example.invalid:80",
        "*.invalid:80",
        "[fe80::1%eth0]:80",
        "[fe80::1%0]:80",
        "[2001:db8::1%3]:80",
        "[::1%3]:80",
    ] {
        assert!(DestinationAuthority::parse(bad).is_err(), "{bad}");
    }
}

#[test]
fn policy_normalizes_ipv4_mapped_ipv6_before_listener_and_scope_checks() {
    let listener: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let mapped_listener: SocketAddr = "[::ffff:127.0.0.1]:8080".parse().unwrap();
    let mapped_private: SocketAddr = "[::ffff:10.0.0.1]:443".parse().unwrap();
    let mut policy = DestinationPolicy::new(listener);
    assert!(!policy.evaluate(mapped_listener).allowed);
    assert!(!policy.evaluate(mapped_private).allowed);
    policy.grant_for_test("10.0.0.1:443".parse().unwrap());
    assert!(policy.evaluate(mapped_private).allowed);
}

#[tokio::test]
async fn connector_reports_cancellation_as_a_reachable_terminal_stage() {
    let cancellation = UpstreamCancellation::default();
    cancellation.cancel();
    let short = Duration::from_millis(50);
    let error = connect_upstream_cancellable(
        &DestinationAuthority::parse("127.0.0.1:9").unwrap(),
        &DestinationPolicy::new("127.0.0.1:8".parse().unwrap()),
        UpstreamBudgets {
            dns: short,
            connect: short,
            read: short,
            write: short,
        },
        &cancellation,
    )
    .await
    .unwrap_err();
    assert_eq!(error.stage, UpstreamStage::Cancelled);
    assert_eq!(error.code, "cancelled");
}

#[test]
fn policy_refuses_listener_and_private_destinations_without_exact_grant() {
    let listener: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let mut policy = DestinationPolicy::new(listener);
    assert!(!policy.evaluate(listener).allowed);
    let origin: SocketAddr = "127.0.0.1:8081".parse().unwrap();
    assert!(!policy.evaluate(origin).allowed);
    policy.grant_for_test(origin);
    assert!(policy.evaluate(origin).allowed);
    assert!(!policy.evaluate(listener).allowed);
}

#[tokio::test]
async fn connector_uses_exact_grant_and_bounded_io() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut bytes = [0_u8; 4];
        stream.read_exact(&mut bytes).await.unwrap();
        stream.write_all(&bytes).await.unwrap();
    });
    let authority = DestinationAuthority::parse(&origin.to_string()).unwrap();
    let mut policy = DestinationPolicy::new("127.0.0.1:9".parse().unwrap());
    policy.grant_for_test(origin);
    let budgets = UpstreamBudgets {
        dns: Duration::from_secs(1),
        connect: Duration::from_secs(1),
        read: Duration::from_secs(1),
        write: Duration::from_secs(1),
    };
    let mut stream = connect_upstream(&authority, &policy, budgets)
        .await
        .unwrap();
    assert_eq!(stream.peer_addr().unwrap(), origin);
    stream.write_all(b"test").await.unwrap();
    let mut response = [0_u8; 4];
    stream.read(&mut response).await.unwrap();
    assert_eq!(&response, b"test");
    task.await.unwrap();
}

#[tokio::test]
async fn ipv6_literal_connects_to_the_exact_selected_peer() {
    let Ok(listener) = TcpListener::bind("[::1]:0").await else {
        return;
    };
    let origin = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        let (mut stream, peer) = listener.accept().await.unwrap();
        assert!(peer.is_ipv6());
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut bytes = [0_u8; 4];
        stream.read_exact(&mut bytes).await.unwrap();
        stream.write_all(&bytes).await.unwrap();
    });
    let authority = DestinationAuthority::parse(&origin.to_string()).unwrap();
    let mut policy = DestinationPolicy::new("[::1]:9".parse().unwrap());
    policy.grant_for_test(origin);
    let second = Duration::from_secs(1);
    let mut stream = connect_upstream(
        &authority,
        &policy,
        UpstreamBudgets {
            dns: second,
            connect: second,
            read: second,
            write: second,
        },
    )
    .await
    .unwrap();
    assert_eq!(stream.peer_addr().unwrap(), origin);
    assert!(stream.local_addr().unwrap().is_ipv6());
    stream.write_all(b"ipv6").await.unwrap();
    let mut response = [0_u8; 4];
    stream.read(&mut response).await.unwrap();
    assert_eq!(&response, b"ipv6");
    task.await.unwrap();
}

#[tokio::test]
async fn connector_refuses_private_destination_without_connecting() {
    let authority = DestinationAuthority::parse("127.0.0.1:9").unwrap();
    let policy = DestinationPolicy::new("127.0.0.1:8".parse().unwrap());
    let short = Duration::from_millis(50);
    let error = connect_upstream(
        &authority,
        &policy,
        UpstreamBudgets {
            dns: short,
            connect: short,
            read: short,
            write: short,
        },
    )
    .await
    .unwrap_err();
    assert_eq!(error.stage, UpstreamStage::Policy);
}

#[tokio::test]
async fn tls_connector_verifies_the_chain_and_requested_identity() {
    async fn origin() -> (
        SocketAddr,
        CertificateDer<'static>,
        tokio::task::JoinHandle<()>,
    ) {
        let generated = rcgen::generate_simple_self_signed(vec!["127.0.0.1".to_string()]).unwrap();
        let certificate = CertificateDer::from(generated.cert);
        let private = PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der());
        let provider = std::sync::Arc::new(rustls::crypto::ring::default_provider());
        let config = rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(vec![certificate.clone()], private.into())
            .unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let _ = tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(config))
                .accept(stream)
                .await;
        });
        (address, certificate, task)
    }

    let budgets = UpstreamBudgets {
        dns: Duration::from_secs(1),
        connect: Duration::from_secs(1),
        read: Duration::from_secs(1),
        write: Duration::from_secs(1),
    };
    let (address, certificate, task) = origin().await;
    let mut roots = rustls::RootCertStore::empty();
    roots.add(certificate).unwrap();
    let authority = DestinationAuthority::parse(&address.to_string()).unwrap();
    let mut policy = DestinationPolicy::new("127.0.0.1:9".parse().unwrap());
    policy.grant_for_test(address);
    let stream = connect_tls_upstream(
        &authority,
        &policy,
        budgets,
        tls_client_config_with_roots(roots).unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(stream.peer_addr().unwrap(), address);
    assert_eq!(stream.local_addr().unwrap().is_ipv4(), address.is_ipv4());
    task.await.unwrap();

    let (address, certificate, task) = origin().await;
    let mut roots = rustls::RootCertStore::empty();
    roots.add(certificate).unwrap();
    let authority = DestinationAuthority::parse(&format!("localhost:{}", address.port())).unwrap();
    let mut policy = DestinationPolicy::new("127.0.0.1:9".parse().unwrap());
    policy.grant_for_test(address);
    let error = connect_tls_upstream(
        &authority,
        &policy,
        budgets,
        tls_client_config_with_roots(roots).unwrap(),
    )
    .await
    .unwrap_err();
    assert_eq!(error.stage, UpstreamStage::Tls);
    assert_eq!(error.code, "certificate-validation");
    task.await.unwrap();
}
