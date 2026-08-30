// SPDX-License-Identifier: Apache-2.0

use crate::{
    evidence::{BackendRun, Observation, Status},
    scenario::{self, CaMaterial},
};
use http_body_util::BodyExt;
use hudsucker::{
    certificate_authority::{CertificateAuthority, RcgenAuthority},
    futures::{Sink, SinkExt, Stream, StreamExt},
    hyper::{Request, Response},
    hyper_util::{
        client::legacy::{connect::HttpConnector, Client},
        rt::TokioExecutor,
    },
    rustls::{self, ServerConfig},
    tokio_tungstenite::{
        self,
        tungstenite::{self, Message},
    },
    Body, HttpContext, HttpHandler, Proxy, RequestOrResponse, WebSocketContext, WebSocketHandler,
};
use std::{
    fmt,
    fs::{File, OpenOptions},
    io::Write,
    net::SocketAddr,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};

const CACHE_CAPACITY: u64 = 32;

#[derive(Clone, Default)]
struct RecordingHandler {
    observations: Arc<Mutex<Vec<Observation>>>,
    current_scenario: Option<String>,
}

impl RecordingHandler {
    fn take(&self) -> Vec<Observation> {
        std::mem::take(&mut *self.observations.lock().expect("observation lock"))
    }
}

impl HttpHandler for RecordingHandler {
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        req: Request<Body>,
    ) -> RequestOrResponse {
        let (parts, body) = req.into_parts();
        let scenario = scenario_from_path(parts.uri.path());
        if parts.method == hudsucker::hyper::Method::CONNECT {
            return RequestOrResponse::Request(Request::from_parts(parts, body));
        }
        self.current_scenario = Some(scenario.to_string());
        match body.collect().await {
            Ok(collected) => {
                let bytes = collected.to_bytes();
                let kind = if scenario == "websocket" {
                    "proxy-handshake-request"
                } else {
                    "proxy-request"
                };
                let observation = if scenario == "websocket" {
                    Observation::complete_empty(
                        scenario,
                        kind,
                        Some(&format!("{:?}", parts.version)),
                    )
                } else {
                    Observation::complete(
                        scenario,
                        kind,
                        Some(&format!("{:?}", parts.version)),
                        &bytes,
                    )
                };
                self.observations
                    .lock()
                    .expect("observation lock")
                    .push(observation);
                RequestOrResponse::Request(Request::from_parts(
                    parts,
                    Body::from(http_body_util::Full::new(bytes)),
                ))
            }
            Err(error) => {
                let kind = if scenario == "websocket" {
                    "proxy-handshake-request"
                } else {
                    "proxy-request"
                };
                self.observations
                    .lock()
                    .expect("observation lock")
                    .push(Observation::result(
                        scenario,
                        kind,
                        Status::Failed,
                        error.to_string(),
                    ));
                RequestOrResponse::Request(Request::from_parts(parts, Body::empty()))
            }
        }
    }

    async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<Body>) -> Response<Body> {
        let (parts, body) = res.into_parts();
        match body.collect().await {
            Ok(collected) => {
                let bytes = collected.to_bytes();
                let scenario = self.current_scenario.as_deref().unwrap_or("unknown");
                let kind = if scenario == "websocket" {
                    "proxy-handshake-response"
                } else {
                    "proxy-response"
                };
                let observation = if scenario == "websocket" {
                    Observation::complete_empty(
                        scenario,
                        kind,
                        Some(&format!("{:?}", parts.version)),
                    )
                } else {
                    Observation::complete(
                        scenario,
                        kind,
                        Some(&format!("{:?}", parts.version)),
                        &bytes,
                    )
                };
                self.observations
                    .lock()
                    .expect("observation lock")
                    .push(observation);
                Response::from_parts(parts, Body::from(http_body_util::Full::new(bytes)))
            }
            Err(error) => {
                let scenario = self.current_scenario.as_deref().unwrap_or("unknown");
                let kind = if scenario == "websocket" {
                    "proxy-handshake-response"
                } else {
                    "proxy-response"
                };
                self.observations
                    .lock()
                    .expect("observation lock")
                    .push(Observation::result(
                        scenario,
                        kind,
                        Status::Failed,
                        error.to_string(),
                    ));
                Response::from_parts(parts, Body::empty())
            }
        }
    }
}

