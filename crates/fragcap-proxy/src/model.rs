// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::time::Duration;

/// Validated, effect-free native proxy configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeProxyConfig {
    pub(crate) listen: SocketAddr,
    pub(crate) max_connections: NonZeroUsize,
    pub(crate) per_connection_buffer_bytes: NonZeroUsize,
    pub(crate) shutdown_timeout: Duration,
}

impl NativeProxyConfig {
    pub fn new(
        listen: SocketAddr,
        max_connections: usize,
        per_connection_buffer_bytes: usize,
        shutdown_timeout: Duration,
    ) -> Result<Self, ConfigError> {
        if !listen.ip().is_loopback() {
            return Err(ConfigError::new(
                "non-loopback-listen",
                format!("native proxy endpoint must be loopback, got {listen}"),
            ));
        }
        let max_connections = NonZeroUsize::new(max_connections).ok_or_else(|| {
            ConfigError::new("zero-connection-limit", "connection limit must be non-zero")
        })?;
        let per_connection_buffer_bytes = NonZeroUsize::new(per_connection_buffer_bytes)
            .ok_or_else(|| {
                ConfigError::new("zero-buffer-limit", "connection buffer must be non-zero")
            })?;
        if shutdown_timeout.is_zero() {
            return Err(ConfigError::new(
                "zero-shutdown-timeout",
                "shutdown timeout must be non-zero",
            ));
        }
        Ok(Self {
            listen,
            max_connections,
            per_connection_buffer_bytes,
            shutdown_timeout,
        })
    }

    pub fn listen(&self) -> SocketAddr {
        self.listen
    }

    pub fn max_connections(&self) -> usize {
        self.max_connections.get()
    }

    pub fn per_connection_buffer_bytes(&self) -> usize {
        self.per_connection_buffer_bytes.get()
    }

    pub fn shutdown_timeout(&self) -> Duration {
        self.shutdown_timeout
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigError {
    pub code: &'static str,
    pub detail: String,
}

impl ConfigError {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for ConfigError {}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendKind {
    NativeRust,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackendCapabilities {
    pub foundation_listener: bool,
    pub forwards_upstream: bool,
    pub observes_http: bool,
    pub inspects_tls: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackendIdentity {
    pub kind: BackendKind,
    pub name: &'static str,
    pub version: &'static str,
    pub capabilities: BackendCapabilities,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleState {
    Running,
    Stopping,
    Stopped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeFailure {
    pub code: &'static str,
    pub detail: String,
    pub connection_id: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeObservation {
    pub state: LifecycleState,
    pub endpoint: SocketAddr,
    pub accepted_connections: u64,
    pub authenticated_connections: u64,
    pub authentication_refused: u64,
    pub saturated_connections: u64,
    pub completed_connections: u64,
    pub failed_connections: u64,
    pub forced_connections: u64,
    pub live_connections: usize,
    pub peak_live_connections: usize,
    pub failures: Vec<RuntimeFailure>,
}

impl RuntimeObservation {
    pub(crate) fn running(endpoint: SocketAddr) -> Self {
        Self {
            state: LifecycleState::Running,
            endpoint,
            accepted_connections: 0,
            authenticated_connections: 0,
            authentication_refused: 0,
            saturated_connections: 0,
            completed_connections: 0,
            failed_connections: 0,
            forced_connections: 0,
            live_connections: 0,
            peak_live_connections: 0,
            failures: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShutdownReport {
    pub observation: RuntimeObservation,
    pub listener_released: bool,
    pub joined_tasks: u64,
    pub incomplete_tasks: u64,
    pub residue: bool,
}

impl ShutdownReport {
    pub fn is_clean(&self) -> bool {
        self.listener_released
            && self.observation.state == LifecycleState::Stopped
            && self.observation.live_connections == 0
            && self.incomplete_tasks == 0
            && !self.residue
            && self.observation.failures.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartError {
    pub code: &'static str,
    pub detail: String,
}

impl StartError {
    pub(crate) fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for StartError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObserveError {
    pub code: &'static str,
    pub detail: String,
}

impl ObserveError {
    pub(crate) fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for ObserveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for ObserveError {}
