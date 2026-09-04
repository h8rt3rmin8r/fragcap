// SPDX-License-Identifier: Apache-2.0

//! Isolated controller for the native proxy performance campaigns.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

mod metrics;
mod workloads;

const REGISTRY: &str = "../native-proxy-budgets-v1.json";
const WORKER_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone, Copy)]
struct Driver {
    name: &'static str,
}

fn driver(protocol: &str) -> Option<Driver> {
    Some(match protocol {
        "http1" => Driver { name: "http1" },
        "http2" => Driver { name: "http2" },
        "websocket" => Driver { name: "websocket" },
        "grpc" => Driver { name: "grpc" },
        "tcp" => Driver { name: "tcp" },
        "udp" => Driver { name: "udp" },
        "quic" => Driver { name: "quic" },
        _ => return None,
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(error) => {
            eprintln!("native-performance: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> io::Result<bool> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--worker") {
        return worker(&args);
    }
    let profile = argument(&args, "--profile").unwrap_or("short");
    let output =
        argument(&args, "--output").ok_or_else(|| io::Error::other("--output is required"))?;
    if !matches!(profile, "short" | "soak") {
        return Err(io::Error::other("--profile must be short or soak"));
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.parent().and_then(Path::parent).ok_or_else(|| {
        io::Error::other("performance harness must remain under the repository root")
    })?;
    let registry_bytes = fs::read(manifest.join(REGISTRY))?;
    let registry: Value = serde_json::from_slice(&registry_bytes).map_err(io::Error::other)?;
    prepare(root)?;
    let profile_value = &registry["profiles"][profile];
    let windows = profile_value["windows"]
        .as_u64()
        .ok_or_else(|| io::Error::other("profile windows missing"))?;
    let warmup_windows = profile_value["warmup_windows"]
        .as_u64()
        .ok_or_else(|| io::Error::other("profile warmup_windows missing"))?;
    let sample_interval = Duration::from_secs(
        profile_value["sample_seconds"]
            .as_u64()
            .ok_or_else(|| io::Error::other("profile sample_seconds missing"))?,
    );
    let progress_interval = sample_interval.saturating_sub(WORKER_TIMEOUT);
    let minimum_duration = Duration::from_secs(
        profile_value["minimum_duration_seconds"]
            .as_u64()
            .unwrap_or(0),
    );
    let output = PathBuf::from(output);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut writer = BufWriter::new(File::create(&output)?);
    let registry_digest = stable_digest(&registry_bytes);
    let expected_cases: Vec<&str> = registry["cases"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|case| case["id"].as_str())
        .collect();
    write_line(
        &mut writer,
        &json!({"schema_version":1,"kind":"campaign.header","sequence":0,"profile":profile,"registry_digest":registry_digest,"product_version":product_version(root),"source_revision":command_text(root,"git", &["rev-parse","HEAD"]),"source_dirty":!command_success(root,"git", &["diff","--quiet"]),"operating_system":std::env::consts::OS,"architecture":std::env::consts::ARCH,"logical_cpu_count":std::thread::available_parallelism().map_or(0,usize::from),"build_profile":if cfg!(debug_assertions){"debug"}else{"release"},"timer":"std::time::Instant monotonic microseconds","comparability_class":format!("{}:{}:{}:{}",std::env::consts::OS,std::env::consts::ARCH,if cfg!(debug_assertions){"debug"}else{"release"},registry_digest),"expected_cases":expected_cases,"started_unix_ms":unix_ms()}),
    )?;
    let started = Instant::now();
    let mut sequence = 0_u64;
    let mut observed = BTreeSet::new();
    let mut passed = true;
    let mut private_ranges = BTreeMap::<String, (u64, u64)>::new();
    let mut last_campaign_sample = Instant::now();
    loop {
        for case in registry["cases"]
            .as_array()
            .ok_or_else(|| io::Error::other("cases missing"))?
        {
            let id = case["id"]
                .as_str()
                .ok_or_else(|| io::Error::other("case id missing"))?;
            let protocol = case["protocol"]
                .as_str()
                .ok_or_else(|| io::Error::other("case protocol missing"))?;
            let retention = case["retention"]
                .as_str()
                .ok_or_else(|| io::Error::other("case retention missing"))?;
            let selected = driver(protocol)
                .ok_or_else(|| io::Error::other(format!("no driver for {protocol}")))?;
            let maximum_retries = registry["evaluation"]["maximum_retries"]
                .as_u64()
                .unwrap_or(0);
            for _ in 0..warmup_windows {
                let _ = run_measurement(root, selected, protocol, retention)?;
            }
            let mut attempt = 0_u64;
            let case_passed = loop {
                attempt += 1;
                let mut samples = Vec::new();
                let mut attempt_passed = true;
                for window in 0..windows {
                    let result = run_measurement(root, selected, protocol, retention)?;
                    let range = private_ranges
                        .entry(id.to_string())
                        .or_insert((u64::MAX, 0));
                    range.0 = range.0.min(result.process.private_bytes);
                    range.1 = range.1.max(result.process.private_bytes);
                    sequence += 1;
                    write_line(
                        &mut writer,
                        &json!({"schema_version":1,"kind":"case.sample","sequence":sequence,"case_id":id,"driver":selected.name,"attempt":attempt,"window":window + 1,"direct_microseconds":result.direct_microseconds,"proxy_microseconds":result.proxy_microseconds,"added_microseconds":result.proxy_microseconds.saturating_sub(result.direct_microseconds),"useful_bytes":result.useful_bytes,"throughput_bytes_per_second":rate(result.useful_bytes,result.proxy_microseconds),"metrics_available":result.process.available,"cpu_microseconds":result.process.cpu_microseconds,"peak_working_set_bytes":result.process.working_set_bytes,"private_bytes":result.process.private_bytes,"artifact_bytes":result.artifact_bytes,"payload_bytes_observed":result.payload_bytes_observed,"payload_bytes_retained":result.payload_bytes_retained,"payload_bytes_omitted":result.payload_bytes_omitted,"payload_bytes_queue_dropped":result.payload_bytes_queue_dropped,"payload_bytes_storage_dropped":result.payload_bytes_storage_dropped,"shutdown_microseconds":result.shutdown_microseconds,"clean_shutdown":result.clean_shutdown,"task_peak":result.task_peak,"task_current":result.task_current,"task_spawned":result.task_spawned,"task_completed":result.task_completed,"task_aborted":result.task_aborted,"cache_peak_entries":result.cache_peak_entries,"cache_peak_bytes":result.cache_peak_bytes,"queue_peak":result.queue_peak,"queue_current":result.queue_current,"failure_details_dropped":result.failure_details_dropped,"application_events_dropped":result.application_events_dropped,"success":result.success}),
                    )?;
                    if profile == "soak" && last_campaign_sample.elapsed() >= progress_interval {
                        sequence += 1;
                        write_line(
                            &mut writer,
                            &json!({"schema_version":1,"kind":"campaign.sample","sequence":sequence,"elapsed_seconds":started.elapsed().as_secs()}),
                        )?;
                        last_campaign_sample = Instant::now();
                    }
                    samples.push(result);
                    attempt_passed &= result.success;
                }
                attempt_passed &= samples.len() == windows as usize;
                let evaluation = evaluate_case(case, &registry["evaluation"], &samples);
                attempt_passed &= evaluation.passed;
                if should_retry(&evaluation, attempt, maximum_retries) {
                    continue;
                }
                attempt_passed &= !evaluation.guard_band;
                sequence += 1;
                write_line(
                    &mut writer,
                    &json!({"schema_version":1,"kind":"case.terminal","sequence":sequence,"case_id":id,"protocol":protocol,"retention":retention,"attempts":attempt,"windows":samples.len(),"median_throughput_bytes_per_second":evaluation.median_throughput,"median_throughput_ratio_basis_points":evaluation.median_ratio_basis_points,"added_p95_microseconds":evaluation.added_p95_microseconds,"timing_breaching_windows":evaluation.timing_breaching_windows,"hard_invariant_failures":evaluation.hard_invariant_failures,"guard_band_terminal":evaluation.guard_band,"passed":attempt_passed,"measurement":"paired useful-payload direct and production-proxy exchange with application artifact writer","conservation_equation":"payload_bytes_observed = payload_bytes_retained + payload_bytes_omitted + payload_bytes_queue_dropped + payload_bytes_storage_dropped"}),
                )?;
                break attempt_passed;
            };
            observed.insert(id.to_string());
            passed &= case_passed;
        }
        if profile == "short" || started.elapsed() >= minimum_duration {
            break;
        }
    }
    let private_memory_span_bytes = private_ranges
        .values()
        .map(|(minimum, maximum)| maximum.saturating_sub(*minimum))
        .max()
        .unwrap_or(0);
    passed &= private_memory_span_bytes
        <= registry["evaluation"]["maximum_private_memory_growth_bytes"]
            .as_u64()
            .unwrap_or(0);
    sequence += 1;
    write_line(
        &mut writer,
        &json!({"schema_version":1,"kind":"campaign.terminal","sequence":sequence,"profile":profile,"registry_digest":registry_digest,"duration_seconds":started.elapsed().as_secs(),"expected_cases":expected_cases,"observed_cases":observed,"private_memory_span_bytes":private_memory_span_bytes,"complete":observed.len()==14 && (profile=="short" || started.elapsed()>=minimum_duration),"passed":passed,"ended_unix_ms":unix_ms()}),
    )?;
    writer.flush()?;
    Ok(passed)
}

fn worker(args: &[String]) -> io::Result<bool> {
    let protocol = args
        .get(2)
        .ok_or_else(|| io::Error::other("worker protocol missing"))?;
    let retention = args
        .get(3)
        .ok_or_else(|| io::Error::other("worker retention missing"))?;
    let result = workloads::measure(protocol, retention)?;
    println!(
        "{}",
        json!({"schema_version":1,"kind":"worker.terminal","direct_microseconds":result.direct_microseconds,"proxy_microseconds":result.proxy_microseconds,"useful_bytes":result.useful_bytes,"artifact_bytes":result.resources.artifact_bytes,"payload_bytes_observed":result.resources.payload_bytes_observed,"payload_bytes_retained":result.resources.payload_bytes_retained,"payload_bytes_omitted":result.resources.payload_bytes_omitted,"payload_bytes_queue_dropped":result.resources.payload_bytes_queue_dropped,"payload_bytes_storage_dropped":result.resources.payload_bytes_storage_dropped,"shutdown_microseconds":result.shutdown_microseconds,"clean_shutdown":result.clean_shutdown,"task_peak":result.resources.task_peak,"task_current":result.resources.task_current,"task_spawned":result.resources.task_spawned,"task_completed":result.resources.task_completed,"task_aborted":result.resources.task_aborted,"cache_peak_entries":result.resources.cache_peak_entries,"cache_peak_bytes":result.resources.cache_peak_bytes,"queue_peak":result.resources.queue_peak,"queue_current":result.resources.queue_current,"failure_details_dropped":result.resources.failure_details_dropped,"application_events_dropped":result.resources.application_events_dropped})
    );
    Ok(result.clean_shutdown)
}

#[derive(Clone, Copy, Default)]
struct WorkerResult {
    success: bool,
    direct_microseconds: u64,
    proxy_microseconds: u64,
    useful_bytes: u64,
    shutdown_microseconds: u64,
    clean_shutdown: bool,
    task_peak: u64,
    task_current: u64,
    task_spawned: u64,
    task_completed: u64,
    task_aborted: u64,
    cache_peak_entries: u64,
    cache_peak_bytes: u64,
    queue_peak: u64,
    queue_current: u64,
    failure_details_dropped: u64,
    application_events_dropped: u64,
    artifact_bytes: u64,
    payload_bytes_observed: u64,
    payload_bytes_retained: u64,
    payload_bytes_omitted: u64,
    payload_bytes_queue_dropped: u64,
    payload_bytes_storage_dropped: u64,
    process: metrics::ProcessSample,
}

struct CaseEvaluation {
    passed: bool,
    median_throughput: u64,
    median_ratio_basis_points: u64,
    added_p95_microseconds: u64,
    timing_breaching_windows: u64,
    hard_invariant_failures: u64,
    guard_band: bool,
}

fn evaluate_case(case: &Value, limits: &Value, samples: &[WorkerResult]) -> CaseEvaluation {
    let floor = case["minimum_throughput_bytes_per_second"]
        .as_u64()
        .unwrap_or(u64::MAX);
    let ratio_floor = case["minimum_throughput_ratio_basis_points"]
        .as_u64()
        .unwrap_or(u64::MAX);
    let added_ceiling = case["maximum_added_p95_microseconds"].as_u64().unwrap_or(0);
    let mut throughput: Vec<u64> = samples
        .iter()
        .map(|sample| rate(sample.useful_bytes, sample.proxy_microseconds))
        .collect();
    let mut ratios: Vec<u64> = samples
        .iter()
        .map(|sample| {
            sample.direct_microseconds.saturating_mul(10_000) / sample.proxy_microseconds.max(1)
        })
        .collect();
    let mut added: Vec<u64> = samples
        .iter()
        .map(|sample| {
            sample
                .proxy_microseconds
                .saturating_sub(sample.direct_microseconds)
        })
        .collect();
    throughput.sort_unstable();
    ratios.sort_unstable();
    added.sort_unstable();
    let median = |values: &[u64]| values.get(values.len() / 2).copied().unwrap_or(0);
    let median_throughput = median(&throughput);
    let median_ratio_basis_points = median(&ratios);
    let p95_index = samples
        .len()
        .saturating_mul(95)
        .div_ceil(100)
        .saturating_sub(1);
    let added_p95_microseconds = added.get(p95_index).copied().unwrap_or(u64::MAX);
    let timing_breaching_windows = samples
        .iter()
        .filter(|sample| {
            rate(sample.useful_bytes, sample.proxy_microseconds) < floor
                || sample.direct_microseconds.saturating_mul(10_000)
                    / sample.proxy_microseconds.max(1)
                    < ratio_floor
                || sample
                    .proxy_microseconds
                    .saturating_sub(sample.direct_microseconds)
                    > added_ceiling
        })
        .count() as u64;
    let shutdown_ceiling = limits["maximum_shutdown_milliseconds"]
        .as_u64()
        .unwrap_or(0)
        .saturating_mul(1000);
    let task_ceiling = limits["maximum_worker_tasks"].as_u64().unwrap_or(0);
    let cache_entries = limits["maximum_leaf_cache_entries"].as_u64().unwrap_or(0);
    let cache_bytes = limits["maximum_leaf_cache_bytes"].as_u64().unwrap_or(0);
    let queue_ceiling = limits["maximum_application_queue"].as_u64().unwrap_or(0);
    let memory_ceiling = limits["maximum_worker_memory_bytes"].as_u64().unwrap_or(0);
    let artifact_ceiling = limits["maximum_artifact_bytes"].as_u64().unwrap_or(0);
    let cpu_per_mib_ceiling = limits["maximum_cpu_microseconds_per_mib"]
        .as_u64()
        .unwrap_or(0);
    let hard_invariant_failures = samples
        .iter()
        .filter(|sample| {
            !sample.success
                || sample.useful_bytes != case["useful_bytes_per_window"].as_u64().unwrap_or(0)
                || !sample.process.available
                || sample
                    .process
                    .private_bytes
                    .max(sample.process.working_set_bytes)
                    > memory_ceiling
                || sample.artifact_bytes > artifact_ceiling
                || sample.process.cpu_microseconds.saturating_mul(1024 * 1024)
                    / sample.useful_bytes.max(1)
                    > cpu_per_mib_ceiling
                || !sample.clean_shutdown
                || sample.shutdown_microseconds > shutdown_ceiling
                || sample.task_peak > task_ceiling
                || sample.task_current != 0
                || sample.task_spawned
                    != sample
                        .task_completed
                        .saturating_add(sample.task_aborted)
                        .saturating_add(sample.task_current)
                || sample.cache_peak_entries > cache_entries
                || sample.cache_peak_bytes > cache_bytes
                || sample.queue_peak > queue_ceiling
                || sample.queue_current != 0
                || sample.failure_details_dropped != 0
                || sample.application_events_dropped != 0
                || sample.payload_bytes_observed < sample.useful_bytes
                || sample.payload_bytes_queue_dropped != 0
                || sample.payload_bytes_storage_dropped != 0
                || (case["retention"].as_str() == Some("off") && sample.payload_bytes_retained != 0)
                || (case["retention"].as_str() == Some("on") && sample.payload_bytes_retained == 0)
                || sample.payload_bytes_observed
                    != sample
                        .payload_bytes_retained
                        .saturating_add(sample.payload_bytes_omitted)
                        .saturating_add(sample.payload_bytes_queue_dropped)
                        .saturating_add(sample.payload_bytes_storage_dropped)
        })
        .count() as u64;
    let minimum_breaches = limits["minimum_breaching_windows"].as_u64().unwrap_or(0);
    let timing_passed = !((median_throughput < floor
        || median_ratio_basis_points < ratio_floor
        || added_p95_microseconds > added_ceiling)
        && timing_breaching_windows >= minimum_breaches);
    let guard_percent = limits["guard_band_percent"].as_u64().unwrap_or(0);
    let guard_band = near_threshold(median_throughput, floor, guard_percent)
        || near_threshold(median_ratio_basis_points, ratio_floor, guard_percent)
        || near_threshold(added_p95_microseconds, added_ceiling, guard_percent);
    CaseEvaluation {
        passed: timing_passed && hard_invariant_failures == 0,
        median_throughput,
        median_ratio_basis_points,
        added_p95_microseconds,
        timing_breaching_windows,
        hard_invariant_failures,
        guard_band,
    }
}

fn near_threshold(value: u64, threshold: u64, percent: u64) -> bool {
    value.abs_diff(threshold) <= threshold.saturating_mul(percent) / 100
}

fn should_retry(evaluation: &CaseEvaluation, attempt: u64, maximum_retries: u64) -> bool {
    evaluation.guard_band && evaluation.hard_invariant_failures == 0 && attempt <= maximum_retries
}

fn run_measurement(
    _root: &Path,
    _driver: Driver,
    protocol: &str,
    retention: &str,
) -> io::Result<WorkerResult> {
    let mut command = Command::new(std::env::current_exe()?);
    command
        .args(["--worker", protocol, retention])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_windows(&mut command);
    let mut child = command.spawn()?;
    let mut process = metrics::sample(&child);
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            process.merge(metrics::sample(&child));
            break status;
        }
        process.merge(metrics::sample(&child));
        if started.elapsed() >= WORKER_TIMEOUT {
            child.kill()?;
            let _ = child.wait();
            return Err(io::Error::other(format!(
                "{protocol}-{retention} worker exceeded {} seconds",
                WORKER_TIMEOUT.as_secs()
            )));
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let mut output = String::new();
    child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("worker output unavailable"))?
        .read_to_string(&mut output)?;
    let value: Value = serde_json::from_str(output.trim()).map_err(|error| {
        let mut diagnostic = String::new();
        if let Some(mut stderr) = child.stderr.take() {
            let _ = stderr.read_to_string(&mut diagnostic);
        }
        io::Error::other(format!(
            "{protocol}-{retention} worker returned invalid output: {error}; {}",
            diagnostic.trim()
        ))
    })?;
    Ok(worker_result(status.success(), &value, process))
}

fn worker_result(success: bool, value: &Value, process: metrics::ProcessSample) -> WorkerResult {
    let number = |field| value[field].as_u64().unwrap_or(0);
    WorkerResult {
        success,
        direct_microseconds: number("direct_microseconds"),
        proxy_microseconds: number("proxy_microseconds"),
        useful_bytes: number("useful_bytes"),
        shutdown_microseconds: number("shutdown_microseconds"),
        clean_shutdown: value["clean_shutdown"].as_bool().unwrap_or(false),
        task_peak: number("task_peak"),
        task_current: number("task_current"),
        task_spawned: number("task_spawned"),
        task_completed: number("task_completed"),
        task_aborted: number("task_aborted"),
        cache_peak_entries: number("cache_peak_entries"),
        cache_peak_bytes: number("cache_peak_bytes"),
        queue_peak: number("queue_peak"),
        queue_current: number("queue_current"),
        failure_details_dropped: number("failure_details_dropped"),
        application_events_dropped: number("application_events_dropped"),
        artifact_bytes: number("artifact_bytes"),
        payload_bytes_observed: number("payload_bytes_observed"),
        payload_bytes_retained: number("payload_bytes_retained"),
        payload_bytes_omitted: number("payload_bytes_omitted"),
        payload_bytes_queue_dropped: number("payload_bytes_queue_dropped"),
        payload_bytes_storage_dropped: number("payload_bytes_storage_dropped"),
        process,
    }
}

fn rate(bytes: u64, microseconds: u64) -> u64 {
    bytes.saturating_mul(1_000_000) / microseconds.max(1)
}

fn prepare(root: &Path) -> io::Result<()> {
    let mut command = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    command
        .current_dir(root)
        .args([
            "test",
            "--release",
            "--locked",
            "-p",
            "fragcap-proxy",
            "--tests",
            "--no-run",
        ])
        .stdin(Stdio::null());
    hide_windows(&mut command);
    if command.status()?.success() {
        Ok(())
    } else {
        Err(io::Error::other(
            "production-path performance drivers did not compile",
        ))
    }
}

fn command_text(root: &Path, program: &str, args: &[&str]) -> String {
    let mut command = Command::new(program);
    command
        .current_dir(root)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null());
    hide_windows(&mut command);
    command
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|| "unavailable".to_string())
}

fn command_success(root: &Path, program: &str, args: &[&str]) -> bool {
    let mut command = Command::new(program);
    command
        .current_dir(root)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    hide_windows(&mut command);
    command.status().is_ok_and(|status| status.success())
}

fn product_version(root: &Path) -> String {
    fs::read_to_string(root.join("Cargo.toml"))
        .ok()
        .and_then(|text| {
            text.lines()
                .find_map(|line| {
                    let line = line.trim();
                    let (name, value) = line.split_once('=')?;
                    (name.trim() == "version").then(|| value.trim().trim_matches('"'))
                })
                .map(str::to_string)
        })
        .unwrap_or_else(|| "unavailable".to_string())
}

#[cfg(windows)]
fn hide_windows(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_windows(_: &mut Command) {}

fn argument<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}

fn write_line(writer: &mut impl Write, value: &Value) -> io::Result<()> {
    serde_json::to_writer(&mut *writer, value).map_err(io::Error::other)?;
    writer.write_all(b"\n")
}

fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

fn stable_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registry_protocol_has_a_real_driver() {
        for protocol in ["http1", "http2", "websocket", "grpc", "tcp", "udp", "quic"] {
            assert_eq!(driver(protocol).unwrap().name, protocol);
        }
    }

    #[test]
    fn digest_is_stable_and_sensitive() {
        assert_eq!(stable_digest(b"abc"), stable_digest(b"abc"));
        assert_ne!(stable_digest(b"abc"), stable_digest(b"abd"));
    }

    #[test]
    fn guard_band_is_exactly_symmetric() {
        assert!(near_threshold(95, 100, 5));
        assert!(near_threshold(105, 100, 5));
        assert!(!near_threshold(94, 100, 5));
        assert!(!near_threshold(106, 100, 5));
    }

    #[test]
    fn retention_off_cannot_pass_with_retained_payload() {
        let registry: Value =
            serde_json::from_str(include_str!("../../native-proxy-budgets-v1.json")).unwrap();
        let case = &registry["cases"][0];
        let sample = WorkerResult {
            success: true,
            direct_microseconds: 50_000,
            proxy_microseconds: 100_000,
            useful_bytes: case["useful_bytes_per_window"].as_u64().unwrap(),
            clean_shutdown: true,
            task_peak: 1,
            task_spawned: 1,
            task_completed: 1,
            artifact_bytes: 1,
            payload_bytes_observed: 1,
            payload_bytes_retained: 1,
            process: metrics::ProcessSample {
                available: true,
                ..metrics::ProcessSample::default()
            },
            ..WorkerResult::default()
        };
        let evaluation = evaluate_case(case, &registry["evaluation"], &[sample; 7]);
        assert!(!evaluation.passed);
        assert_eq!(evaluation.hard_invariant_failures, 7);
    }

    #[test]
    fn hard_invariant_failure_is_never_retried_through_a_timing_guard_band() {
        let evaluation = CaseEvaluation {
            passed: false,
            median_throughput: 100,
            median_ratio_basis_points: 100,
            added_p95_microseconds: 100,
            timing_breaching_windows: 0,
            hard_invariant_failures: 1,
            guard_band: true,
        };
        assert!(!should_retry(&evaluation, 1, 1));
    }
}
