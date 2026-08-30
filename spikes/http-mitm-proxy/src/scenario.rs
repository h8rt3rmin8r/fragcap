// SPDX-License-Identifier: Apache-2.0

use crate::evidence::{Observation, Status};
use async_http_proxy::http_connect_tokio;
use http_body_util::Full;
use http_mitm_proxy::futures::{SinkExt, StreamExt};
use hyper::{Method, Request, Response, Version, body::Incoming, service::service_fn};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto,
};
use rcgen::{BasicConstraints, CertificateParams, DistinguishedName, IsCa, Issuer, KeyPair};
use std::{convert::Infallible, error::Error, net::SocketAddr};
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};
use tokio_tungstenite::tungstenite::Message;

pub const REQUEST_BODY: &[u8] = b"fragcap-s099-request-body";
pub const RESPONSE_BODY: &[u8] = b"fragcap-s099-response-body";
pub const WEBSOCKET_BODY: &str = "fragcap-s099-websocket";

pub struct CaMaterial {
    cert_pem: String,
    key_pem: String,
}

impl CaMaterial {
    pub fn generate() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let mut params = CertificateParams::default();
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.distinguished_name = DistinguishedName::new();
        let key = KeyPair::generate()?;
        let cert = params.self_signed(&key)?;
        Ok(Self {
            cert_pem: cert.pem(),
            key_pem: key.serialize_pem(),
        })
    }

    pub fn issuer(&self) -> Result<Issuer<'static, KeyPair>, Box<dyn Error + Send + Sync>> {
        Ok(Issuer::from_ca_cert_pem(
            &self.cert_pem,
            KeyPair::from_pem(&self.key_pem)?,
        )?)
    }

    pub fn cert_pem(&self) -> &str {
        &self.cert_pem
    }

    fn reqwest_certificate(&self) -> reqwest::Certificate {
        reqwest::Certificate::from_pem(self.cert_pem.as_bytes()).expect("generated CA PEM")
    }

    fn native_connector(
        &self,
    ) -> Result<tokio_native_tls::TlsConnector, Box<dyn Error + Send + Sync>> {
        let cert = native_tls::Certificate::from_pem(self.cert_pem.as_bytes())?;
        let connector = native_tls::TlsConnector::builder()
            .add_root_certificate(cert)
            .build()?;
        Ok(tokio_native_tls::TlsConnector::from(connector))
    }
}

pub struct Origin {
    pub addr: SocketAddr,
    stop: oneshot::Sender<()>,
    task: JoinHandle<()>,
}

impl Origin {
    pub async fn start() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
        let addr = listener.local_addr()?;
        let (stop, mut stopped) = oneshot::channel();
        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut stopped => break,
                    accepted = listener.accept() => {
                        let Ok((stream, _)) = accepted else { break };
                        tokio::spawn(async move {
                            let _ = auto::Builder::new(TokioExecutor::new())
                                .serve_connection_with_upgrades(TokioIo::new(stream), service_fn(origin_service))
                                .await;
                        });
                    }
                }
            }
        });
        Ok(Self { addr, stop, task })
    }

    pub async fn stop(self) {
        let _ = self.stop.send(());
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), self.task).await;
    }
}

async fn origin_service(
    req: Request<Incoming>,
) -> Result<Response<Full<bytes::Bytes>>, Infallible> {
    if hyper_tungstenite::is_upgrade_request(&req) {
        let (response, websocket) = hyper_tungstenite::upgrade(req, None).expect("valid upgrade");
        tokio::spawn(async move {
            if let Ok(mut websocket) = websocket.await {
                while let Some(Ok(message)) = websocket.next().await {
                    if message.is_close() {
                        break;
                    }
                    let _ = websocket.send(message).await;
                }
            }
        });
        return Ok(response.map(|_| Full::new(bytes::Bytes::new())));
    }
    let protocol = if req.version() == Version::HTTP_2 {
        "h2"
    } else {
        "http1"
    };
    let status = if req.method() == Method::POST {
        200
    } else {
        405
    };
    Ok(Response::builder()
        .status(status)
        .header("x-fragcap-protocol", protocol)
        .body(Full::new(bytes::Bytes::from_static(RESPONSE_BODY)))
        .expect("static response"))
}

