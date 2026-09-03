// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use fragcap::deep_capture::*;

type Ledger = Rc<RefCell<Vec<String>>>;

struct Targets(Ledger);
impl TargetResolver for Targets {
    fn resolve(&mut self, config: &SessionConfig) -> Result<PreparedTarget, PreflightRefusal> {
        self.0.borrow_mut().push("target.resolve".into());
        Ok(PreparedTarget {
            id: 7,
            handle: config.target.clone(),
            launch_case: config.launch_case.expect("test declares launch case"),
        })
    }

    fn validate_compatibility(
        &mut self,
        _: &PreparedTarget,
        _: &SessionConfig,
    ) -> Result<(), PreflightRefusal> {
        self.0.borrow_mut().push("target.compatibility".into());
        Ok(())
    }
}

struct Endpoint;
impl EndpointAllocator for Endpoint {
    fn select(&mut self) -> Result<LoopbackEndpoint, PreflightRefusal> {
        Ok(LoopbackEndpoint::new("127.0.0.1:31337".parse().unwrap()).unwrap())
    }
}

struct Clock;
impl SessionClock for Clock {
    fn wall_now(&mut self) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(42)
    }
    fn monotonic_elapsed(&mut self) -> Duration {
        Duration::ZERO
    }
}

struct ScriptClock(Rc<RefCell<VecDeque<Duration>>>);
impl SessionClock for ScriptClock {
    fn wall_now(&mut self) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(42)
    }
    fn monotonic_elapsed(&mut self) -> Duration {
        let mut values = self.0.borrow_mut();
        if values.len() > 1 {
            values.pop_front().expect("scripted instant")
        } else {
            *values.front().expect("final scripted instant")
        }
    }
}

struct Ids(VecDeque<String>);
impl IdentifierSource for Ids {
    fn next_id(&mut self, _: &'static str) -> Result<String, PreflightRefusal> {
        Ok(self.0.pop_front().expect("two identifiers"))
    }
}

struct Proxy(Ledger);
impl ProxyBackend for Proxy {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            name: "controlled".into(),
            version: "1".into(),
        }
    }
    fn start(&mut self, _: &SessionPlan, _: Budget) -> Result<Box<dyn ProxyLease>, StageFailure> {
        self.0.borrow_mut().push("proxy.start".into());
        Ok(Box::new(ProxyRun(self.0.clone())))
    }
}

struct ProxyRun(Ledger);
impl ProxyLease for ProxyRun {
    fn route(&self) -> Result<ProxyRoute, StageFailure> {
        Ok(test_route())
    }

    fn observations(&mut self, _: Budget) -> Result<Vec<CompatibilityObservation>, StageFailure> {
        self.0.borrow_mut().push("proxy.observe".into());
        Ok(vec![CompatibilityObservation {
            flow_id: fragcap::FlowId::new(9),
            proxy_connection_id: "proxy-1".into(),
            client_peer: None,
            proxy_local: None,
            observed_at: "1970-01-01T00:00:42Z".into(),
            process_id: Some(99),
            process_image: Some("controlled.exe".into()),
            role: Some("client".into()),
            attribution: Some("live".into()),
            packet_observations: 1,
            packet_observations_unretained: 0,
            correlation_state: fragcap::deep_capture::CorrelationState::Matched,
            correlation_reason: "exact-flow-and-owner".into(),
            protocol: "https".into(),
            inspectability: Inspectability::Full,
            method: Some("GET".into()),
            url: Some("https://example.invalid/".into()),
            status: Some(200),
            reason: Some("controlled final-client flow".into()),
        }])
    }
    fn stop(&mut self, _: Budget) -> CleanupResult {
        self.0.borrow_mut().push("proxy.stop".into());
        released("proxy-process")
    }
    fn cleanup(&mut self, _: Budget) -> Vec<CleanupResult> {
        self.0.borrow_mut().push("proxy.cleanup".into());
        vec![released("proxy-material")]
    }
}

struct BudgetProxy(Rc<RefCell<Option<Duration>>>);
impl ProxyBackend for BudgetProxy {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            name: "budget-proxy".into(),
            version: "1".into(),
        }
    }
    fn start(&mut self, _: &SessionPlan, _: Budget) -> Result<Box<dyn ProxyLease>, StageFailure> {
        Ok(Box::new(BudgetProxyRun(self.0.clone())))
    }
}

