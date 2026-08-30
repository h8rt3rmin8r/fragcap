// SPDX-License-Identifier: Apache-2.0

use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn tcp_round_trip(payload: &[u8]) -> Vec<u8> {
    use std::io::{Read, Write};
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (ready_tx, ready_rx) = mpsc::sync_channel(0);
    let server = thread::spawn(move || {
        ready_tx.send(()).unwrap();
        let (mut stream, _) = listener.accept().unwrap();
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).unwrap();
        stream.write_all(&bytes).unwrap();
        bytes
    });
    ready_rx.recv().unwrap();
    let mut client = TcpStream::connect(address).unwrap();
    client.write_all(payload).unwrap();
    client.shutdown(std::net::Shutdown::Write).unwrap();
    let mut echoed = Vec::new();
    client.read_to_end(&mut echoed).unwrap();
    assert_eq!(server.join().unwrap(), payload);
    echoed
}

pub fn udp_round_trip(payload: &[u8]) -> Vec<u8> {
    let server = UdpSocket::bind("127.0.0.1:0").unwrap();
    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    server
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    client.connect(server.local_addr().unwrap()).unwrap();
    server.connect(client.local_addr().unwrap()).unwrap();
    client.send(payload).unwrap();
    let mut bytes = [0_u8; 256];
    let length = server.recv(&mut bytes).unwrap();
    server.send(&bytes[..length]).unwrap();
    let length = client.recv(&mut bytes).unwrap();
    bytes[..length].to_vec()
}

pub async fn quic_round_trip(payload: &[u8]) -> Vec<u8> {
    use quinn::{ClientConfig, Endpoint, ServerConfig};
    use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};

    let generated = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let certificate = CertificateDer::from(generated.cert);
    let private_key = PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der());
    let server_config =
        ServerConfig::with_single_cert(vec![certificate.clone()], private_key.into()).unwrap();
    let server = Endpoint::server(server_config, "127.0.0.1:0".parse().unwrap()).unwrap();
    let server_address = server.local_addr().unwrap();
    let server_task = tokio::spawn(async move {
        let incoming = server.accept().await.unwrap();
        let connection = incoming.await.unwrap();
        let (mut send, mut receive) = connection.accept_bi().await.unwrap();
        let bytes = receive.read_to_end(256).await.unwrap();
        send.write_all(&bytes).await.unwrap();
        send.finish().unwrap();
        connection.closed().await;
        server.wait_idle().await;
        bytes
    });

    let mut roots = rustls::RootCertStore::empty();
    roots.add(certificate).unwrap();
    let mut client = Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
    client
        .set_default_client_config(ClientConfig::with_root_certificates(Arc::new(roots)).unwrap());
    let connection = client
        .connect(server_address, "localhost")
        .unwrap()
        .await
        .unwrap();
    let (mut send, mut receive) = connection.open_bi().await.unwrap();
    send.write_all(payload).await.unwrap();
    send.finish().unwrap();
    let echoed = receive.read_to_end(256).await.unwrap();
    connection.close(0_u32.into(), b"complete");
    client.wait_idle().await;
    assert_eq!(server_task.await.unwrap(), payload);
    echoed
}
