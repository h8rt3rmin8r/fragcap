// SPDX-License-Identifier: Apache-2.0

//! The structured lifecycle events of specification section 17.5, and their
//! hand-rolled newline-delimited JSON.
//!
//! `serde_json` stays a test-only dependency by policy (slice S07), so the event
//! set, which is small and fixed, is serialized by hand over the sink crate's
//! JSON string escaper rather than by adding a serializer to the runtime graph.
//! Reusing that one escaper is what makes the event strings and the sink output
//! agree on escaping by construction.
//!
//! Each record carries an RFC3339 `Z` timestamp, formatted from a
//! [`SystemTime`] with a small civil-date conversion so no date crate is
//! pulled in.

use std::time::{SystemTime, UNIX_EPOCH};

use fragcap::write_json_string;

/// A lifecycle event, emitted on standard error under `--json`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// The session armed: the capture handle is open and the watcher attached.
    SessionArmed { interfaces: Vec<String> },
    /// A stage matched a process.
    StageMatched {
        role: String,
        pid: u32,
        process: String,
    },
    /// A matched stage's process exited.
    StageExited { role: String, pid: u32 },
    /// The capture filter narrowed to this many active endpoints.
    FilterNarrowed { endpoints: usize },
    /// The session completed, carrying the headline counters.
    SessionComplete {
        packets: u64,
        attributed: u64,
        dropped: u64,
        watching_discarded: u64,
        discarded_out_of_window: u64,
        /// Packets excluded because they belong to a process this capture does
        /// not cover (slice S064).
        ///
        /// Carried here and not only in the human summary because `--json`
        /// suppresses that summary entirely: without these two fields a machine
        /// consumer sees a capture that observed thousands of packets, wrote a
        /// few, and accounted for none of the difference. Every discard path has
        /// a named counter on every surface, not just the one a person reads
        /// (P-4).
        scope_discarded: u64,
        /// Packets excluded on scope grounds that carried no attribution, so it
        /// is not known whether they were the capture's.
        scope_unresolved_discarded: u64,
    },
    /// A streaming consumer left, carrying its per-consumer accounting. Distinct
    /// from the capture-wide `dropped` in `session.complete`.
    StreamConsumer {
        transport: String,
        id: String,
        written: u64,
        dropped: u64,
        reason: String,
    },
    /// A ring-mode capture finished, carrying the count of packets evicted from
    /// the rolling window. The sink's own retention accounting, distinct from the
    /// capture-wide `dropped`: an eviction is the operator's declared window scope,
    /// not a capture loss, but it is surfaced so the omission is never silent.
    RingEvicted { evicted: u64 },
    /// A Deep Capture preflight decision.
    DeepCapturePreflight {
        status: String,
        blockers: usize,
        warnings: usize,
        target: String,
        proxy_backend: String,
        trust_state: String,
    },
    /// The exact target-scoped routing and bypass plan, emitted before authorization.
    DeepCaptureRoutingPlan {
        operator_rules: Vec<String>,
        infrastructure: String,
        environment_variables: Vec<String>,
    },
    /// A complete compatibility calibration plan, emitted before confirmation.
    DeepCaptureCalibrationPlan {
        target: String,
        phase: String,
        declared_launch_case: String,
        observed_launch_case: String,
        proxy_backend: String,
        proxy_backend_version: String,
        routing_strategy: String,
        address_family: String,
        protocol: String,
        fragcap_version: String,
        target_version: Option<String>,
        bundle: String,
        trust_action: String,
        launch_timeout_secs: u64,
        observation_timeout_secs: u64,
        shutdown_timeout_secs: u64,
        cleanup_timeout_secs: u64,
    },
    /// A compatibility calibration phase transition or terminal outcome.
    DeepCaptureCalibrationPhase {
        session_id: Option<String>,
        target: String,
        phase: String,
        launch_case: String,
        proxy_backend: String,
        proxy_backend_version: String,
        routing_strategy: String,
        address_family: String,
        protocol: String,
        fragcap_version: String,
        target_version: Option<String>,
        stage: String,
        status: String,
        reason: String,
    },
    /// An explicit warm-to-cold plan, emitted before the operator acts.
    DeepCaptureRestartPlan {
        target: String,
        warm_case: String,
        images: Vec<String>,
        deadline_secs: u64,
    },
    /// A warm-to-cold transition or terminal pre-effect outcome.
    DeepCaptureRestart {
        target: String,
        stage: String,
        status: String,
        warm_case: String,
        cold_case: Option<String>,
        reason: String,
    },
    /// The Deep Capture proxy backend started.
    DeepCaptureProxyStarted {
        session_id: String,
        backend: String,
        version: String,
        listen_addr: String,
        listen_port: u16,
    },
    /// A live TLS key-log file is ready for an analyzer to follow.
    DeepCaptureKeyLogReady { session_id: String, path: String },
    /// Deep Capture trust state was confirmed or changed.
    DeepCaptureTrust {
        session_id: String,
        state: String,
        action: String,
        thumbprint: Option<String>,
    },
    /// Deep Capture issued a managed launch.
    DeepCaptureLaunch {
        session_id: String,
        launch_case: String,
        scoped_proxy: bool,
        target: String,
    },
    /// Deep Capture observed application traffic.
    DeepCaptureApplication {
        session_id: String,
        flow_id: Option<String>,
        proxy_connection_id: String,
        protocol: String,
        inspectability: String,
        classification_schema_version: u32,
        family: String,
        detection: String,
        classification_reason: Option<String>,
    },
    /// Deep Capture wrote a bundle artifact.
    DeepCaptureBundle {
        session_id: String,
        role: String,
        path: String,
        sensitivity: String,
        required: bool,
    },
    /// Deep Capture cleanup reported a resource outcome.
    DeepCaptureCleanup {
        session_id: String,
        resource: String,
        status: String,
        reason: String,
    },
    /// Deep Capture completed.
    DeepCaptureComplete {
        session_id: String,
        manifest: String,
        status: String,
        cleanup_status: String,
        inspectable: u64,
        metadata_only: u64,
        unsupported: u64,
        unknown: u64,
        failed: u64,
        decrypted_unknown: u64,
        encrypted_opaque: u64,
        unavailable: u64,
        classification_reasons: Vec<(String, u64)>,
        unclassified_lost: u64,
    },
    /// A periodic snapshot of the live counters the human status block also
    /// renders (slice S069), for a `--json` consumer watching a long-running
    /// capture. Carries no holder-tally breakdown: that is a human-display aid
    /// (see the S069 `contracts/capture-progress-event.md`), and a `--json`
    /// consumer already has per-packet attribution in the captured file itself.
    ///
    /// `etw`+`windows`-gated because its one real constructor,
    /// `crate::orchestrator::capture_progress_event`, lives inside
    /// `drive_live`, which is gated the same way (an ETW event stream has no
    /// non-Windows meaning); see `lib.rs`'s note on `mod live_status` for why
    /// that keeps `cargo clippy --all-targets --all-features` clean on every
    /// platform rather than leaving this reachable-but-uncalled elsewhere.
    #[cfg(all(feature = "etw", windows))]
    CaptureProgress {
        elapsed_secs: u64,
        packets: u64,
        bytes: u64,
        active_endpoints: usize,
        watching_discarded: u64,
        discarded_out_of_window: u64,
        buffer_dropped: u64,
        sink_dropped: u64,
        scope_discarded: u64,
        scope_unresolved_discarded: u64,
    },
}