struct BudgetProxyRun(Rc<RefCell<Option<Duration>>>);
impl ProxyLease for BudgetProxyRun {
    fn route(&self) -> Result<ProxyRoute, StageFailure> {
        Ok(test_route())
    }

    fn observations(
        &mut self,
        budget: Budget,
    ) -> Result<Vec<CompatibilityObservation>, StageFailure> {
        *self.0.borrow_mut() = Some(budget.remaining());
        Ok(Vec::new())
    }
    fn stop(&mut self, _: Budget) -> CleanupResult {
        released("proxy-process")
    }
    fn cleanup(&mut self, _: Budget) -> Vec<CleanupResult> {
        vec![released("proxy-material")]
    }
}

struct SharedClock(Rc<RefCell<Duration>>);
impl SessionClock for SharedClock {
    fn wall_now(&mut self) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(42)
    }
    fn monotonic_elapsed(&mut self) -> Duration {
        *self.0.borrow()
    }
}

struct TimedCapture(Rc<RefCell<Duration>>);
impl CaptureRunner for TimedCapture {
    fn prepare(
        &mut self,
        _: &SessionConfig,
        _: &PreparedTarget,
        _: LoopbackEndpoint,
    ) -> Result<PreparedCapture, PreflightRefusal> {
        Ok(PreparedCapture {
            token: "timed".into(),
        })
    }
    fn run(
        &mut self,
        _: &PreparedCapture,
        _: &AppliedRoute,
        _: Budget,
    ) -> Result<CaptureRunResult, StageFailure> {
        *self.0.borrow_mut() = Duration::from_secs(8);
        Ok(CaptureRunResult {
            observations: Vec::new(),
            interrupted: false,
        })
    }
    fn stop(&mut self, _: Budget) -> CleanupResult {
        *self.0.borrow_mut() = Duration::from_secs(9);
        released("capture")
    }
}

struct Trust(Ledger);
impl TrustManager for Trust {
    fn acquire(
        &mut self,
        _: &SessionPlan,
        _: &ProxyRoute,
        _: Budget,
    ) -> Result<Box<dyn TrustLease>, StageFailure> {
        self.0.borrow_mut().push("trust.acquire".into());
        Ok(Box::new(TrustRun(self.0.clone())))
    }
}
struct TrustRun(Ledger);
impl TrustLease for TrustRun {
    fn cleanup(&mut self, _: Budget) -> CleanupResult {
        self.0.borrow_mut().push("trust.cleanup".into());
        released("trust-entry")
    }
}

struct Launch(Ledger);
impl LaunchAdapter for Launch {
    fn launch(
        &mut self,
        _: &PreparedTarget,
        _: LaunchCase,
        route: &AppliedRoute,
        _: Budget,
    ) -> Result<Box<dyn LaunchLease>, StageFailure> {
        assert_eq!(route.proxy().endpoint().port(), 31_337);
        self.0.borrow_mut().push("launch.start".into());
        Ok(Box::new(LaunchRun(self.0.clone())))
    }
}
struct LaunchRun(Ledger);
impl LaunchLease for LaunchRun {
    fn cleanup(&mut self, _: Budget) -> CleanupResult {
        self.0.borrow_mut().push("launch.cleanup".into());
        released("target-process")
    }
}

struct Capture(Ledger);
impl CaptureRunner for Capture {
    fn prepare(
        &mut self,
        _: &SessionConfig,
        _: &PreparedTarget,
        _: LoopbackEndpoint,
    ) -> Result<PreparedCapture, PreflightRefusal> {
        self.0.borrow_mut().push("capture.prepare".into());
        Ok(PreparedCapture {
            token: "prepared".into(),
        })
    }
    fn run(
        &mut self,
        prepared: &PreparedCapture,
        _: &AppliedRoute,
        _: Budget,
    ) -> Result<CaptureRunResult, StageFailure> {
        assert_eq!(prepared.token, "prepared");
        self.0.borrow_mut().push("capture.run".into());
        Ok(CaptureRunResult {
            observations: Vec::new(),
            interrupted: false,
        })
    }
    fn stop(&mut self, _: Budget) -> CleanupResult {
        self.0.borrow_mut().push("capture.stop".into());
        released("capture")
    }
}

