// SPDX-License-Identifier: Apache-2.0

// Integration coverage for the guided calibration front door.

mod common;

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use common::{run, run_with_authorization};
use fragcap::profile::FidelityTier;
use fragcap::targets::{
    resolved_client_launch, ClassificationSource, CompatibilityAddressFamily,
    CompatibilityEvidenceSource, CompatibilityFact, CompatibilityFactKey, CompatibilityLaunchCase,
    CompatibilityProtocol, CompatibilityRoutingStrategy, Store, TargetClassification, TargetEntry,
};

const STABLE_ID: i64 = 75_000;

struct FixedAuthorization {
    terminal: bool,
    response: Vec<u8>,
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
    assert_eq!(arguments.get_one::<i64>("id"), Some(&target_id));
    assert_eq!(
        arguments
            .get_one::<PathBuf>("local_db")
            .map(PathBuf::as_path),
        Some(local)
    );
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
        let mut fact = CompatibilityFact::new(
            id,
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
    id
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
        "--bundle",
        "--duration",
        "--wait",
        "--max-packets",
        "--max-bytes",
        "--interface",
        "--no-payload",
        "--authorize-stdin",
        "--restart-warm",
    ] {
        assert!(
            help.contains(required),
            "help must contain {required}:\n{help}"
        );
    }
    for excluded in [
        "--calibration-protocol",
        "--launch-case",
        "--proxy-family",
        "--proxy-bypass",
        "--trust-ca",
        "--yes",
        "--controlled-target",
    ] {
        assert!(!help.contains(excluded), "help leaked {excluded}:\n{help}");
    }
}

#[test]
fn calibrate_requires_one_registered_target_selector() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    Store::open(&local).expect("scratch local store");
    let (code, _out, err) = run(&[
        "calibrate",
        "missing-target",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(err.contains("no target matches"), "refusal: {err}");
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
}

#[test]
fn missing_routing_runs_one_controlled_reachability_attempt() {
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
    assert_eq!(events.matches("deep_capture.authorization_plan").count(), 1);
    assert!(events.contains("\"phase\":\"reachability\""));
    assert!(events.contains("\"protocol\":\"routing\""));
    assert!(events.contains("\"event\":\"calibration.guidance\""));
    assert!(events.contains("\"status\":\"completed\""));
    assert_next_command_selects_store(&events, "calibrate", STABLE_ID, &local);

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
    assert!(events.contains("--restart-warm"));
    assert_next_command_selects_store(&events, "calibrate", 82_001, &local);
    assert!(!events.contains("deep_capture.restart_plan"));

    let (code, _out, events) = run(&[
        "--json",
        "calibrate",
        "--id",
        "82001",
        "--restart-warm",
        "--local-db",
        local.to_str().unwrap(),
    ]);
    assert_eq!(code, 2, "events:\n{events}");
    assert!(events.contains("deep_capture.restart_plan"));
    assert!(events.contains("operator-close step requires a terminal"));
    assert!(!events.contains("deep_capture.authorization_plan"));
}

#[test]
fn malformed_topology_reports_all_typed_limitations_before_effects() {
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
    assert!(events.contains("\"action\":\"refused\""));
    assert!(events.contains("ambiguous-launch-declaration"));
    assert!(events.contains("A.exe") && events.contains("B.exe"));
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
    assert!(guidance.contains("status=not-completed"));
    assert!(!guidance.contains("status=completed"));
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
    assert!(events.contains("\"reason\":\"delegated-session-error\""));
    assert!(!events.contains("\"status\":\"completed\""));
    assert!(!invalid_bundle.exists());
}
