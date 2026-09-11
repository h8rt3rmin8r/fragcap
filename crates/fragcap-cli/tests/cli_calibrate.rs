// SPDX-License-Identifier: Apache-2.0

// Integration coverage for the guided calibration front door.

mod common;

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use common::{run, run_with_authorization};
use fragcap::profile::FidelityTier;
use fragcap::targets::{
    resolved_client_launch, CalibrationWorkflowCheckpoint, CalibrationWorkflowPhase,
    CalibrationWorkflowState, CalibrationWorkflowUpdateOutcome, ClassificationSource,
    CompatibilityAddressFamily, CompatibilityEvidenceSource, CompatibilityFact,
    CompatibilityFactKey, CompatibilityLaunchCase, CompatibilityProtocol,
    CompatibilityRoutingStrategy, Store, TargetClassification, TargetEntry,
};

const STABLE_ID: i64 = 75_000;

struct FixedAuthorization {
    terminal: bool,
    response: Vec<u8>,
}

struct EchoAuthorization {
    calls: usize,
}

struct RecordingEchoAuthorization {
    plan_ids: Vec<String>,
}

struct RecordingDeclineAuthorization {
    plan_ids: Vec<String>,
}

struct AcceptThenDeclineAuthorization {
    calls: usize,
}

struct AcceptThenInterruptAuthorization {
    calls: usize,
}

struct SecondPlanTargetDriftAuthorization {
    calls: usize,
    local: PathBuf,
}

struct BoundedEchoAuthorization {
    calls: usize,
    accepted: usize,
}

struct DriftAuthorization;

struct AmbiguityAuthorization;

struct SteamClientDriftAuthorization;

struct SteamClientAmbiguityAuthorization;

enum TargetMutation {
    Change,
    Delete,
}

struct TargetMutationAuthorization {
    local: PathBuf,
    mutation: TargetMutation,
}

impl fragcap_cli::DeepCaptureAuthorizationInput for EchoAuthorization {
    fn is_terminal(&self) -> bool {
        false
    }

    fn read_response(&mut self, plan_id: &str, exact: bool) -> std::io::Result<Vec<u8>> {
        assert!(
            exact,
            "structured registration and session plans must be exact"
        );
        self.calls += 1;
        Ok(format!("{plan_id}\n").into_bytes())
    }
}

impl fragcap_cli::DeepCaptureAuthorizationInput for RecordingEchoAuthorization {
    fn is_terminal(&self) -> bool {
        false
    }

    fn read_response(&mut self, plan_id: &str, exact: bool) -> std::io::Result<Vec<u8>> {
        assert!(exact);
        self.plan_ids.push(plan_id.to_string());
        Ok(format!("{plan_id}\n").into_bytes())
    }
}

impl fragcap_cli::DeepCaptureAuthorizationInput for RecordingDeclineAuthorization {
    fn is_terminal(&self) -> bool {
        true
    }

    fn read_response(&mut self, plan_id: &str, exact: bool) -> std::io::Result<Vec<u8>> {
        assert!(!exact);
        self.plan_ids.push(plan_id.to_string());
        Ok(b"no\n".to_vec())
    }
}

impl fragcap_cli::DeepCaptureAuthorizationInput for AcceptThenDeclineAuthorization {
    fn is_terminal(&self) -> bool {
        true
    }

    fn read_response(&mut self, _plan_id: &str, exact: bool) -> std::io::Result<Vec<u8>> {
        assert!(!exact);
        self.calls += 1;
        Ok(if self.calls == 1 {
            b"yes\n".to_vec()
        } else {
            b"no\n".to_vec()
        })
    }
}

impl fragcap_cli::DeepCaptureAuthorizationInput for AcceptThenInterruptAuthorization {
    fn is_terminal(&self) -> bool {
        false
    }

    fn read_response(&mut self, plan_id: &str, exact: bool) -> std::io::Result<Vec<u8>> {
        assert!(exact);
        self.calls += 1;
        if self.calls == 1 {
            Ok(format!("{plan_id}\n").into_bytes())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "controlled authorization interrupt",
            ))
        }
    }
}

impl fragcap_cli::DeepCaptureAuthorizationInput for SecondPlanTargetDriftAuthorization {
    fn is_terminal(&self) -> bool {
        false
    }

    fn read_response(&mut self, plan_id: &str, exact: bool) -> std::io::Result<Vec<u8>> {
        assert!(exact);
        self.calls += 1;
        if self.calls == 2 {
            let mut store = Store::open(&self.local).expect("open target for second-plan drift");
            let mut target = store
                .target_by_stable_id(STABLE_ID)
                .expect("read target")
                .expect("target exists");
            target.name = "Changed During Sequence".to_string();
            assert!(store.update_target(&target).expect("drift target"));
        }
        Ok(format!("{plan_id}\n").into_bytes())
    }
}

impl fragcap_cli::DeepCaptureAuthorizationInput for BoundedEchoAuthorization {
    fn is_terminal(&self) -> bool {
        false
    }

    fn read_response(&mut self, plan_id: &str, exact: bool) -> std::io::Result<Vec<u8>> {
        assert!(exact);
        self.calls += 1;
        if self.calls <= self.accepted {
            Ok(format!("{plan_id}\n").into_bytes())
        } else {
            Ok(b"not-the-current-plan\n".to_vec())
        }
    }
}

impl fragcap_cli::DeepCaptureAuthorizationInput for DriftAuthorization {
    fn is_terminal(&self) -> bool {
        false
    }

    fn read_response(&mut self, plan_id: &str, exact: bool) -> std::io::Result<Vec<u8>> {
        assert!(exact);
        std::env::set_var("FRAGCAP_CONTROLLED_TARGET_REGISTRATION_DRIFT", "1");
        Ok(format!("{plan_id}\n").into_bytes())
    }
}

impl fragcap_cli::DeepCaptureAuthorizationInput for AmbiguityAuthorization {
    fn is_terminal(&self) -> bool {
        false
    }

    fn read_response(&mut self, plan_id: &str, exact: bool) -> std::io::Result<Vec<u8>> {
        assert!(exact);
        std::env::set_var("FRAGCAP_CONTROLLED_TARGET_REGISTRATION_AMBIGUOUS", "1");
        Ok(format!("{plan_id}\n").into_bytes())
    }
}

impl fragcap_cli::DeepCaptureAuthorizationInput for SteamClientDriftAuthorization {
    fn is_terminal(&self) -> bool {
        false
    }

    fn read_response(&mut self, plan_id: &str, exact: bool) -> std::io::Result<Vec<u8>> {
        assert!(exact);
        assert!(plan_id.starts_with("steam-client-setup-v1:"));
        std::env::set_var("FRAGCAP_CONTROLLED_TARGET_STEAM_CLIENT_DRIFT", "1");
        Ok(format!("{plan_id}\n").into_bytes())
    }
}

impl fragcap_cli::DeepCaptureAuthorizationInput for SteamClientAmbiguityAuthorization {
    fn is_terminal(&self) -> bool {
        false
    }

    fn read_response(&mut self, plan_id: &str, exact: bool) -> std::io::Result<Vec<u8>> {
        assert!(exact);
        assert!(plan_id.starts_with("steam-client-setup-v1:"));
        std::env::set_var("FRAGCAP_CONTROLLED_TARGET_STEAM_CLIENT_AMBIGUOUS", "1");
        Ok(format!("{plan_id}\n").into_bytes())
    }
}

impl fragcap_cli::DeepCaptureAuthorizationInput for TargetMutationAuthorization {
    fn is_terminal(&self) -> bool {
        false
    }

    fn read_response(&mut self, plan_id: &str, exact: bool) -> std::io::Result<Vec<u8>> {
        assert!(exact);
        assert!(plan_id.starts_with("steam-client-setup-v1:"));
        let mut store = Store::open(&self.local).expect("open target store during confirmation");
        let mut target = store
            .target_by_anchor("steam:75000")
            .expect("read target")
            .expect("target exists");
        match self.mutation {
            TargetMutation::Change => {
                target.name = "Changed Sample Target".to_string();
                assert!(store.update_target(&target).expect("change target"));
            }
            TargetMutation::Delete => {
                assert!(store
                    .delete_target(target.id.expect("stored row"))
                    .expect("delete target"));
            }
        }
        Ok(format!("{plan_id}\n").into_bytes())
    }
}

impl fragcap_cli::DeepCaptureAuthorizationInput for FixedAuthorization {
    fn is_terminal(&self) -> bool {
        self.terminal
    }

    fn read_response(&mut self, _plan_id: &str, _exact: bool) -> std::io::Result<Vec<u8>> {
        Ok(std::mem::take(&mut self.response))
    }
}

fn controlled_environment() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn powershell_words(command: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut characters = command.chars().peekable();
    let mut quoted = false;
    while let Some(character) = characters.next() {
        match character {
            '"' => quoted = !quoted,
            '`' => {
                let escaped = characters.next().expect("complete PowerShell escape");
                word.push(match escaped {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    value => value,
                });
            }
            value if value.is_whitespace() && !quoted => {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
            }
            value => word.push(value),
        }
    }
    assert!(
        !quoted,
        "generated command must close every quoted argument"
    );
    if !word.is_empty() {
        words.push(word);
    }
    words
}

fn assert_next_command_selects_store(events: &str, subcommand: &str, target_id: i64, local: &Path) {
    let command = events
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|event| event["event"] == "calibration.guidance")
        .filter_map(|event| event["next_command"].as_str().map(str::to_string))
        .next_back()
        .expect("one guided next command");
    let matches = fragcap_cli::command()
        .try_get_matches_from(powershell_words(&command))
        .expect("generated next command parses");
    let (actual_subcommand, arguments) = matches.subcommand().expect("generated subcommand");
    assert_eq!(actual_subcommand, subcommand);
    if subcommand == "calibrate" {
        let workflow_id = *arguments
            .get_one::<i64>("resume")
            .expect("calibration continuation selects one workflow");
        let store = Store::open(local).expect("continuation store");
        assert_eq!(
            store
                .calibration_workflow(workflow_id)
                .expect("workflow query")
                .expect("workflow")
                .target
                .stable_id,
            target_id
        );
    } else {
        assert_eq!(arguments.get_one::<i64>("id"), Some(&target_id));
    }
    assert_eq!(
        arguments
            .get_one::<PathBuf>("local_db")
            .map(PathBuf::as_path),
        Some(local)
    );
}

fn calibration_choices(events: &str) -> Vec<serde_json::Value> {
    events
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|event| event["event"] == "calibration.choice_required")
        .and_then(|event| event["choices"].as_array().cloned())
        .expect("one structured calibration choice set")
}