struct Facts(Ledger);
impl CompatibilityRepository for Facts {
    fn append(&mut self, _: &PreparedTarget, fact: &CompatibilityFact) -> FactWriteStatus {
        assert!(!fact.kind.is_empty());
        self.0.borrow_mut().push("fact.append".into());
        FactWriteStatus::Appended
    }
}

struct Artifacts(Ledger);
impl ArtifactSink for Artifacts {
    fn validate_destination(&mut self, _: &Path) -> Result<(), PreflightRefusal> {
        Ok(())
    }
    fn finalize(&mut self, bundle: &Path, _: &TerminalSnapshot) -> Vec<ArtifactResult> {
        ["compatibility", "fact-writes", "cleanup", "manifest"]
            .into_iter()
            .map(|role| {
                self.0.borrow_mut().push(format!("artifact.{role}"));
                ArtifactResult {
                    role: role.into(),
                    path: bundle.join(format!("{role}.json")),
                    sensitivity: Sensitivity::Metadata,
                    required: true,
                    status: ArtifactStatus::Written,
                }
            })
            .collect()
    }
}

struct Events(Ledger);
impl EventSink for Events {
    fn emit(&mut self, event: &DeepCaptureEvent) -> Result<(), StageFailure> {
        let kind = match event {
            DeepCaptureEvent::Plan { .. } => "plan",
            DeepCaptureEvent::ProxyStarted { .. } => "proxy-started",
            DeepCaptureEvent::TrustAcquired { .. } => "trust-acquired",
            DeepCaptureEvent::LaunchStarted { .. } => "launch-started",
            DeepCaptureEvent::Started { .. } => "started",
            DeepCaptureEvent::Observation { .. } => "observation",
            DeepCaptureEvent::Cleanup { .. } => "cleanup",
            DeepCaptureEvent::Terminal { .. } => "terminal",
            _ => "future",
        };
        self.0.borrow_mut().push(format!("event.{kind}"));
        Ok(())
    }
}

struct FailingEvents(Ledger);
impl EventSink for FailingEvents {
    fn emit(&mut self, event: &DeepCaptureEvent) -> Result<(), StageFailure> {
        self.0.borrow_mut().push("event.attempt".into());
        if matches!(event, DeepCaptureEvent::Observation { .. }) {
            Err(StageFailure::new(
                Stage::EventDelivery,
                "controlled-event-failure",
                "controlled event sink rejected the observation",
            ))
        } else {
            Ok(())
        }
    }
}

struct FailingProxy(Ledger);
impl ProxyBackend for FailingProxy {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            name: "controlled-failure".into(),
            version: "1".into(),
        }
    }

    fn start(&mut self, _: &SessionPlan, _: Budget) -> Result<Box<dyn ProxyLease>, StageFailure> {
        self.0.borrow_mut().push("proxy.start.failed".into());
        Err(StageFailure::new(
            Stage::ProxyStart,
            "controlled-proxy-failure",
            "controlled proxy failed before acquisition",
        ))
    }
}

struct FailingTrust(Ledger);
impl TrustManager for FailingTrust {
    fn acquire(
        &mut self,
        _: &SessionPlan,
        _: &ProxyRoute,
        _: Budget,
    ) -> Result<Box<dyn TrustLease>, StageFailure> {
        self.0.borrow_mut().push("trust.acquire.failed".into());
        Err(StageFailure::new(
            Stage::Trust,
            "trust-failed",
            "controlled trust failure",
        ))
    }
}

struct FailingLaunch(Ledger);
impl LaunchAdapter for FailingLaunch {
    fn launch(
        &mut self,
        _: &PreparedTarget,
        _: LaunchCase,
        _: &AppliedRoute,
        _: Budget,
    ) -> Result<Box<dyn LaunchLease>, StageFailure> {
        self.0.borrow_mut().push("launch.start.failed".into());
        Err(StageFailure::new(
            Stage::Launch,
            "launch-failed",
            "controlled launch failure",
        ))
    }
}