impl WebSocketHandler for RecordingHandler {
    async fn handle_websocket(
        self,
        ctx: WebSocketContext,
        mut stream: impl Stream<Item = Result<Message, tungstenite::Error>> + Unpin + Send + 'static,
        mut sink: impl Sink<Message, Error = tungstenite::Error> + Unpin + Send + 'static,
    ) {
        while let Some(message) = stream.next().await {
            match message {
                Ok(message) => {
                    let direction = match ctx {
                        WebSocketContext::ClientToServer { .. } => "client-to-server",
                        WebSocketContext::ServerToClient { .. } => "server-to-client",
                    };
                    let bytes = message.clone().into_data();
                    let mut observation = Observation::complete(
                        "websocket",
                        "proxy-message",
                        Some("websocket"),
                        &bytes,
                    );
                    observation.direction = Some(direction.to_string());
                    self.observations
                        .lock()
                        .expect("observation lock")
                        .push(observation);
                    if sink.send(message).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    }
}

fn scenario_from_path(path: &str) -> &'static str {
    if path.contains("https-http2") {
        "https-http2"
    } else if path.contains("https-http1") {
        "https-http1"
    } else if path.contains("websocket") {
        "websocket"
    } else {
        "http1"
    }
}

struct KeyLogAuthority {
    inner: RcgenAuthority,
    log: Arc<ResearchKeyLog>,
}

impl CertificateAuthority for KeyLogAuthority {
    async fn gen_server_config(&self, authority: &http::uri::Authority) -> Arc<ServerConfig> {
        let original = self.inner.gen_server_config(authority).await;
        let mut config = original.as_ref().clone();
        config.key_log = self.log.clone();
        Arc::new(config)
    }
}

struct ResearchKeyLog {
    file: Mutex<File>,
}

impl fmt::Debug for ResearchKeyLog {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResearchKeyLog")
            .finish_non_exhaustive()
    }
}

impl ResearchKeyLog {
    fn new(path: &Path) -> std::io::Result<Self> {
        Ok(Self {
            file: Mutex::new(
                OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(path)?,
            ),
        })
    }
}

impl rustls::KeyLog for ResearchKeyLog {
    fn log(&self, label: &str, client_random: &[u8], secret: &[u8]) {
        let mut file = self.file.lock().expect("key-log lock");
        let _ = writeln!(file, "{label} {} {}", hex(client_random), hex(secret));
        let _ = file.flush();
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

struct RunningProxy {
    addr: SocketAddr,
    stop: oneshot::Sender<()>,
    task: JoinHandle<Result<(), hudsucker::Error>>,
    handler: RecordingHandler,
}

async fn start_proxy(ca: &CaMaterial, key_log: &Path) -> Result<RunningProxy, String> {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .map_err(|error| error.to_string())?;
    let addr = listener.local_addr().map_err(|error| error.to_string())?;
    let (stop, stopped) = oneshot::channel();
    let log = Arc::new(ResearchKeyLog::new(key_log).map_err(|error| error.to_string())?);
    let authority = KeyLogAuthority {
        inner: ca
            .authority(CACHE_CAPACITY)
            .map_err(|error| error.to_string())?,
        log,
    };
    let handler = RecordingHandler::default();
    let native = native_tls::TlsConnector::builder()
        .add_root_certificate(
            native_tls::Certificate::from_pem(ca.cert_pem().as_bytes())
                .map_err(|error| error.to_string())?,
        )
        .build()
        .map_err(|error| error.to_string())?;
    let websocket_connector = tokio_tungstenite::Connector::NativeTls(native.clone());
    let mut http = HttpConnector::new();
    http.enforce_http(false);
    let https: hyper_tls::HttpsConnector<HttpConnector> =
        (http, tokio_native_tls::TlsConnector::from(native)).into();
    let client: Client<_, Body> = Client::builder(TokioExecutor::new()).build(https);
    let proxy = Proxy::builder()
        .with_listener(listener)
        .with_ca(authority)
        .with_client(client)
        .with_http_handler(handler.clone())
        .with_websocket_handler(handler.clone())
        .with_websocket_connector(websocket_connector)
        .with_graceful_shutdown(async {
            let _ = stopped.await;
        })
        .build()
        .map_err(|error| error.to_string())?;
    let task = tokio::spawn(proxy.start());
    Ok(RunningProxy {
        addr,
        stop,
        task,
        handler,
    })
}

async fn stop_proxy(proxy: RunningProxy) -> (Status, Vec<Observation>) {
    let observations = proxy.handler.take();
    let _ = proxy.stop.send(());
    let status = match tokio::time::timeout(Duration::from_secs(5), proxy.task).await {
        Ok(Ok(Ok(()))) => Status::Complete,
        _ => Status::Failed,
    };
    (status, observations)
}

async fn stop_proxy_with_active_connection(proxy: RunningProxy) -> Status {
    let connection = match tokio::net::TcpStream::connect(proxy.addr).await {
        Ok(connection) => connection,
        Err(_) => return Status::Failed,
    };
    let stopping = tokio::spawn(stop_proxy(proxy));
    tokio::time::sleep(Duration::from_millis(50)).await;
    drop(connection);
    match stopping.await {
        Ok((status, _)) => status,
        Err(_) => Status::Failed,
    }
}

pub async fn run() -> BackendRun {
    let directory = match tempfile::tempdir() {
        Ok(directory) => directory,
        Err(error) => return BackendRun::failed("hudsucker", "0.23.0", error.to_string()),
    };
    let ca = match CaMaterial::generate() {
        Ok(ca) => ca,
        Err(error) => return BackendRun::failed("hudsucker", "0.23.0", error.to_string()),
    };
    let origins = match scenario::Origins::start(&ca).await {
        Ok(origins) => origins,
        Err(error) => return BackendRun::failed("hudsucker", "0.23.0", error.to_string()),
    };
    let key_log = directory.path().join("candidate.keys");
    let proxy = match start_proxy(&ca, &key_log).await {
        Ok(proxy) => proxy,
        Err(error) => return BackendRun::failed("hudsucker", "0.23.0", error),
    };
    let mut observations = scenario::exercise(proxy.addr, &origins).await;
    let (first_shutdown, mut proxy_observations) = stop_proxy(proxy).await;
    observations.append(&mut proxy_observations);
    let mut shutdown_trials = vec![first_shutdown];
    for index in 1..10 {
        let path = directory.path().join(format!("candidate-{index}.keys"));
        match start_proxy(&ca, &path).await {
            Ok(proxy) if index == 1 => {
                let status = stop_proxy_with_active_connection(proxy).await;
                observations.push(Observation::result(
                    "lifecycle",
                    "active-connection-shutdown",
                    status,
                    "accepted connection released during bounded graceful shutdown",
                ));
                shutdown_trials.push(status);
            }
            Ok(proxy) => shutdown_trials.push(stop_proxy(proxy).await.0),
            Err(_) => shutdown_trials.push(Status::Failed),
        }
    }
    origins.stop().await;
    let key_log_lines = std::fs::read_to_string(&key_log)
        .map(|content| content.lines().count())
        .unwrap_or(0);
    observations.push(Observation::result(
        "matrix",
        "har-source",
        Status::Complete,
        "public handlers expose method, URI, version, headers, status, and complete synthetic bodies",
    ));
    let mut run = BackendRun {
        backend: "hudsucker".to_string(),
        version: "0.23.0".to_string(),
        platform: "windows-x86_64".to_string(),
        loopback_only: true,
        trust_store_mutated: false,
        cache_capacity: Some(CACHE_CAPACITY),
        key_log_lines,
        shutdown_trials,
        observations,
        limitations: vec![
            "RcgenAuthority exposes capacity and tracing but no public cache enumeration or explicit invalidation".to_string(),
            "full body buffering is a measurement technique, not a production backpressure design".to_string(),
        ],
    };
    run.sort();
    run
}