fn seed_target(local: &Path, with_current_routing: bool) -> i64 {
    let mut store = Store::open(local).expect("scratch local store");
    let entry = TargetEntry {
        id: None,
        stable_id: STABLE_ID,
        handle: "sample-target".to_string(),
        name: "Sample Target".to_string(),
        classification: TargetClassification::Game,
        classification_source: ClassificationSource::User,
        fidelity: FidelityTier::Authored,
        provenance: None,
        anchor: None,
        launch_entries: Some(resolved_client_launch("client.exe")),
        install_root: None,
        evidence: None,
        detection_scan: None,
        folder_name: None,
        executable_hint: Some("client.exe".to_string()),
    };
    let id = store.insert_target(&entry).expect("insert target");
    if with_current_routing {
        insert_current_routing_fact(&mut store, id);
    }
    id
}

fn seed_missing_steam_target(local: &Path) -> i64 {
    let mut store = Store::open(local).expect("scratch local store");
    let stable_id = fragcap::targets::identifier::anchored_id("steam:75000");
    let entry = TargetEntry {
        id: None,
        stable_id,
        handle: "sample_target".to_string(),
        name: "Sample Target".to_string(),
        classification: TargetClassification::Game,
        classification_source: ClassificationSource::Platform,
        fidelity: FidelityTier::Observed,
        provenance: Some(serde_json::json!({"source": "steam"})),
        anchor: Some("steam:75000".to_string()),
        launch_entries: None,
        install_root: Some("C:\\Games\\Sample Target".to_string()),
        evidence: None,
        detection_scan: None,
        folder_name: Some("Sample Target".to_string()),
        executable_hint: Some("client.exe".to_string()),
    };
    store.insert_target(&entry).expect("insert target");
    stable_id
}

fn insert_current_routing_fact(store: &mut Store, target_id: i64) {
    let mut fact = CompatibilityFact::new(
        target_id,
        CompatibilityFactKey::ProxyRouting,
        "reached-client",
        CompatibilityEvidenceSource::UserConfirmed,
    )
    .expect("compatibility fact");
    fact.launch_case = Some(CompatibilityLaunchCase::DirectExeCold);
    fact.fragcap_version = Some(env!("CARGO_PKG_VERSION").to_string());
    fact.proxy_backend = Some("fragcap-native".to_string());
    fact.proxy_backend_version = Some(env!("CARGO_PKG_VERSION").to_string());
    fact.routing_strategy = Some(CompatibilityRoutingStrategy::ChildEnvironment);
    fact.address_family = Some(CompatibilityAddressFamily::Ipv4);
    fact.protocol = Some(CompatibilityProtocol::NotApplicable);
    store
        .insert_compatibility_fact(&fact)
        .expect("insert compatibility fact");
}

fn insert_protocol_fact(store: &mut Store, row_id: i64, protocol: CompatibilityProtocol) {
    let mut fact = CompatibilityFact::new(
        row_id,
        CompatibilityFactKey::Inspectability,
        "full",
        CompatibilityEvidenceSource::UserConfirmed,
    )
    .expect("compatibility fact");
    fact.launch_case = Some(CompatibilityLaunchCase::DirectExeCold);
    fact.fragcap_version = Some(env!("CARGO_PKG_VERSION").to_string());
    fact.proxy_backend = Some("fragcap-native".to_string());
    fact.proxy_backend_version = Some(env!("CARGO_PKG_VERSION").to_string());
    fact.routing_strategy = Some(CompatibilityRoutingStrategy::ChildEnvironment);
    fact.address_family = Some(CompatibilityAddressFamily::Ipv4);
    fact.protocol = Some(protocol);
    store
        .insert_compatibility_fact(&fact)
        .expect("insert compatibility fact");
}

#[cfg(windows)]
fn seed_topology(
    local: &Path,
    stable_id: i64,
    handle: &str,
    anchor: Option<&str>,
    launch_entries: serde_json::Value,
    launch_case: CompatibilityLaunchCase,
) -> i64 {
    let mut store = Store::open(local).expect("scratch local store");
    let entry = TargetEntry {
        id: None,
        stable_id,
        handle: handle.to_string(),
        name: handle.to_string(),
        classification: TargetClassification::Game,
        classification_source: ClassificationSource::User,
        fidelity: FidelityTier::Authored,
        provenance: None,
        anchor: anchor.map(str::to_string),
        launch_entries: Some(launch_entries),
        install_root: Some("C:\\Games\\Fixture".to_string()),
        evidence: None,
        detection_scan: None,
        folder_name: None,
        executable_hint: None,
    };
    let row_id = store.insert_target(&entry).expect("insert target");
    insert_routing_fact(
        &mut store,
        row_id,
        launch_case,
        "reached-client",
        false,
        CompatibilityAddressFamily::Ipv4,
        true,
    );
    row_id
}

#[cfg(windows)]
fn insert_routing_fact(
    store: &mut Store,
    row_id: i64,
    launch_case: CompatibilityLaunchCase,
    value: &str,
    stale: bool,
    family: CompatibilityAddressFamily,
    complete: bool,
) {
    let mut fact = CompatibilityFact::new(
        row_id,
        CompatibilityFactKey::ProxyRouting,
        value,
        CompatibilityEvidenceSource::UserConfirmed,
    )
    .expect("compatibility fact");
    if complete {
        fact.launch_case = Some(launch_case);
        fact.fragcap_version = Some(env!("CARGO_PKG_VERSION").to_string());
        fact.proxy_backend = Some("fragcap-native".to_string());
        fact.proxy_backend_version = Some(env!("CARGO_PKG_VERSION").to_string());
        fact.routing_strategy = Some(CompatibilityRoutingStrategy::ChildEnvironment);
        fact.address_family = Some(family);
        fact.protocol = Some(CompatibilityProtocol::NotApplicable);
    }
    fact.stale = stale;
    store
        .insert_compatibility_fact(&fact)
        .expect("insert compatibility fact");
}

#[test]
fn calibrate_is_listed_and_documents_its_bounded_contract() {
    let (code, root, err) = run(&["--help"]);
    assert_eq!(code, 0, "stderr:\n{err}");
    assert!(root.contains("calibrate"), "root help:\n{root}");

    let (code, help, err) = run(&["calibrate", "--help"]);
    assert_eq!(code, 0, "stderr:\n{err}");
    for required in [
        "SELECTOR",
        "--target",
        "--id",
        "--resume",
        "--pause-for",
        "--candidate",
        "--launch-case",
        "--routing-strategy",
        "--proxy-family",
        "--bundle",
        "--duration",
        "--wait",
        "--max-packets",
        "--max-bytes",
        "--interface",
        "--no-payload",
        "--authorize-stdin",
        "--restart-warm",
        "--protocol",
    ] {
        assert!(
            help.contains(required),
            "help must contain {required}:\n{help}"
        );
    }
    for excluded in [
        "--calibration-protocol",
        "--proxy-bypass",
        "--trust-ca",
        "--yes",
        "--controlled-target",
    ] {
        assert!(!help.contains(excluded), "help leaked {excluded}:\n{help}");
    }
}