struct FailingCapture(Ledger);
impl CaptureRunner for FailingCapture {
    fn prepare(
        &mut self,
        _: &SessionConfig,
        _: &PreparedTarget,
        _: LoopbackEndpoint,
    ) -> Result<PreparedCapture, PreflightRefusal> {
        Ok(PreparedCapture {
            token: "failed-capture".into(),
        })
    }
    fn run(
        &mut self,
        _: &PreparedCapture,
        _: &AppliedRoute,
        _: Budget,
    ) -> Result<CaptureRunResult, StageFailure> {
        self.0.borrow_mut().push("capture.run.failed".into());
        Err(StageFailure::new(
            Stage::Capture,
            "capture-failed",
            "controlled capture failure",
        ))
    }
    fn stop(&mut self, _: Budget) -> CleanupResult {
        self.0.borrow_mut().push("capture.stop".into());
        released("capture")
    }
}

struct InterruptedCapture(Ledger);
impl CaptureRunner for InterruptedCapture {
    fn prepare(
        &mut self,
        _: &SessionConfig,
        _: &PreparedTarget,
        _: LoopbackEndpoint,
    ) -> Result<PreparedCapture, PreflightRefusal> {
        Ok(PreparedCapture {
            token: "interrupted-capture".into(),
        })
    }
    fn run(
        &mut self,
        _: &PreparedCapture,
        _: &AppliedRoute,
        _: Budget,
    ) -> Result<CaptureRunResult, StageFailure> {
        self.0.borrow_mut().push("capture.interrupted".into());
        Ok(CaptureRunResult {
            observations: Vec::new(),
            interrupted: true,
        })
    }
    fn stop(&mut self, _: Budget) -> CleanupResult {
        released("capture")
    }
}

struct FailingFacts(Ledger);
impl CompatibilityRepository for FailingFacts {
    fn append(&mut self, _: &PreparedTarget, _: &CompatibilityFact) -> FactWriteStatus {
        self.0.borrow_mut().push("fact.append.failed".into());
        FactWriteStatus::Failed {
            code: "fact-failed".into(),
            detail: "controlled fact failure".into(),
        }
    }
}

struct FailingArtifacts(Ledger);
impl ArtifactSink for FailingArtifacts {
    fn validate_destination(&mut self, _: &Path) -> Result<(), PreflightRefusal> {
        Ok(())
    }
    fn finalize(&mut self, bundle: &Path, _: &TerminalSnapshot) -> Vec<ArtifactResult> {
        self.0.borrow_mut().push("artifact.failed".into());
        vec![ArtifactResult {
            role: "manifest".into(),
            path: bundle.join("manifest.json"),
            sensitivity: Sensitivity::Metadata,
            required: true,
            status: ArtifactStatus::Failed {
                code: "artifact-failed".into(),
                detail: "controlled artifact failure".into(),
            },
        }]
    }
}

struct FailingCleanupTrust(Ledger);
impl TrustManager for FailingCleanupTrust {
    fn acquire(
        &mut self,
        _: &SessionPlan,
        _: &ProxyRoute,
        _: Budget,
    ) -> Result<Box<dyn TrustLease>, StageFailure> {
        Ok(Box::new(FailingCleanupTrustRun(self.0.clone())))
    }
}
struct FailingCleanupTrustRun(Ledger);
impl TrustLease for FailingCleanupTrustRun {
    fn cleanup(&mut self, _: Budget) -> CleanupResult {
        self.0.borrow_mut().push("trust.cleanup.failed".into());
        CleanupResult {
            resource: "trust-entry".into(),
            status: CleanupStatus::Failed,
            reason: "controlled cleanup failure".into(),
        }
    }
}

fn released(resource: &str) -> CleanupResult {
    CleanupResult {
        resource: resource.into(),
        status: CleanupStatus::Released,
        reason: "released".into(),
    }
}

fn test_route() -> ProxyRoute {
    ProxyRoute::new(
        LoopbackEndpoint::new("127.0.0.1:31337".parse().unwrap()).unwrap(),
        (
            "http://fragcap:test@127.0.0.1:31337".to_string().into(),
            "socks5h://fragcap:test@127.0.0.1:31337".to_string().into(),
        ),
        "Basic test".to_string(),
        vec![1, 2, 3],
        "test-thumbprint".to_string(),
        1,
        None,
    )
}