impl Event {
    /// The `event` discriminator string.
    fn kind(&self) -> &'static str {
        match self {
            Event::SessionArmed { .. } => "session.armed",
            Event::StageMatched { .. } => "stage.matched",
            Event::StageExited { .. } => "stage.exited",
            Event::FilterNarrowed { .. } => "filter.narrowed",
            Event::SessionComplete { .. } => "session.complete",
            Event::StreamConsumer { .. } => "stream.consumer",
            Event::RingEvicted { .. } => "ring.evicted",
            Event::DeepCapturePreflight { .. } => "deep_capture.preflight",
            Event::DeepCaptureRoutingPlan { .. } => "deep_capture.routing_plan",
            Event::DeepCaptureCalibrationPlan { .. } => "deep_capture.calibration_plan",
            Event::DeepCaptureCalibrationPhase { .. } => "deep_capture.calibration_phase",
            Event::DeepCaptureRestartPlan { .. } => "deep_capture.restart_plan",
            Event::DeepCaptureRestart { .. } => "deep_capture.restart",
            Event::DeepCaptureProxyStarted { .. } => "deep_capture.proxy_started",
            Event::DeepCaptureKeyLogReady { .. } => "deep_capture.key_log_ready",
            Event::DeepCaptureTrust { .. } => "deep_capture.trust",
            Event::DeepCaptureLaunch { .. } => "deep_capture.launch",
            Event::DeepCaptureApplication { .. } => "deep_capture.application",
            Event::DeepCaptureBundle { .. } => "deep_capture.bundle",
            Event::DeepCaptureCleanup { .. } => "deep_capture.cleanup",
            Event::DeepCaptureComplete { .. } => "deep_capture.complete",
            #[cfg(all(feature = "etw", windows))]
            Event::CaptureProgress { .. } => "capture.progress",
        }
    }

    /// Render this event as one NDJSON line (no trailing newline), stamped with
    /// `now`.
    pub fn render(&self, now: SystemTime) -> String {
        let mut line = String::from("{\"ts\":");
        write_json_string(&rfc3339_utc(now), &mut line);
        line.push_str(",\"event\":");
        write_json_string(self.kind(), &mut line);
        match self {
            Event::SessionArmed { interfaces } => {
                line.push_str(",\"interfaces\":[");
                for (i, name) in interfaces.iter().enumerate() {
                    if i > 0 {
                        line.push(',');
                    }
                    write_json_string(name, &mut line);
                }
                line.push(']');
            }
            Event::StageMatched { role, pid, process } => {
                line.push_str(",\"role\":");
                write_json_string(role, &mut line);
                line.push_str(",\"pid\":");
                line.push_str(&pid.to_string());
                line.push_str(",\"proc\":");
                write_json_string(process, &mut line);
            }
            Event::StageExited { role, pid } => {
                line.push_str(",\"role\":");
                write_json_string(role, &mut line);
                line.push_str(",\"pid\":");
                line.push_str(&pid.to_string());
            }
            Event::FilterNarrowed { endpoints } => {
                line.push_str(",\"endpoints\":");
                line.push_str(&endpoints.to_string());
            }
            Event::SessionComplete {
                packets,
                attributed,
                dropped,
                watching_discarded,
                discarded_out_of_window,
                scope_discarded,
                scope_unresolved_discarded,
            } => {
                line.push_str(",\"packets\":");
                line.push_str(&packets.to_string());
                line.push_str(",\"attributed\":");
                line.push_str(&attributed.to_string());
                line.push_str(",\"dropped\":");
                line.push_str(&dropped.to_string());
                line.push_str(",\"watching_discarded\":");
                line.push_str(&watching_discarded.to_string());
                line.push_str(",\"discarded_out_of_window\":");
                line.push_str(&discarded_out_of_window.to_string());
                line.push_str(",\"scope_discarded\":");
                line.push_str(&scope_discarded.to_string());
                line.push_str(",\"scope_unresolved_discarded\":");
                line.push_str(&scope_unresolved_discarded.to_string());
            }
            Event::StreamConsumer {
                transport,
                id,
                written,
                dropped,
                reason,
            } => {
                line.push_str(",\"transport\":");
                write_json_string(transport, &mut line);
                line.push_str(",\"id\":");
                write_json_string(id, &mut line);
                line.push_str(",\"written\":");
                line.push_str(&written.to_string());
                line.push_str(",\"dropped\":");
                line.push_str(&dropped.to_string());
                line.push_str(",\"reason\":");
                write_json_string(reason, &mut line);
            }
            Event::RingEvicted { evicted } => {
                line.push_str(",\"evicted\":");
                line.push_str(&evicted.to_string());
            }
            Event::DeepCapturePreflight {
                status,
                blockers,
                warnings,
                target,
                proxy_backend,
                trust_state,
            } => {
                line.push_str(",\"status\":");
                write_json_string(status, &mut line);
                line.push_str(",\"blockers\":");
                line.push_str(&blockers.to_string());
                line.push_str(",\"warnings\":");
                line.push_str(&warnings.to_string());
                line.push_str(",\"target\":");
                write_json_string(target, &mut line);
                line.push_str(",\"proxy_backend\":");
                write_json_string(proxy_backend, &mut line);
                line.push_str(",\"trust_state\":");
                write_json_string(trust_state, &mut line);
            }
            Event::DeepCaptureRoutingPlan {
                operator_rules,
                infrastructure,
                environment_variables,
            } => {
                line.push_str(",\"policy_version\":1,\"operator_rules\":[");
                for (index, rule) in operator_rules.iter().enumerate() {
                    if index > 0 {
                        line.push(',');
                    }
                    write_json_string(rule, &mut line);
                }
                line.push_str("],\"infrastructure\":");
                write_json_string(infrastructure, &mut line);
                line.push_str(",\"environment_variables\":[");
                for (index, variable) in environment_variables.iter().enumerate() {
                    if index > 0 {
                        line.push(',');
                    }
                    write_json_string(variable, &mut line);
                }
                line.push_str("],\"dns_matching\":\"requested-authority-before-resolution\",\"resolved_address_policy\":\"evaluate-every-answer-every-attempt\",\"fallback\":\"none\"");
            }
            Event::DeepCaptureCalibrationPlan {
                target,
                phase,
                declared_launch_case,
                observed_launch_case,
                proxy_backend,
                proxy_backend_version,
                routing_strategy,
                address_family,
                protocol,
                fragcap_version,
                target_version,
                bundle,
                trust_action,
                launch_timeout_secs,
                observation_timeout_secs,
                shutdown_timeout_secs,
                cleanup_timeout_secs,
            } => {
                line.push_str(",\"target\":");
                write_json_string(target, &mut line);
                line.push_str(",\"phase\":");
                write_json_string(phase, &mut line);
                line.push_str(",\"declared_launch_case\":");
                write_json_string(declared_launch_case, &mut line);
                line.push_str(",\"observed_launch_case\":");
                write_json_string(observed_launch_case, &mut line);
                line.push_str(",\"proxy_backend\":");
                write_json_string(proxy_backend, &mut line);
                line.push_str(",\"proxy_backend_version\":");
                write_json_string(proxy_backend_version, &mut line);
                line.push_str(",\"proxy_mode\":\"launch-scoped-env\"");
                line.push_str(",\"routing_strategy\":");
                write_json_string(routing_strategy, &mut line);
                line.push_str(",\"address_family\":");
                write_json_string(address_family, &mut line);
                line.push_str(",\"protocol\":");
                write_json_string(protocol, &mut line);
                line.push_str(",\"fragcap_version\":");
                write_json_string(fragcap_version, &mut line);
                line.push_str(",\"target_version\":");
                match target_version {
                    Some(target_version) => write_json_string(target_version, &mut line),
                    None => line.push_str("null"),
                }
                line.push_str(",\"bundle\":");
                write_json_string(bundle, &mut line);
                line.push_str(",\"trust_action\":");
                write_json_string(trust_action, &mut line);
                line.push_str(",\"launch_timeout_secs\":");
                line.push_str(&launch_timeout_secs.to_string());
                line.push_str(",\"observation_timeout_secs\":");
                line.push_str(&observation_timeout_secs.to_string());
                line.push_str(",\"shutdown_timeout_secs\":");
                line.push_str(&shutdown_timeout_secs.to_string());
                line.push_str(",\"cleanup_timeout_secs\":");
                line.push_str(&cleanup_timeout_secs.to_string());
                line.push_str(",\"system_proxy_change\":false,\"publishes_evidence\":false");
            }
            Event::DeepCaptureCalibrationPhase {
                session_id,
                target,
                phase,
                launch_case,
                proxy_backend,
                proxy_backend_version,
                routing_strategy,
                address_family,
                protocol,
                fragcap_version,
                target_version,
                stage,
                status,
                reason,
            } => {
                line.push_str(",\"session_id\":");
                match session_id {
                    Some(session_id) => write_json_string(session_id, &mut line),
                    None => line.push_str("null"),
                }
                line.push_str(",\"target\":");
                write_json_string(target, &mut line);
                line.push_str(",\"phase\":");
                write_json_string(phase, &mut line);
                line.push_str(",\"launch_case\":");
                write_json_string(launch_case, &mut line);
                line.push_str(",\"proxy_backend\":");
                write_json_string(proxy_backend, &mut line);
                line.push_str(",\"proxy_backend_version\":");
                write_json_string(proxy_backend_version, &mut line);
                line.push_str(",\"routing_strategy\":");
                write_json_string(routing_strategy, &mut line);
                line.push_str(",\"address_family\":");
                write_json_string(address_family, &mut line);
                line.push_str(",\"protocol\":");
                write_json_string(protocol, &mut line);
                line.push_str(",\"fragcap_version\":");
                write_json_string(fragcap_version, &mut line);
                line.push_str(",\"target_version\":");
                match target_version {
                    Some(target_version) => write_json_string(target_version, &mut line),
                    None => line.push_str("null"),
                }
                line.push_str(",\"stage\":");
                write_json_string(stage, &mut line);
                line.push_str(",\"status\":");
                write_json_string(status, &mut line);
                line.push_str(",\"reason\":");
                write_json_string(reason, &mut line);
            }
            Event::DeepCaptureRestartPlan {
                target,
                warm_case,
                images,
                deadline_secs,
            } => {
                line.push_str(",\"target\":");
                write_json_string(target, &mut line);
                line.push_str(",\"warm_case\":");
                write_json_string(warm_case, &mut line);
                line.push_str(",\"images\":[");
                for (index, image) in images.iter().enumerate() {
                    if index > 0 {
                        line.push(',');
                    }
                    write_json_string(image, &mut line);
                }
                line.push_str("],\"deadline_secs\":");
                line.push_str(&deadline_secs.to_string());
                line.push_str(
                    ",\"identity\":\"image-name-observation-only\",\"process_control\":\"none\"",
                );
            }
            Event::DeepCaptureRestart {
                target,
                stage,
                status,
                warm_case,
                cold_case,
                reason,
            } => {
                line.push_str(",\"target\":");
                write_json_string(target, &mut line);
                line.push_str(",\"stage\":");
                write_json_string(stage, &mut line);
                line.push_str(",\"status\":");
                write_json_string(status, &mut line);
                line.push_str(",\"warm_case\":");
                write_json_string(warm_case, &mut line);
                line.push_str(",\"cold_case\":");
                match cold_case {
                    Some(cold_case) => write_json_string(cold_case, &mut line),
                    None => line.push_str("null"),
                }
                line.push_str(",\"reason\":");
                write_json_string(reason, &mut line);
            }
            Event::DeepCaptureProxyStarted {
                session_id,
                backend,
                version,
                listen_addr,
                listen_port,
            } => {
                line.push_str(",\"session_id\":");
                write_json_string(session_id, &mut line);
                line.push_str(",\"backend\":");
                write_json_string(backend, &mut line);
                line.push_str(",\"version\":");
                write_json_string(version, &mut line);
                line.push_str(",\"listen_addr\":");
                write_json_string(listen_addr, &mut line);
                line.push_str(",\"listen_port\":");
                line.push_str(&listen_port.to_string());
            }
            Event::DeepCaptureKeyLogReady { session_id, path } => {
                line.push_str(",\"session_id\":");
                write_json_string(session_id, &mut line);
                line.push_str(",\"path\":");
                write_json_string(path, &mut line);
            }
            Event::DeepCaptureTrust {
                session_id,
                state,
                action,
                thumbprint,
            } => {
                line.push_str(",\"session_id\":");
                write_json_string(session_id, &mut line);
                line.push_str(",\"state\":");
                write_json_string(state, &mut line);
                line.push_str(",\"action\":");
                write_json_string(action, &mut line);
                line.push_str(",\"thumbprint\":");
                match thumbprint {
                    Some(thumbprint) => write_json_string(thumbprint, &mut line),
                    None => line.push_str("null"),
                }
            }
            Event::DeepCaptureLaunch {
                session_id,
                launch_case,
                scoped_proxy,
                target,
            } => {
                line.push_str(",\"session_id\":");
                write_json_string(session_id, &mut line);
                line.push_str(",\"launch_case\":");
                write_json_string(launch_case, &mut line);
                line.push_str(",\"scoped_proxy\":");
                line.push_str(if *scoped_proxy { "true" } else { "false" });
                line.push_str(",\"target\":");
                write_json_string(target, &mut line);
            }
            Event::DeepCaptureApplication {
                session_id,
                flow_id,
                proxy_connection_id,
                protocol,
                inspectability,
                classification_schema_version,
                family,
                detection,
                classification_reason,
            } => {
                line.push_str(",\"session_id\":");
                write_json_string(session_id, &mut line);
                line.push_str(",\"flow_id\":");
                if let Some(flow_id) = flow_id {
                    write_json_string(flow_id, &mut line);
                } else {
                    line.push_str("null");
                }
                line.push_str(",\"proxy_connection_id\":");
                write_json_string(proxy_connection_id, &mut line);
                line.push_str(",\"protocol\":");
                write_json_string(protocol, &mut line);
                line.push_str(",\"inspectability\":");
                write_json_string(inspectability, &mut line);
                line.push_str(",\"classification_schema_version\":");
                line.push_str(&classification_schema_version.to_string());
                line.push_str(",\"family\":");
                write_json_string(family, &mut line);
                line.push_str(",\"detection\":");
                write_json_string(detection, &mut line);
                line.push_str(",\"classification_reason\":");
                if let Some(reason) = classification_reason {
                    write_json_string(reason, &mut line);
                } else {
                    line.push_str("null");
                }
            }
            Event::DeepCaptureBundle {
                session_id,
                role,
                path,
                sensitivity,
                required,
            } => {
                line.push_str(",\"session_id\":");
                write_json_string(session_id, &mut line);
                line.push_str(",\"role\":");
                write_json_string(role, &mut line);
                line.push_str(",\"path\":");
                write_json_string(path, &mut line);
                line.push_str(",\"sensitivity\":");
                write_json_string(sensitivity, &mut line);
                line.push_str(",\"required\":");
                line.push_str(if *required { "true" } else { "false" });
            }
            Event::DeepCaptureCleanup {
                session_id,
                resource,
                status,
                reason,
            } => {
                line.push_str(",\"session_id\":");
                write_json_string(session_id, &mut line);
                line.push_str(",\"resource\":");
                write_json_string(resource, &mut line);
                line.push_str(",\"status\":");
                write_json_string(status, &mut line);
                line.push_str(",\"reason\":");
                write_json_string(reason, &mut line);
            }
            Event::DeepCaptureComplete {
                session_id,
                manifest,
                status,
                cleanup_status,
                inspectable,
                metadata_only,
                unsupported,
                unknown,
                failed,
                decrypted_unknown,
                encrypted_opaque,
                unavailable,
                classification_reasons,
                unclassified_lost,
            } => {
                line.push_str(",\"session_id\":");
                write_json_string(session_id, &mut line);
                line.push_str(",\"manifest\":");
                write_json_string(manifest, &mut line);
                line.push_str(",\"status\":");
                write_json_string(status, &mut line);
                line.push_str(",\"cleanup_status\":");
                write_json_string(cleanup_status, &mut line);
                line.push_str(",\"inspectable\":");
                line.push_str(&inspectable.to_string());
                line.push_str(",\"metadata_only\":");
                line.push_str(&metadata_only.to_string());
                line.push_str(",\"unsupported\":");
                line.push_str(&unsupported.to_string());
                line.push_str(",\"unknown\":");
                line.push_str(&unknown.to_string());
                line.push_str(",\"failed\":");
                line.push_str(&failed.to_string());
                line.push_str(",\"decrypted_unknown\":");
                line.push_str(&decrypted_unknown.to_string());
                line.push_str(",\"encrypted_opaque\":");
                line.push_str(&encrypted_opaque.to_string());
                line.push_str(",\"unavailable\":");
                line.push_str(&unavailable.to_string());
                line.push_str(",\"classification_reasons\":{");
                for (index, (reason, count)) in classification_reasons.iter().enumerate() {
                    if index > 0 {
                        line.push(',');
                    }
                    write_json_string(reason, &mut line);
                    line.push(':');
                    line.push_str(&count.to_string());
                }
                line.push('}');
                line.push_str(",\"unclassified_lost\":");
                line.push_str(&unclassified_lost.to_string());
            }
            #[cfg(all(feature = "etw", windows))]
            Event::CaptureProgress {
                elapsed_secs,
                packets,
                bytes,
                active_endpoints,
                watching_discarded,
                discarded_out_of_window,
                buffer_dropped,
                sink_dropped,
                scope_discarded,
                scope_unresolved_discarded,
            } => {
                line.push_str(",\"elapsed_secs\":");
                line.push_str(&elapsed_secs.to_string());
                line.push_str(",\"packets\":");
                line.push_str(&packets.to_string());
                line.push_str(",\"bytes\":");
                line.push_str(&bytes.to_string());
                line.push_str(",\"active_endpoints\":");
                line.push_str(&active_endpoints.to_string());
                line.push_str(",\"watching_discarded\":");
                line.push_str(&watching_discarded.to_string());
                line.push_str(",\"discarded_out_of_window\":");
                line.push_str(&discarded_out_of_window.to_string());
                line.push_str(",\"buffer_dropped\":");
                line.push_str(&buffer_dropped.to_string());
                line.push_str(",\"sink_dropped\":");
                line.push_str(&sink_dropped.to_string());
                line.push_str(",\"scope_discarded\":");
                line.push_str(&scope_discarded.to_string());
                line.push_str(",\"scope_unresolved_discarded\":");
                line.push_str(&scope_unresolved_discarded.to_string());
            }
        }
        line.push('}');
        line
    }
}