#[test]
fn resume_is_an_explicit_mutually_exclusive_workflow_selector() {
    for args in [
        vec!["calibrate", "sample-target", "--resume", "1"],
        vec!["calibrate", "--id", "75000", "--resume", "1"],
        vec!["calibrate", "--resume", "1", "--protocol", "https"],
        vec![
            "calibrate",
            "--resume",
            "1",
            "--candidate",
            "candidate-v1:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ],
        vec![
            "calibrate",
            "--resume",
            "1",
            "--launch-case",
            "direct-exe-cold",
        ],
        vec![
            "calibrate",
            "--resume",
            "1",
            "--routing-strategy",
            "child-environment",
        ],
        vec!["calibrate", "--resume", "1", "--proxy-family", "ipv6"],
    ] {
        let (code, _out, err) = run(&args);
        assert_eq!(code, 2, "args={args:?} stderr={err}");
    }
    let (code, _out, err) = run(&["calibrate", "--pause-for", "login"]);
    assert_eq!(code, 2);
    assert!(err.contains("--resume"), "stderr={err}");
}

#[test]
fn resume_refuses_a_missing_workflow_without_effects() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let (code, _out, err) = run(&[
        "calibrate",
        "--resume",
        "404",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 2, "stderr={err}");
    assert!(
        err.contains("workflow 404 does not exist in the selected local store"),
        "stderr={err}"
    );
    assert!(!err.contains("authorization_plan"), "stderr={err}");
}

#[test]
fn resume_requires_a_positive_workflow_identifier() {
    let (code, _out, err) = run(&["calibrate", "--resume", "0"]);
    assert_eq!(code, 2, "stderr={err}");
    assert!(
        err.contains("workflow identifier must be positive"),
        "stderr={err}"
    );

    let (code, _out, err) = run(&["calibrate", "--resume", "-1"]);
    assert_eq!(code, 2, "stderr={err}");
}

#[test]
fn completed_workflow_is_durable_and_resume_is_a_no_effect_status_check() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    seed_target(&local, true);
    let (code, _out, events) = run(&[
        "--json",
        "calibrate",
        "--id",
        "75000",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "events:\n{events}");
    let guidance: serde_json::Value = events
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .find(|event: &serde_json::Value| event["event"] == "calibration.guidance")
        .unwrap();
    let workflow_id = guidance["workflow_id"].as_i64().expect("workflow id");
    assert_eq!(guidance["workflow_state"], "completed");
    assert_eq!(guidance["workflow_revision"], 2);
    assert!(guidance["resume_command"]
        .as_str()
        .unwrap()
        .contains(&format!("--resume {workflow_id}")));

    let (code, _out, resumed) = run(&[
        "--json",
        "calibrate",
        "--resume",
        &workflow_id.to_string(),
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "events:\n{resumed}");
    assert!(resumed.contains("\"status\":\"workflow-complete\""));
    assert!(!resumed.contains("deep_capture.authorization_plan"));
}

#[test]
fn resume_reassesses_current_facts_and_suppresses_completed_work() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    seed_target(&local, true);
    let mut store = Store::open(&local).unwrap();
    let target = store.target_by_stable_id(STABLE_ID).unwrap().unwrap();
    let workflow = store
        .create_calibration_workflow(&target, &[CompatibilityProtocol::Https], 100)
        .unwrap();
    insert_protocol_fact(
        &mut store,
        target.id.expect("stored target row"),
        CompatibilityProtocol::Https,
    );
    drop(store);

    let (code, _out, events) = run(&[
        "--json",
        "calibrate",
        "--resume",
        &workflow.id.to_string(),
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "events:\n{events}");
    assert!(events.contains("\"workflow_state\":\"completed\""));
    assert!(events.contains("\"reason\":\"current-protocol-evidence\""));
    assert!(!events.contains("deep_capture.authorization_plan"));
    assert!(!events.contains("deep_capture.session"));
}

#[test]
fn explicit_operator_pauses_are_durable_and_effect_free() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    seed_target(&local, true);
    let mut store = Store::open(&local).unwrap();
    let target = store.target_by_stable_id(STABLE_ID).unwrap().unwrap();
    let workflow_id = store
        .create_calibration_workflow(&target, &[CompatibilityProtocol::Https], 100)
        .unwrap()
        .id;
    drop(store);

    for reason in [
        "login",
        "eula",
        "update",
        "anti-cheat",
        "gameplay",
        "shutdown",
        "interrupted",
    ] {
        let (code, _out, events) = run(&[
            "--json",
            "calibrate",
            "--resume",
            &workflow_id.to_string(),
            "--pause-for",
            reason,
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
        ]);
        assert_eq!(code, 0, "reason={reason} events:\n{events}");
        assert!(events.contains(&format!("\"pause_reason\":\"{reason}\"")));
        assert!(events.contains("\"workflow_state\":\"paused\""));
        assert!(!events.contains("deep_capture.authorization_plan"));
        assert!(!events.contains("deep_capture.session"));
    }
}

#[test]
fn a_completed_workflow_cannot_be_changed_back_to_paused() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    seed_target(&local, true);
    let (_, _, initial) = run(&[
        "--json",
        "calibrate",
        "--id",
        "75000",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    let workflow_id = initial
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|event| event["event"] == "calibration.guidance")
        .and_then(|event| event["workflow_id"].as_i64())
        .unwrap();

    let (code, _out, err) = run(&[
        "calibrate",
        "--resume",
        &workflow_id.to_string(),
        "--pause-for",
        "login",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 2, "stderr={err}");
    assert!(
        err.contains("completed calibration workflow"),
        "stderr={err}"
    );
    let store = Store::open(&local).unwrap();
    assert_eq!(
        store
            .calibration_workflow(workflow_id)
            .unwrap()
            .unwrap()
            .state,
        CalibrationWorkflowState::Completed
    );
}

#[test]
fn an_in_flight_checkpoint_resumes_as_an_effect_free_interruption() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    seed_target(&local, true);
    let mut store = Store::open(&local).unwrap();
    let target = store.target_by_stable_id(STABLE_ID).unwrap().unwrap();
    let workflow = store
        .create_calibration_workflow(&target, &[CompatibilityProtocol::Https], 100)
        .unwrap();
    let checkpoint = CalibrationWorkflowCheckpoint {
        requested_protocols: workflow.requested_protocols.clone(),
        observed_protocols: Vec::new(),
        completed_protocols: Vec::new(),
        remaining_protocols: workflow.remaining_protocols.clone(),
        attempted_case_keys: vec!["case-1".to_string()],
        attempt_ordinal: 1,
        attempt_phase: Some(CalibrationWorkflowPhase::Tls),
        attempt_protocol: Some(CompatibilityProtocol::Https),
        attempt_key: Some("case-1".to_string()),
        state: CalibrationWorkflowState::InFlight,
        pause_reason: None,
    };
    assert!(matches!(
        store
            .update_calibration_workflow(workflow.id, workflow.revision, &checkpoint, 101)
            .unwrap(),
        CalibrationWorkflowUpdateOutcome::Applied(_)
    ));
    drop(store);

    let (code, _out, events) = run(&[
        "--json",
        "calibrate",
        "--resume",
        &workflow.id.to_string(),
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "events:\n{events}");
    assert!(events.contains("\"workflow_state\":\"paused\""));
    assert!(events.contains("\"pause_reason\":\"interrupted\""));
    assert!(events.contains("\"reason\":\"prior-process-interrupted\""));
    assert!(!events.contains("deep_capture.authorization_plan"));
}

#[test]
fn changed_target_authority_refuses_a_durable_workflow() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    seed_target(&local, true);
    let mut store = Store::open(&local).unwrap();
    let mut target = store.target_by_stable_id(STABLE_ID).unwrap().unwrap();
    let workflow = store
        .create_calibration_workflow(&target, &[CompatibilityProtocol::Https], 100)
        .unwrap();
    target.name = "Changed Target".to_string();
    assert!(store.update_target(&target).unwrap());
    drop(store);

    let (code, _out, err) = run(&[
        "calibrate",
        "--resume",
        &workflow.id.to_string(),
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 2, "stderr={err}");
    assert!(err.contains("target authority changed"), "stderr={err}");
    let store = Store::open(&local).unwrap();
    assert_eq!(
        store
            .calibration_workflow(workflow.id)
            .unwrap()
            .unwrap()
            .state,
        CalibrationWorkflowState::Refused
    );
}

#[test]
fn protocol_candidates_are_repeatable_and_routing_is_refused_before_effects() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let bundle = dir.path().join("must-not-exist");
    seed_target(&local, true);

    let (code, _out, events) = run(&[
        "--json",
        "calibrate",
        "--id",
        "75000",
        "--protocol",
        "https",
        "--protocol",
        "http1",
        "--protocol",
        "https",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "authorization is required for the selected attempt: {events}"
    );
    assert!(
        events.contains("\"action\":\"run-protocol\""),
        "events:\n{events}"
    );
    assert!(events.contains("\"requested_protocols\":[\"http1\",\"https\"]"));
    assert!(!bundle.exists());

    let (code, _out, err) = run(&[
        "calibrate",
        "--id",
        "75000",
        "--protocol",
        "routing",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(err.contains("invalid value 'routing'"), "refusal: {err}");
    assert!(!bundle.exists());
}

#[test]
fn current_routing_without_candidates_reports_unknown_coverage() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    seed_target(&local, true);
    let (code, _out, events) = run(&[
        "--json",
        "calibrate",
        "--id",
        "75000",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "events:\n{events}");
    let guidance: serde_json::Value = serde_json::from_str(events.lines().next().unwrap()).unwrap();
    assert_eq!(guidance["status"], "ready");
    assert_eq!(guidance["reason"], "current-routing-evidence");
    assert_eq!(guidance["requested_protocols"], serde_json::json!([]));
    assert_eq!(guidance["observed_protocols"], serde_json::json!([]));
    assert_eq!(guidance["completed_protocols"], serde_json::json!([]));
    assert_eq!(guidance["remaining_protocols"], serde_json::json!([]));
    assert_eq!(guidance["attempt"], serde_json::Value::Null);
    assert_eq!(guidance["maximum_attempts"], serde_json::Value::Null);
    assert_eq!(guidance["phase"], serde_json::Value::Null);
    assert_eq!(guidance["protocol"], serde_json::Value::Null);
    assert_next_command_selects_store(&events, "deep-capture", STABLE_ID, &local);
}

#[test]
fn current_positive_candidate_reports_requested_coverage_without_effects() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let bundle = dir.path().join("must-not-exist");
    let row_id = seed_target(&local, true);
    let mut store = Store::open(&local).unwrap();
    insert_protocol_fact(&mut store, row_id, CompatibilityProtocol::Https);
    drop(store);

    let (code, _out, events) = run(&[
        "--json",
        "calibrate",
        "--id",
        "75000",
        "--protocol",
        "https",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "events:\n{events}");
    let guidance: serde_json::Value = serde_json::from_str(events.lines().next().unwrap()).unwrap();
    assert_eq!(guidance["status"], "requested-coverage-complete");
    assert_eq!(
        guidance["completed_protocols"],
        serde_json::json!(["https"])
    );
    assert_eq!(guidance["remaining_protocols"], serde_json::json!([]));
    assert!(!events.contains("deep_capture.authorization_plan"));
    assert!(!bundle.exists());
}

#[test]
fn missing_routing_runs_all_observed_protocols_before_handoff() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let bundle = dir.path().join("reachability-with-candidates");
    seed_target(&local, false);
    std::env::set_var(
        "FRAGCAP_CONTROLLED_TARGET_EXECUTABLE",
        env!("CARGO_BIN_EXE_fragcap"),
    );

    let (code, _out, events) = run(&[
        "--json",
        "calibrate",
        "--id",
        "75000",
        "--authorize-stdin",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
        "--duration",
        "5s",
        "--wait",
        "7s",
    ]);
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE");

    assert_eq!(code, 0, "events:\n{events}");
    assert_eq!(events.matches("deep_capture.authorization_plan").count(), 3);
    let guidance: serde_json::Value = events
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .rfind(|event: &serde_json::Value| event["event"] == "calibration.guidance")
        .unwrap();
    assert_eq!(
        guidance["observed_protocols"],
        serde_json::json!(["http1", "https"])
    );
    assert_eq!(
        guidance["completed_protocols"],
        serde_json::json!(["http1", "https"])
    );
    assert_eq!(guidance["remaining_protocols"], serde_json::json!([]));
    assert_eq!(guidance["status"], "observed-coverage-complete");
    assert_next_command_selects_store(&events, "deep-capture", STABLE_ID, &local);
}

#[test]
fn current_routing_runs_requested_and_newly_observed_protocol_attempts() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let bundle = dir.path().join("protocol");
    let row_id = seed_target(&local, true);
    std::env::set_var(
        "FRAGCAP_CONTROLLED_TARGET_EXECUTABLE",
        env!("CARGO_BIN_EXE_fragcap"),
    );

    let (code, _out, events) = run(&[
        "--json",
        "calibrate",
        "--id",
        "75000",
        "--protocol",
        "https",
        "--authorize-stdin",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
        "--duration",
        "5s",
        "--wait",
        "7s",
    ]);
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE");

    assert_eq!(code, 0, "events:\n{events}");
    assert_eq!(events.matches("deep_capture.authorization_plan").count(), 2);
    assert!(events.contains("\"action\":\"run-protocol\""));
    assert!(events.contains("\"phase\":\"tls\""));
    assert!(events.contains("\"protocol\":\"https\""));
    let guidance: serde_json::Value = events
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .rfind(|event: &serde_json::Value| event["event"] == "calibration.guidance")
        .unwrap();
    assert_eq!(
        guidance["status"], "requested-coverage-complete",
        "events:\n{events}"
    );
    assert_eq!(
        guidance["requested_protocols"],
        serde_json::json!(["https"])
    );
    assert!(guidance["observed_protocols"]
        .as_array()
        .unwrap()
        .iter()
        .any(|protocol| protocol == "https"));
    assert!(guidance["completed_protocols"]
        .as_array()
        .unwrap()
        .iter()
        .any(|protocol| protocol == "https"));

    let store = Store::open(&local).unwrap();
    let facts = store.compatibility_facts_for_target(row_id).unwrap();
    assert!(facts.iter().any(|fact| {
        fact.key == CompatibilityFactKey::Inspectability
            && fact.value == "full"
            && fact.protocol == Some(CompatibilityProtocol::Https)
    }));
}

#[test]
fn partial_protocol_attempt_reassesses_facts_but_never_claims_completion() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let bundle = dir.path().join("partial-protocol");
    seed_target(&local, true);
    std::env::set_var(
        "FRAGCAP_CONTROLLED_TARGET_EXECUTABLE",
        env!("CARGO_BIN_EXE_fragcap"),
    );
    std::env::set_var("FRAGCAP_CONTROLLED_TARGET_FAIL_AFTER", "2");

    let (code, _out, events) = run(&[
        "--json",
        "calibrate",
        "--id",
        "75000",
        "--protocol",
        "https",
        "--authorize-stdin",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
        "--duration",
        "5s",
        "--wait",
        "7s",
    ]);
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_FAIL_AFTER");
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE");

    assert_eq!(code, 1, "events:\n{events}");
    let guidance: serde_json::Value = events
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .rfind(|event: &serde_json::Value| event["event"] == "calibration.guidance")
        .unwrap();
    assert_eq!(guidance["status"], "failed");
    assert_eq!(guidance["reason"], "delegated-session-terminal-failure");
    assert_eq!(guidance["next_command"], serde_json::Value::Null);
    assert_ne!(guidance["status"], "completed");
}

#[test]
fn calibrate_reports_a_clean_stored_and_discovered_miss() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    Store::open(&local).expect("scratch local store");
    let (code, _out, err) = run(&[
        "calibrate",
        "missing-target",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(
        err.contains("no stored or discovered target exactly matches"),
        "refusal: {err}"
    );
    assert!(err.contains("considered 1, produced 1"), "refusal: {err}");
}

#[test]
fn calibrate_preserves_shared_ambiguity_diagnostics() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let mut store = Store::open(&local).unwrap();
    for (stable_id, handle) in [(76_001, "first"), (76_002, "second")] {
        let target = TargetEntry {
            id: None,
            stable_id,
            handle: handle.to_string(),
            name: "Same Display Name".to_string(),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::User,
            fidelity: FidelityTier::Authored,
            provenance: None,
            anchor: None,
            launch_entries: Some(resolved_client_launch("absent.exe")),
            install_root: None,
            evidence: None,
            detection_scan: None,
            folder_name: None,
            executable_hint: None,
        };
        store.insert_target(&target).unwrap();
    }
    drop(store);
    let (code, _out, err) = run(&[
        "calibrate",
        "Same Display Name",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(err.contains("selector is ambiguous"), "diagnostic: {err}");
    assert!(err.contains("first") && err.contains("second"));
}

#[test]
fn calibrate_target_inputs_are_mutually_exclusive() {
    for args in [
        vec!["calibrate", "sample-target", "--id", "75000"],
        vec!["calibrate", "--target", "sample-target", "--id", "75000"],
        vec!["calibrate", "sample-target", "--target", "sample-target"],
    ] {
        let (code, _out, err) = run(&args);
        assert_eq!(code, 2, "stderr:\n{err}");
        assert!(err.contains("cannot be used with"), "stderr:\n{err}");
    }
}

#[test]
fn unregistered_target_decline_and_invalid_exact_input_write_no_target_row() {
    let _environment = controlled_environment().lock().unwrap();
    for (selector, json, response, expected_status, expected_code) in [
        ("75000", false, b"no\n".to_vec(), "declined", 0),
        ("75000", true, b"wrong-plan\n".to_vec(), "invalid", 2),
        ("sample target", true, Vec::new(), "closed", 0),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let local = dir.path().join("local.db");
        let mut authorization = FixedAuthorization {
            terminal: !json,
            response,
        };
        let mut args = vec!["calibrate", selector, "--controlled-target", "--local-db"];
        let local_text = local.to_string_lossy().into_owned();
        args.push(&local_text);
        if json {
            args.splice(0..0, ["--json"]);
            args.push("--authorize-stdin");
        }
        let (code, _out, events) = run_with_authorization(&args, &mut authorization);
        assert_eq!(code, expected_code, "diagnostics:\n{events}");
        assert!(
            events.contains("Target registration plan")
                || events.contains("calibration.registration_plan")
        );
        assert!(events.contains(expected_status), "diagnostics:\n{events}");
        assert!(Store::open(&local).unwrap().targets().unwrap().is_empty());
    }
}

#[test]
fn confirmed_discovered_target_registers_then_authors_the_steam_client_once() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let bundle = dir.path().join("must-not-exist");
    let mut authorization = BoundedEchoAuthorization {
        calls: 0,
        accepted: 2,
    };
    let (code, out, events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "75000",
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
            "--bundle",
            bundle.to_str().unwrap(),
            "--duration",
            "5s",
            "--wait",
            "7s",
        ],
        &mut authorization,
    );
    assert_eq!(code, 2, "events:\n{events}");
    assert!(out.is_empty());
    assert_eq!(
        authorization.calls, 3,
        "registration and client setup are authorized independently from the refused session"
    );
    assert_eq!(events.matches("calibration.registration_plan").count(), 1);
    assert_eq!(events.matches("calibration.steam_client_plan").count(), 1);
    assert_eq!(events.matches("deep_capture.authorization_plan").count(), 1);
    assert!(events.contains("\"discovery_considered\":1"));
    assert!(events.contains("\"discovery_produced\":1"));
    assert!(events.contains("\"discovery_access_error\":0"));
    assert!(events.contains("\"discovery_warning_count\":0"));
    assert!(events.contains("\"status\":\"registered\""));
    assert!(events.contains("\"status\":\"applied\""));
    assert!(events.contains("\"continued\":true"));
    assert!(events.contains("steam-protocol-cold"));
    assert!(!bundle.exists());

    let store = Store::open(&local).unwrap();
    let target = store.target_by_anchor("steam:75000").unwrap().unwrap();
    assert_eq!(store.targets().unwrap().len(), 1);
    assert_eq!(
        target.launch_entries,
        Some(resolved_client_launch("client.exe"))
    );
    assert_eq!(target.fidelity, FidelityTier::Authored);
    assert!(store
        .compatibility_facts_for_target(target.id.unwrap())
        .unwrap()
        .is_empty());
    drop(store);

    let mut repeated_authorization = BoundedEchoAuthorization {
        calls: 0,
        accepted: 1,
    };
    let (repeated_code, _out, repeated_events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "75000",
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
        ],
        &mut repeated_authorization,
    );
    assert_eq!(repeated_code, 2, "events:\n{repeated_events}");
    assert_eq!(repeated_authorization.calls, 2);
    assert!(repeated_events.contains("\"status\":\"already-present\""));
    assert!(!repeated_events.contains("calibration.steam_client_plan"));
    assert_eq!(Store::open(&local).unwrap().targets().unwrap().len(), 1);
}

