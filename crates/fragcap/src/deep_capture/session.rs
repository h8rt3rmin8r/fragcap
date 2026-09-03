// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use super::*;
use crate::targets::CompatibilityProtocol;

const MAX_LAUNCH: Duration = Duration::from_secs(120);
const MAX_OBSERVATION: Duration = Duration::from_secs(300);
const MAX_SHUTDOWN: Duration = Duration::from_secs(30);
const MAX_CLEANUP: Duration = Duration::from_secs(60);

/// Entry point for side-effect-free Deep Capture preparation.
pub struct DeepCapture;

impl DeepCapture {
    /// Resolve one immutable session plan without starting any external effect.
    pub fn preflight(
        mut config: SessionConfig,
        adapters: &mut AdapterSet<'_>,
    ) -> Result<PreparedSession, PreflightRefusal> {
        validate_config(&config)?;
        let target = adapters.targets.resolve(&config)?;
        if config
            .launch_case
            .is_some_and(|launch_case| target.launch_case != launch_case)
        {
            return Err(PreflightRefusal::new(
                "launch-case-mismatch",
                "declared launch case does not match the resolved target",
            ));
        }
        adapters.targets.validate_compatibility(&target, &config)?;
        adapters.artifacts.validate_destination(&config.bundle)?;
        let endpoint = adapters.endpoints.select()?;
        let capture = adapters.capture.prepare(&config, &target, endpoint)?;
        let routing = RoutingPlan::child_environment(endpoint, &config.proxy_bypass)?;
        adapters.routing.prepare(&target, &routing)?;
        let session_id = adapters.identifiers.next_id("session")?;
        let plan_id = PlanId::new(adapters.identifiers.next_id("plan")?);
        config.deadlines = cap_deadlines(config.deadlines);
        let plan = SessionPlan {
            id: plan_id,
            session_id,
            target,
            mode: config.mode,
            calibration_protocol: config.calibration_protocol,
            controlled: config.controlled,
            proxy_backend: adapters.proxy.descriptor(),
            endpoint,
            bundle: config.bundle.clone(),
            routing,
            trust_ca: config.trust_ca,
            client_identity: config.client_identity,
            artifacts: ArtifactRequests {
                har: config.har,
                key_log: config.key_log,
                sensitive_retention: config.sensitive_retention,
            },
            deadlines: config.deadlines,
        };
        Ok(PreparedSession { capture, plan })
    }
}

/// Side-effect-free result consumed into exactly one session owner.
#[derive(Debug)]
pub struct PreparedSession {
    capture: PreparedCapture,
    plan: SessionPlan,
}

impl PreparedSession {
    /// Reviewable immutable plan.
    pub fn plan(&self) -> &SessionPlan {
        &self.plan
    }

    /// Consume preparation and all adapters into one checked coordinator.
    pub fn into_session<'a>(self, adapters: AdapterSet<'a>) -> DeepCaptureSession<'a> {
        DeepCaptureSession {
            capture: self.capture,
            plan: self.plan,
            adapters,
            state: LifecycleState::Prepared,
            proxy: None,
            trust: None,
            trust_target: None,
            routing: None,
            launch: None,
            observations: Vec::new(),
            classification_records_lost: 0,
            application_classification_summary: None,
            failures: Vec::new(),
            fact_writes: Vec::new(),
            cleanup: Vec::new(),
            artifacts: Vec::new(),
            event_failures: Vec::new(),
            sequence: 0,
            interrupted: false,
            facts_persisted: false,
            observation_started: None,
            route_verification: None,
            resource_journal: None,
            cleanup_lifecycle: None,
        }
    }
}

/// Single owner of one Deep Capture lifecycle and all acquired resources.
pub struct DeepCaptureSession<'a> {
    capture: PreparedCapture,
    plan: SessionPlan,
    adapters: AdapterSet<'a>,
    state: LifecycleState,
    proxy: Option<Box<dyn ProxyLease>>,
    trust: Option<Box<dyn TrustLease>>,
    trust_target: Option<String>,
    routing: Option<Box<dyn RoutingLease>>,
    launch: Option<Box<dyn LaunchLease>>,
    observations: Vec<CompatibilityObservation>,
    classification_records_lost: u64,
    application_classification_summary: Option<ClassificationSummary>,
    failures: Vec<StageFailure>,
    fact_writes: Vec<FactWriteResult>,
    cleanup: Vec<CleanupResult>,
    artifacts: Vec<ArtifactResult>,
    event_failures: Vec<EventDeliveryFailure>,
    sequence: u64,
    interrupted: bool,
    facts_persisted: bool,
    observation_started: Option<Duration>,
    route_verification: Option<RouteVerification>,
    resource_journal: Option<ResourceJournal>,
    cleanup_lifecycle: Option<LifecycleWriter>,
}

