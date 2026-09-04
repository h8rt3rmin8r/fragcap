// SPDX-License-Identifier: Apache-2.0

//! Static authority for the native Deep Capture performance gate.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Component, Path};

use serde_json::Value;

const REGISTRY: &str = "performance/native-proxy-budgets-v1.json";
const REFERENCE: &str = "performance/native-proxy-reference-v1.json";
const SOAK_SUMMARY: &str = "performance/native-proxy-soak-v1.json";
const PROTOCOLS: &[&str] = &["grpc", "http1", "http2", "quic", "tcp", "udp", "websocket"];
const RETENTION: &[&str] = &["off", "on"];

pub fn run(root: &Path, report: Option<&Path>) -> io::Result<usize> {
    let registry_bytes = fs::read(root.join(REGISTRY))?;
    let value: Value = serde_json::from_slice(&registry_bytes).map_err(io::Error::other)?;
    let problems = validate(root, &value)?;
    let mut problems = problems;
    if let Some(report) = report {
        problems.extend(validate_report(
            &fs::read_to_string(report)?,
            &stable_digest(&registry_bytes),
            &value,
        ));
    }
    for problem in &problems {
        eprintln!("performance: {problem}");
    }
    if problems.is_empty() {
        println!("performance: schema 1, 14 frozen cases, short and two-hour soak profiles");
    }
    Ok(problems.len())
}

fn validate(root: &Path, value: &Value) -> io::Result<Vec<String>> {
    let mut problems = Vec::new();
    if value["schema_version"].as_u64() != Some(1) {
        problems.push("schema_version must be 1".into());
    }
    if value["issue"].as_u64() != Some(326) {
        problems.push("issue must be 326".into());
    }
    positive(value, "workload_seed", "registry", &mut problems);
    required_string(value, "reviewed_on", "registry", &mut problems);
    exact_strings(&value["matrix"], "protocols", PROTOCOLS, &mut problems);
    exact_strings(&value["matrix"], "retention", RETENTION, &mut problems);

    let profiles = &value["profiles"];
    positive(
        &profiles["short"],
        "windows",
        "short profile",
        &mut problems,
    );
    positive(
        &profiles["short"],
        "warmup_windows",
        "short profile",
        &mut problems,
    );
    if profiles["short"]["windows"].as_u64() != Some(7) {
        problems.push("short profile must use exactly seven windows".into());
    }
    if profiles["soak"]["minimum_duration_seconds"].as_u64() != Some(7_200) {
        problems.push("soak profile must require exactly 7200 seconds".into());
    }
    if profiles["soak"]["sample_seconds"]
        .as_u64()
        .is_none_or(|value| value > 60)
    {
        problems.push("soak profile must sample at least once per 60 seconds".into());
    }

    let evaluation = &value["evaluation"];
    for field in [
        "guard_band_percent",
        "minimum_breaching_windows",
        "maximum_retries",
        "maximum_shutdown_milliseconds",
        "maximum_private_memory_growth_bytes",
        "maximum_worker_memory_bytes",
        "maximum_artifact_bytes",
        "maximum_cpu_microseconds_per_mib",
        "maximum_worker_tasks",
        "maximum_application_queue",
        "maximum_leaf_cache_entries",
        "maximum_leaf_cache_bytes",
    ] {
        positive(evaluation, field, "evaluation", &mut problems);
    }
    if evaluation["maximum_unaccounted_units"].as_u64() != Some(0) {
        problems.push("maximum_unaccounted_units must be zero".into());
    }

    let Some(cases) = value["cases"].as_array() else {
        problems.push("cases must be an array".into());
        return Ok(problems);
    };
    let expected: BTreeSet<String> = PROTOCOLS
        .iter()
        .flat_map(|protocol| {
            RETENTION
                .iter()
                .map(move |retention| format!("{protocol}-{retention}"))
        })
        .collect();
    let mut observed = BTreeSet::new();
    for case in cases {
        let protocol = required_string(case, "protocol", "case", &mut problems);
        let retention = required_string(case, "retention", "case", &mut problems);
        let id = required_string(case, "id", "case", &mut problems);
        if id != format!("{protocol}-{retention}") {
            problems.push(format!(
                "case {id} identity does not match {protocol}-{retention}"
            ));
        }
        if !observed.insert(id.clone()) {
            problems.push(format!("duplicate case {id}"));
        }
        for field in [
            "useful_bytes_per_window",
            "concurrency",
            "minimum_throughput_bytes_per_second",
            "minimum_throughput_ratio_basis_points",
            "maximum_added_p95_microseconds",
        ] {
            positive(case, field, &format!("case {id}"), &mut problems);
        }
        let evidence = required_string(case, "evidence", &format!("case {id}"), &mut problems);
        if safe_relative(&evidence) && !root.join(&evidence).is_file() {
            problems.push(format!("case {id} evidence does not exist: {evidence}"));
        } else if !safe_relative(&evidence) {
            problems.push(format!("case {id} evidence is not a safe relative path"));
        }
    }
    if observed != expected {
        problems.push(format!(
            "case matrix drift: expected={expected:?}, found={observed:?}"
        ));
    }

    for path in [
        "performance/native-proxy/Cargo.toml",
        "performance/native-proxy/Cargo.lock",
        "performance/native-proxy/src/main.rs",
        SOAK_SUMMARY,
        ".github/workflows/performance.yml",
        "docs/security/deep-capture-performance.md",
    ] {
        if !root.join(path).is_file() {
            problems.push(format!("required performance authority is missing: {path}"));
        }
    }
    if let Ok(workflow) = fs::read_to_string(root.join(".github/workflows/performance.yml")) {
        for required in [
            "windows-latest",
            "ubuntu-latest",
            "workflow_dispatch",
            "schedule",
            "7200",
        ] {
            if !workflow.contains(required) {
                problems.push(format!("performance workflow does not name {required}"));
            }
        }
    }
    validate_reference(root, &expected, &mut problems)?;
    validate_soak_summary(root, value, &mut problems)?;
    validate_runtime_inventory(root, &mut problems)?;
    Ok(problems)
}