#[test]
fn steam_client_decline_and_invalid_structured_input_preserve_the_target() {
    let _environment = controlled_environment().lock().unwrap();
    for (json, response, expected_status, expected_code) in [
        (false, b"no\n".to_vec(), "declined", 0),
        (true, b"wrong-plan\n".to_vec(), "invalid", 2),
        (true, Vec::new(), "closed", 0),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let local = dir.path().join("local.db");
        let stable_id = seed_missing_steam_target(&local);
        let id = stable_id.to_string();
        let mut authorization = FixedAuthorization {
            terminal: !json,
            response,
        };
        let mut args = vec![
            "calibrate",
            "--id",
            &id,
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
        ];
        if json {
            args.splice(0..0, ["--json"]);
            args.push("--authorize-stdin");
        }
        let (code, _out, events) = run_with_authorization(&args, &mut authorization);
        assert_eq!(code, expected_code, "events:\n{events}");
        assert!(
            events.contains("calibration.steam_client_plan")
                || events.contains("Steam client setup plan")
        );
        assert!(events.contains(expected_status), "events:\n{events}");
        let target = Store::open(&local)
            .unwrap()
            .target_by_stable_id(stable_id)
            .unwrap()
            .unwrap();
        assert_eq!(target.launch_entries, None);
        assert_eq!(target.fidelity, FidelityTier::Observed);
    }
}

#[test]
fn every_present_launch_value_bypasses_steam_client_setup_unchanged() {
    let _environment = controlled_environment().lock().unwrap();
    for launch_entries in [
        serde_json::json!({"observed_exe": "launcher.exe", "socket_holder": "unresolved"}),
        serde_json::json!([]),
        serde_json::json!("historical-malformed-value"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let local = dir.path().join("local.db");
        let stable_id = seed_missing_steam_target(&local);
        let mut store = Store::open(&local).unwrap();
        let mut target = store.target_by_stable_id(stable_id).unwrap().unwrap();
        target.launch_entries = Some(launch_entries.clone());
        assert!(store.update_target(&target).unwrap());
        drop(store);

        let id = stable_id.to_string();
        let (code, _out, events) = run(&[
            "--json",
            "calibrate",
            "--id",
            &id,
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
        ]);
        assert_eq!(code, 2, "events:\n{events}");
        assert!(!events.contains("calibration.steam_client_plan"));
        let after = Store::open(&local)
            .unwrap()
            .target_by_stable_id(stable_id)
            .unwrap()
            .unwrap();
        assert_eq!(after.launch_entries, Some(launch_entries));
    }
}

#[test]
fn steam_client_setup_refuses_discovery_drift_before_mutation() {
    let _environment = controlled_environment().lock().unwrap();
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_STEAM_CLIENT_DRIFT");
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let stable_id = seed_missing_steam_target(&local);
    let id = stable_id.to_string();
    let mut authorization = SteamClientDriftAuthorization;
    let (code, _out, events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "--id",
            &id,
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
        ],
        &mut authorization,
    );
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_STEAM_CLIENT_DRIFT");
    assert_eq!(code, 2, "events:\n{events}");
    assert!(events.contains("steam-client-candidate-not-reproduced"));
    assert_eq!(
        Store::open(&local)
            .unwrap()
            .target_by_stable_id(stable_id)
            .unwrap()
            .unwrap()
            .launch_entries,
        None
    );
}

#[test]
fn steam_client_setup_refuses_ambiguous_candidate_reproduction() {
    let _environment = controlled_environment().lock().unwrap();
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_STEAM_CLIENT_AMBIGUOUS");
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let stable_id = seed_missing_steam_target(&local);
    let id = stable_id.to_string();
    let mut authorization = SteamClientAmbiguityAuthorization;
    let (code, _out, events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "--id",
            &id,
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
        ],
        &mut authorization,
    );
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_STEAM_CLIENT_AMBIGUOUS");
    assert_eq!(code, 2, "events:\n{events}");
    assert!(events.contains("steam-client-plan-changed-after-confirmation"));
    assert_eq!(
        Store::open(&local)
            .unwrap()
            .target_by_stable_id(stable_id)
            .unwrap()
            .unwrap()
            .launch_entries,
        None
    );
}

