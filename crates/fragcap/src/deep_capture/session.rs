// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use super::*;

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
        let capture = adapters.capture.prepare(&config, &target)?;
        let endpoint = adapters.endpoints.select()?;
        let session_id = adapters.identifiers.next_id("session")?;
        let plan_id = PlanId::new(adapters.identifiers.next_id("plan")?);
        config.deadlines = cap_deadlines(config.deadlines);
        let plan = SessionPlan {
            id: plan_id,
            session_id,
            target,
            mode: config.mode,
            controlled: config.controlled,
            proxy_backend: adapters.proxy.descriptor(),
            endpoint,
            bundle: config.bundle.clone(),
            trust_ca: config.trust_ca,
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
            launch: None,
            observations: Vec::new(),
            failures: Vec::new(),
            fact_writes: Vec::new(),
            cleanup: Vec::new(),
            artifacts: Vec::new(),
            event_failures: Vec::new(),
            sequence: 0,
            interrupted: false,
            facts_persisted: false,
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
    launch: Option<Box<dyn LaunchLease>>,
    observations: Vec<CompatibilityObservation>,
    failures: Vec<StageFailure>,
    fact_writes: Vec<FactWriteResult>,
    cleanup: Vec<CleanupResult>,
    artifacts: Vec<ArtifactResult>,
    event_failures: Vec<EventDeliveryFailure>,
    sequence: u64,
    interrupted: bool,
    facts_persisted: bool,
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
        let budget = self.remaining_budget(started, self.plan.deadlines.launch);
        match self.adapters.proxy.start(&self.plan, budget) {
            Ok(lease) => self.proxy = Some(lease),
            Err(error) => {
                self.failures.push(error);
                self.state = LifecycleState::Stopped;
                return Ok(());
            }
        }
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
            match self.adapters.trust.acquire(&self.plan, budget) {
                Ok(lease) => self.trust = Some(lease),
                Err(error) => {
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
        match self.adapters.launch.launch(
            &self.plan.target,
            self.plan.target.launch_case,
            self.plan.endpoint,
            budget,
        ) {
            Ok(lease) => self.launch = Some(lease),
            Err(error) => {
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
        let budget = self.remaining_budget(started, self.plan.deadlines.observation);
        match self
            .adapters
            .capture
            .run(&self.capture, self.plan.endpoint, budget)
        {
            Ok(result) => {
                self.interrupted |= result.interrupted;
                self.extend_observations(result.observations);
            }
            Err(error) => self.failures.push(error),
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
        let budget = self.remaining_budget(started, self.plan.deadlines.shutdown);
        let capture = self.adapters.capture.stop(budget);
        self.record_cleanup(capture);
        if let Some(proxy) = self.proxy.as_mut() {
            let elapsed = self.adapters.clock.monotonic_elapsed();
            let budget = Budget::new(
                self.plan
                    .deadlines
                    .shutdown
                    .saturating_sub(elapsed.saturating_sub(started)),
            );
            let result = proxy.stop(budget);
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
            match proxy.observations(Budget::new(self.plan.deadlines.observation)) {
                Ok(observations) => self.extend_observations(observations),
                Err(error) => self.failures.push(error),
            }
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
        let mut snapshot = self.snapshot();
        if !self.event_failures.is_empty() && snapshot.outcome == SessionOutcome::Complete {
            snapshot.outcome = SessionOutcome::Partial;
        }
        self.write_bundle(&snapshot);
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
            let result =
                launch.cleanup(self.remaining_budget(started, self.plan.deadlines.cleanup));
            self.record_cleanup(result);
        }
        if let Some(mut trust) = self.trust.take() {
            let result = trust.cleanup(self.remaining_budget(started, self.plan.deadlines.cleanup));
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
            let budget = self.remaining_budget(started, self.plan.deadlines.cleanup);
            for result in proxy.cleanup(budget) {
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
        self.cleanup.push(result);
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
            outcome,
            observations: self.observations.clone(),
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
        final_owner_index: candidate.final_owner_index,
    })
    .collect()
}

fn with_sequence(event: DeepCaptureEvent, sequence: u64) -> DeepCaptureEvent {
    match event {
        DeepCaptureEvent::Plan { plan, .. } => DeepCaptureEvent::Plan { sequence, plan },
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