fn validate_soak_summary(
    root: &Path,
    registry: &Value,
    problems: &mut Vec<String>,
) -> io::Result<()> {
    let summary: Value =
        serde_json::from_slice(&fs::read(root.join(SOAK_SUMMARY))?).map_err(io::Error::other)?;
    let registry_digest = stable_digest(&fs::read(root.join(REGISTRY))?);
    if summary["schema_version"].as_u64() != Some(1)
        || summary["profile"].as_str() != Some("soak")
        || summary["complete"].as_bool() != Some(true)
        || summary["passed"].as_bool() != Some(true)
        || summary["contains_sensitive_data"].as_bool() != Some(false)
    {
        problems.push("soak summary metadata is incomplete".into());
    }
    if summary["registry_digest"].as_str() != Some(registry_digest.as_str()) {
        problems.push("soak summary registry digest is stale".into());
    }
    if summary["duration_seconds"].as_u64().unwrap_or(0) < 7_200
        || summary["required_cases"].as_u64() != Some(14)
        || summary["failed_case_terminals"].as_u64() != Some(0)
    {
        problems.push("soak summary does not prove complete passing coverage".into());
    }
    let terminals = summary["case_terminals"].as_u64().unwrap_or(0);
    if terminals == 0
        || !terminals.is_multiple_of(14)
        || summary["complete_cycles"].as_u64() != Some(terminals / 14)
        || summary["case_samples"].as_u64() != Some(terminals.saturating_mul(7))
    {
        problems.push("soak summary cycle and sample counts do not reconcile".into());
    }
    for field in [
        "application_events_dropped",
        "payload_bytes_queue_dropped",
        "payload_bytes_storage_dropped",
    ] {
        if summary[field].as_u64() != Some(0) {
            problems.push(format!("soak summary requires zero {field}"));
        }
    }
    if summary["private_memory_span_bytes"]
        .as_u64()
        .unwrap_or(u64::MAX)
        > registry["evaluation"]["maximum_private_memory_growth_bytes"]
            .as_u64()
            .unwrap_or(0)
    {
        problems.push("soak summary exceeds the private-memory growth budget".into());
    }
    if summary["maximum_event_queue_peak"]
        .as_u64()
        .unwrap_or(u64::MAX)
        > registry["evaluation"]["maximum_application_queue"]
            .as_u64()
            .unwrap_or(0)
    {
        problems.push("soak summary exceeds the application queue budget".into());
    }
    Ok(())
}