#[test]
fn steam_client_setup_reports_initial_discovery_ambiguity() {
    let _environment = controlled_environment().lock().unwrap();
    std::env::set_var("FRAGCAP_CONTROLLED_TARGET_STEAM_CLIENT_AMBIGUOUS", "1");
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let stable_id = seed_missing_steam_target(&local);
    let id = stable_id.to_string();
    let (code, _out, events) = run(&[
        "--json",
        "calibrate",
        "--id",
        &id,
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 2, "events:\n{events}");
    let choices = calibration_choices(&events);
    assert_eq!(choices.len(), 2);
    assert!(events.contains("\"scope\":\"steam-client\""));
    assert!(events.contains("\"process_control\":\"none\""));
    assert!(!events.contains("calibration.steam_client_plan"));

    let (human_code, human_out, human) = run(&[
        "calibrate",
        "--id",
        &id,
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(human_code, 2, "stderr:\n{human}");
    assert!(human_out.is_empty());
    assert!(human.contains("client.exe"), "stderr:\n{human}");
    assert!(human.contains("other-client.exe"), "stderr:\n{human}");
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_STEAM_CLIENT_AMBIGUOUS");

    assert_eq!(
        Store::open(&local)
            .unwrap()
            .target_by_stable_id(stable_id)
            .unwrap()
            .unwrap()
            .launch_entries,
        None
    );
}

#[test]
fn explicit_candidate_selects_one_ambiguous_discovered_target() {
    let _environment = controlled_environment().lock().unwrap();
    std::env::set_var("FRAGCAP_CONTROLLED_TARGET_REGISTRATION_AMBIGUOUS", "1");
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let base = [
        "--json",
        "calibrate",
        "Sample Target",
        "--authorize-stdin",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ];
    let (code, _out, events) = run(&base);
    assert_eq!(code, 2, "events:\n{events}");
    let choices = calibration_choices(&events);
    assert_eq!(choices.len(), 2);
    let selected = choices
        .iter()
        .find(|choice| choice["identity"] == "steam:75000")
        .and_then(|choice| choice["id"].as_str())
        .unwrap()
        .to_string();
    assert!(Store::open(&local).unwrap().targets().unwrap().is_empty());

    let mut authorization = BoundedEchoAuthorization {
        calls: 0,
        accepted: 2,
    };
    let (code, _out, selected_events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "Sample Target",
            "--candidate",
            &selected,
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
        ],
        &mut authorization,
    );
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_REGISTRATION_AMBIGUOUS");
    assert_eq!(code, 2, "events:\n{selected_events}");
    assert!(selected_events.contains("\"status\":\"registered\""));
    assert_eq!(Store::open(&local).unwrap().targets().unwrap().len(), 1);
}

#[test]
fn explicit_candidate_selects_one_ambiguous_steam_client() {
    let _environment = controlled_environment().lock().unwrap();
    std::env::set_var("FRAGCAP_CONTROLLED_TARGET_STEAM_CLIENT_AMBIGUOUS", "1");
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let stable_id = seed_missing_steam_target(&local);
    let id = stable_id.to_string();
    let (code, _out, events) = run(&[
        "--json",
        "calibrate",
        "--id",
        &id,
        "--authorize-stdin",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 2, "events:\n{events}");
    let choices = calibration_choices(&events);
    let selected = choices
        .iter()
        .find(|choice| choice["executable_hint"] == "client.exe")
        .and_then(|choice| choice["id"].as_str())
        .unwrap()
        .to_string();

    let mut authorization = BoundedEchoAuthorization {
        calls: 0,
        accepted: 1,
    };
    let (code, _out, selected_events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "--id",
            &id,
            "--candidate",
            &selected,
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
        ],
        &mut authorization,
    );
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_STEAM_CLIENT_AMBIGUOUS");
    assert_eq!(code, 2, "events:\n{selected_events}");
    assert!(selected_events.contains("\"status\":\"applied\""));
    let target = Store::open(&local)
        .unwrap()
        .target_by_stable_id(stable_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        target.launch_entries,
        Some(resolved_client_launch("client.exe"))
    );
}

#[test]
fn explicit_candidate_selects_one_ambiguous_stored_client() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let mut store = Store::open(&local).unwrap();
    let target = TargetEntry {
        id: None,
        stable_id: 85_001,
        handle: "ambiguous-stored-client".to_string(),
        name: "Ambiguous Stored Client".to_string(),
        classification: TargetClassification::Game,
        classification_source: ClassificationSource::User,
        fidelity: FidelityTier::Observed,
        provenance: None,
        anchor: None,
        launch_entries: Some(serde_json::json!([
            {"executable":"shared-client.exe","arguments":"--first","role":"client"},
            {"executable":"shared-client.exe","arguments":"--second","role":"client"}
        ])),
        install_root: Some("C:\\Games\\Ambiguous".to_string()),
        evidence: None,
        detection_scan: None,
        folder_name: None,
        executable_hint: None,
    };
    store.insert_target(&target).unwrap();
    drop(store);

    let (code, _out, choices) = run(&[
        "--json",
        "calibrate",
        "--id",
        "85001",
        "--authorize-stdin",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 2, "events:\n{choices}");
    assert!(choices.contains("\"scope\":\"stored-client\""));
    assert!(!choices.contains("calibration.stored_client_plan"));
    let selected = calibration_choices(&choices)
        .into_iter()
        .find(|choice| {
            choice["identity"]
                .as_str()
                .and_then(|identity| serde_json::from_str::<serde_json::Value>(identity).ok())
                .is_some_and(|identity| identity["arguments"] == "--second")
        })
        .and_then(|choice| choice["id"].as_str().map(str::to_string))
        .unwrap();

    let mut authorization = BoundedEchoAuthorization {
        calls: 0,
        accepted: 1,
    };
    let (code, _out, events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "--id",
            "85001",
            "--candidate",
            &selected,
            "--authorize-stdin",
            "--local-db",
            local.to_str().unwrap(),
        ],
        &mut authorization,
    );
    assert!(matches!(code, 0 | 2), "events:\n{events}");
    assert!(events.contains("calibration.stored_client_plan"));
    assert!(events.contains("\"reason\":\"stored-client-persisted\""));
    let plan_event = events
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|event| event["event"] == "calibration.stored_client_plan")
        .expect("one stored client plan");
    assert!(plan_event["plan_id"]
        .as_str()
        .unwrap()
        .starts_with("stored-client-selection-v1:"));
    let canonical: serde_json::Value =
        serde_json::from_str(plan_event["canonical_json"].as_str().unwrap()).unwrap();
    assert_eq!(canonical["target"]["stable_id"], 85_001);
    assert_eq!(canonical["proposed_executable"], "shared-client.exe");
    assert_eq!(
        canonical["resulting_launch_entries"][0]["arguments"],
        "--second"
    );
    assert_eq!(canonical["resulting_launch_entries"][0]["role"], "client");
    assert_eq!(
        canonical["operation"],
        "select-stored-client-if-unchanged-v1"
    );
    assert_eq!(canonical["no_effects"].as_array().unwrap().len(), 5);
    let selected_target = Store::open(&local)
        .unwrap()
        .target_by_stable_id(85_001)
        .unwrap()
        .unwrap();
    assert_eq!(
        selected_target.launch_entries,
        Some(serde_json::json!([{
            "executable":"shared-client.exe",
            "arguments":"--second",
            "role":"client"
        }]))
    );
    assert_eq!(selected_target.fidelity, FidelityTier::Authored);
}