fn config() -> SessionConfig {
    static NEXT_BUNDLE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let bundle_id = NEXT_BUNDLE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    SessionConfig {
        target: "controlled-target".into(),
        launch_case: Some(LaunchCase::Controlled),
        mode: SessionMode::TlsCalibration,
        controlled: true,
        bundle: std::env::temp_dir().join(format!(
            "fragcap-deep-capture-session-{}-{bundle_id}",
            std::process::id()
        )),
        trust_ca: true,
        har: true,
        key_log: false,
        client_identity: false,
        sensitive_retention: SensitiveRetention::Retain,
        deadlines: Deadlines::default(),
    }
}

fn adapters(ledger: &Ledger) -> AdapterSet<'_> {
    AdapterSet {
        targets: Box::new(Targets(ledger.clone())),
        endpoints: Box::new(Endpoint),
        clock: Box::new(Clock),
        identifiers: Box::new(Ids(VecDeque::from(["session-1".into(), "plan-1".into()]))),
        proxy: Box::new(Proxy(ledger.clone())),
        trust: Box::new(Trust(ledger.clone())),
        routing: Box::new(ChildEnvironmentRouting),
        launch: Box::new(Launch(ledger.clone())),
        capture: Box::new(Capture(ledger.clone())),
        facts: Box::new(Facts(ledger.clone())),
        artifacts: Box::new(Artifacts(ledger.clone())),
        events: Box::new(Events(ledger.clone())),
    }
}

#[test]
fn controlled_consumer_runs_complete_lifecycle_without_cli() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    let prepared = DeepCapture::preflight(config(), &mut environment).expect("preflight");
    assert_eq!(
        prepared.plan().artifacts,
        ArtifactRequests {
            har: true,
            key_log: false,
            sensitive_retention: SensitiveRetention::Retain,
        }
    );
    let authorization = Authorization::approved(prepared.plan().id.clone());
    let report = prepared
        .into_session(environment)
        .run_to_completion(authorization);

    assert!(report.is_complete());
    assert_eq!(report.snapshot.observations.len(), 1);
    assert_eq!(report.snapshot.fact_writes.len(), 5);
    assert_eq!(report.snapshot.cleanup.len(), 6);
    assert_eq!(report.snapshot.artifacts, prepared_artifact_requests());
    assert_eq!(report.artifacts.last().expect("manifest").role, "manifest");
    let calls = ledger.borrow();
    assert!(
        calls.iter().position(|call| call == "fact.append")
            < calls.iter().position(|call| call == "launch.cleanup")
    );
    assert!(
        calls.iter().position(|call| call == "proxy.cleanup")
            < calls
                .iter()
                .position(|call| call == "artifact.compatibility")
    );
    assert!(
        calls.iter().position(|call| call == "event.proxy-started")
            < calls.iter().position(|call| call == "trust.acquire")
    );
    assert!(
        calls.iter().position(|call| call == "event.trust-acquired")
            < calls.iter().position(|call| call == "launch.start")
    );
}

fn prepared_artifact_requests() -> ArtifactRequests {
    ArtifactRequests {
        har: true,
        key_log: false,
        sensitive_retention: SensitiveRetention::Retain,
    }
}

#[test]
fn refused_preflight_has_no_external_effects() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    let mut invalid = config();
    invalid.target.clear();
    let refusal = DeepCapture::preflight(invalid, &mut environment).expect_err("refused");
    assert_eq!(refusal.code, "target-empty");
    assert!(ledger.borrow().is_empty());
}

#[test]
fn stale_authorization_has_no_external_effects() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    let prepared = DeepCapture::preflight(config(), &mut environment).expect("preflight");
    ledger.borrow_mut().clear();
    let report = prepared
        .into_session(environment)
        .run_to_completion(Authorization::approved(PlanId::new("stale")));
    assert_eq!(report.snapshot.outcome, SessionOutcome::Failed);
    assert_eq!(report.snapshot.failures[0].code, "plan-id-mismatch");
    assert!(ledger.borrow().is_empty());
}

