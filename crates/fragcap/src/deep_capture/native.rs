// SPDX-License-Identifier: Apache-2.0

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use fragcap_proxy::{NativeProxyBackend as RuntimeBackend, NativeProxyConfig, ShutdownReport};

use super::{
    BackendDescriptor, Budget, CleanupResult, CleanupStatus, CompatibilityObservation,
    ProxyBackend, ProxyLease, SessionPlan, Stage, StageFailure,
};

/// Finite native runtime limits selected by the library consumer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeProxyLimits {
    pub max_connections: usize,
    pub per_connection_buffer_bytes: usize,
    pub shutdown_timeout: Duration,
}

impl Default for NativeProxyLimits {
    fn default() -> Self {
        Self {
            max_connections: 128,
            per_connection_buffer_bytes: 16 * 1024,
            shutdown_timeout: Duration::from_secs(10),
        }
    }
}

/// Library-owned native implementation of the Deep Capture proxy seam.
///
/// S102 owns listener and task lifecycle only. It deliberately returns no
/// application observations until the protocol issues under #278 land.
pub struct NativeProxyAdapter {
    limits: NativeProxyLimits,
}

impl NativeProxyAdapter {
    pub fn new(limits: NativeProxyLimits) -> Self {
        Self { limits }
    }

    pub fn limits(&self) -> NativeProxyLimits {
        self.limits
    }
}

impl Default for NativeProxyAdapter {
    fn default() -> Self {
        Self::new(NativeProxyLimits::default())
    }
}

impl ProxyBackend for NativeProxyAdapter {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            name: "fragcap-native".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn start(
        &mut self,
        plan: &SessionPlan,
        budget: Budget,
    ) -> Result<Box<dyn ProxyLease>, StageFailure> {
        let endpoint = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), plan.endpoint.port);
        let config = NativeProxyConfig::new(
            endpoint,
            self.limits.max_connections,
            self.limits.per_connection_buffer_bytes,
            self.limits.shutdown_timeout,
        )
        .map_err(|error| StageFailure::new(Stage::ProxyStart, error.code, error.detail))?;
        let mut backend = RuntimeBackend::new(config);
        let lease = backend
            .start(budget.remaining())
            .map_err(|error| StageFailure::new(Stage::ProxyStart, error.code, error.detail))?;
        Ok(Box::new(NativeProxyLease { lease }))
    }
}

struct NativeProxyLease {
    lease: fragcap_proxy::NativeProxyLease,
}

impl ProxyLease for NativeProxyLease {
    fn observations(
        &mut self,
        budget: Budget,
    ) -> Result<Vec<CompatibilityObservation>, StageFailure> {
        self.lease
            .observation(budget.remaining())
            .map_err(|error| StageFailure::new(Stage::Observe, error.code, error.detail))?;
        Ok(Vec::new())
    }

    fn stop(&mut self, budget: Budget) -> CleanupResult {
        let report = self.lease.stop(budget.remaining());
        cleanup_result("native-proxy-listener", &report)
    }

    fn cleanup(&mut self, budget: Budget) -> Vec<CleanupResult> {
        let report = self.lease.cleanup(budget.remaining());
        vec![cleanup_result("native-proxy-runtime", &report)]
    }
}

fn cleanup_result(resource: &str, report: &ShutdownReport) -> CleanupResult {
    let status = if report.is_clean() {
        CleanupStatus::Released
    } else if report.residue || report.incomplete_tasks > 0 {
        CleanupStatus::TimedOut
    } else {
        CleanupStatus::Failed
    };
    CleanupResult {
        resource: resource.to_string(),
        status,
        reason: format!(
            "accepted={}, completed={}, failed={}, forced={}, incomplete={}, failures={}",
            report.observation.accepted_connections,
            report.observation.completed_connections,
            report.observation.failed_connections,
            report.observation.forced_connections,
            report.incomplete_tasks,
            report.observation.failures.len()
        ),
    }
}