impl DeepCaptureSession<'_> {
    /// Current checked lifecycle state.
    pub fn state(&self) -> LifecycleState {
        self.state
    }

    /// Start proxy, optional trust, and managed launch after exact-plan approval.
    pub fn start(&mut self, authorization: Authorization) -> Result<(), InvalidTransition> {
        self.require(Operation::Start, &[LifecycleState::Prepared])?;
        match authorization {
            Authorization::Approved { plan_id } if plan_id == self.plan.id => {}
            Authorization::Approved { .. } => {
                self.fail(
                    Stage::Authorization,
                    "plan-id-mismatch",
                    "authorization names a different prepared plan",
                );
                self.state = LifecycleState::Terminal;
                return Ok(());
            }
            Authorization::Declined => {
                self.fail(
                    Stage::Authorization,
                    "declined",
                    "caller declined the prepared plan",
                );
                self.state = LifecycleState::Terminal;
                return Ok(());
            }
        }

        self.emit(DeepCaptureEvent::Plan {
            sequence: 0,
            plan: self.plan.clone(),
        });
        let started = self.adapters.clock.monotonic_elapsed();
        if let Err(error) = self.adapters.artifacts.prepare(&self.plan) {
            self.failures.push(error);
            self.state = LifecycleState::Stopped;
            return Ok(());
        }
        match ResourceJournal::create(
            &self.plan.bundle,
            &self.plan.session_id,
            self.plan.id.as_str(),
        ) {
            Ok(journal) => self.resource_journal = Some(journal),
            Err(error) => {
                self.fail(
                    Stage::Bundle,
                    "resource-journal-open-failed",
                    error.to_string(),
                );
                self.state = LifecycleState::Stopped;
                return Ok(());
            }
        }
        match LifecycleWriter::create(
            self.plan.bundle.join(CLEANUP_LIFECYCLE),
            "cleanup",
            &self.plan.session_id,
        ) {
            Ok(writer) => self.cleanup_lifecycle = Some(writer),
            Err(error) => {
                self.fail(
                    Stage::Bundle,
                    "cleanup-lifecycle-open-failed",
                    error.to_string(),
                );
                self.state = LifecycleState::Stopped;
                return Ok(());
            }
        }
        let budget = self.remaining_budget(started, self.plan.deadlines.launch);
        let proxy_target = self.plan.endpoint.address().to_string();
        if !self.record_resource(
            "proxy-listener",
            ResourceKind::Proxy,
            &proxy_target,
            "close-loopback-listener",
            ResourceState::Pending,
            "listener bind must follow this durable obligation",
        ) || !self.record_resource(
            "proxy-runtime",
            ResourceKind::Proxy,
            &proxy_target,
            "join-proxy-tasks",
            ResourceState::Pending,
            "runtime task ownership must follow this durable obligation",
        ) {
            self.state = LifecycleState::Stopped;
            return Ok(());
        }
        let route = match self.adapters.proxy.start(&self.plan, budget) {
            Ok(lease) => {
                self.record_resource(
                    "proxy-listener",
                    ResourceKind::Proxy,
                    &proxy_target,
                    "close-loopback-listener",
                    ResourceState::Applied,
                    "native listener acquired",
                );
                self.record_resource(
                    "proxy-runtime",
                    ResourceKind::Proxy,
                    &proxy_target,
                    "join-proxy-tasks",
                    ResourceState::Applied,
                    "native runtime tasks acquired",
                );
                let route = match lease.route() {
                    Ok(route) => route,
                    Err(error) => {
                        self.failures.push(error);
                        self.proxy = Some(lease);
                        self.state = LifecycleState::Stopped;
                        return Ok(());
                    }
                };
                self.proxy = Some(lease);
                self.emit(DeepCaptureEvent::ProxyStarted {
                    sequence: 0,
                    session_id: self.plan.session_id.clone(),
                });
                route
            }
            Err(error) => {
                self.record_resource(
                    "proxy-listener",
                    ResourceKind::Proxy,
                    &proxy_target,
                    "close-loopback-listener",
                    ResourceState::NotApplied,
                    &error.detail,
                );
                self.record_resource(
                    "proxy-runtime",
                    ResourceKind::Proxy,
                    &proxy_target,
                    "join-proxy-tasks",
                    ResourceState::NotApplied,
                    &error.detail,
                );
                self.failures.push(error);
                self.state = LifecycleState::Stopped;
                return Ok(());
            }
        };
        if self.deadline_expired(started, self.plan.deadlines.launch) {
            self.fail(
                Stage::ProxyStart,
                "launch-deadline-exceeded",
                "proxy start returned after the launch deadline",
            );
            self.state = LifecycleState::Stopped;
            return Ok(());
        }

        if self.plan.trust_ca {
            let budget = self.remaining_budget(started, self.plan.deadlines.launch);
            let trust_target = format!("sha1:{}", route.ca_sha1_thumbprint());
            if !self.record_resource(
                "trust-entry",
                ResourceKind::Trust,
                &trust_target,
                "remove-current-user-root-by-exact-thumbprint",
                ResourceState::Pending,
                "trust mutation must follow this durable obligation",
            ) {
                self.state = LifecycleState::Stopped;
                return Ok(());
            }
            match self.adapters.trust.acquire(&self.plan, &route, budget) {
                Ok(lease) => {
                    self.record_resource(
                        "trust-entry",
                        ResourceKind::Trust,
                        &trust_target,
                        "remove-current-user-root-by-exact-thumbprint",
                        ResourceState::Applied,
                        "current-user trust entry acquired",
                    );
                    self.trust = Some(lease);
                    self.trust_target = Some(trust_target);
                    self.emit(DeepCaptureEvent::TrustAcquired {
                        sequence: 0,
                        session_id: self.plan.session_id.clone(),
                    });
                }
                Err(error) => {
                    self.record_resource(
                        "trust-entry",
                        ResourceKind::Trust,
                        &trust_target,
                        "remove-current-user-root-by-exact-thumbprint",
                        ResourceState::Failed,
                        &error.detail,
                    );
                    self.failures.push(error);
                    self.state = LifecycleState::Stopped;
                    return Ok(());
                }
            }
            if self.deadline_expired(started, self.plan.deadlines.launch) {
                self.fail(
                    Stage::Trust,
                    "launch-deadline-exceeded",
                    "trust acquisition returned after the launch deadline",
                );
                self.state = LifecycleState::Stopped;
                return Ok(());
            }
        }

        let budget = self.remaining_budget(started, self.plan.deadlines.launch);
        let route_target = self.plan.routing.strategy.as_str().to_string();
        if !self.record_resource(
            "route",
            ResourceKind::Route,
            &route_target,
            "remove-target-scoped-route",
            ResourceState::Pending,
            "route application must follow this durable obligation",
        ) {
            self.state = LifecycleState::Stopped;
            return Ok(());
        }
        match self.adapters.routing.apply(&self.plan, route, budget) {
            Ok(lease) => {
                self.record_resource(
                    "route",
                    ResourceKind::Route,
                    &route_target,
                    "remove-target-scoped-route",
                    ResourceState::Applied,
                    "target-scoped route material resolved",
                );
                self.routing = Some(lease);
            }
            Err(error) => {
                self.record_resource(
                    "route",
                    ResourceKind::Route,
                    &route_target,
                    "remove-target-scoped-route",
                    ResourceState::Failed,
                    &error.detail,
                );
                self.failures.push(error);
                self.state = LifecycleState::Stopped;
                return Ok(());
            }
        }

        let budget = self.remaining_budget(started, self.plan.deadlines.launch);
        let launch_target = format!("target:{}", self.plan.target.id);
        if !self.record_resource(
            "managed-child",
            ResourceKind::Launch,
            &launch_target,
            "stop-managed-child",
            ResourceState::Pending,
            "managed launch must follow this durable obligation",
        ) {
            self.state = LifecycleState::Stopped;
            return Ok(());
        }
        match self.adapters.launch.launch(
            &self.plan.target,
            self.plan.target.launch_case,
            self.routing
                .as_ref()
                .expect("routing was applied")
                .applied(),
            budget,
        ) {
            Ok(lease) => {
                self.record_resource(
                    "managed-child",
                    ResourceKind::Launch,
                    &launch_target,
                    "stop-managed-child",
                    ResourceState::Applied,
                    "managed child acquired",
                );
                self.launch = Some(lease);
                self.emit(DeepCaptureEvent::LaunchStarted {
                    sequence: 0,
                    session_id: self.plan.session_id.clone(),
                });
            }
            Err(error) => {
                self.record_resource(
                    "managed-child",
                    ResourceKind::Launch,
                    &launch_target,
                    "stop-managed-child",
                    ResourceState::Failed,
                    &error.detail,
                );
                self.failures.push(error);
                self.state = LifecycleState::Stopped;
                return Ok(());
            }
        }
        if self.deadline_expired(started, self.plan.deadlines.launch) {
            self.fail(
                Stage::Launch,
                "launch-deadline-exceeded",
                "managed launch returned after the launch deadline",
            );
            self.state = LifecycleState::Stopped;
            return Ok(());
        }
        self.state = LifecycleState::Running;
        self.emit(DeepCaptureEvent::Started {
            sequence: 0,
            session_id: self.plan.session_id.clone(),
        });
        Ok(())
    }

    /// Run ordinary Capture and collect proxy observations.
    pub fn observe(&mut self) -> Result<(), InvalidTransition> {
        self.require(Operation::Observe, &[LifecycleState::Running])?;
        let started = self.adapters.clock.monotonic_elapsed();
        self.observation_started = Some(started);
        let budget = self.remaining_budget(started, self.plan.deadlines.observation);
        let capture_target = self.capture.token.clone();
        if !self.record_resource(
            "capture",
            ResourceKind::Capture,
            &capture_target,
            "stop-capture",
            ResourceState::Pending,
            "capture start must follow this durable obligation",
        ) {
            self.state = LifecycleState::Observed;
            return Ok(());
        }
        match self.adapters.capture.run(
            &self.capture,
            self.routing
                .as_ref()
                .expect("routing was applied")
                .applied(),
            budget,
        ) {
            Ok(result) => {
                self.record_resource(
                    "capture",
                    ResourceKind::Capture,
                    &capture_target,
                    "stop-capture",
                    ResourceState::Applied,
                    "capture runner started and returned",
                );
                self.interrupted |= result.interrupted;
                self.extend_observations(result.observations);
            }
            Err(error) => {
                self.record_resource(
                    "capture",
                    ResourceKind::Capture,
                    &capture_target,
                    "stop-capture",
                    ResourceState::Failed,
                    &error.detail,
                );
                self.failures.push(error);
            }
        }
        if self.deadline_expired(started, self.plan.deadlines.observation) {
            self.fail(
                Stage::Capture,
                "observation-deadline-exceeded",
                "Capture returned after the observation deadline",
            );
        }
        self.state = LifecycleState::Observed;
        Ok(())
    }

    /// Stop Capture and the proxy with finite budgets.
    pub fn stop(&mut self) -> Result<(), InvalidTransition> {
        self.require(
            Operation::Stop,
            &[
                LifecycleState::Running,
                LifecycleState::Observed,
                LifecycleState::Stopped,
            ],
        )?;
        if self.state == LifecycleState::Stopped {
            return Err(InvalidTransition {
                operation: Operation::Stop,
                actual: self.state,
                allowed: &[LifecycleState::Running, LifecycleState::Observed],
            });
        }
        let started = self.adapters.clock.monotonic_elapsed();
        let capture_target = self.capture.token.clone();
        self.record_resource(
            "capture",
            ResourceKind::Capture,
            &capture_target,
            "stop-capture",
            ResourceState::CleanupPending,
            "bounded capture stop attempt",
        );
        let budget = self.remaining_budget(started, self.plan.deadlines.shutdown);
        let capture = self.adapters.capture.stop(budget);
        self.record_cleanup_transition(
            "capture",
            ResourceKind::Capture,
            &capture_target,
            "stop-capture",
            &capture,
        );
        self.record_cleanup(capture);
        if self.proxy.is_some() {
            let elapsed = self.adapters.clock.monotonic_elapsed();
            let budget = Budget::new(
                self.plan
                    .deadlines
                    .shutdown
                    .saturating_sub(elapsed.saturating_sub(started)),
            );
            let proxy_target = self.plan.endpoint.address().to_string();
            self.record_resource(
                "proxy-listener",
                ResourceKind::Proxy,
                &proxy_target,
                "close-loopback-listener",
                ResourceState::CleanupPending,
                "bounded listener stop attempt",
            );
            let proxy = self.proxy.as_mut().expect("proxy presence checked");
            let result = proxy.stop(budget);
            self.record_cleanup_transition(
                "proxy-listener",
                ResourceKind::Proxy,
                &proxy_target,
                "close-loopback-listener",
                &result,
            );
            self.record_cleanup(result);
        }
        if self.deadline_expired(started, self.plan.deadlines.shutdown) {
            self.fail(
                Stage::ProxyStop,
                "shutdown-deadline-exceeded",
                "stop operations returned after the shutdown deadline",
            );
        }
        if let Some(proxy) = self.proxy.as_mut() {
            let observation_started = self.observation_started.unwrap_or(started);
            let elapsed = self.adapters.clock.monotonic_elapsed();
            let budget = Budget::new(
                self.plan
                    .deadlines
                    .observation
                    .saturating_sub(elapsed.saturating_sub(observation_started)),
            );
            match proxy.observations(budget) {
                Ok(observations) => {
                    self.classification_records_lost = self
                        .classification_records_lost
                        .saturating_add(proxy.observations_lost());
                    self.application_classification_summary =
                        proxy.application_classification_summary();
                    self.extend_observations(observations);
                }
                Err(error) => self.failures.push(error),
            }
            if self.deadline_expired(observation_started, self.plan.deadlines.observation)
                && !self
                    .failures
                    .iter()
                    .any(|failure| failure.code == "observation-deadline-exceeded")
            {
                self.fail(
                    Stage::Observe,
                    "observation-deadline-exceeded",
                    "proxy observations returned after the shared observation deadline",
                );
            }
        }
        if let Some(routing) = self.routing.as_ref() {
            self.route_verification = Some(routing.verify(&self.observations));
        }
        self.state = LifecycleState::Stopped;
        Ok(())
    }

    /// Persist evidence-backed facts, clean resources, and freeze terminal truth.
    pub fn finalize(&mut self) -> Result<TerminalReport, InvalidTransition> {
        self.require(
            Operation::Finalize,
            &[LifecycleState::Stopped, LifecycleState::Finalizing],
        )?;
        self.state = LifecycleState::Finalizing;
        self.persist_facts();
        self.cleanup_resources();
        self.prepare_lifecycle_authority();
        let mut snapshot = self.snapshot();
        if !self.event_failures.is_empty() && snapshot.outcome == SessionOutcome::Complete {
            snapshot.outcome = SessionOutcome::Partial;
        }
        let failures_before_publication = self.failures.len();
        self.write_bundle(&snapshot);
        self.settle_lifecycle_authority(self.failures.len() == failures_before_publication);
        let reconciled_snapshot = self.snapshot();
        self.reconcile_bundle(&reconciled_snapshot);
        snapshot.failures = self.failures.clone();
        if self.required_reporting_failed() && snapshot.outcome == SessionOutcome::Complete {
            snapshot.outcome = SessionOutcome::Partial;
        }
        self.emit(DeepCaptureEvent::Terminal {
            sequence: 0,
            report: snapshot.clone(),
        });
        if !self.event_failures.is_empty() && snapshot.outcome == SessionOutcome::Complete {
            snapshot.outcome = SessionOutcome::Partial;
        }
        self.state = LifecycleState::Terminal;
        Ok(TerminalReport {
            snapshot,
            artifacts: std::mem::take(&mut self.artifacts),
            event_failures: std::mem::take(&mut self.event_failures),
        })
    }

    /// Explicitly clean a stopped session. Finalization performs this
    /// automatically; calling it separately moves the session to finalizing.
    pub fn cleanup(&mut self) -> Result<(), InvalidTransition> {
        self.require(Operation::Cleanup, &[LifecycleState::Stopped])?;
        self.state = LifecycleState::Finalizing;
        self.cleanup_resources();
        Ok(())
    }

    /// Drive the safe end-to-end sequence and always attempt finalization after
    /// a started effect, even when an adapter reports failure.
    pub fn run_to_completion(mut self, authorization: Authorization) -> TerminalReport {
        let _ = self.start(authorization);
        if self.state == LifecycleState::Running {
            let _ = self.observe();
        }
        if matches!(
            self.state,
            LifecycleState::Running | LifecycleState::Observed
        ) {
            let _ = self.stop();
        }
        if self.state == LifecycleState::Stopped {
            return self
                .finalize()
                .expect("stopped is a valid finalization state");
        }
        let snapshot = self.snapshot();
        TerminalReport {
            snapshot,
            artifacts: std::mem::take(&mut self.artifacts),
            event_failures: std::mem::take(&mut self.event_failures),
        }
    }

    fn require(
        &self,
        operation: Operation,
        allowed: &'static [LifecycleState],
    ) -> Result<(), InvalidTransition> {
        if allowed.contains(&self.state) {
            Ok(())
        } else {
            Err(InvalidTransition {
                operation,
                actual: self.state,
                allowed,
            })
        }
    }

    fn extend_observations(&mut self, observations: Vec<CompatibilityObservation>) {
        for observation in observations {
            self.emit(DeepCaptureEvent::Observation {
                sequence: 0,
                session_id: self.plan.session_id.clone(),
                observation: observation.clone(),
            });
            self.observations.push(observation);
        }
    }

    fn persist_facts(&mut self) {
        if self.facts_persisted {
            return;
        }
        self.facts_persisted = true;
        for fact in facts_from_observations(&self.plan, &self.observations) {
            let status = self.adapters.facts.append(&self.plan.target, &fact);
            if let FactWriteStatus::Failed { code, detail } = &status {
                self.fail(Stage::Facts, code.clone(), detail.clone());
            }
            self.fact_writes.push(FactWriteResult { fact, status });
        }
    }

    fn cleanup_resources(&mut self) {
        let started = self.adapters.clock.monotonic_elapsed();
        if let Some(mut launch) = self.launch.take() {
            let target = format!("target:{}", self.plan.target.id);
            self.record_resource(
                "managed-child",
                ResourceKind::Launch,
                &target,
                "stop-managed-child",
                ResourceState::CleanupPending,
                "bounded child cleanup attempt",
            );
            let result =
                launch.cleanup(self.remaining_budget(started, self.plan.deadlines.cleanup));
            self.record_cleanup_transition(
                "managed-child",
                ResourceKind::Launch,
                &target,
                "stop-managed-child",
                &result,
            );
            self.record_cleanup(result);
        }
        if let Some(mut routing) = self.routing.take() {
            let target = self.plan.routing.strategy.as_str().to_string();
            self.record_resource(
                "route",
                ResourceKind::Route,
                &target,
                "remove-target-scoped-route",
                ResourceState::CleanupPending,
                "bounded route cleanup attempt",
            );
            let result =
                routing.cleanup(self.remaining_budget(started, self.plan.deadlines.cleanup));
            self.record_cleanup_transition(
                "route",
                ResourceKind::Route,
                &target,
                "remove-target-scoped-route",
                &result,
            );
            self.record_cleanup(result);
        }
        if let Some(mut trust) = self.trust.take() {
            let target = self
                .trust_target
                .take()
                .expect("acquired trust retains its exact recovery target");
            self.record_resource(
                "trust-entry",
                ResourceKind::Trust,
                &target,
                "remove-current-user-root-by-exact-thumbprint",
                ResourceState::CleanupPending,
                "bounded exact trust cleanup attempt",
            );
            let result = trust.cleanup(self.remaining_budget(started, self.plan.deadlines.cleanup));
            self.record_cleanup_transition(
                "trust-entry",
                ResourceKind::Trust,
                &target,
                "remove-current-user-root-by-exact-thumbprint",
                &result,
            );
            self.record_cleanup(result);
        } else if !self
            .cleanup
            .iter()
            .any(|result| result.resource == "trust-entry")
        {
            self.record_cleanup(CleanupResult {
                resource: "trust-entry".into(),
                status: CleanupStatus::NotNeeded,
                reason: "session did not acquire trust".into(),
            });
        }
        if let Some(mut proxy) = self.proxy.take() {
            let target = self.plan.endpoint.address().to_string();
            self.record_resource(
                "proxy-runtime",
                ResourceKind::Proxy,
                &target,
                "join-proxy-tasks",
                ResourceState::CleanupPending,
                "bounded runtime cleanup attempt",
            );
            let budget = self.remaining_budget(started, self.plan.deadlines.cleanup);
            let results = proxy.cleanup(budget);
            let combined = if results
                .iter()
                .any(|result| result.status == CleanupStatus::Failed)
            {
                CleanupResult {
                    resource: "proxy-runtime".into(),
                    status: CleanupStatus::Failed,
                    reason: "one or more native proxy resources failed cleanup".into(),
                }
            } else if results
                .iter()
                .any(|result| result.status == CleanupStatus::TimedOut)
            {
                CleanupResult {
                    resource: "proxy-runtime".into(),
                    status: CleanupStatus::TimedOut,
                    reason: "one or more native proxy resources timed out during cleanup".into(),
                }
            } else {
                CleanupResult {
                    resource: "proxy-runtime".into(),
                    status: CleanupStatus::Released,
                    reason: "all native proxy resources completed cleanup".into(),
                }
            };
            self.record_cleanup_transition(
                "proxy-runtime",
                ResourceKind::Proxy,
                &target,
                "join-proxy-tasks",
                &combined,
            );
            for result in results {
                self.record_cleanup(result);
            }
        }
        if self.deadline_expired(started, self.plan.deadlines.cleanup) {
            self.fail(
                Stage::Cleanup,
                "cleanup-deadline-exceeded",
                "cleanup operations returned after the total cleanup deadline",
            );
        }
    }

    fn record_cleanup(&mut self, result: CleanupResult) {
        if matches!(
            result.status,
            CleanupStatus::Failed | CleanupStatus::TimedOut
        ) {
            self.fail(
                Stage::Cleanup,
                "cleanup-incomplete",
                format!("{}: {}", result.resource, result.reason),
            );
        }
        self.emit(DeepCaptureEvent::Cleanup {
            sequence: 0,
            session_id: self.plan.session_id.clone(),
            result: result.clone(),
        });
        if let Some(writer) = self.cleanup_lifecycle.as_mut() {
            let status = match result.status {
                CleanupStatus::Released => "succeeded",
                CleanupStatus::NotNeeded => "not-needed",
                CleanupStatus::TimedOut => "timed-out",
                CleanupStatus::Failed => "failed",
            };
            if let Err(error) = writer.append(
                "cleanup.adapter-result",
                serde_json::json!({
                    "resource_id": result.resource,
                    "status": status,
                    "reason": result.reason,
                }),
            ) {
                self.fail(
                    Stage::Bundle,
                    "cleanup-lifecycle-write-failed",
                    error.to_string(),
                );
            }
        }
        self.cleanup.push(result);
    }

    fn record_resource(
        &mut self,
        resource_id: &str,
        kind: ResourceKind,
        target: &str,
        action: &str,
        state: ResourceState,
        detail: &str,
    ) -> bool {
        let transition = ResourceTransition::new(
            resource_id,
            kind,
            target,
            format!("session:{}", self.plan.session_id),
            action,
            state,
            detail,
        );
        let sequence = match self.resource_journal.as_mut() {
            Some(journal) => match journal.append(transition) {
                Ok(sequence) => sequence,
                Err(error) => {
                    self.fail(
                        Stage::Bundle,
                        "resource-journal-write-failed",
                        error.to_string(),
                    );
                    return false;
                }
            },
            None => {
                self.fail(
                    Stage::Bundle,
                    "resource-journal-unavailable",
                    "an external effect was attempted without a resource journal",
                );
                return false;
            }
        };
        if let Some(writer) = self.cleanup_lifecycle.as_mut() {
            let record_type = match state {
                ResourceState::Pending => "cleanup.obligation",
                ResourceState::Applied => "cleanup.acquired",
                ResourceState::CleanupPending => "cleanup.attempt",
                ResourceState::Released
                | ResourceState::Retained
                | ResourceState::Failed
                | ResourceState::TimedOut
                | ResourceState::NotApplied => "cleanup.result",
            };
            if let Err(error) = writer.append(
                record_type,
                serde_json::json!({
                    "journal_sequence": sequence,
                    "resource_id": resource_id,
                    "kind": kind.as_str(),
                    "target": target,
                    "action": action,
                    "state": state.as_str(),
                    "detail": detail,
                }),
            ) {
                self.fail(
                    Stage::Bundle,
                    "cleanup-lifecycle-write-failed",
                    error.to_string(),
                );
            }
        }
        true
    }

    fn record_cleanup_transition(
        &mut self,
        resource_id: &str,
        kind: ResourceKind,
        target: &str,
        action: &str,
        result: &CleanupResult,
    ) {
        let state = match result.status {
            CleanupStatus::Released | CleanupStatus::NotNeeded => ResourceState::Released,
            CleanupStatus::TimedOut => ResourceState::TimedOut,
            CleanupStatus::Failed => ResourceState::Failed,
        };
        self.record_resource(resource_id, kind, target, action, state, &result.reason);
    }

    fn prepare_lifecycle_authority(&mut self) {
        if self.resource_journal.is_none() {
            return;
        }
        let bundle_target = self.plan.bundle.display().to_string();
        self.record_resource(
            "bundle-evidence",
            ResourceKind::Artifact,
            &bundle_target,
            "retain-authorized-evidence",
            ResourceState::Pending,
            "artifact retention must follow this durable obligation",
        );
        if let Some(writer) = self.cleanup_lifecycle.as_mut() {
            if let Err(error) = writer.finish() {
                self.fail(
                    Stage::Bundle,
                    "cleanup-lifecycle-finish-failed",
                    error.to_string(),
                );
            }
        }
        if let Some(journal) = self.resource_journal.as_mut() {
            if let Err(error) = journal.finish() {
                self.fail(
                    Stage::Bundle,
                    "resource-journal-finish-failed",
                    error.to_string(),
                );
            }
        }
    }

    fn settle_lifecycle_authority(&mut self, publication_succeeded: bool) {
        let journal_path = self
            .resource_journal
            .take()
            .map(|journal| journal.path().to_path_buf());
        let cleanup_path = self
            .cleanup_lifecycle
            .take()
            .map(|writer| writer.path().to_path_buf());
        if let Some(path) = journal_path {
            match ResourceJournal::resume(&path) {
                Ok(journal) => self.resource_journal = Some(journal),
                Err(error) => self.fail(
                    Stage::Bundle,
                    "resource-journal-resume-failed",
                    error.to_string(),
                ),
            }
        }
        if let Some(path) = cleanup_path {
            match LifecycleWriter::resume(&path) {
                Ok(writer) => self.cleanup_lifecycle = Some(writer),
                Err(error) => self.fail(
                    Stage::Bundle,
                    "cleanup-lifecycle-resume-failed",
                    error.to_string(),
                ),
            }
        }
        let bundle_target = self.plan.bundle.display().to_string();
        self.record_resource(
            "bundle-evidence",
            ResourceKind::Artifact,
            &bundle_target,
            "retain-authorized-evidence",
            if publication_succeeded {
                ResourceState::Retained
            } else {
                ResourceState::Failed
            },
            if publication_succeeded {
                "bundle publication completed before retention was declared"
            } else {
                "bundle publication failed; retention was not declared"
            },
        );
        if let Some(writer) = self.cleanup_lifecycle.as_mut() {
            if let Err(error) = writer.finish() {
                self.fail(
                    Stage::Bundle,
                    "cleanup-lifecycle-finish-failed",
                    error.to_string(),
                );
            }
        }
        if let Some(journal) = self.resource_journal.as_mut() {
            if let Err(error) = journal.finish() {
                self.fail(
                    Stage::Bundle,
                    "resource-journal-finish-failed",
                    error.to_string(),
                );
            }
        }
    }

    fn snapshot(&mut self) -> TerminalSnapshot {
        let outcome = if self.interrupted {
            SessionOutcome::Interrupted
        } else if self.failures.is_empty() {
            SessionOutcome::Complete
        } else if self.observations.is_empty() {
            SessionOutcome::Failed
        } else {
            SessionOutcome::Partial
        };
        TerminalSnapshot {
            session_id: self.plan.session_id.clone(),
            plan_id: self.plan.id.clone(),
            target: self.plan.target.clone(),
            mode: self.plan.mode,
            controlled: self.plan.controlled,
            artifacts: self.plan.artifacts,
            outcome,
            observations: self.observations.clone(),
            classification_records_lost: self.classification_records_lost,
            application_classification_summary: self.application_classification_summary.clone(),
            route_verification: self.route_verification.clone(),
            failures: self.failures.clone(),
            fact_writes: self.fact_writes.clone(),
            cleanup: self.cleanup.clone(),
            deadlines: self.plan.deadlines,
            finished_at: self.adapters.clock.wall_now(),
        }
    }

    fn write_bundle(&mut self, snapshot: &TerminalSnapshot) {
        for result in self
            .adapters
            .artifacts
            .finalize(&self.plan.bundle, snapshot)
        {
            if let ArtifactStatus::Failed { code, detail } = &result.status {
                self.fail(Stage::Bundle, code.clone(), detail.clone());
            }
            self.artifacts.push(result);
        }
    }

    fn reconcile_bundle(&mut self, snapshot: &TerminalSnapshot) {
        for result in self
            .adapters
            .artifacts
            .reconcile(&self.plan.bundle, snapshot)
        {
            if let ArtifactStatus::Failed { code, detail } = &result.status {
                self.fail(Stage::Bundle, code.clone(), detail.clone());
            }
            self.artifacts.push(result);
        }
    }

    fn emit(&mut self, event: DeepCaptureEvent) {
        self.sequence += 1;
        let event = with_sequence(event, self.sequence);
        if let Err(error) = self.adapters.events.emit(&event) {
            self.event_failures.push(EventDeliveryFailure {
                sequence: self.sequence,
                code: error.code,
                detail: error.detail,
            });
        }
    }

    fn fail(&mut self, stage: Stage, code: impl Into<String>, detail: impl Into<String>) {
        self.failures.push(StageFailure::new(stage, code, detail));
    }

    fn required_reporting_failed(&self) -> bool {
        !self.event_failures.is_empty()
            || self.artifacts.iter().any(|artifact| {
                artifact.required && !matches!(artifact.status, ArtifactStatus::Written)
            })
    }

    fn remaining_budget(&mut self, started: Duration, total: Duration) -> Budget {
        let elapsed = self.adapters.clock.monotonic_elapsed();
        Budget::new(total.saturating_sub(elapsed.saturating_sub(started)))
    }

    fn deadline_expired(&mut self, started: Duration, total: Duration) -> bool {
        self.adapters
            .clock
            .monotonic_elapsed()
            .saturating_sub(started)
            > total
    }
}