fn validate_reference(
    root: &Path,
    expected: &BTreeSet<String>,
    problems: &mut Vec<String>,
) -> io::Result<()> {
    let reference: Value =
        serde_json::from_slice(&fs::read(root.join(REFERENCE))?).map_err(io::Error::other)?;
    let registry_digest = stable_digest(&fs::read(root.join(REGISTRY))?);
    if reference["schema_version"].as_u64() != Some(1)
        || reference["profile"].as_str() != Some("short")
        || reference["complete"].as_bool() != Some(true)
        || reference["contains_sensitive_data"].as_bool() != Some(false)
    {
        problems.push("short reference metadata is incomplete".into());
    }
    if reference["registry_digest"].as_str() != Some(registry_digest.as_str()) {
        problems.push("short reference registry digest is stale".into());
    }
    let Some(campaigns) = reference["campaigns"].as_array() else {
        problems.push("short reference campaigns must be an array".into());
        return Ok(());
    };
    if campaigns.len() != 2 {
        problems.push("short reference must contain exactly two campaigns".into());
        return Ok(());
    }
    for (index, campaign) in campaigns.iter().enumerate() {
        let found: BTreeSet<String> = campaign
            .as_object()
            .into_iter()
            .flat_map(|values| values.keys().cloned())
            .collect();
        if &found != expected {
            problems.push(format!(
                "short reference campaign {} matrix drift",
                index + 1
            ));
        }
        if campaign
            .as_object()
            .into_iter()
            .flat_map(|values| values.values())
            .any(|value| {
                value["throughput"].as_u64().is_none_or(|value| value == 0)
                    || value["latency"].as_u64().is_none_or(|value| value == 0)
            })
        {
            problems.push(format!(
                "short reference campaign {} has an invalid sample",
                index + 1
            ));
        }
    }
    let mut maximum_delta = 0.0_f64;
    for id in expected {
        for metric in ["throughput", "latency"] {
            let first = campaigns[0][id][metric].as_u64().unwrap_or(0) as f64;
            let second = campaigns[1][id][metric].as_u64().unwrap_or(0) as f64;
            if first > 0.0 && second > 0.0 {
                maximum_delta =
                    maximum_delta.max((first - second).abs() / first.max(second) * 100.0);
            }
        }
    }
    let declared = reference["maximum_pair_delta_percent"]
        .as_f64()
        .unwrap_or(f64::INFINITY);
    if (declared - maximum_delta).abs() > 0.11 {
        problems.push(format!("short reference maximum delta is stale: declared={declared:.1}, measured={maximum_delta:.1}"));
    }
    if maximum_delta > 75.0 || reference["comparison_tolerance_percent"].as_u64() != Some(75) {
        problems.push("short reference exceeds the 75 percent comparison tolerance".into());
    }
    Ok(())
}