#[test]
fn malformed_and_unused_candidates_stop_before_workflow_creation() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    seed_target(&local, true);
    let (code, _out, malformed) = run(&[
        "calibrate",
        "--id",
        "75000",
        "--candidate",
        "candidate-v1:not-a-digest",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(malformed.contains("64 lowercase hexadecimal"));

    let valid = format!("candidate-v1:{}", "0".repeat(64));
    let (code, _out, unused) = run(&[
        "calibrate",
        "--id",
        "75000",
        "--candidate",
        &valid,
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(
        unused.contains("was not consumed"),
        "diagnostics:\n{unused}"
    );
    assert!(Store::open(&local)
        .unwrap()
        .calibration_workflow(1)
        .unwrap()
        .is_none());

    let discovery_local = dir.path().join("discovery.db");
    let (code, _out, stale) = run(&[
        "--json",
        "calibrate",
        "Sample Target",
        "--candidate",
        &valid,
        "--authorize-stdin",
        "--controlled-target",
        "--local-db",
        discovery_local.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(stale.contains("no current ambiguous target-registration choice"));
    assert!(Store::open(&discovery_local)
        .unwrap()
        .targets()
        .unwrap()
        .is_empty());
}

#[test]
fn duplicate_candidate_authority_is_refused_without_a_target_write() {
    let _environment = controlled_environment().lock().unwrap();
    std::env::set_var("FRAGCAP_CONTROLLED_TARGET_REGISTRATION_DUPLICATE", "1");
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let (code, _out, events) = run(&[
        "--json",
        "calibrate",
        "Sample Target",
        "--authorize-stdin",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_REGISTRATION_DUPLICATE");
    assert_eq!(code, 2, "events:\n{events}");
    assert!(events.contains("duplicate candidate authority"));
    assert!(events.contains("calibration.choice_required"));
    assert!(Store::open(&local).unwrap().targets().unwrap().is_empty());
}

#[test]
fn exact_case_intent_is_persisted_rendered_and_reused_on_resume() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    seed_target(&local, true);
    let mut authorization = BoundedEchoAuthorization {
        calls: 0,
        accepted: 0,
    };
    let (code, _out, events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "--id",
            "75000",
            "--launch-case",
            "direct-exe-cold",
            "--routing-strategy",
            "child-environment",
            "--proxy-family",
            "ipv6",
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
        ],
        &mut authorization,
    );
    assert_eq!(code, 2, "events:\n{events}");
    assert_eq!(authorization.calls, 1);
    assert!(events.contains("\"launch_case_assertion\":\"direct-exe-cold\""));
    assert!(events.contains("\"routing_strategy\":\"child-environment\""));
    assert!(events.contains("\"address_family\":\"ipv6\""));
    let authorization_plan = events
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|event| event["event"] == "deep_capture.authorization_plan")
        .expect("the delegated low-level attempt must disclose its plan");
    assert_eq!(
        authorization_plan["plan"]["proxy"]["address_family"],
        "ipv6"
    );
    assert!(events.contains("\"reason\":\"delegated-session-refused\""));
    let store = Store::open(&local).unwrap();
    let workflow = store.calibration_workflow(1).unwrap().unwrap();
    assert_eq!(
        workflow.selected_launch_case,
        Some(CompatibilityLaunchCase::DirectExeCold)
    );
    assert_eq!(
        workflow.routing_strategy,
        CompatibilityRoutingStrategy::ChildEnvironment
    );
    assert_eq!(workflow.address_family, CompatibilityAddressFamily::Ipv6);
    drop(store);

    let (code, _out, resumed) = run(&[
        "--json",
        "calibrate",
        "--resume",
        "1",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 2, "events:\n{resumed}");
    assert!(resumed.contains("\"address_family\":\"ipv6\""));
    assert!(resumed.contains("\"launch_case_assertion\":\"direct-exe-cold\""));
}

#[test]
fn unsupported_routing_and_mismatched_launch_assertions_refuse_before_effects() {
    for (extra, expected) in [
        (
            vec!["--routing-strategy", "socks"],
            "unsupported-routing-strategy",
        ),
        (
            vec!["--launch-case", "steam-protocol-cold"],
            "launch-case-assertion-mismatch",
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let local = dir.path().join("local.db");
        seed_target(&local, true);
        let mut args = vec![
            "--json",
            "calibrate",
            "--id",
            "75000",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
        ];
        args.extend(extra);
        let (code, _out, events) = run(&args);
        assert_eq!(code, 2, "events:\n{events}");
        assert!(events.contains(expected), "events:\n{events}");
        assert!(!events.contains("deep_capture.authorization_plan"));
        assert!(!events.contains("deep_capture.session"));
        assert_eq!(
            Store::open(&local)
                .unwrap()
                .calibration_workflow(1)
                .unwrap()
                .unwrap()
                .state,
            CalibrationWorkflowState::Refused
        );
    }
}

#[test]
fn steam_client_setup_refuses_changed_or_missing_target_authority() {
    let _environment = controlled_environment().lock().unwrap();
    for mutation in [TargetMutation::Change, TargetMutation::Delete] {
        let dir = tempfile::tempdir().unwrap();
        let local = dir.path().join("local.db");
        let stable_id = seed_missing_steam_target(&local);
        let id = stable_id.to_string();
        let mut authorization = TargetMutationAuthorization {
            local: local.clone(),
            mutation,
        };
        let (code, _out, events) = run_with_authorization(
            &[
                "--json",
                "calibrate",
                "--id",
                &id,
                "--authorize-stdin",
                "--controlled-target",
                "--local-db",
                local.to_str().unwrap(),
            ],
            &mut authorization,
        );
        assert_eq!(code, 2, "events:\n{events}");
        assert!(events.contains("\"status\":\"drifted\""));
        assert!(!events.contains("authored-client-persisted"));
        if let Some(target) = Store::open(&local)
            .unwrap()
            .target_by_stable_id(stable_id)
            .unwrap()
        {
            assert_eq!(target.launch_entries, None);
        }
    }
}

#[test]
fn confirmed_registration_refuses_rediscovery_drift_before_writing_a_target() {
    let _environment = controlled_environment().lock().unwrap();
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_REGISTRATION_DRIFT");
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let mut authorization = DriftAuthorization;
    let (code, _out, events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "75000",
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
        ],
        &mut authorization,
    );
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_REGISTRATION_DRIFT");
    assert_eq!(code, 2, "events:\n{events}");
    assert!(events.contains("\"status\":\"drifted\""));
    assert!(Store::open(&local).unwrap().targets().unwrap().is_empty());
}

#[test]
fn confirmed_registration_preserves_rediscovery_ambiguity_diagnostics() {
    let _environment = controlled_environment().lock().unwrap();
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_REGISTRATION_AMBIGUOUS");
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let mut authorization = AmbiguityAuthorization;
    let (code, _out, events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "Sample Target",
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
        ],
        &mut authorization,
    );
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_REGISTRATION_AMBIGUOUS");
    assert_eq!(code, 2, "events:\n{events}");
    assert!(events.contains("\"status\":\"drifted\""));
    assert!(
        events.contains("calibration.choice_required"),
        "events:\n{events}"
    );
    assert!(events.contains("steam:75000"), "events:\n{events}");
    assert!(
        events.contains("C:\\\\Other Games\\\\Sample Target\\\\client.exe"),
        "events:\n{events}"
    );
    assert!(Store::open(&local).unwrap().targets().unwrap().is_empty());
}

#[test]
fn numeric_discovery_selector_continues_by_registered_stable_id() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let stable_id = fragcap::targets::identifier::anchored_id("steam:75000");
    let mut store = Store::open(&local).unwrap();
    let target = TargetEntry {
        id: None,
        stable_id,
        handle: "sample-target".to_string(),
        name: "Sample Target".to_string(),
        classification: TargetClassification::Game,
        classification_source: ClassificationSource::Platform,
        fidelity: FidelityTier::Observed,
        provenance: None,
        anchor: Some("steam:75000".to_string()),
        launch_entries: Some(resolved_client_launch("client.exe")),
        install_root: Some("C:\\Games\\Sample Target".to_string()),
        evidence: None,
        detection_scan: None,
        folder_name: Some("Sample Target".to_string()),
        executable_hint: Some("client.exe".to_string()),
    };
    let row_id = store.insert_target(&target).unwrap();
    insert_current_routing_fact(&mut store, row_id);
    drop(store);

    let mut authorization = EchoAuthorization { calls: 0 };
    let (code, out, events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "75000",
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
        ],
        &mut authorization,
    );
    assert_eq!(code, 0, "events:\n{events}");
    assert!(out.is_empty());
    assert_eq!(
        authorization.calls, 2,
        "registration and the selected calibration attempt are authorized"
    );
    assert!(events.contains("\"status\":\"already-present\""));
    assert!(events.contains("\"status\":\"ready\""));
    assert_next_command_selects_store(&events, "calibrate", stable_id, &local);
}

#[test]
fn current_exact_routing_emits_ready_guidance_without_session_effects() {
    let dir = tempfile::tempdir().unwrap();
    let store_dir = dir.path().join("custom store $fixture");
    std::fs::create_dir(&store_dir).unwrap();
    let local = store_dir.join("local.db");
    let bundle = dir.path().join("must-not-exist");
    seed_target(&local, true);
    let id_arg = STABLE_ID.to_string();

    let (code, out, events) = run(&[
        "--json",
        "calibrate",
        "--id",
        &id_arg,
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
    ]);

    assert_eq!(code, 0, "events:\n{events}");
    assert!(out.is_empty());
    assert!(events.contains("\"event\":\"calibration.guidance\""));
    assert!(events.contains("\"status\":\"ready\""));
    assert_next_command_selects_store(&events, "deep-capture", STABLE_ID, &local);
    assert!(!events.contains("deep_capture.authorization_plan"));
    assert!(!bundle.exists(), "ready guidance must not create a bundle");
}

#[test]
fn guidance_obeys_human_suppression_and_json_remains_machine_readable() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    seed_target(&local, true);
    for flag in ["--quiet", "--silent"] {
        let (code, out, err) = run(&[
            flag,
            "calibrate",
            "--id",
            "75000",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
        ]);
        assert_eq!(code, 0, "stderr:\n{err}");
        assert!(out.is_empty());
        assert!(err.is_empty(), "{flag} guidance leaked: {err}");
    }
    let (code, out, events) = run(&[
        "--json",
        "--quiet",
        "calibrate",
        "--id",
        "75000",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "events:\n{events}");
    assert!(out.is_empty());
    let event: serde_json::Value = serde_json::from_str(events.lines().next().unwrap()).unwrap();
    assert_eq!(event["event"], "calibration.guidance");
    assert!(event["next_command"].is_string());
    assert_next_command_selects_store(&events, "deep-capture", STABLE_ID, &local);

    let (code, out, human) = run(&[
        "calibrate",
        "--id",
        "75000",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "stderr:\n{human}");
    assert!(out.is_empty());
    assert!(human.contains("workflow_id="), "stderr:\n{human}");
    assert!(human.contains("workflow_revision="), "stderr:\n{human}");
    assert!(
        human.contains("workflow_state=completed"),
        "stderr:\n{human}"
    );
    assert!(human.contains("pause_reason=none"), "stderr:\n{human}");
    assert!(human.contains("resume_command=fragcap calibrate --resume"));
}

#[test]
fn missing_routing_runs_reachability_then_observed_protocol() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let bundle = dir.path().join("reachability");
    let row_id = seed_target(&local, false);
    let id_arg = STABLE_ID.to_string();
    std::env::set_var(
        "FRAGCAP_CONTROLLED_TARGET_EXECUTABLE",
        env!("CARGO_BIN_EXE_fragcap"),
    );

    let (code, out, events) = run(&[
        "--json",
        "calibrate",
        "--id",
        &id_arg,
        "--authorize-stdin",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
        "--duration",
        "5s",
        "--wait",
        "7s",
    ]);
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE");

    assert_eq!(code, 0, "events:\n{events}");
    assert!(out.is_empty());
    assert_eq!(events.matches("deep_capture.authorization_plan").count(), 3);
    let plan: serde_json::Value = events
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .find(|event: &serde_json::Value| event["event"] == "deep_capture.authorization_plan")
        .unwrap();
    assert_eq!(plan["plan"]["target"]["stable_id"], STABLE_ID);
    assert_eq!(plan["plan"]["target"]["handle"], "sample-target");
    assert_eq!(plan["plan"]["launch"]["observed_case"], "direct-exe-cold");
    assert_eq!(
        plan["plan"]["launch"]["process_control"],
        "managed launch only; no process handle or forced termination"
    );
    assert_eq!(
        plan["plan"]["proxy"]["routing_scope"],
        "managed child environment only"
    );
    assert_eq!(plan["plan"]["proxy"]["address_family"], "ipv4");
    assert_eq!(plan["plan"]["proxy"]["system_proxy_change"], false);
    assert_eq!(plan["plan"]["deadlines_milliseconds"]["launch"], 7_000);
    assert_eq!(plan["plan"]["deadlines_milliseconds"]["observation"], 5_000);
    assert_eq!(plan["plan"]["trust"]["action"], "none");
    assert_eq!(
        plan["plan"]["capture"]["scope"],
        "selected target process tree"
    );
    assert_eq!(plan["plan"]["cleanup"].as_array().unwrap().len(), 4);
    assert_eq!(
        plan["plan"]["artifacts"]["bundle"],
        bundle.display().to_string()
    );
    assert!(events.contains("\"phase\":\"reachability\""));
    assert!(events.contains("\"protocol\":\"routing\""));
    assert!(events.contains("\"event\":\"calibration.guidance\""));
    assert!(events.contains("\"status\":\"completed\""));
    assert_next_command_selects_store(&events, "deep-capture", STABLE_ID, &local);

    let store = Store::open(&local).unwrap();
    let facts = store.compatibility_facts_for_target(row_id).unwrap();
    assert!(facts.iter().any(|fact| {
        fact.key == CompatibilityFactKey::ProxyRouting
            && fact.value == "reached-client"
            && fact.launch_case == Some(CompatibilityLaunchCase::DirectExeCold)
            && fact.protocol == Some(CompatibilityProtocol::NotApplicable)
    }));
}

#[test]
#[cfg(windows)]
fn direct_steam_and_publisher_current_cases_preserve_topology_and_durable_handoff() {
    for (stable_id, handle, anchor, launches, launch_case, expected_topology) in [
        (
            81_001,
            "direct-fixture",
            None,
            serde_json::json!([{ "executable": "absent-direct-fixture.exe", "role": "client" }]),
            CompatibilityLaunchCase::DirectExeCold,
            "direct",
        ),
        (
            81_002,
            "steam-fixture",
            Some("steam:123"),
            serde_json::json!([{ "executable": "absent-steam-fixture.exe", "role": "client" }]),
            CompatibilityLaunchCase::SteamProtocolCold,
            "steam",
        ),
        (
            81_003,
            "publisher-fixture",
            None,
            serde_json::json!([
                { "executable": "absent-launcher-fixture.exe", "role": "launcher" },
                { "executable": "absent-client-fixture.exe", "role": "client" }
            ]),
            CompatibilityLaunchCase::PublisherLauncherCold,
            "publisher",
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let local = dir.path().join("local.db");
        seed_topology(&local, stable_id, handle, anchor, launches, launch_case);
        let id_arg = stable_id.to_string();
        let (code, out, events) = run(&[
            "--json",
            "calibrate",
            "--id",
            &id_arg,
            "--local-db",
            local.to_str().unwrap(),
        ]);
        assert_eq!(code, 0, "events:\n{events}");
        assert!(out.is_empty());
        let event: serde_json::Value =
            serde_json::from_str(events.lines().next().unwrap()).unwrap();
        assert_eq!(event["topology"], expected_topology);
        assert_eq!(event["selected_launch_case"], launch_case.as_str());
        assert!(event["launch_case_assertion"].is_null());
        assert_eq!(event["routing_strategy"], "child-environment");
        assert_eq!(event["address_family"], "ipv4");
        if event["status"] == "ready" {
            assert_next_command_selects_store(&events, "deep-capture", stable_id, &local);
        } else {
            assert_eq!(expected_topology, "steam");
            assert_eq!(event["status"], "warm");
            assert_next_command_selects_store(&events, "calibrate", stable_id, &local);
        }
    }
}

#[test]
#[cfg(windows)]
fn warm_target_is_guidance_only_until_restart_is_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let image = std::env::current_exe()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    seed_topology(
        &local,
        82_001,
        "warm-fixture",
        None,
        serde_json::json!([{ "executable": image, "role": "client" }]),
        CompatibilityLaunchCase::DirectExeWarm,
    );

    let (code, _out, events) = run(&[
        "--json",
        "calibrate",
        "--id",
        "82001",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "events:\n{events}");
    assert!(events.contains("\"action\":\"operator-action\""));
    assert!(events.contains("\"observed_launch_case\":\"direct-exe-warm\""));
    assert!(events.contains("\"process_control\":\"none\""));
    assert!(events.contains("\"workflow_state\":\"paused\""));
    assert!(events.contains("\"pause_reason\":\"shutdown\""));
    assert!(events.contains("--restart-warm"));
    assert_next_command_selects_store(&events, "calibrate", 82_001, &local);
    assert!(!events.contains("deep_capture.restart_plan"));

    let (code, _out, events) = run(&[
        "--json",
        "calibrate",
        "--resume",
        "1",
        "--restart-warm",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 2, "events:\n{events}");
    assert!(events.contains("deep_capture.restart_plan"));
    assert!(events.contains("operator-close step requires a terminal"));
    assert!(!events.contains("deep_capture.authorization_plan"));
    let workflow = Store::open(&local)
        .unwrap()
        .calibration_workflow(1)
        .unwrap()
        .unwrap();
    assert_eq!(workflow.state, CalibrationWorkflowState::Paused);
    assert_eq!(workflow.pause_reason.unwrap().as_str(), "shutdown");
}

#[test]
fn ambiguous_stored_topology_requires_choice_before_effects() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let mut store = Store::open(&local).unwrap();
    let target = TargetEntry {
        id: None,
        stable_id: 83_001,
        handle: "ambiguous-fixture".to_string(),
        name: "Ambiguous Fixture".to_string(),
        classification: TargetClassification::Game,
        classification_source: ClassificationSource::User,
        fidelity: FidelityTier::Authored,
        provenance: None,
        anchor: None,
        launch_entries: Some(serde_json::json!([
            { "executable": "A.exe" },
            { "executable": "B.exe" }
        ])),
        install_root: None,
        evidence: None,
        detection_scan: None,
        folder_name: None,
        executable_hint: None,
    };
    store.insert_target(&target).unwrap();
    drop(store);
    let bundle = dir.path().join("must-not-exist");

    let (code, _out, events) = run(&[
        "--json",
        "calibrate",
        "--id",
        "83001",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
    ]);
    assert_eq!(code, 2, "events:\n{events}");
    assert!(events.contains("calibration.choice_required"));
    assert!(events.contains("\"scope\":\"stored-client\""));
    assert!(events.contains("A.exe") && events.contains("B.exe"));
    assert!(!events.contains("calibration.stored_client_plan"));
    assert!(!bundle.exists());
}

#[test]
#[cfg(windows)]
fn every_non_positive_routing_state_preserves_the_proposal_reason() {
    for (index, expected, configure) in [
        (0_i64, "missing", 0_u8),
        (1, "negative", 1),
        (2, "stale", 2),
        (3, "legacy-incomplete", 3),
        (4, "context-mismatch", 4),
        (5, "conflict", 5),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let local = dir.path().join("local.db");
        let stable_id = 84_000 + index;
        let mut store = Store::open(&local).unwrap();
        let target = TargetEntry {
            id: None,
            stable_id,
            handle: format!("reason-{expected}"),
            name: format!("Reason {expected}"),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::User,
            fidelity: FidelityTier::Authored,
            provenance: None,
            anchor: None,
            launch_entries: Some(serde_json::json!([{
                "executable": format!("absent-reason-{index}.exe"),
                "role": "client"
            }])),
            install_root: None,
            evidence: None,
            detection_scan: None,
            folder_name: None,
            executable_hint: None,
        };
        let row_id = store.insert_target(&target).unwrap();
        match configure {
            0 => {}
            1 => insert_routing_fact(
                &mut store,
                row_id,
                CompatibilityLaunchCase::DirectExeCold,
                "no-proxy-traffic",
                false,
                CompatibilityAddressFamily::Ipv4,
                true,
            ),
            2 => insert_routing_fact(
                &mut store,
                row_id,
                CompatibilityLaunchCase::DirectExeCold,
                "reached-client",
                true,
                CompatibilityAddressFamily::Ipv4,
                true,
            ),
            3 => insert_routing_fact(
                &mut store,
                row_id,
                CompatibilityLaunchCase::DirectExeCold,
                "reached-client",
                false,
                CompatibilityAddressFamily::Ipv4,
                false,
            ),
            4 => insert_routing_fact(
                &mut store,
                row_id,
                CompatibilityLaunchCase::DirectExeCold,
                "reached-client",
                false,
                CompatibilityAddressFamily::Ipv6,
                true,
            ),
            5 => {
                for value in ["reached-client", "no-proxy-traffic"] {
                    insert_routing_fact(
                        &mut store,
                        row_id,
                        CompatibilityLaunchCase::DirectExeCold,
                        value,
                        false,
                        CompatibilityAddressFamily::Ipv4,
                        true,
                    );
                }
            }
            _ => unreachable!(),
        }
        drop(store);
        let id_arg = stable_id.to_string();
        let bundle = dir.path().join("must-not-exist");
        let (code, _out, events) = run(&[
            "--json",
            "calibrate",
            "--id",
            &id_arg,
            "--authorize-stdin",
            "--local-db",
            local.to_str().unwrap(),
            "--bundle",
            bundle.to_str().unwrap(),
        ]);
        assert_ne!(code, 0, "the absent executable cannot start: {events}");
        let selected: serde_json::Value = events
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .find(|event: &serde_json::Value| event["event"] == "calibration.guidance")
            .unwrap();
        assert_eq!(selected["reason"], expected, "events:\n{events}");
        assert_eq!(selected["status"], "selected");
        assert!(!bundle.exists());
    }
}

#[test]
fn declined_and_wrong_authorization_never_claim_completion() {
    let declined_dir = tempfile::tempdir().unwrap();
    let declined_local = declined_dir.path().join("local.db");
    let declined_bundle = declined_dir.path().join("bundle");
    seed_target(&declined_local, false);
    let mut decline = FixedAuthorization {
        terminal: true,
        response: b"no\n".to_vec(),
    };
    let (code, _out, guidance) = run_with_authorization(
        &[
            "calibrate",
            "--id",
            "75000",
            "--controlled-target",
            "--local-db",
            declined_local.to_str().unwrap(),
            "--bundle",
            declined_bundle.to_str().unwrap(),
        ],
        &mut decline,
    );
    assert_eq!(code, 0, "guidance:\n{guidance}");
    assert!(guidance.contains("status=declined"));
    assert!(!guidance.contains("status=completed"));
    assert!(guidance.contains("fragcap calibrate --resume"));
    assert!(!declined_bundle.exists());

    let invalid_dir = tempfile::tempdir().unwrap();
    let invalid_local = invalid_dir.path().join("local.db");
    let invalid_bundle = invalid_dir.path().join("bundle");
    seed_target(&invalid_local, false);
    let mut invalid = FixedAuthorization {
        terminal: false,
        response: format!("plan-v1:{}\n", "0".repeat(64)).into_bytes(),
    };
    let (code, _out, events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "--id",
            "75000",
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            invalid_local.to_str().unwrap(),
            "--bundle",
            invalid_bundle.to_str().unwrap(),
        ],
        &mut invalid,
    );
    assert_eq!(code, 2, "events:\n{events}");
    assert!(events.contains("\"status\":\"invalid\""));
    assert!(events.contains("\"reason\":\"delegated-session-refused\""));
    assert!(!events.contains("\"status\":\"completed\""));
    assert!(!invalid_bundle.exists());
}

#[test]
fn one_invocation_runs_reachability_and_all_current_useful_protocol_attempts() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let bundle = dir.path().join("sequence");
    let row_id = seed_target(&local, false);
    let mut authorization = RecordingEchoAuthorization {
        plan_ids: Vec::new(),
    };
    std::env::set_var(
        "FRAGCAP_CONTROLLED_TARGET_EXECUTABLE",
        env!("CARGO_BIN_EXE_fragcap"),
    );

    let (code, _out, events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "--id",
            "75000",
            "--protocol",
            "https",
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
            "--bundle",
            bundle.to_str().unwrap(),
            "--duration",
            "5s",
            "--wait",
            "7s",
        ],
        &mut authorization,
    );
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE");

    assert_eq!(code, 0, "events:\n{events}");
    assert_eq!(authorization.plan_ids.len(), 3, "events:\n{events}");
    for (index, plan) in authorization.plan_ids.iter().enumerate() {
        assert!(authorization.plan_ids[index + 1..]
            .iter()
            .all(|other| other != plan));
    }
    assert_eq!(events.matches("deep_capture.authorization_plan").count(), 3);
    let selected_attempts = events
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|event| event["event"] == "calibration.guidance" && event["status"] == "selected")
        .map(|event| {
            (
                event["attempt"].as_u64().unwrap(),
                event["maximum_attempts"].as_u64().unwrap(),
                event["phase"].as_str().unwrap().to_string(),
                event["protocol"].as_str().unwrap().to_string(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        selected_attempts,
        vec![
            (1, 14, "reachability".to_string(), "routing".to_string()),
            (2, 14, "tls".to_string(), "http1".to_string()),
            (3, 14, "tls".to_string(), "https".to_string()),
        ]
    );
    assert!(bundle.join("manifest.json").is_file());
    assert!(dir
        .path()
        .join("sequence-attempt-02-tls-http1")
        .join("manifest.json")
        .is_file());
    assert!(dir
        .path()
        .join("sequence-attempt-03-tls-https")
        .join("manifest.json")
        .is_file());
    let guidance: serde_json::Value = events
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .rfind(|event: &serde_json::Value| event["event"] == "calibration.guidance")
        .unwrap();
    assert_eq!(guidance["status"], "requested-coverage-complete");
    assert_eq!(
        guidance["completed_protocols"],
        serde_json::json!(["http1", "https"])
    );
    assert_eq!(guidance["remaining_protocols"], serde_json::json!([]));
    assert_next_command_selects_store(&events, "deep-capture", STABLE_ID, &local);

    let store = Store::open(&local).unwrap();
    let facts = store.compatibility_facts_for_target(row_id).unwrap();
    assert!(facts
        .iter()
        .any(|fact| fact.key == CompatibilityFactKey::ProxyRouting));
    for protocol in [CompatibilityProtocol::Http1, CompatibilityProtocol::Https] {
        assert!(facts.iter().any(|fact| {
            fact.key == CompatibilityFactKey::Inspectability
                && fact.protocol == Some(protocol)
                && fact.value == "full"
        }));
    }
}

#[test]
fn declining_a_later_plan_stops_before_its_bundle_or_fact_effects() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let bundle = dir.path().join("decline-sequence");
    let row_id = seed_target(&local, false);
    let mut authorization = AcceptThenDeclineAuthorization { calls: 0 };
    std::env::set_var(
        "FRAGCAP_CONTROLLED_TARGET_EXECUTABLE",
        env!("CARGO_BIN_EXE_fragcap"),
    );

    let (code, _out, guidance) = run_with_authorization(
        &[
            "calibrate",
            "--id",
            "75000",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
            "--bundle",
            bundle.to_str().unwrap(),
            "--duration",
            "5s",
            "--wait",
            "7s",
        ],
        &mut authorization,
    );
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE");

    assert_eq!(code, 0, "guidance:\n{guidance}");
    assert_eq!(authorization.calls, 2);
    assert!(guidance.contains("status=declined"));
    assert!(bundle.join("manifest.json").is_file());
    assert!(!dir
        .path()
        .join("decline-sequence-attempt-02-tls-http1")
        .exists());
    let store = Store::open(&local).unwrap();
    let facts = store.compatibility_facts_for_target(row_id).unwrap();
    assert!(facts
        .iter()
        .any(|fact| fact.key == CompatibilityFactKey::ProxyRouting));
    assert!(!facts
        .iter()
        .any(|fact| fact.key == CompatibilityFactKey::Inspectability));
}

#[test]
fn interrupting_a_later_authorization_retains_interrupted_guidance() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let bundle = dir.path().join("interrupt-sequence");
    seed_target(&local, false);
    let mut authorization = AcceptThenInterruptAuthorization { calls: 0 };
    std::env::set_var(
        "FRAGCAP_CONTROLLED_TARGET_EXECUTABLE",
        env!("CARGO_BIN_EXE_fragcap"),
    );

    let (code, _out, events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "--id",
            "75000",
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
            "--bundle",
            bundle.to_str().unwrap(),
            "--duration",
            "5s",
            "--wait",
            "7s",
        ],
        &mut authorization,
    );
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE");

    assert_eq!(code, 1, "events:\n{events}");
    assert_eq!(authorization.calls, 2);
    assert!(events.contains("\"status\":\"interrupted\""));
    assert!(events.contains("\"reason\":\"delegated-session-interrupted\""));
    assert!(bundle.join("manifest.json").is_file());
    assert!(!dir
        .path()
        .join("interrupt-sequence-attempt-02-tls-http1")
        .exists());
}