impl Drop for DeepCaptureSession<'_> {
    fn drop(&mut self) {
        if matches!(
            self.state,
            LifecycleState::Running | LifecycleState::Observed
        ) {
            let _ = self.stop();
        }
        if matches!(
            self.state,
            LifecycleState::Stopped | LifecycleState::Finalizing
        ) {
            self.cleanup_resources();
        }
    }
}

fn validate_config(config: &SessionConfig) -> Result<(), PreflightRefusal> {
    if config.target.trim().is_empty() {
        return Err(PreflightRefusal::new(
            "target-empty",
            "target cannot be empty",
        ));
    }
    if config.mode == SessionMode::ReachabilityCalibration
        && (config.trust_ca || config.har || config.key_log)
    {
        return Err(PreflightRefusal::new(
            "reachability-tls-options",
            "reachability calibration cannot acquire trust or request TLS artifacts",
        ));
    }
    match (config.mode, config.calibration_protocol) {
        (SessionMode::Capture, None) => {}
        (SessionMode::ReachabilityCalibration, Some(CompatibilityProtocol::Routing)) => {}
        (SessionMode::TlsCalibration, Some(protocol))
            if protocol != CompatibilityProtocol::Routing
                && protocol != CompatibilityProtocol::NotApplicable => {}
        (SessionMode::Capture, Some(_)) => {
            return Err(PreflightRefusal::new(
                "calibration-protocol-unexpected",
                "ordinary Deep Capture cannot declare a calibration protocol",
            ));
        }
        _ => {
            return Err(PreflightRefusal::new(
                "calibration-protocol",
                "calibration requires one exact phase-appropriate protocol case",
            ));
        }
    }
    if config.mode != SessionMode::ReachabilityCalibration && !config.trust_ca {
        return Err(PreflightRefusal::new(
            "trust-not-authorized",
            "HTTPS inspection requires explicit CA trust authorization",
        ));
    }
    Ok(())
}