fn validate_report(text: &str, registry_digest: &str, registry: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    let mut expected_sequence = 0_u64;
    let mut header = 0_u64;
    let mut terminal = 0_u64;
    let mut cases = BTreeSet::new();
    let mut case_samples = BTreeMap::<String, Vec<Value>>::new();
    let mut profile = None;
    let mut last_progress_seconds = 0_u64;
    for (index, line) in text.lines().enumerate() {
        let value: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(error) => {
                problems.push(format!(
                    "report line {} is invalid JSON: {error}",
                    index + 1
                ));
                continue;
            }
        };
        if value["schema_version"].as_u64() != Some(1) {
            problems.push(format!("report line {} has an unknown schema", index + 1));
        }
        if value["sequence"].as_u64() != Some(expected_sequence) {
            problems.push(format!("report line {} breaks sequence", index + 1));
        }
        expected_sequence = expected_sequence.saturating_add(1);
        match value["kind"].as_str() {
            Some("campaign.header") => {
                header += 1;
                profile = value["profile"].as_str().map(str::to_string);
                if value["registry_digest"].as_str() != Some(registry_digest) {
                    problems.push("report header registry digest is stale".into());
                }
                for field in [
                    "profile",
                    "product_version",
                    "source_revision",
                    "operating_system",
                    "architecture",
                    "build_profile",
                    "timer",
                    "comparability_class",
                ] {
                    if value[field].as_str().is_none_or(str::is_empty) {
                        problems.push(format!("report header requires {field}"));
                    }
                }
                if value["logical_cpu_count"]
                    .as_u64()
                    .is_none_or(|value| value == 0)
                {
                    problems.push("report header requires logical_cpu_count".into());
                }
            }
            Some("case.sample") => {
                if let Some(id) = value["case_id"].as_str() {
                    case_samples
                        .entry(id.to_string())
                        .or_default()
                        .push(value.clone());
                } else {
                    problems.push("report sample requires case_id".into());
                }
                if value["metrics_available"].as_bool() != Some(true) {
                    problems.push("report sample lacks required process metrics".into());
                }
                for field in [
                    "useful_bytes",
                    "direct_microseconds",
                    "proxy_microseconds",
                    "throughput_bytes_per_second",
                    "cpu_microseconds",
                    "peak_working_set_bytes",
                    "private_bytes",
                    "artifact_bytes",
                    "payload_bytes_observed",
                    "payload_bytes_retained",
                    "payload_bytes_omitted",
                    "payload_bytes_queue_dropped",
                    "payload_bytes_storage_dropped",
                    "queue_peak",
                    "queue_current",
                    "task_peak",
                    "task_current",
                    "task_spawned",
                    "task_completed",
                    "task_aborted",
                    "cache_peak_entries",
                    "cache_peak_bytes",
                    "failure_details_dropped",
                    "application_events_dropped",
                    "shutdown_microseconds",
                ] {
                    if value[field].as_u64().is_none() {
                        problems.push(format!("report sample requires numeric {field}"));
                    }
                }
            }
            Some("campaign.sample") => {
                let elapsed = value["elapsed_seconds"].as_u64().unwrap_or(0);
                if elapsed == 0 || elapsed.saturating_sub(last_progress_seconds) > 60 {
                    problems.push("soak report exceeds the 60-second progress cadence".into());
                }
                last_progress_seconds = elapsed;
            }
            Some("case.terminal") => {
                if let Some(id) = value["case_id"].as_str() {
                    if !cases.insert(id.to_string()) && profile.as_deref() != Some("soak") {
                        problems.push(format!("report duplicates terminal case {id}"));
                    }
                    let case = registry["cases"].as_array().and_then(|cases| {
                        cases.iter().find(|case| case["id"].as_str() == Some(id))
                    });
                    if let Some(case) = case {
                        validate_case_terminal(
                            id,
                            &value,
                            case_samples.get(id).map(Vec::as_slice).unwrap_or_default(),
                            case,
                            &registry["evaluation"],
                            &mut problems,
                        );
                    } else {
                        problems.push(format!("report terminal has unknown case {id}"));
                    }
                }
                if value["passed"].as_bool() != Some(true) {
                    problems.push("report contains a failed case".into());
                }
                if value["conservation_equation"].as_str().is_none() {
                    problems.push("report case terminal lacks its conservation equation".into());
                }
            }
            Some("campaign.terminal") => {
                terminal += 1;
                if value["complete"].as_bool() != Some(true)
                    || value["passed"].as_bool() != Some(true)
                    || value["registry_digest"].as_str() != Some(registry_digest)
                {
                    problems.push("report campaign terminal is not complete and passing".into());
                }
                let private_span = case_samples
                    .values()
                    .map(|samples| {
                        let values = samples
                            .iter()
                            .filter_map(|sample| sample["private_bytes"].as_u64())
                            .collect::<Vec<_>>();
                        values
                            .iter()
                            .max()
                            .zip(values.iter().min())
                            .map_or(0, |(maximum, minimum)| maximum.saturating_sub(*minimum))
                    })
                    .max()
                    .unwrap_or(0);
                if value["private_memory_span_bytes"].as_u64() != Some(private_span)
                    || private_span
                        > registry["evaluation"]["maximum_private_memory_growth_bytes"]
                            .as_u64()
                            .unwrap_or(0)
                {
                    problems.push(
                        "report campaign private-memory span disagrees with samples or exceeds its budget"
                            .into(),
                    );
                }
                if profile.as_deref() == Some("soak") {
                    let duration = value["duration_seconds"].as_u64().unwrap_or(0);
                    if duration < 7_200 {
                        problems.push("soak report is shorter than 7200 seconds".into());
                    }
                    if duration.saturating_sub(last_progress_seconds) > 60 {
                        problems
                            .push("soak terminal exceeds the 60-second progress cadence".into());
                    }
                }
            }
            _ => problems.push(format!("report line {} has an unknown kind", index + 1)),
        }
    }
    if header != 1 || terminal != 1 || cases.len() != 14 {
        problems.push(format!(
            "report structure requires one header, fourteen cases, and one terminal: header={header}, cases={}, terminal={terminal}",
            cases.len()
        ));
    }
    let expected: BTreeSet<String> = PROTOCOLS
        .iter()
        .flat_map(|protocol| {
            RETENTION
                .iter()
                .map(move |retention| format!("{protocol}-{retention}"))
        })
        .collect();
    if cases != expected {
        problems.push(format!(
            "report case matrix drift: expected={expected:?}, found={cases:?}"
        ));
    }
    for id in &expected {
        let count = case_samples.get(id).map_or(0, Vec::len);
        let valid = if profile.as_deref() == Some("soak") {
            count >= 7 && count % 7 == 0
        } else {
            matches!(count, 7 | 14)
        };
        if !valid {
            problems.push(format!(
                "report case {id} requires seven windows or one seven-window retry"
            ));
        }
    }
    problems
}

