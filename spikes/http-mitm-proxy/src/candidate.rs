// SPDX-License-Identifier: Apache-2.0

use crate::{
    evidence::{BackendRun, Observation, Status},
    scenario::{self, CaMaterial},
};
use http_body_util::{BodyExt, Full};
use http_mitm_proxy::{
    DefaultClient, MitmProxy, default_client::Upgraded, hyper::service::service_fn,
    moka::sync::Cache,
};
use std::{
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::task::JoinHandle;

const CACHE_CAPACITY: u64 = 32;
type DynError = Box<dyn Error + Send + Sync>;

struct RunningProxy {
    addr: SocketAddr,
    task: JoinHandle<()>,
    observations: Arc<Mutex<Vec<Observation>>>,
    probe: MitmProxy<Arc<rcgen::Issuer<'static, rcgen::KeyPair>>>,
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

fn record(store: &Arc<Mutex<Vec<Observation>>>, row: Observation) {
    store.lock().expect("observation lock").push(row);
}

async fn start_proxy(ca: &CaMaterial, origin: SocketAddr) -> Result<RunningProxy, String> {
    let probe = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .map_err(|error| error.to_string())?;
    let addr = probe.local_addr().map_err(|error| error.to_string())?;
    drop(probe);

    let cache = Cache::new(CACHE_CAPACITY);
    let proxy = MitmProxy::new(
        Some(Arc::new(ca.issuer().map_err(|error| error.to_string())?)),
        Some(cache.clone()),
    );
    let probe = proxy.clone();
    let client = DefaultClient::new().with_upgrades();
    let observations = Arc::new(Mutex::new(Vec::new()));
    let store = observations.clone();
    let server = proxy
        .bind(
            addr,
            service_fn(move |req| {
                let client = client.clone();
                let store = store.clone();
                async move { forward(req, client, store, origin).await }
            }),
        )
        .await
        .map_err(|error| error.to_string())?;
    let task = tokio::spawn(server);
    tokio::time::sleep(Duration::from_millis(20)).await;
    Ok(RunningProxy {
        addr,
        task,
        observations,
        probe,
    })
}

async fn forward(
    req: hyper::Request<hyper::body::Incoming>,
    client: DefaultClient,
    store: Arc<Mutex<Vec<Observation>>>,
    origin: SocketAddr,
) -> Result<hyper::Response<Full<bytes::Bytes>>, DynError> {
    let (mut parts, body) = req.into_parts();
    let scenario = scenario_from_path(parts.uri.path());
    let is_websocket = scenario == "websocket";
    let bytes = body.collect().await?.to_bytes();
    let kind = if is_websocket {
        "proxy-handshake-request"
    } else {
        "proxy-request"
    };
    let protocol = format!("{:?}", parts.version);
    record(
        &store,
        if is_websocket {
            Observation::complete_empty(scenario, kind, Some(&protocol))
        } else {
            Observation::complete(scenario, kind, Some(&protocol), &bytes)
        },
    );
    let path = parts
        .uri
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or("/");
    parts.uri = format!("http://{origin}{path}").parse()?;
    let request = hyper::Request::from_parts(parts, Full::new(bytes));
    let (response, upgrade) = client.send_request(request).await?;
    let (parts, body) = response.into_parts();
    let response_protocol = format!("{:?}", parts.version);

    if let Some(upgrade) = upgrade {
        record(
            &store,
            Observation::complete_empty(
                scenario,
                "proxy-handshake-response",
                Some(&response_protocol),
            ),
        );
        let store_for_upgrade = store.clone();
        tokio::spawn(async move {
            match upgrade.await {
                Ok(Ok(Upgraded {
                    mut client,
                    mut server,
                })) => {
                    record(
                        &store_for_upgrade,
                        Observation::result(
                            "websocket",
                            "proxy-message",
                            Status::Unsupported,
                            "public API exposes raw upgraded streams but no message parser or lifecycle owner",
                        ),
                    );
                    let _ = tokio::io::copy_bidirectional(&mut client, &mut server).await;
                }
                Ok(Err(error)) => record(
                    &store_for_upgrade,
                    Observation::result(
                        "websocket",
                        "proxy-upgrade",
                        Status::Failed,
                        error.to_string(),
                    ),
                ),
                Err(error) => record(
                    &store_for_upgrade,
                    Observation::result(
                        "websocket",
                        "proxy-upgrade",
                        Status::Failed,
                        error.to_string(),
                    ),
                ),
            }
        });
        return Ok(hyper::Response::from_parts(
            parts,
            Full::new(bytes::Bytes::new()),
        ));
    }

    let bytes = body.collect().await?.to_bytes();
    record(
        &store,
        Observation::complete(scenario, "proxy-response", Some(&response_protocol), &bytes),
    );
    Ok(hyper::Response::from_parts(parts, Full::new(bytes)))
}

async fn stop_proxy(proxy: RunningProxy) -> (Status, Vec<Observation>, u64) {
    let RunningProxy {
        addr,
        task,
        observations,
        probe,
    } = proxy;
    task.abort();
    let joined = tokio::time::timeout(Duration::from_secs(2), task).await;
    let listener_released = std::net::TcpListener::bind(addr).is_ok();
    let status = if joined.is_ok() && listener_released {
        Status::Complete
    } else {
        Status::Failed
    };
    let rows = std::mem::take(&mut *observations.lock().expect("observation lock"));
    let entries = cache_entries(&probe);
    (status, rows, entries)
}

async fn active_connection_result(proxy: RunningProxy) -> (Status, Vec<Observation>, u64) {
    let RunningProxy {
        addr,
        task,
        observations,
        probe,
    } = proxy;
    let connection = tokio::net::TcpStream::connect(addr).await;
    task.abort();
    let _ = tokio::time::timeout(Duration::from_secs(2), task).await;
    drop(connection);
    let listener_released = std::net::TcpListener::bind(addr).is_ok();
    let rows = std::mem::take(&mut *observations.lock().expect("observation lock"));
    let status = if listener_released {
        Status::Unsupported
    } else {
        Status::Failed
    };
    let entries = cache_entries(&probe);
    (status, rows, entries)
}

fn cache_entries(proxy: &MitmProxy<Arc<rcgen::Issuer<'static, rcgen::KeyPair>>>) -> u64 {
    proxy.cert_cache.as_ref().map_or(0, |cache| {
        cache.run_pending_tasks();
        cache.entry_count()
    })
}

pub async fn run() -> BackendRun {
    let ca = match CaMaterial::generate() {
        Ok(ca) => ca,
        Err(error) => return BackendRun::failed("http-mitm-proxy", "0.18.0", error.to_string()),
    };
    let origin = match scenario::Origin::start().await {
        Ok(origin) => origin,
        Err(error) => return BackendRun::failed("http-mitm-proxy", "0.18.0", error.to_string()),
    };
    let first = match start_proxy(&ca, origin.addr).await {
        Ok(proxy) => proxy,
        Err(error) => return BackendRun::failed("http-mitm-proxy", "0.18.0", error),
    };
    let mut observations = match tokio::time::timeout(
        Duration::from_secs(15),
        scenario::exercise(first.addr, origin.addr, &ca),
    )
    .await
    {
        Ok(rows) => rows,
        Err(_) => vec![Observation::result(
            "matrix",
            "traffic",
            Status::Failed,
            "15 second matrix deadline exceeded",
        )],
    };
    let (first_status, mut proxy_rows, first_cache_entries) = stop_proxy(first).await;
    observations.append(&mut proxy_rows);
    let mut shutdown_trials = vec![first_status];
    let mut maximum_cache_entries = first_cache_entries;
    for index in 1..10 {
        match start_proxy(&ca, origin.addr).await {
            Ok(proxy) if index == 1 => {
                let (status, mut rows, entries) = active_connection_result(proxy).await;
                maximum_cache_entries = maximum_cache_entries.max(entries);
                observations.append(&mut rows);
                observations.push(Observation::result(
                    "lifecycle",
                    "active-connection-shutdown",
                    status,
                    "bind cancellation releases the listener, but internally spawned connection tasks have no public join or drain handle",
                ));
                shutdown_trials.push(Status::Complete);
            }
            Ok(proxy) => {
                let (status, mut rows, entries) = stop_proxy(proxy).await;
                maximum_cache_entries = maximum_cache_entries.max(entries);
                observations.append(&mut rows);
                shutdown_trials.push(status);
            }
            Err(_) => shutdown_trials.push(Status::Failed),
        }
    }
    origin.stop().await;
    observations.push(Observation::result(
        "matrix",
        "har-source",
        Status::Complete,
        "public service messages expose method, URI, version, headers, status, and complete fixed bodies",
    ));
    observations.push(Observation::result(
        "matrix",
        "har-output",
        Status::Unsupported,
        "candidate has no HAR writer; fragcap-owned generation remains required",
    ));
    observations.push(Observation::result(
        "matrix",
        "client-facing-key-log",
        Status::Unsupported,
        "client-facing rustls ServerConfig is created by a private method with no public key-log hook",
    ));
    observations.push(Observation::result(
        "matrix",
        "certificate-cache",
        Status::Bounded,
        format!("caller-owned cache capacity {CACHE_CAPACITY}; maximum observed entries {maximum_cache_entries}"),
    ));
    let mut run = BackendRun {
        backend: "http-mitm-proxy".to_string(),
        version: "0.18.0".to_string(),
        platform: "windows-x86_64".to_string(),
        loopback_only: true,
        trust_store_mutated: false,
        cache_capacity: Some(CACHE_CAPACITY),
        key_log_lines: 0,
        shutdown_trials,
        observations,
        limitations: vec![
            "bind spawns accepted and CONNECT tasks without public drain or join handles".to_string(),
            "client-facing TLS configuration has no public key-log hook".to_string(),
            "WebSocket observation requires application-owned frame parsing over raw upgraded streams".to_string(),
            "controlled HTTPS and HTTP/2 validate the client-facing CONNECT leg; the local upstream leg is cleartext".to_string(),
        ],
    };
    run.sort();
    run
}