#[test]
fn invalid_transition_does_not_call_an_adapter() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    let prepared = DeepCapture::preflight(config(), &mut environment).expect("preflight");
    ledger.borrow_mut().clear();
    let mut session = prepared.into_session(environment);
    let error = session.observe().expect_err("observe before start");
    assert_eq!(error.operation, Operation::Observe);
    assert_eq!(error.actual, LifecycleState::Prepared);
    assert!(ledger.borrow().is_empty());
}

#[test]
fn event_delivery_failure_does_not_skip_cleanup_or_erase_evidence() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    environment.events = Box::new(FailingEvents(ledger.clone()));
    let prepared = DeepCapture::preflight(config(), &mut environment).expect("preflight");
    let authorization = Authorization::approved(prepared.plan().id.clone());
    let report = prepared
        .into_session(environment)
        .run_to_completion(authorization);

    assert!(!report.is_complete());
    assert_eq!(report.snapshot.outcome, SessionOutcome::Partial);
    assert_eq!(report.snapshot.observations.len(), 1);
    assert_eq!(report.event_failures.len(), 1);
    assert!(ledger.borrow().iter().any(|call| call == "proxy.cleanup"));
    assert!(ledger.borrow().iter().any(|call| call == "trust.cleanup"));
}

#[test]
fn proxy_start_failure_still_returns_a_typed_terminal_report() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    environment.proxy = Box::new(FailingProxy(ledger.clone()));
    let prepared = DeepCapture::preflight(config(), &mut environment).expect("preflight");
    let authorization = Authorization::approved(prepared.plan().id.clone());
    let report = prepared
        .into_session(environment)
        .run_to_completion(authorization);

    assert_eq!(report.snapshot.outcome, SessionOutcome::Failed);
    assert_eq!(report.snapshot.failures[0].code, "controlled-proxy-failure");
    assert!(!ledger.borrow().iter().any(|call| call == "trust.acquire"));
    assert!(!ledger.borrow().iter().any(|call| call == "launch.start"));
    assert!(ledger
        .borrow()
        .iter()
        .any(|call| call == "artifact.manifest"));
}

#[test]
fn a_late_success_is_a_failure_and_its_resource_is_still_cleaned() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    environment.clock = Box::new(ScriptClock(Rc::new(RefCell::new(VecDeque::from([
        Duration::ZERO,
        Duration::ZERO,
        Duration::from_secs(2),
    ])))));
    let mut bounded = config();
    bounded.deadlines.launch = Duration::from_secs(1);
    let prepared = DeepCapture::preflight(bounded, &mut environment).expect("preflight");
    let authorization = Authorization::approved(prepared.plan().id.clone());
    let report = prepared
        .into_session(environment)
        .run_to_completion(authorization);

    assert_eq!(report.snapshot.outcome, SessionOutcome::Failed);
    assert!(report
        .snapshot
        .failures
        .iter()
        .any(|failure| failure.code == "launch-deadline-exceeded"));
    assert!(ledger.borrow().iter().any(|call| call == "proxy.cleanup"));
    assert!(!ledger.borrow().iter().any(|call| call == "trust.acquire"));
}

#[test]
fn proxy_observation_uses_only_the_original_observation_budget_remaining() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let elapsed = Rc::new(RefCell::new(Duration::ZERO));
    let proxy_budget = Rc::new(RefCell::new(None));
    let mut environment = adapters(&ledger);
    environment.clock = Box::new(SharedClock(elapsed.clone()));
    environment.capture = Box::new(TimedCapture(elapsed));
    environment.proxy = Box::new(BudgetProxy(proxy_budget.clone()));
    let mut bounded = config();
    bounded.deadlines.observation = Duration::from_secs(10);

    let prepared = DeepCapture::preflight(bounded, &mut environment).expect("preflight");
    let authorization = Authorization::approved(prepared.plan().id.clone());
    let report = prepared
        .into_session(environment)
        .run_to_completion(authorization);

    assert_eq!(*proxy_budget.borrow(), Some(Duration::from_secs(1)));
    assert!(report.snapshot.failures.is_empty());
}