/// Format a `SystemTime` as an RFC3339 UTC timestamp with a `Z` suffix and
/// second resolution.
///
/// Second resolution is enough for a lifecycle record; the point is a
/// standard, sortable, timezone-unambiguous stamp, not sub-second precision.
pub fn rfc3339_utc(now: SystemTime) -> String {
    let secs = now
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (hour, minute, second) = (rem / 3_600, (rem % 3_600) / 60, rem % 60);
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Days since the Unix epoch to a civil `(year, month, day)`.
///
/// Howard Hinnant's public-domain algorithm. Kept here rather than reached for
/// through a date crate, which would be a runtime dependency for one small
/// formatter.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (year + i64::from(month <= 2), month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn at_epoch(secs: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(secs)
    }

    #[test]
    fn the_epoch_formats_as_the_start_of_1970() {
        assert_eq!(rfc3339_utc(UNIX_EPOCH), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn a_known_instant_formats_correctly() {
        // 2021-01-01T00:00:00Z is 1609459200 seconds after the epoch.
        assert_eq!(rfc3339_utc(at_epoch(1_609_459_200)), "2021-01-01T00:00:00Z");
        // One more second, to catch a fencepost in the time-of-day split.
        assert_eq!(rfc3339_utc(at_epoch(1_609_459_201)), "2021-01-01T00:00:01Z");
    }

    #[test]
    fn each_event_renders_its_kind_and_fields() {
        let now = UNIX_EPOCH;
        let armed = Event::SessionArmed {
            interfaces: vec!["eth0".to_string()],
        }
        .render(now);
        assert!(armed.contains("\"event\":\"session.armed\""));
        assert!(armed.contains("\"interfaces\":[\"eth0\"]"));

        let matched = Event::StageMatched {
            role: "client".to_string(),
            pid: 4242,
            process: "game.exe".to_string(),
        }
        .render(now);
        assert!(matched.contains("\"event\":\"stage.matched\""));
        assert!(matched.contains("\"role\":\"client\""));
        assert!(matched.contains("\"pid\":4242"));
        assert!(matched.contains("\"proc\":\"game.exe\""));

        let complete = Event::SessionComplete {
            packets: 10,
            attributed: 9,
            dropped: 0,
            watching_discarded: 3,
            discarded_out_of_window: 1,
            scope_discarded: 0,
            scope_unresolved_discarded: 0,
        }
        .render(now);
        assert!(complete.contains("\"packets\":10"));
        assert!(complete.contains("\"attributed\":9"));
        assert!(complete.contains("\"dropped\":0"));
        assert!(complete.contains("\"watching_discarded\":3"));
        assert!(complete.contains("\"discarded_out_of_window\":1"));

        let ring = Event::RingEvicted { evicted: 17 }.render(now);
        assert!(ring.contains("\"event\":\"ring.evicted\""));
        assert!(ring.contains("\"evicted\":17"));

        let key_log = Event::DeepCaptureKeyLogReady {
            session_id: "session-1".to_string(),
            path: "/session/tls-keylog.log".to_string(),
        }
        .render(now);
        assert!(key_log.contains("\"event\":\"deep_capture.key_log_ready\""));
        assert!(key_log.contains("\"session_id\":\"session-1\""));
        assert!(key_log.contains("\"path\":\"/session/tls-keylog.log\""));

        let plan = Event::DeepCaptureCalibrationPlan {
            target: "sample-target".to_string(),
            phase: "reachability".to_string(),
            declared_launch_case: "direct-exe-warm".to_string(),
            observed_launch_case: "direct-exe-warm".to_string(),
            proxy_backend: "controlled".to_string(),
            proxy_backend_version: "0.8.0".to_string(),
            routing_strategy: "child-environment".to_string(),
            address_family: "ipv4".to_string(),
            protocol: "routing".to_string(),
            fragcap_version: "0.8.0".to_string(),
            target_version: None,
            bundle: "bundle".to_string(),
            trust_action: "none".to_string(),
            launch_timeout_secs: 30,
            observation_timeout_secs: 60,
            shutdown_timeout_secs: 10,
            cleanup_timeout_secs: 15,
        }
        .render(now);
        assert!(plan.contains("\"event\":\"deep_capture.calibration_plan\""));
        assert!(plan.contains("\"system_proxy_change\":false"));

        let phase = Event::DeepCaptureCalibrationPhase {
            session_id: Some("session-1".to_string()),
            target: "controlled".to_string(),
            phase: "tls".to_string(),
            launch_case: "direct-exe-cold".to_string(),
            proxy_backend: "fragcap-native".to_string(),
            proxy_backend_version: "0.8.0".to_string(),
            routing_strategy: "child-environment".to_string(),
            address_family: "ipv4".to_string(),
            protocol: "https".to_string(),
            fragcap_version: "0.8.0".to_string(),
            target_version: None,
            stage: "complete".to_string(),
            status: "metadata-only".to_string(),
            reason: "observed metadata".to_string(),
        }
        .render(now);
        assert!(phase.contains("\"event\":\"deep_capture.calibration_phase\""));
        assert!(phase.contains("\"status\":\"metadata-only\""));

        let restart_plan = Event::DeepCaptureRestartPlan {
            target: "sample-target".to_string(),
            warm_case: "direct-exe-warm".to_string(),
            images: vec!["client.exe".to_string()],
            deadline_secs: 120,
        }
        .render(now);
        assert!(restart_plan.contains("\"event\":\"deep_capture.restart_plan\""));
        assert!(restart_plan.contains("\"process_control\":\"none\""));

        let restart = Event::DeepCaptureRestart {
            target: "sample-target".to_string(),
            stage: "reprepare".to_string(),
            status: "cold-ready".to_string(),
            warm_case: "direct-exe-warm".to_string(),
            cold_case: Some("direct-exe-cold".to_string()),
            reason: "current facts are cold".to_string(),
        }
        .render(now);
        assert!(restart.contains("\"event\":\"deep_capture.restart\""));
        assert!(restart.contains("\"cold_case\":\"direct-exe-cold\""));
    }

    // `Event::CaptureProgress` is `etw`+`windows`-gated (see its own doc
    // comment), so this test is too; the other event variants' tests above
    // run on every platform.
    #[cfg(all(feature = "etw", windows))]
    #[test]
    fn capture_progress_renders_its_kind_and_fields() {
        let now = UNIX_EPOCH;
        let progress = Event::CaptureProgress {
            elapsed_secs: 135,
            packets: 4102,
            bytes: 812_004,
            active_endpoints: 3,
            watching_discarded: 0,
            discarded_out_of_window: 0,
            buffer_dropped: 0,
            sink_dropped: 0,
            scope_discarded: 0,
            scope_unresolved_discarded: 0,
        }
        .render(now);
        assert!(progress.contains("\"event\":\"capture.progress\""));
        assert!(progress.contains("\"elapsed_secs\":135"));
        assert!(progress.contains("\"packets\":4102"));
        assert!(progress.contains("\"bytes\":812004"));
        assert!(progress.contains("\"active_endpoints\":3"));
    }

    #[test]
    fn every_line_starts_with_the_timestamp_then_the_event() {
        let line = Event::FilterNarrowed { endpoints: 3 }.render(UNIX_EPOCH);
        assert!(line.starts_with("{\"ts\":\"1970-01-01T00:00:00Z\",\"event\":\"filter.narrowed\""));
        assert!(line.contains("\"endpoints\":3"));
        assert!(line.ends_with('}'));
    }
}