fn cap_deadlines(deadlines: Deadlines) -> Deadlines {
    Deadlines {
        launch: deadlines.launch.min(MAX_LAUNCH),
        observation: deadlines.observation.min(MAX_OBSERVATION),
        shutdown: deadlines.shutdown.min(MAX_SHUTDOWN),
        cleanup: deadlines.cleanup.min(MAX_CLEANUP),
    }
}

fn facts_from_observations(
    plan: &SessionPlan,
    observations: &[CompatibilityObservation],
) -> Vec<CompatibilityFact> {
    let calibration = match plan.mode {
        SessionMode::Capture => None,
        SessionMode::ReachabilityCalibration => Some(CalibrationPhase::Reachability),
        SessionMode::TlsCalibration => Some(CalibrationPhase::Tls),
    };
    compatibility_fact_candidates(
        plan.target.launch_case.as_str(),
        observations,
        plan.controlled,
        calibration,
        plan.calibration_protocol,
    )
    .into_iter()
    .map(|candidate| CompatibilityFact {
        kind: candidate.key.as_str().to_string(),
        value: candidate.value,
        evidence: candidate
            .final_owner_index
            .and_then(|index| observations.get(index))
            .and_then(|observation| observation.reason.clone())
            .unwrap_or_else(|| "scrubbed Deep Capture observation".into()),
        phase: candidate.phase,
        protocol: candidate.protocol,
        final_owner_index: candidate.final_owner_index,
    })
    .collect()
}