#[derive(Clone, Copy)]
struct ReportCaseEvaluation {
    passed: bool,
    median_throughput: u64,
    median_ratio_basis_points: u64,
    added_p95_microseconds: u64,
    timing_breaching_windows: u64,
    hard_invariant_failures: u64,
    guard_band: bool,
}

fn validate_case_terminal(
    id: &str,
    terminal: &Value,
    all_samples: &[Value],
    case: &Value,
    limits: &Value,
    problems: &mut Vec<String>,
) {
    let windows = terminal["windows"].as_u64().unwrap_or(0) as usize;
    if windows == 0 || all_samples.len() < windows {
        problems.push(format!(
            "report case {id} terminal has no complete sample window"
        ));
        return;
    }
    let evaluation =
        evaluate_report_case(&all_samples[all_samples.len() - windows..], case, limits);
    for (field, expected) in [
        (
            "median_throughput_bytes_per_second",
            evaluation.median_throughput,
        ),
        (
            "median_throughput_ratio_basis_points",
            evaluation.median_ratio_basis_points,
        ),
        ("added_p95_microseconds", evaluation.added_p95_microseconds),
        (
            "timing_breaching_windows",
            evaluation.timing_breaching_windows,
        ),
        (
            "hard_invariant_failures",
            evaluation.hard_invariant_failures,
        ),
    ] {
        if terminal[field].as_u64() != Some(expected) {
            problems.push(format!(
                "report case {id} terminal {field} disagrees with its samples"
            ));
        }
    }
    if terminal["guard_band_terminal"].as_bool() != Some(evaluation.guard_band) {
        problems.push(format!(
            "report case {id} terminal guard band disagrees with its samples"
        ));
    }
    if !evaluation.passed || evaluation.guard_band {
        problems.push(format!(
            "report case {id} samples do not satisfy the canonical budget"
        ));
    }
}

