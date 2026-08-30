// SPDX-License-Identifier: Apache-2.0

use crate::evidence::{Observation, Status};
use async_http_proxy::http_connect_tokio;
use hudsucker::{
    certificate_authority::{CertificateAuthority, RcgenAuthority},
    futures::{SinkExt, StreamExt},
    hyper::{body::Incoming, service::service_fn, Method, Request, Response, Version},
    hyper_util::{
        rt::{TokioExecutor, TokioIo},
        server::conn::auto,
    },
    rcgen::{BasicConstraints, CertificateParams, DistinguishedName, IsCa, KeyPair},
    rustls::crypto::aws_lc_rs,
    tokio_tungstenite::{self, tungstenite::Message},
    Body,
};
use std::{convert::Infallible, error::Error, net::SocketAddr};
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};
use tokio_rustls::TlsAcceptor;

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

    pub fn cert_pem(&self) -> &str {
        &self.cert_pem
    }

    fn reqwest_certificate(&self) -> reqwest::Certificate {
        reqwest::Certificate::from_pem(self.cert_pem.as_bytes())
            .expect("generated CA certificate must remain valid PEM")
    }

    pub fn authority(
        &self,
        cache_size: u64,
    ) -> Result<RcgenAuthority, Box<dyn Error + Send + Sync>> {
        let key = KeyPair::from_pem(&self.key_pem)?;
        let cert = CertificateParams::from_ca_cert_pem(&self.cert_pem)?.self_signed(&key)?;
        Ok(RcgenAuthority::new(
            key,
            cert,
            cache_size,
            aws_lc_rs::default_provider(),
        ))
    }
}

pub struct Origins {
    pub http: SocketAddr,
    pub https: SocketAddr,
    stops: Vec<oneshot::Sender<()>>,
    tasks: Vec<JoinHandle<()>>,
}

impl Origins {
    pub async fn start(ca: &CaMaterial) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let (http, http_stop, http_task) = start_http().await?;
        let (https, https_stop, https_task) = start_https(ca.authority(8)?).await?;
        Ok(Self {
            http,
            https,
            stops: vec![http_stop, https_stop],
            tasks: vec![http_task, https_task],
        })
    }

    pub async fn stop(self) {
        for stop in self.stops {
            let _ = stop.send(());
        }
        for task in self.tasks {
            let _ = tokio::time::timeout(std::time::Duration::from_secs(3), task).await;
        }
    }
}

async fn origin_service(req: Request<Incoming>) -> Result<Response<Body>, Infallible> {
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
        return Ok(response.map(Body::from));
    }

    let protocol = match req.version() {
        Version::HTTP_2 => "h2",
        _ => "http1",
    };
    let status = if req.method() == Method::POST {
        200
    } else {
        405
    };
    Ok(Response::builder()
        .status(status)
        .header("x-fragcap-protocol", protocol)
        .body(Body::from(RESPONSE_BODY))
        .expect("static response"))
}

async fn start_http(
) -> Result<(SocketAddr, oneshot::Sender<()>, JoinHandle<()>), Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
    let addr = listener.local_addr()?;
    let (stop_tx, mut stop_rx) = oneshot::channel();
    let task = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut stop_rx => break,
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
    Ok((addr, stop_tx, task))
}

async fn start_https(
    ca: impl CertificateAuthority,
) -> Result<(SocketAddr, oneshot::Sender<()>, JoinHandle<()>), Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
    let addr = listener.local_addr()?;
    let config = ca.gen_server_config(&"localhost".parse()?).await;
    let acceptor = TlsAcceptor::from(config);
    let (stop_tx, mut stop_rx) = oneshot::channel();
    let task = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut stop_rx => break,
                accepted = listener.accept() => {
                    let Ok((stream, _)) = accepted else { break };
                    let acceptor = acceptor.clone();
                    tokio::spawn(async move {
                        let Ok(stream) = acceptor.accept(stream).await else { return };
                        let _ = auto::Builder::new(TokioExecutor::new())
                            .serve_connection_with_upgrades(TokioIo::new(stream), service_fn(origin_service))
                            .await;
                    });
                }
            }
        }
    });
    Ok((addr, stop_tx, task))
}

pub async fn exercise(proxy: SocketAddr, origins: &Origins, ca: &CaMaterial) -> Vec<Observation> {
    let mut results = Vec::new();
    let proxy_url = format!("http://{proxy}");

    results.extend(
        request(
            reqwest::Client::builder().http1_only(),
            &proxy_url,
            format!("http://{}/http1", origins.http),
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
            format!("https://localhost:{}/https-http1", origins.https.port()),
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
            format!("https://localhost:{}/https-http2", origins.https.port()),
            "https-http2",
        )
        .await,
    );
    results.push(websocket(proxy, origins.http).await);
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
            )]
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

async fn websocket(proxy: SocketAddr, origin: SocketAddr) -> Observation {
    let result = async {
        let mut stream = tokio::net::TcpStream::connect(proxy).await?;
        http_connect_tokio(&mut stream, &origin.ip().to_string(), origin.port()).await?;
        let (mut websocket, _) =
            tokio_tungstenite::client_async(format!("ws://{origin}/websocket"), stream).await?;
        websocket
            .send(Message::Text(WEBSOCKET_BODY.to_string()))
            .await?;
        let message = websocket.next().await.ok_or("websocket closed")??;
        let bytes = message.into_data();
        let _ = websocket.close(None).await;
        Ok::<_, Box<dyn Error + Send + Sync>>(bytes)
    }
    .await;
    match result {
        Ok(bytes) if bytes == WEBSOCKET_BODY.as_bytes() => {
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