fn with_sequence(event: DeepCaptureEvent, sequence: u64) -> DeepCaptureEvent {
    match event {
        DeepCaptureEvent::Plan { plan, .. } => DeepCaptureEvent::Plan { sequence, plan },
        DeepCaptureEvent::ProxyStarted { session_id, .. } => DeepCaptureEvent::ProxyStarted {
            sequence,
            session_id,
        },
        DeepCaptureEvent::TrustAcquired { session_id, .. } => DeepCaptureEvent::TrustAcquired {
            sequence,
            session_id,
        },
        DeepCaptureEvent::LaunchStarted { session_id, .. } => DeepCaptureEvent::LaunchStarted {
            sequence,
            session_id,
        },
        DeepCaptureEvent::Started { session_id, .. } => DeepCaptureEvent::Started {
            sequence,
            session_id,
        },
        DeepCaptureEvent::Observation {
            session_id,
            observation,
            ..
        } => DeepCaptureEvent::Observation {
            sequence,
            session_id,
            observation,
        },
        DeepCaptureEvent::Cleanup {
            session_id, result, ..
        } => DeepCaptureEvent::Cleanup {
            sequence,
            session_id,
            result,
        },
        DeepCaptureEvent::Terminal { report, .. } => {
            DeepCaptureEvent::Terminal { sequence, report }
        }
    }
}
