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

struct Endpoint(Ledger);
impl EndpointAllocator for Endpoint {
    fn select(&mut self) -> Result<LoopbackEndpoint, PreflightRefusal> {
        self.0.borrow_mut().push("endpoint.select".into());
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
            classification: ProtocolClassification::new(
                TrafficFamily::Https,
                DetectionState::Identified,
                InspectabilityState::Full,
                None,
            )
            .unwrap(),
        }])
    }
    fn observations_lost(&self) -> u64 {
        2
    }
    fn application_classification_summary(&self) -> Option<ClassificationSummary> {
        let classification = ProtocolClassification::new(
            TrafficFamily::Http2,
            DetectionState::Identified,
            InspectabilityState::Full,
            None,
        )
        .unwrap();
        Some(ClassificationSummary::from_classifications(
            [&classification],
            4,
        ))
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

struct Routing(Ledger, ChildEnvironmentRouting);

impl RoutingAdapter for Routing {
    fn prepare(
        &mut self,
        target: &PreparedTarget,
        plan: &RoutingPlan,
    ) -> Result<(), PreflightRefusal> {
        self.1.prepare(target, plan)
    }

    fn apply(
        &mut self,
        session: &SessionPlan,
        proxy: ProxyRoute,
        budget: Budget,
    ) -> Result<Box<dyn RoutingLease>, StageFailure> {
        self.0.borrow_mut().push("routing.apply".into());
        self.1.apply(session, proxy, budget)
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

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum FailingEventPoint {
    Started,
    Terminal,
}

#[allow(dead_code)]
struct FailingNamedEvent(Ledger, FailingEventPoint);
impl EventSink for FailingNamedEvent {
    fn emit(&mut self, event: &DeepCaptureEvent) -> Result<(), StageFailure> {
        self.0.borrow_mut().push("event.attempt".into());
        let selected = matches!(
            (self.1, event),
            (FailingEventPoint::Started, DeepCaptureEvent::Started { .. })
                | (
                    FailingEventPoint::Terminal,
                    DeepCaptureEvent::Terminal { .. }
                )
        );
        if selected {
            Err(StageFailure::new(
                Stage::EventDelivery,
                "controlled-event-failure",
                "controlled event sink rejected the selected lifecycle event",
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

#[allow(dead_code)]
struct FailingProxyCleanup(Ledger);
impl ProxyBackend for FailingProxyCleanup {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            name: "controlled-cleanup-failure".into(),
            version: "1".into(),
        }
    }

    fn start(&mut self, _: &SessionPlan, _: Budget) -> Result<Box<dyn ProxyLease>, StageFailure> {
        Ok(Box::new(FailingProxyCleanupRun(self.0.clone())))
    }
}

#[allow(dead_code)]
struct FailingProxyCleanupRun(Ledger);
impl ProxyLease for FailingProxyCleanupRun {
    fn route(&self) -> Result<ProxyRoute, StageFailure> {
        Ok(test_route())
    }

    fn observations(&mut self, _: Budget) -> Result<Vec<CompatibilityObservation>, StageFailure> {
        Ok(Vec::new())
    }

    fn stop(&mut self, _: Budget) -> CleanupResult {
        self.0.borrow_mut().push("proxy.stop.failed".into());
        CleanupResult {
            resource: "proxy-process".into(),
            status: CleanupStatus::Failed,
            reason: "controlled network reset during proxy stop".into(),
        }
    }

    fn cleanup(&mut self, _: Budget) -> Vec<CleanupResult> {
        self.0.borrow_mut().push("proxy.cleanup.failed".into());
        vec![CleanupResult {
            resource: "proxy-task".into(),
            status: CleanupStatus::Failed,
            reason: "controlled proxy task panic retained by join".into(),
        }]
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

struct FailingRouting(Ledger);
impl RoutingAdapter for FailingRouting {
    fn prepare(&mut self, _: &PreparedTarget, _: &RoutingPlan) -> Result<(), PreflightRefusal> {
        Ok(())
    }

    fn apply(
        &mut self,
        _: &SessionPlan,
        _: ProxyRoute,
        _: Budget,
    ) -> Result<Box<dyn RoutingLease>, StageFailure> {
        self.0.borrow_mut().push("route.apply.failed".into());
        Err(StageFailure::new(
            Stage::Routing,
            "route-permission-denied",
            "controlled route application denial",
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

#[allow(dead_code)]
struct TimedOutStopCapture(Ledger);
impl CaptureRunner for TimedOutStopCapture {
    fn prepare(
        &mut self,
        _: &SessionConfig,
        _: &PreparedTarget,
        _: LoopbackEndpoint,
    ) -> Result<PreparedCapture, PreflightRefusal> {
        Ok(PreparedCapture {
            token: "timed-out-stop".into(),
        })
    }

    fn run(
        &mut self,
        _: &PreparedCapture,
        _: &AppliedRoute,
        _: Budget,
    ) -> Result<CaptureRunResult, StageFailure> {
        Ok(CaptureRunResult {
            observations: Vec::new(),
            interrupted: false,
        })
    }

    fn stop(&mut self, _: Budget) -> CleanupResult {
        self.0.borrow_mut().push("capture.stop.timed-out".into());
        CleanupResult {
            resource: "capture".into(),
            status: CleanupStatus::TimedOut,
            reason: "controlled capture stop timeout".into(),
        }
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

struct FailingBoundary {
    boundary: String,
    side: BoundarySide,
    family: String,
    driver: String,
    ledger: Ledger,
}

impl BoundaryController for FailingBoundary {
    fn check(&mut self, boundary: &str, side: BoundarySide) -> Result<(), StageFailure> {
        self.ledger
            .borrow_mut()
            .push(format!("boundary.{boundary}.{side:?}"));
        if boundary == self.boundary && side == self.side {
            Err(StageFailure::new(
                Stage::Bundle,
                format!("controlled-{}", self.family),
                format!("{} at {boundary}:{side:?}", self.driver),
            ))
        } else {
            Ok(())
        }
    }
}

#[allow(dead_code)]
struct CorruptingArtifacts(Ledger);
impl ArtifactSink for CorruptingArtifacts {
    fn validate_destination(&mut self, _: &Path) -> Result<(), PreflightRefusal> {
        Ok(())
    }

    fn finalize(&mut self, bundle: &Path, _: &TerminalSnapshot) -> Vec<ArtifactResult> {
        self.0
            .borrow_mut()
            .push("artifact.written-before-corruption".into());
        vec![ArtifactResult {
            role: "application".into(),
            path: bundle.join("application.jsonl"),
            sensitivity: Sensitivity::Payload,
            required: true,
            status: ArtifactStatus::Written,
        }]
    }

    fn reconcile(&mut self, bundle: &Path, _: &TerminalSnapshot) -> Vec<ArtifactResult> {
        self.0.borrow_mut().push("artifact.corrupt".into());
        vec![ArtifactResult {
            role: "manifest".into(),
            path: bundle.join("manifest.json"),
            sensitivity: Sensitivity::Metadata,
            required: true,
            status: ArtifactStatus::Failed {
                code: "artifact-corrupt".into(),
                detail: "controlled corruption after initial artifact publication".into(),
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
        "0123456789abcdef0123456789abcdef01234567".to_string(),
        1,
        None,
    )
}

fn config() -> SessionConfig {
    static NEXT_BUNDLE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let bundle_id = NEXT_BUNDLE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let run_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("test clock follows the Unix epoch")
        .as_nanos();
    SessionConfig {
        target: "controlled-target".into(),
        launch_case: Some(LaunchCase::Controlled),
        mode: SessionMode::TlsCalibration,
        calibration_protocol: Some(fragcap::targets::CompatibilityProtocol::Https),
        controlled: true,
        bundle: std::env::temp_dir().join(format!(
            "fragcap-deep-capture-session-{}-{run_id}-{bundle_id}",
            std::process::id()
        )),
        trust_ca: true,
        har: true,
        key_log: false,
        client_identity: false,
        proxy_bypass: Vec::new(),
        sensitive_retention: SensitiveRetention::Retain,
        deadlines: Deadlines::default(),
    }
}

fn adapters(ledger: &Ledger) -> AdapterSet<'_> {
    AdapterSet {
        boundaries: Box::new(AllowBoundaries),
        targets: Box::new(Targets(ledger.clone())),
        endpoints: Box::new(Endpoint(ledger.clone())),
        clock: Box::new(Clock),
        identifiers: Box::new(Ids(VecDeque::from(["session-1".into(), "plan-1".into()]))),
        proxy: Box::new(Proxy(ledger.clone())),
        trust: Box::new(Trust(ledger.clone())),
        routing: Box::new(Routing(ledger.clone(), ChildEnvironmentRouting)),
        launch: Box::new(Launch(ledger.clone())),
        capture: Box::new(Capture(ledger.clone())),
        facts: Box::new(Facts(ledger.clone())),
        artifacts: Box::new(Artifacts(ledger.clone())),
        events: Box::new(Events(ledger.clone())),
    }
}

#[test]
fn malformed_bypass_refuses_before_listener_selection() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    let mut invalid = config();
    invalid.proxy_bypass = vec!["*".to_string()];
    let refusal = DeepCapture::preflight(invalid, &mut environment).unwrap_err();
    assert_eq!(refusal.code, "proxy-bypass-invalid");
    assert!(!ledger.borrow().iter().any(|call| call == "endpoint.select"));
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
    let classification = report.snapshot.classification_summary();
    assert_eq!(classification.observations, 1);
    assert_eq!(classification.unclassified_lost, 4);
    assert_eq!(classification.by_family.get("http2"), Some(&1));
    assert_eq!(
        classification.observations + classification.unclassified_lost,
        5
    );
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
    assert_eq!(
        ledger
            .borrow()
            .iter()
            .filter(|call| call.as_str() == "proxy.stop")
            .count(),
        1
    );
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
    assert_eq!(
        ledger
            .borrow()
            .iter()
            .filter(|call| call.as_str() == "proxy.stop")
            .count(),
        1
    );
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
fn route_failure_still_cleans_trust_and_proxy_without_launching() {
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let mut environment = adapters(&ledger);
    environment.routing = Box::new(FailingRouting(ledger.clone()));
    let report = run_with(environment);
    assert!(report
        .snapshot
        .failures
        .iter()
        .any(|failure| failure.code == "route-permission-denied"));
    let calls = ledger.borrow();
    assert!(calls.iter().any(|call| call == "trust.cleanup"));
    assert!(calls.iter().any(|call| call == "proxy.cleanup"));
    assert!(!calls.iter().any(|call| call == "launch.start"));
    assert!(report.snapshot.fact_writes.iter().all(|write| {
        write.fact.kind == "launch-case" && write.fact.final_owner_index.is_none()
    }));
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
    assert!(matches!(
        report.artifacts[0].status,
        ArtifactStatus::Failed { .. }
    ));
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

#[test]
fn generated_failure_matrix_exercises_production_authorities() {
    let registry: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/security/deep-capture-failures.v1.json"
    ))
    .expect("failure registry");
    let mut scenarios = Vec::new();
    for boundary in registry["effects"].as_array().unwrap() {
        for side in ["before", "after"] {
            scenarios.push((
                format!("{}:{side}", boundary["id"].as_str().unwrap()),
                boundary["id"].as_str().unwrap(),
                true,
                side,
                boundary[side]["family"].as_str().unwrap(),
                boundary[side]["driver"].as_str().unwrap(),
                &boundary[side]["expected"],
            ));
        }
    }
    for boundary in registry["transitions"].as_array().unwrap() {
        for side in ["before", "after"] {
            scenarios.push((
                format!("{}:{side}", boundary["id"].as_str().unwrap()),
                boundary["id"].as_str().unwrap(),
                false,
                side,
                boundary[side]["family"].as_str().unwrap(),
                boundary[side]["driver"].as_str().unwrap(),
                &boundary[side]["expected"],
            ));
        }
    }
    assert_eq!(scenarios.len(), 30);

    for (scenario, boundary, is_effect, side, family, driver, expected) in scenarios {
        let ledger = Rc::new(RefCell::new(Vec::new()));
        let mut environment = adapters(&ledger);
        environment.boundaries = Box::new(FailingBoundary {
            boundary: boundary.to_string(),
            side: if side == "before" {
                BoundarySide::Before
            } else {
                BoundarySide::After
            },
            family: family.to_string(),
            driver: driver.to_string(),
            ledger: ledger.clone(),
        });
        if !is_effect && boundary == "prepared-stopped" {
            environment.proxy = Box::new(FailingProxy(ledger.clone()));
        }

        let mut scenario_config = config();
        if family == "timeout" {
            scenario_config.deadlines.launch = Duration::from_secs(1);
        }
        let bundle = scenario_config.bundle.clone();
        let prepared = DeepCapture::preflight(scenario_config, &mut environment)
            .unwrap_or_else(|error| panic!("{scenario}: preflight failed: {}", error.code));
        let authorization = if !is_effect && boundary == "prepared-terminal" {
            Authorization::approved(PlanId::new("stale-matrix-plan"))
        } else {
            Authorization::approved(prepared.plan().id.clone())
        };
        let mut session = prepared.into_session(environment);
        let report = if !is_effect && boundary == "running-stopped" {
            session.start(authorization).expect("matrix start");
            session.stop().expect("matrix direct stop");
            session.finalize().expect("matrix finalization")
        } else {
            session.run_to_completion(authorization)
        };
        let boundary_marker = format!(
            "boundary.{boundary}.{}",
            if side == "before" { "Before" } else { "After" }
        );
        assert_eq!(
            ledger
                .borrow()
                .iter()
                .filter(|entry| *entry == &boundary_marker)
                .count(),
            1,
            "{scenario}: selected production boundary was not reached exactly once"
        );
        assert!(
            report
                .snapshot
                .failures
                .iter()
                .any(|failure| failure.code == format!("controlled-{family}")),
            "{scenario}: selected boundary failure was not retained"
        );

        let journal = if !is_effect && boundary == "prepared-terminal" {
            assert!(
                !bundle.join("resource-journal.jsonl").exists(),
                "{scenario}: refused authorization created a journal"
            );
            None
        } else {
            let journal = read_resource_journal(&bundle.join("resource-journal.jsonl"))
                .unwrap_or_else(|error| panic!("{scenario}: journal truth: {error}"));
            assert!(
                !journal.transitions.is_empty(),
                "{scenario}: journal contains no effect decisions"
            );
            let recovery = journal.recovery_plan();
            let decisions = recovery
                .actions
                .iter()
                .map(|action| action.resource_id.as_str())
                .chain(
                    recovery
                        .refusals
                        .iter()
                        .map(|refusal| refusal.resource_id.as_str()),
                )
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(
                decisions.len(),
                recovery.actions.len() + recovery.refusals.len(),
                "{scenario}: recovery truth contains duplicate decisions"
            );
            Some(journal)
        };

        if !is_effect {
            assert!(
                report.snapshot.lifecycle_transitions.iter().any(|edge| {
                    format!("{:?}-{:?}", edge.from, edge.to).to_ascii_lowercase() == boundary
                }),
                "{scenario}: named lifecycle edge was not traversed"
            );
        }
        assert_expected_vector(
            &scenario,
            expected,
            &report,
            &ledger.borrow(),
            journal.as_ref(),
        );
        if is_effect {
            assert_production_effect_side(
                &scenario,
                boundary,
                side,
                &ledger.borrow(),
                journal.as_ref().unwrap(),
            );
        }
    }
}

fn assert_expected_vector(
    scenario: &str,
    expected: &serde_json::Value,
    report: &TerminalReport,
    calls: &[String],
    journal: Option<&JournalPrefix>,
) {
    let value = |dimension: &str| {
        expected[dimension]
            .as_str()
            .unwrap_or_else(|| panic!("{scenario}: missing {dimension} expectation"))
    };
    let terminal = match value("terminal") {
        "failed" => report.snapshot.outcome == SessionOutcome::Failed,
        "partial" => report.snapshot.outcome == SessionOutcome::Partial,
        "interrupted" => report.snapshot.outcome == SessionOutcome::Interrupted,
        "interrupted-or-failed" => matches!(
            report.snapshot.outcome,
            SessionOutcome::Interrupted | SessionOutcome::Failed
        ),
        "interrupted-or-partial" => matches!(
            report.snapshot.outcome,
            SessionOutcome::Interrupted | SessionOutcome::Partial
        ),
        "partial-or-failed" => matches!(
            report.snapshot.outcome,
            SessionOutcome::Partial | SessionOutcome::Failed
        ),
        other => panic!("{scenario}: unsupported terminal expectation {other}"),
    };
    assert!(
        terminal,
        "{scenario}: terminal expectation {}",
        value("terminal")
    );

    let failed_artifact = report
        .artifacts
        .iter()
        .any(|result| matches!(result.status, ArtifactStatus::Failed { .. }));
    let artifact = match value("artifact") {
        "none" => report.artifacts.is_empty(),
        "failed-not-complete" => failed_artifact && !report.is_complete(),
        "incomplete" => !report.is_complete(),
        "independent" => !report.artifacts.is_empty(),
        other => panic!("{scenario}: unsupported artifact expectation {other}"),
    };
    assert!(
        artifact,
        "{scenario}: artifact expectation {}",
        value("artifact")
    );

    let failed_fact = report
        .snapshot
        .fact_writes
        .iter()
        .any(|result| matches!(result.status, FactWriteStatus::Failed { .. }));
    let fact = match value("fact") {
        "none" => report.snapshot.observations.is_empty(),
        "failed-counted" => failed_fact,
        "independent" => {
            !report.snapshot.fact_writes.is_empty()
                && report.snapshot.fact_writes.iter().all(|result| {
                    matches!(result.status, FactWriteStatus::Appended)
                        && !result.fact.evidence.is_empty()
                })
        }
        "no-positive-invention" => {
            report.snapshot.observations.is_empty()
                || report
                    .snapshot
                    .fact_writes
                    .iter()
                    .all(|result| !result.fact.evidence.is_empty())
        }
        other => panic!("{scenario}: unsupported fact expectation {other}"),
    };
    assert!(fact, "{scenario}: fact expectation {}", value("fact"));

    let event = match value("event") {
        "none" => {
            report.event_failures.is_empty()
                && !calls
                    .iter()
                    .any(|call| call == "event.terminal" || call == "event.attempt")
        }
        "failed-counted" => !report.event_failures.is_empty(),
        "terminal-attempted" => calls
            .iter()
            .any(|call| call == "event.terminal" || call == "event.attempt"),
        other => panic!("{scenario}: unsupported event expectation {other}"),
    };
    assert!(event, "{scenario}: event expectation {}", value("event"));

    let cleanup = &report.snapshot.cleanup;
    let has_cleanup_failure = cleanup.iter().any(|result| {
        matches!(
            result.status,
            CleanupStatus::Failed | CleanupStatus::TimedOut
        )
    });
    let cleanup_matches = match value("cleanup") {
        "none" => cleanup.is_empty(),
        "later-obligations-not-applied" => !calls.iter().any(|call| call == "launch.start"),
        "proxy-attempted" => calls
            .iter()
            .any(|call| call.starts_with("proxy.stop") || call.starts_with("proxy.cleanup")),
        "route-and-earlier-attempted" => calls.iter().any(|call| call == "proxy.cleanup"),
        "child-and-earlier-attempted" => calls.iter().any(|call| call == "launch.cleanup"),
        "earlier-resources-attempted" | "acquired-attempted" | "owned-only" => !cleanup.is_empty(),
        "all-acquired-attempted" | "all-attempted" | "already-attempted" => {
            !cleanup.is_empty() && calls.iter().any(|call| call == "proxy.cleanup")
        }
        "later-cleanup-attempted" => has_cleanup_failure && cleanup.len() > 1,
        other => panic!("{scenario}: unsupported cleanup expectation {other}"),
    };
    assert!(
        cleanup_matches,
        "{scenario}: cleanup expectation {}",
        value("cleanup")
    );

    let recovery = journal
        .map(JournalPrefix::recovery_plan)
        .unwrap_or_default();
    let has_state = |state| {
        journal.is_some_and(|journal| {
            journal
                .transitions
                .iter()
                .any(|transition| transition.state == state)
        })
    };
    let journal_matches = match value("journal") {
        "none" => journal.is_none(),
        "not-applied" => has_state(ResourceState::NotApplied),
        "failed" => has_state(ResourceState::Failed),
        "timed-out" => has_state(ResourceState::TimedOut),
        "none-or-complete" => journal.is_none() || recovery == RecoveryPlan::default(),
        "terminal-or-recoverable" | "complete-or-recoverable" => journal.is_some(),
        "failed-or-recoverable" | "retained-or-failed" => {
            has_state(ResourceState::Failed)
                || has_state(ResourceState::Retained)
                || !recovery.actions.is_empty()
                || !recovery.refusals.is_empty()
        }
        other => panic!("{scenario}: unsupported journal expectation {other}"),
    };
    assert!(
        journal_matches,
        "{scenario}: journal expectation {}",
        value("journal")
    );

    let recovery_matches = match value("recovery") {
        "none" => recovery.actions.is_empty() && recovery.refusals.is_empty(),
        "exact" | "pending-obligation-exact" => {
            !recovery.actions.is_empty() && recovery.refusals.is_empty()
        }
        "exact-or-none" => recovery.refusals.is_empty(),
        "exact-or-refused" => !recovery.actions.is_empty() || !recovery.refusals.is_empty(),
        "exact-trust-action" => {
            recovery
                .actions
                .iter()
                .any(|action| action.kind == ResourceKind::Trust)
                || recovery
                    .refusals
                    .iter()
                    .any(|refusal| refusal.resource_id == "trust-entry")
        }
        other => panic!("{scenario}: unsupported recovery expectation {other}"),
    };
    assert!(
        recovery_matches,
        "{scenario}: recovery expectation {}",
        value("recovery")
    );
}

fn assert_production_effect_side(
    scenario: &str,
    boundary: &str,
    side: &str,
    calls: &[String],
    journal: &JournalPrefix,
) {
    let transitions = journal
        .transitions
        .iter()
        .filter(|transition| transition.resource_id == boundary)
        .collect::<Vec<_>>();
    assert!(
        !transitions.is_empty(),
        "{scenario}: effect has no production journal row"
    );
    let applied = transitions.iter().any(|transition| {
        matches!(
            transition.state,
            ResourceState::Applied | ResourceState::Retained
        )
    });
    let invoked = match boundary {
        "proxy-listener" | "proxy-runtime" => calls.iter().any(|call| call == "proxy.start"),
        "trust-entry" => calls.iter().any(|call| call == "trust.acquire"),
        "route" => calls.iter().any(|call| call == "routing.apply"),
        "managed-child" => calls.iter().any(|call| call == "launch.start"),
        "capture" => calls.iter().any(|call| call == "capture.run"),
        "bundle-evidence" => calls.iter().any(|call| call.starts_with("artifact.")),
        other => panic!("{scenario}: unknown effect {other}"),
    };
    match side {
        "before" => {
            assert!(!invoked, "{scenario}: before-side effect was invoked");
            assert!(!applied, "{scenario}: before-side effect was applied");
        }
        "after" => {
            assert!(invoked, "{scenario}: after-side effect was never invoked");
            if boundary != "bundle-evidence" {
                assert!(applied, "{scenario}: after-side effect was never applied");
            }
        }
        other => panic!("{scenario}: unknown side {other}"),
    }
}