fn evaluate_report_case(samples: &[Value], case: &Value, limits: &Value) -> ReportCaseEvaluation {
    let number = |sample: &Value, field: &str| sample[field].as_u64().unwrap_or(u64::MAX);
    let rate = |sample: &Value| {
        number(sample, "useful_bytes").saturating_mul(1_000_000)
            / number(sample, "proxy_microseconds").max(1)
    };
    let ratio = |sample: &Value| {
        number(sample, "direct_microseconds").saturating_mul(10_000)
            / number(sample, "proxy_microseconds").max(1)
    };
    let added = |sample: &Value| {
        number(sample, "proxy_microseconds").saturating_sub(number(sample, "direct_microseconds"))
    };
    let mut throughput = samples.iter().map(rate).collect::<Vec<_>>();
    let mut ratios = samples.iter().map(ratio).collect::<Vec<_>>();
    let mut added_values = samples.iter().map(added).collect::<Vec<_>>();
    throughput.sort_unstable();
    ratios.sort_unstable();
    added_values.sort_unstable();
    let median = |values: &[u64]| values.get(values.len() / 2).copied().unwrap_or(0);
    let median_throughput = median(&throughput);
    let median_ratio_basis_points = median(&ratios);
    let p95_index = samples
        .len()
        .saturating_mul(95)
        .div_ceil(100)
        .saturating_sub(1);
    let added_p95_microseconds = added_values.get(p95_index).copied().unwrap_or(u64::MAX);
    let throughput_floor = case["minimum_throughput_bytes_per_second"]
        .as_u64()
        .unwrap_or(u64::MAX);
    let ratio_floor = case["minimum_throughput_ratio_basis_points"]
        .as_u64()
        .unwrap_or(u64::MAX);
    let added_ceiling = case["maximum_added_p95_microseconds"].as_u64().unwrap_or(0);
    let timing_breaching_windows = samples
        .iter()
        .filter(|sample| {
            rate(sample) < throughput_floor
                || ratio(sample) < ratio_floor
                || added(sample) > added_ceiling
        })
        .count() as u64;
    let shutdown_ceiling = limits["maximum_shutdown_milliseconds"]
        .as_u64()
        .unwrap_or(0)
        .saturating_mul(1000);
    let useful = case["useful_bytes_per_window"].as_u64().unwrap_or(0);
    let hard_invariant_failures = samples
        .iter()
        .filter(|sample| {
            let observed = number(sample, "payload_bytes_observed");
            let retained = number(sample, "payload_bytes_retained");
            let omitted = number(sample, "payload_bytes_omitted");
            let queue_dropped = number(sample, "payload_bytes_queue_dropped");
            let storage_dropped = number(sample, "payload_bytes_storage_dropped");
            sample["success"].as_bool() != Some(true)
                || number(sample, "useful_bytes") != useful
                || number(sample, "throughput_bytes_per_second") != rate(sample)
                || observed < useful
                || observed
                    != retained
                        .saturating_add(omitted)
                        .saturating_add(queue_dropped)
                        .saturating_add(storage_dropped)
                || queue_dropped != 0
                || storage_dropped != 0
                || (case["retention"].as_str() == Some("off") && retained != 0)
                || (case["retention"].as_str() == Some("on") && retained == 0)
                || number(sample, "cpu_microseconds").saturating_mul(1024 * 1024) / useful.max(1)
                    > number(limits, "maximum_cpu_microseconds_per_mib")
                || number(sample, "peak_working_set_bytes").max(number(sample, "private_bytes"))
                    > number(limits, "maximum_worker_memory_bytes")
                || number(sample, "artifact_bytes") > number(limits, "maximum_artifact_bytes")
                || number(sample, "shutdown_microseconds") > shutdown_ceiling
                || sample["clean_shutdown"].as_bool() != Some(true)
                || number(sample, "task_peak") > number(limits, "maximum_worker_tasks")
                || number(sample, "task_current") != 0
                || number(sample, "task_spawned")
                    != number(sample, "task_completed")
                        .saturating_add(number(sample, "task_aborted"))
                        .saturating_add(number(sample, "task_current"))
                || number(sample, "cache_peak_entries")
                    > number(limits, "maximum_leaf_cache_entries")
                || number(sample, "cache_peak_bytes") > number(limits, "maximum_leaf_cache_bytes")
                || number(sample, "queue_peak") > number(limits, "maximum_application_queue")
                || number(sample, "queue_current") != 0
                || number(sample, "failure_details_dropped") != 0
                || number(sample, "application_events_dropped") != 0
        })
        .count() as u64;
    let minimum_breaches = limits["minimum_breaching_windows"].as_u64().unwrap_or(0);
    let timing_passed = !((median_throughput < throughput_floor
        || median_ratio_basis_points < ratio_floor
        || added_p95_microseconds > added_ceiling)
        && timing_breaching_windows >= minimum_breaches);
    let guard_percent = limits["guard_band_percent"].as_u64().unwrap_or(0);
    let near = |value: u64, threshold: u64| {
        value.abs_diff(threshold) <= threshold.saturating_mul(guard_percent) / 100
    };
    ReportCaseEvaluation {
        passed: timing_passed && hard_invariant_failures == 0,
        median_throughput,
        median_ratio_basis_points,
        added_p95_microseconds,
        timing_breaching_windows,
        hard_invariant_failures,
        guard_band: near(median_throughput, throughput_floor)
            || near(median_ratio_basis_points, ratio_floor)
            || near(added_p95_microseconds, added_ceiling),
    }
}