pub async fn exercise(proxy: SocketAddr, origin: SocketAddr, ca: &CaMaterial) -> Vec<Observation> {
    let proxy_url = format!("http://{proxy}");
    let mut results = Vec::new();
    results.extend(
        request(
            reqwest::Client::builder().http1_only(),
            &proxy_url,
            format!("http://{origin}/http1"),
            "http1",
        )
        .await,
    );
    results.extend(
        request(
            reqwest::Client::builder()
                .http1_only()
                .add_root_certificate(ca.reqwest_certificate()),
            &proxy_url,
            format!("https://localhost:{}/https-http1", origin.port()),
            "https-http1",
        )
        .await,
    );
    results.extend(
        request(
            reqwest::Client::builder()
                .http2_prior_knowledge()
                .add_root_certificate(ca.reqwest_certificate()),
            &proxy_url,
            format!("https://localhost:{}/https-http2", origin.port()),
            "https-http2",
        )
        .await,
    );
    results.push(websocket(proxy, origin, ca).await);
    results
}

async fn request(
    builder: reqwest::ClientBuilder,
    proxy: &str,
    url: String,
    scenario: &str,
) -> Vec<Observation> {
    let client = match builder
        .proxy(reqwest::Proxy::all(proxy).expect("proxy URL"))
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            return vec![Observation::result(
                scenario,
                "client-response",
                Status::Failed,
                error.to_string(),
            )];
        }
    };
    match client.post(url).body(REQUEST_BODY).send().await {
        Ok(response) => {
            let protocol = format!("{:?}", response.version());
            match response.bytes().await {
                Ok(bytes) if bytes.as_ref() == RESPONSE_BODY => vec![Observation::complete(
                    scenario,
                    "client-response",
                    Some(&protocol),
                    &bytes,
                )],
                Ok(bytes) => vec![Observation::result(
                    scenario,
                    "client-response",
                    Status::Truncated,
                    format!(
                        "expected {} bytes, received {}",
                        RESPONSE_BODY.len(),
                        bytes.len()
                    ),
                )],
                Err(error) => vec![Observation::result(
                    scenario,
                    "client-response",
                    Status::Failed,
                    error.to_string(),
                )],
            }
        }
        Err(error) => vec![Observation::result(
            scenario,
            "client-response",
            Status::Failed,
            error.to_string(),
        )],
    }
}

async fn websocket(proxy: SocketAddr, origin: SocketAddr, ca: &CaMaterial) -> Observation {
    let result = async {
        let mut stream = tokio::net::TcpStream::connect(proxy).await?;
        http_connect_tokio(&mut stream, "localhost", origin.port()).await?;
        let tls = ca.native_connector()?.connect("localhost", stream).await?;
        let (mut websocket, _) = tokio_tungstenite::client_async(
            format!("wss://localhost:{}/websocket", origin.port()),
            tls,
        )
        .await?;
        websocket.send(Message::Text(WEBSOCKET_BODY.into())).await?;
        let message = websocket.next().await.ok_or("websocket closed")??;
        let bytes = message.into_data();
        let _ = websocket.close(None).await;
        Ok::<_, Box<dyn Error + Send + Sync>>(bytes)
    }
    .await;
    match result {
        Ok(bytes) if bytes.as_ref() == WEBSOCKET_BODY.as_bytes() => {
            Observation::complete("websocket", "client-message", Some("websocket"), &bytes)
        }
        Ok(bytes) => Observation::result(
            "websocket",
            "client-message",
            Status::Truncated,
            format!(
                "expected {} bytes, received {}",
                WEBSOCKET_BODY.len(),
                bytes.len()
            ),
        ),
        Err(error) => Observation::result(
            "websocket",
            "client-message",
            Status::Failed,
            error.to_string(),
        ),
    }
}