#[test]
fn target_drift_during_a_later_plan_stops_before_later_effects() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let bundle = dir.path().join("drift-sequence");
    seed_target(&local, false);
    let mut authorization = SecondPlanTargetDriftAuthorization {
        calls: 0,
        local: local.clone(),
    };
    std::env::set_var(
        "FRAGCAP_CONTROLLED_TARGET_EXECUTABLE",
        env!("CARGO_BIN_EXE_fragcap"),
    );

    let (code, _out, events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "--id",
            "75000",
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
            "--bundle",
            bundle.to_str().unwrap(),
            "--duration",
            "5s",
            "--wait",
            "7s",
        ],
        &mut authorization,
    );
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE");

    assert_eq!(code, 2, "events:\n{events}");
    assert_eq!(authorization.calls, 2);
    assert!(events.contains("\"status\":\"drifted\""));
    assert!(events.contains("\"reason\":\"delegated-session-refused\""));
    assert!(bundle.join("manifest.json").is_file());
    assert!(!dir
        .path()
        .join("drift-sequence-attempt-02-tls-http1")
        .exists());
}

#[test]
fn a_later_bundle_collision_stops_before_authorization_or_overwrite() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let bundle = dir.path().join("collision-sequence");
    let collision = dir.path().join("collision-sequence-attempt-02-tls-http1");
    std::fs::create_dir(&collision).unwrap();
    std::fs::write(collision.join("retained.txt"), b"retain me").unwrap();
    seed_target(&local, false);
    let mut authorization = EchoAuthorization { calls: 0 };
    std::env::set_var(
        "FRAGCAP_CONTROLLED_TARGET_EXECUTABLE",
        env!("CARGO_BIN_EXE_fragcap"),
    );

    let (code, _out, events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "--id",
            "75000",
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
            "--bundle",
            bundle.to_str().unwrap(),
            "--duration",
            "5s",
            "--wait",
            "7s",
        ],
        &mut authorization,
    );
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE");

    assert_eq!(code, 2, "events:\n{events}");
    assert_eq!(authorization.calls, 1);
    assert_eq!(events.matches("deep_capture.authorization_plan").count(), 1);
    assert!(events.contains("\"status\":\"refused\""));
    assert!(events.contains("\"reason\":\"bundle-destination-refused\""));
    assert!(events.contains("\"next_command\":\"fragcap calibrate"));
    assert!(events.contains("is not empty"));
    assert_eq!(
        std::fs::read(collision.join("retained.txt")).unwrap(),
        b"retain me"
    );
    assert!(bundle.join("manifest.json").is_file());
}