fn stable_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn validate_runtime_inventory(root: &Path, problems: &mut Vec<String>) -> io::Result<()> {
    let model = fs::read_to_string(root.join("crates/fragcap-proxy/src/model.rs"))?;
    let application = fs::read_to_string(root.join("crates/fragcap-proxy/src/application.rs"))?;
    let writer = fs::read_to_string(root.join("crates/fragcap/src/deep_capture/application.rs"))?;
    let runtime = fs::read_to_string(root.join("crates/fragcap-proxy/src/runtime.rs"))?;
    let certificate = fs::read_to_string(root.join("crates/fragcap-proxy/src/certificate.rs"))?;
    for (name, source) in [
        ("failure_details_dropped_oldest", model.as_str()),
        ("connection_tasks_current", model.as_str()),
        ("leaf_cache_peak_bytes", model.as_str()),
        ("queue_current", application.as_str()),
        ("queue_current.fetch_sub", writer.as_str()),
        ("push_runtime_failure", runtime.as_str()),
        ("peak_entries", certificate.as_str()),
    ] {
        if !source.contains(name) {
            problems.push(format!("runtime performance inventory is missing {name}"));
        }
    }
    Ok(())
}

fn required_string(value: &Value, field: &str, owner: &str, problems: &mut Vec<String>) -> String {
    match value[field].as_str().filter(|value| !value.is_empty()) {
        Some(value) => value.to_string(),
        None => {
            problems.push(format!("{owner} requires non-empty {field}"));
            String::new()
        }
    }
}

fn positive(value: &Value, field: &str, owner: &str, problems: &mut Vec<String>) {
    if value[field].as_u64().is_none_or(|value| value == 0) {
        problems.push(format!("{owner} requires positive {field}"));
    }
}

fn exact_strings(value: &Value, field: &str, expected: &[&str], problems: &mut Vec<String>) {
    let observed: BTreeSet<&str> = value[field]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let expected: BTreeSet<&str> = expected.iter().copied().collect();
    if observed != expected {
        problems.push(format!(
            "{field} drift: expected={expected:?}, found={observed:?}"
        ));
    }
}