#[test]
fn dropping_a_started_session_stops_and_cleans_every_acquired_resource_once() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    let prepared = DeepCapture::preflight(config(), &mut environment).expect("preflight");
    let authorization = Authorization::approved(prepared.plan().id.clone());
    let mut session = prepared.into_session(environment);
    session.start(authorization).expect("start");
    drop(session);

    let calls = ledger.borrow();
    for expected in [
        "capture.stop",
        "proxy.stop",
        "launch.cleanup",
        "trust.cleanup",
        "proxy.cleanup",
    ] {
        assert_eq!(
            calls
                .iter()
                .filter(|call| call.as_str() == expected)
                .count(),
            1,
            "{expected} must be attempted exactly once"
        );
    }
}

fn run_with(mut environment: AdapterSet<'_>) -> TerminalReport {
    let prepared = DeepCapture::preflight(config(), &mut environment).expect("preflight");
    let authorization = Authorization::approved(prepared.plan().id.clone());
    prepared
        .into_session(environment)
        .run_to_completion(authorization)
}

#[test]
fn trust_failure_still_cleans_the_proxy() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    environment.trust = Box::new(FailingTrust(ledger.clone()));
    let report = run_with(environment);
    assert!(report
        .snapshot
        .failures
        .iter()
        .any(|failure| failure.code == "trust-failed"));
    assert!(ledger.borrow().iter().any(|call| call == "proxy.cleanup"));
    assert!(!ledger.borrow().iter().any(|call| call == "launch.start"));
}

#[test]
fn launch_failure_still_cleans_trust_and_proxy() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    environment.launch = Box::new(FailingLaunch(ledger.clone()));
    let report = run_with(environment);
    assert!(report
        .snapshot
        .failures
        .iter()
        .any(|failure| failure.code == "launch-failed"));
    assert!(ledger.borrow().iter().any(|call| call == "trust.cleanup"));
    assert!(ledger.borrow().iter().any(|call| call == "proxy.cleanup"));
}

#[test]
fn capture_failure_still_stops_and_finalizes() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    environment.capture = Box::new(FailingCapture(ledger.clone()));
    let report = run_with(environment);
    assert!(report
        .snapshot
        .failures
        .iter()
        .any(|failure| failure.code == "capture-failed"));
    assert!(ledger.borrow().iter().any(|call| call == "proxy.stop"));
    assert!(ledger
        .borrow()
        .iter()
        .any(|call| call == "artifact.manifest"));
}

#[test]
fn fact_failure_is_partial_and_does_not_skip_cleanup() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    environment.facts = Box::new(FailingFacts(ledger.clone()));
    let report = run_with(environment);
    assert_eq!(report.snapshot.outcome, SessionOutcome::Partial);
    assert!(report
        .snapshot
        .failures
        .iter()
        .any(|failure| failure.code == "fact-failed"));
    assert!(ledger.borrow().iter().any(|call| call == "proxy.cleanup"));
}

#[test]
fn artifact_failure_is_lossless_and_never_complete() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    environment.artifacts = Box::new(FailingArtifacts(ledger.clone()));
    let report = run_with(environment);
    assert_eq!(report.snapshot.outcome, SessionOutcome::Partial);
    assert!(report
        .snapshot
        .failures
        .iter()
        .any(|failure| failure.code == "artifact-failed"));
    assert_eq!(report.artifacts.len(), 1);
}

#[test]
fn cleanup_failure_does_not_skip_other_cleanup_or_bundle_finalization() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    environment.trust = Box::new(FailingCleanupTrust(ledger.clone()));
    let report = run_with(environment);
    assert_eq!(report.snapshot.outcome, SessionOutcome::Partial);
    assert!(ledger.borrow().iter().any(|call| call == "proxy.cleanup"));
    assert!(ledger
        .borrow()
        .iter()
        .any(|call| call == "artifact.manifest"));
}

#[test]
fn capture_interrupt_reaches_the_typed_terminal_outcome() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    environment.capture = Box::new(InterruptedCapture(ledger.clone()));
    let report = run_with(environment);
    assert_eq!(report.snapshot.outcome, SessionOutcome::Interrupted);
}