#[test]
fn omitted_bundle_uses_a_distinct_default_session_root_for_every_attempt() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    seed_target(&local, false);
    let mut authorization = RecordingEchoAuthorization {
        plan_ids: Vec::new(),
    };
    std::env::set_var(
        "FRAGCAP_CONTROLLED_TARGET_EXECUTABLE",
        env!("CARGO_BIN_EXE_fragcap"),
    );

    let (code, _out, events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "--id",
            "75000",
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
            "--duration",
            "5s",
            "--wait",
            "7s",
        ],
        &mut authorization,
    );
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE");

    assert_eq!(code, 0, "events:\n{events}");
    let bundles = events
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|event| event["event"] == "deep_capture.authorization_plan")
        .map(|event| PathBuf::from(event["plan"]["artifacts"]["bundle"].as_str().unwrap()))
        .collect::<Vec<_>>();
    assert_eq!(bundles.len(), 3);
    for (index, bundle) in bundles.iter().enumerate() {
        assert!(bundle.join("manifest.json").is_file());
        assert!(bundles[index + 1..].iter().all(|other| other != bundle));
    }
}

#[test]
fn a_paused_workflow_resumes_with_a_fresh_plan_and_durable_bundle_ordinal() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let bundle = dir.path().join("resumed-workflow");
    seed_target(&local, true);
    let mut decline = RecordingDeclineAuthorization {
        plan_ids: Vec::new(),
    };
    let (code, _out, first_events) = run_with_authorization(
        &[
            "calibrate",
            "--id",
            "75000",
            "--protocol",
            "https",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
            "--bundle",
            bundle.to_str().unwrap(),
        ],
        &mut decline,
    );
    assert_eq!(code, 0, "events:\n{first_events}");
    let first_plan = decline.plan_ids.first().cloned().expect("first plan");
    let store = Store::open(&local).unwrap();
    let workflow = store.calibration_workflow(1).unwrap().unwrap();
    let workflow_id = workflow.id;
    assert_eq!(workflow.state, CalibrationWorkflowState::Paused);
    assert_eq!(workflow.pause_reason.unwrap().as_str(), "authorization");
    drop(store);

    std::env::set_var(
        "FRAGCAP_CONTROLLED_TARGET_EXECUTABLE",
        env!("CARGO_BIN_EXE_fragcap"),
    );
    let mut resume_authorization = RecordingEchoAuthorization {
        plan_ids: Vec::new(),
    };
    let (code, _out, resumed_events) = run_with_authorization(
        &[
            "--json",
            "calibrate",
            "--resume",
            &workflow_id.to_string(),
            "--authorize-stdin",
            "--controlled-target",
            "--local-db",
            local.to_str().unwrap(),
            "--bundle",
            bundle.to_str().unwrap(),
            "--duration",
            "5s",
            "--wait",
            "7s",
        ],
        &mut resume_authorization,
    );
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE");

    assert_eq!(code, 0, "events:\n{resumed_events}");
    assert!(!resume_authorization.plan_ids.is_empty());
    assert!(resume_authorization
        .plan_ids
        .iter()
        .all(|plan| plan != &first_plan));
    assert!(dir
        .path()
        .join("resumed-workflow-attempt-02-tls-https")
        .join("manifest.json")
        .is_file());
    assert!(resumed_events.contains("\"workflow_state\":\"completed\""));
}