fn safe_relative(value: &str) -> bool {
    !value.is_empty()
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_registry_is_complete() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let value: Value = serde_json::from_slice(&fs::read(root.join(REGISTRY)).unwrap()).unwrap();
        assert!(validate(root, &value).unwrap().is_empty());
    }

    #[test]
    fn missing_case_cannot_pass() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let mut value: Value =
            serde_json::from_slice(&fs::read(root.join(REGISTRY)).unwrap()).unwrap();
        value["cases"].as_array_mut().unwrap().pop();
        assert!(validate(root, &value)
            .unwrap()
            .iter()
            .any(|problem| problem.contains("case matrix drift")));
    }

    #[test]
    fn report_requires_complete_monotonic_evidence() {
        let registry: Value = serde_json::from_str(include_str!(
            "../../performance/native-proxy-budgets-v1.json"
        ))
        .unwrap();
        let mut sequence = 0_u64;
        let mut records = vec![serde_json::json!({
            "schema_version": 1,
            "kind": "campaign.header",
            "sequence": sequence,
            "profile": "short",
            "registry_digest": "digest",
            "product_version": "0.8.0",
            "source_revision": "revision",
            "operating_system": "windows",
            "architecture": "x86_64",
            "logical_cpu_count": 8,
            "build_profile": "release",
            "timer": "monotonic",
            "comparability_class": "class"
        })];
        for protocol in PROTOCOLS {
            for retention in RETENTION {
                let id = format!("{protocol}-{retention}");
                let case = registry["cases"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|case| case["id"].as_str() == Some(id.as_str()))
                    .unwrap();
                let useful = case["useful_bytes_per_window"].as_u64().unwrap();
                for _ in 0..7 {
                    sequence += 1;
                    records.push(serde_json::json!({
                        "schema_version":1,"kind":"case.sample","sequence":sequence,
                        "case_id":id,"metrics_available":true,"useful_bytes":useful,
                        "direct_microseconds":90_000,"proxy_microseconds":100_000,
                        "throughput_bytes_per_second":useful * 10,"cpu_microseconds":0,
                        "peak_working_set_bytes":1,"private_bytes":1,"artifact_bytes":1,
                        "payload_bytes_observed":useful,
                        "payload_bytes_retained":if *retention == "on" { useful } else { 0 },
                        "payload_bytes_omitted":if *retention == "off" { useful } else { 0 },
                        "payload_bytes_queue_dropped":0,"payload_bytes_storage_dropped":0,
                        "queue_peak":0,"queue_current":0,"task_peak":1,"task_current":0,
                        "task_spawned":1,"task_completed":1,"task_aborted":0,
                        "cache_peak_entries":0,"cache_peak_bytes":0,
                        "failure_details_dropped":0,"application_events_dropped":0,
                        "shutdown_microseconds":1,"success":true,"clean_shutdown":true
                    }));
                }
                sequence += 1;
                records.push(serde_json::json!({
                    "schema_version":1,"kind":"case.terminal","sequence":sequence,
                    "case_id":id,"passed":true,"attempts":1,"windows":7,
                    "median_throughput_bytes_per_second":useful * 10,
                    "median_throughput_ratio_basis_points":9000,
                    "added_p95_microseconds":10_000,"timing_breaching_windows":0,
                    "hard_invariant_failures":0,"guard_band_terminal":false,
                    "conservation_equation":"observed = terminal"
                }));
            }
        }
        sequence += 1;
        records.push(serde_json::json!({
            "schema_version":1,"kind":"campaign.terminal","sequence":sequence,
            "complete":true,"passed":true,"registry_digest":"digest",
            "private_memory_span_bytes":0
        }));
        let text = records
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        let problems = validate_report(&text, "digest", &registry);
        assert!(problems.is_empty(), "{problems:?}");

        records[1]["throughput_bytes_per_second"] = Value::from(1);
        let fabricated = records
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(validate_report(&fabricated, "digest", &registry)
            .iter()
            .any(|problem| problem.contains("canonical budget")));
        records[1]["throughput_bytes_per_second"] =
            Value::from(records[1]["useful_bytes"].as_u64().unwrap() * 10);

        records[1]["sequence"] = Value::from(99);
        let corrupt = records
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(validate_report(&corrupt, "digest", &registry)
            .iter()
            .any(|problem| problem.contains("breaks sequence")));
    }

    #[test]
    fn soak_report_rejects_progress_and_terminal_gaps_over_sixty_seconds() {
        let registry: Value = serde_json::from_str(include_str!(
            "../../performance/native-proxy-budgets-v1.json"
        ))
        .unwrap();
        let report = [
            serde_json::json!({
                "schema_version":1,"kind":"campaign.header","sequence":0,
                "profile":"soak","registry_digest":"digest","product_version":"0.8.0",
                "source_revision":"revision","operating_system":"windows",
                "architecture":"x86_64","logical_cpu_count":8,"build_profile":"release",
                "timer":"monotonic","comparability_class":"class"
            }),
            serde_json::json!({
                "schema_version":1,"kind":"campaign.sample","sequence":1,
                "elapsed_seconds":61
            }),
            serde_json::json!({
                "schema_version":1,"kind":"campaign.terminal","sequence":2,
                "complete":true,"passed":true,"registry_digest":"digest",
                "duration_seconds":122
            }),
        ]
        .iter()
        .map(Value::to_string)
        .collect::<Vec<_>>()
        .join("\n");
        let problems = validate_report(&report, "digest", &registry);
        assert!(problems
            .iter()
            .any(|problem| problem == "soak report exceeds the 60-second progress cadence"));
        assert!(problems
            .iter()
            .any(|problem| problem == "soak terminal exceeds the 60-second progress cadence"));
    }
}
