// SPDX-License-Identifier: Apache-2.0

//! Bounded process lifecycle evidence and its versioned JSON Lines projection.

use std::collections::{BTreeMap, BTreeSet};

use fragcap_core::{
    CommandLine, FlowSummary, ProcessEvent, ProcessRecord, Timestamp, WatcherReport,
};
use serde_json::{json, Value};

/// Maximum raw process events retained by one capture run.
pub const PROCESS_EVENT_LIMIT: usize = 65_536;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StageTransitionKind {
    Matched,
    Exited,
}

impl StageTransitionKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Matched => "stage.matched",
            Self::Exited => "stage.exited",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StageTransition {
    pub kind: StageTransitionKind,
    pub pid: u32,
    pub role: String,
    pub stage: Option<String>,
    pub at: Timestamp,
}

/// Evidence already observed by the shared capture orchestrator.
#[derive(Clone, Debug)]
pub struct CaptureProcessEvidence {
    pub event_limit: usize,
    pub startup_snapshot: Vec<ProcessRecord>,
    pub snapshot_at: Option<Timestamp>,
    pub events: Vec<ProcessEvent>,
    pub events_unretained: u64,
    pub launch_pid: Option<u32>,
    pub launch_at: Option<Timestamp>,
    pub stage_transitions: Vec<StageTransition>,
    pub stage_transitions_unretained: u64,
    pub watcher_report: Option<WatcherReport>,
    pub unparseable_events: u64,
    pub rundown_ignored: u64,
    pub watcher_ended: bool,
    pub terminal_state: String,
    pub stop_reason: Option<String>,
}

impl Default for CaptureProcessEvidence {
    fn default() -> Self {
        Self {
            event_limit: PROCESS_EVENT_LIMIT,
            startup_snapshot: Vec::new(),
            snapshot_at: None,
            events: Vec::new(),
            events_unretained: 0,
            launch_pid: None,
            launch_at: None,
            stage_transitions: Vec::new(),
            stage_transitions_unretained: 0,
            watcher_report: None,
            unparseable_events: 0,
            rundown_ignored: 0,
            watcher_ended: false,
            terminal_state: "unavailable".to_string(),
            stop_reason: None,
        }
    }
}

impl CaptureProcessEvidence {
    pub fn observe(&mut self, event: &ProcessEvent) -> bool {
        if self.events.len() < self.event_limit {
            self.events.push(event.clone());
            true
        } else {
            self.events_unretained = self.events_unretained.saturating_add(1);
            false
        }
    }

    pub fn observe_stage(&mut self, transition: StageTransition, source_retained: bool) {
        if source_retained && self.stage_transitions.len() < self.event_limit {
            self.stage_transitions.push(transition);
        } else {
            self.stage_transitions_unretained = self.stage_transitions_unretained.saturating_add(1);
        }
    }
}

pub struct ProcessTraceInput<'a> {
    pub session_id: &'a str,
    pub target_id: Option<i64>,
    pub target_handle: &'a str,
    pub launch_case: &'a str,
    pub evidence: &'a CaptureProcessEvidence,
    pub flows: &'a [FlowSummary],
    pub globally_unretained_flow_observations: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessTraceSummary {
    pub finalization: &'static str,
    pub completeness: &'static str,
    pub records: u64,
    pub process_instances: u64,
    pub flow_owner_intervals: u64,
    pub limitations: u64,
    pub events_lost: u64,
    pub unparseable_events: u64,
    pub buffers_lost: u64,
    pub rundown_ignored: u64,
    pub events_unretained: u64,
    pub stage_transitions_unretained: u64,
    pub unresolved_flow_owners: u64,
}

pub struct ProcessTrace {
    pub jsonl: String,
    pub summary: ProcessTraceSummary,
}

#[derive(Clone)]
struct Instance {
    pid: u32,
    parent: u32,
    image: String,
    command_line: Option<String>,
    created: Option<Timestamp>,
    exited: Option<Timestamp>,
    snapshot: bool,
    creation_observed: bool,
}

impl Instance {
    fn id(&self) -> String {
        match (self.snapshot, self.created) {
            (true, Some(at)) => format!("snapshot-{}-{}", self.pid, at.as_nanos()),
            (true, None) => format!("snapshot-{}", self.pid),
            (false, Some(at)) => format!("process-{}-{}", self.pid, at.as_nanos()),
            (false, None) => unreachable!("creation events always carry a timestamp"),
        }
    }

    fn covers(&self, at: Timestamp) -> bool {
        self.created.is_none_or(|start| start <= at) && self.exited.is_none_or(|end| at <= end)
    }
}

fn instance_at(instances: &[Instance], pid: u32, at: Timestamp) -> Option<&Instance> {
    instances
        .iter()
        .filter(|instance| instance.pid == pid && instance.covers(at))
        .max_by_key(|instance| instance.created)
}

fn image_basename(value: &str) -> &str {
    value.rsplit(['\\', '/']).next().unwrap_or(value)
}

fn same_image(left: &str, right: &str) -> bool {
    image_basename(left).eq_ignore_ascii_case(image_basename(right))
}

fn push_record(output: &mut String, records: &mut u64, mut value: Value, sequence: u64) {
    value["sequence"] = json!(sequence);
    output.push_str(&value.to_string());
    output.push('\n');
    *records += 1;
}

/// Reconcile and serialize process evidence without acquiring a new authority.
pub fn build_process_trace(input: ProcessTraceInput<'_>) -> ProcessTrace {
    let evidence = input.evidence;
    let mut instances = evidence
        .startup_snapshot
        .iter()
        .map(|record| Instance {
            pid: record.pid,
            parent: record.parent,
            image: record.image.to_string(),
            command_line: match &record.command_line {
                CommandLine::Observed(value) => Some(value.to_string()),
                CommandLine::Unavailable => None,
            },
            created: record.started.or(evidence.snapshot_at),
            exited: None,
            snapshot: true,
            creation_observed: record.started.is_some(),
        })
        .collect::<Vec<_>>();
    let mut starts = evidence.events.clone();
    starts.sort_by_key(|event| {
        (
            event.at(),
            event.pid(),
            matches!(event, ProcessEvent::Exited { .. }),
        )
    });
    for event in &starts {
        match event {
            ProcessEvent::Started {
                pid,
                parent,
                image,
                command_line,
                at,
            } => {
                let command_line = match command_line {
                    CommandLine::Observed(value) => Some(value.to_string()),
                    CommandLine::Unavailable => None,
                };
                let overlapping_snapshot = evidence.snapshot_at.is_some_and(|taken| *at <= taken)
                    && instances.iter().any(|instance| {
                        instance.snapshot
                            && instance.pid == *pid
                            && instance.exited.is_none()
                            && same_image(&instance.image, image)
                    });
                if overlapping_snapshot {
                    let instance = instances
                        .iter_mut()
                        .find(|instance| {
                            instance.snapshot
                                && instance.pid == *pid
                                && instance.exited.is_none()
                                && same_image(&instance.image, image)
                        })
                        .expect("the overlapping snapshot was just found");
                    instance.parent = *parent;
                    instance.image = image.to_string();
                    instance.command_line = command_line;
                    instance.created = Some(*at);
                    instance.snapshot = false;
                    instance.creation_observed = true;
                } else {
                    instances.push(Instance {
                        pid: *pid,
                        parent: *parent,
                        image: image.to_string(),
                        command_line,
                        created: Some(*at),
                        exited: None,
                        snapshot: false,
                        creation_observed: true,
                    });
                }
            }
            ProcessEvent::Exited { pid, at } => {
                if let Some(instance) = instances
                    .iter_mut()
                    .filter(|item| {
                        item.pid == *pid
                            && item.created.is_none_or(|start| start <= *at)
                            && item.exited.is_none()
                    })
                    .max_by_key(|item| item.created)
                {
                    instance.exited = Some(*at);
                }
            }
            _ => {}
        }
    }

    let mut seeds = BTreeSet::new();
    if let Some(pid) = evidence.launch_pid {
        seeds.insert(pid);
    }
    seeds.extend(
        evidence
            .stage_transitions
            .iter()
            .map(|transition| transition.pid),
    );
    for flow in input.flows {
        for observation in &flow.observations {
            if let Some(attribution) = &observation.attribution {
                seeds.insert(attribution.pid);
            }
        }
    }
    loop {
        let before = seeds.len();
        for instance in &instances {
            if seeds.contains(&instance.pid)
                && Some(instance.pid) != evidence.launch_pid
                && instance.parent != 0
            {
                seeds.insert(instance.parent);
            }
        }
        if seeds.len() == before {
            break;
        }
    }
    instances.retain(|instance| seeds.contains(&instance.pid));
    instances.sort_by_key(|instance| (instance.created, instance.pid));

    let mut output = String::new();
    let mut records = 0_u64;
    let mut sequence = 0_u64;
    push_record(
        &mut output,
        &mut records,
        json!({
            "type": "process-trace.header", "schema_version": 1, "session_id": input.session_id,
            "target_id": input.target_id, "target_handle": input.target_handle,
            "launch_case": input.launch_case, "event_limit": evidence.event_limit,
            "snapshot_authority": if evidence.startup_snapshot.is_empty() { "unavailable" } else { "query-only" },
            "watcher_authority": if evidence.watcher_report.is_some() { "etw" } else { "unavailable" }
        }),
        sequence,
    );
    sequence += 1;

    let mut limitations = BTreeMap::<&'static str, u64>::new();
    if evidence.launch_pid.is_none() {
        limitations.insert("launch-pid-unavailable", 1);
    } else if evidence.launch_at.is_none()
        || evidence
            .launch_pid
            .zip(evidence.launch_at)
            .and_then(|(pid, at)| instance_at(&instances, pid, at))
            .is_none()
    {
        limitations.insert("launch-generation-unavailable", 1);
    }
    push_record(
        &mut output,
        &mut records,
        json!({
            "type": "launch.receipt", "session_id": input.session_id, "pid": evidence.launch_pid,
            "process_instance_id": evidence.launch_pid.and_then(|pid| evidence.launch_at.and_then(|at| instance_at(&instances, pid, at).map(Instance::id))),
            "at_nanos": evidence.launch_at.map(Timestamp::as_nanos)
        }),
        sequence,
    );
    sequence += 1;
    let mut timeline = Vec::<(Option<Timestamp>, u8, u32, String, Value)>::new();

    for instance in &instances {
        let parent_id = instance
            .created
            .and_then(|at| instance_at(&instances, instance.parent, at).map(Instance::id));
        if instance.parent != 0 && parent_id.is_none() && Some(instance.pid) != evidence.launch_pid
        {
            *limitations
                .entry("parent-instance-unavailable")
                .or_default() += 1;
        }
        if instance.snapshot && !instance.creation_observed {
            *limitations
                .entry("snapshot-creation-unavailable")
                .or_default() += 1;
        }
        timeline.push((
            instance.created,
            0,
            instance.pid,
            instance.id(),
            json!({
                "type": if instance.snapshot { "process.snapshot" } else { "process.started" },
                "session_id": input.session_id, "pid": instance.pid, "parent_pid": instance.parent,
                "process_instance_id": instance.id(), "parent_instance_id": parent_id,
                "image": instance.image, "command_line": instance.command_line,
                "at_nanos": instance.created.map(Timestamp::as_nanos),
                "ancestry_authority": if instance.snapshot { "query-snapshot" } else { "creation-event" }
            }),
        ));
    }

    let mut stage_transitions = evidence.stage_transitions.clone();
    stage_transitions.sort_by_key(|transition| {
        (
            transition.at,
            transition.pid,
            matches!(transition.kind, StageTransitionKind::Exited),
            transition.role.clone(),
            transition.stage.clone(),
        )
    });
    for transition in &stage_transitions {
        let instance = instance_at(&instances, transition.pid, transition.at);
        if instance.is_none() {
            *limitations.entry("stage-instance-unavailable").or_default() += 1;
        }
        timeline.push((
            Some(transition.at),
            if transition.kind == StageTransitionKind::Matched {
                1
            } else {
                3
            },
            transition.pid,
            format!("{}:{:?}", transition.role, transition.stage),
            json!({
                "type": transition.kind.as_str(), "session_id": input.session_id, "pid": transition.pid,
                "process_instance_id": instance.map(Instance::id), "role": transition.role,
                "stage": transition.stage, "at_nanos": transition.at.as_nanos(), "authority": "managed-stage-binding"
            }),
        ));
    }

    let mut flow_owner_intervals = 0_u64;
    let mut unresolved_flow_owners = 0_u64;
    for flow in input.flows {
        let mut observations = flow.observations.clone();
        observations.sort_by_key(|observation| {
            (
                observation.timestamp,
                observation.attribution.as_ref().map(|owner| owner.pid),
                observation
                    .attribution
                    .as_ref()
                    .map(|owner| owner.process.to_string()),
                observation
                    .attribution
                    .as_ref()
                    .and_then(|owner| owner.role.as_ref().map(ToString::to_string)),
                observation
                    .attribution
                    .as_ref()
                    .and_then(|owner| owner.stage.as_ref().map(|stage| stage.as_str().to_string())),
            )
        });
        let mut cursor = 0;
        while cursor < observations.len() {
            let observation = &observations[cursor];
            let Some(attribution) = &observation.attribution else {
                unresolved_flow_owners += 1;
                *limitations.entry("flow-owner-unavailable").or_default() += 1;
                cursor += 1;
                continue;
            };
            let instance = instance_at(&instances, attribution.pid, observation.timestamp);
            let instance_id = instance.map(Instance::id);
            if instance.is_none() {
                unresolved_flow_owners += 1;
                *limitations
                    .entry("process-instance-unavailable")
                    .or_default() += 1;
            }
            let mut end = observation.timestamp;
            let mut next = cursor + 1;
            while next < observations.len() {
                let candidate = &observations[next];
                let candidate_instance = candidate
                    .attribution
                    .as_ref()
                    .and_then(|owner| instance_at(&instances, owner.pid, candidate.timestamp))
                    .map(Instance::id);
                if candidate.attribution.as_ref() != Some(attribution)
                    || candidate_instance != instance_id
                {
                    break;
                }
                end = candidate.timestamp;
                next += 1;
            }
            timeline.push((
                Some(observation.timestamp),
                2,
                attribution.pid,
                flow.id.to_string(),
                json!({
                    "type": "socket-owner.interval", "session_id": input.session_id,
                    "flow_id": flow.id.to_string(), "from_nanos": observation.timestamp.as_nanos(),
                    "to_nanos": end.as_nanos(), "pid": attribution.pid,
                    "process_instance_id": instance_id, "process": attribution.process.as_ref(),
                    "role": attribution.role.as_deref(), "stage": attribution.stage.as_ref().map(|stage| stage.as_str()),
                    "fidelity": format!("{:?}", attribution.fidelity).to_ascii_lowercase()
                }),
            ));
            flow_owner_intervals += 1;
            cursor = next;
        }
        if flow.unretained_observations > 0 {
            *limitations.entry("packet-evidence-unretained").or_default() +=
                flow.unretained_observations;
        }
    }
    if input.globally_unretained_flow_observations > 0 {
        *limitations.entry("packet-evidence-unretained").or_default() +=
            input.globally_unretained_flow_observations;
    }

    for instance in &instances {
        if let Some(at) = instance.exited {
            timeline.push((
                Some(at),
                4,
                instance.pid,
                instance.id(),
                json!({
                    "type": "process.exited", "session_id": input.session_id, "pid": instance.pid,
                    "process_instance_id": instance.id(), "at_nanos": at.as_nanos()
                }),
            ));
        } else {
            *limitations.entry("process-exit-unobserved").or_default() += 1;
        }
    }
    timeline.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
            .then_with(|| left.3.cmp(&right.3))
    });
    for (_, _, _, _, record) in timeline {
        push_record(&mut output, &mut records, record, sequence);
        sequence += 1;
    }
    let watcher = evidence.watcher_report.unwrap_or_default();
    let kernel_events_lost = watcher
        .events_lost
        .saturating_sub(evidence.unparseable_events);
    if kernel_events_lost > 0 {
        limitations.insert("watcher-event-loss", kernel_events_lost);
    }
    if evidence.unparseable_events > 0 {
        limitations.insert("watcher-unparseable-event", evidence.unparseable_events);
    }
    if watcher.buffers_lost > 0 {
        limitations.insert("watcher-buffer-loss", watcher.buffers_lost);
    }
    if evidence.watcher_ended {
        limitations.insert("watcher-ended", 1);
    }
    if evidence.events_unretained > 0 {
        limitations.insert("event-retention-overflow", evidence.events_unretained);
    }
    if evidence.stage_transitions_unretained > 0 {
        limitations.insert(
            "stage-transition-retention-overflow",
            evidence.stage_transitions_unretained,
        );
    }
    for (reason, count) in &limitations {
        push_record(
            &mut output,
            &mut records,
            json!({
                "type": "process-trace.limitation", "session_id": input.session_id,
                "reason": reason, "count": count
            }),
            sequence,
        );
        sequence += 1;
    }
    push_record(
        &mut output,
        &mut records,
        json!({
            "type": "session.terminal", "session_id": input.session_id,
            "terminal_state": evidence.terminal_state, "stop_reason": evidence.stop_reason
        }),
        sequence,
    );
    sequence += 1;

    let completeness = if instances.is_empty() {
        "unavailable"
    } else if limitations.is_empty() {
        "complete"
    } else {
        "partial"
    };
    let summary = ProcessTraceSummary {
        finalization: "complete",
        completeness,
        records: records + 1,
        process_instances: instances.len() as u64,
        flow_owner_intervals,
        limitations: limitations.values().sum(),
        events_lost: kernel_events_lost,
        unparseable_events: evidence.unparseable_events,
        buffers_lost: watcher.buffers_lost,
        rundown_ignored: evidence.rundown_ignored,
        events_unretained: evidence.events_unretained,
        stage_transitions_unretained: evidence.stage_transitions_unretained,
        unresolved_flow_owners,
    };
    push_record(
        &mut output,
        &mut records,
        json!({
            "type": "process-trace.trailer", "schema_version": 1, "session_id": input.session_id,
            "records": summary.records, "process_instances": summary.process_instances,
            "flow_owner_intervals": summary.flow_owner_intervals, "limitations": summary.limitations,
            "events_lost": summary.events_lost, "buffers_lost": summary.buffers_lost,
            "unparseable_events": summary.unparseable_events,
            "rundown_ignored": summary.rundown_ignored, "events_unretained": summary.events_unretained,
            "stage_transitions_unretained": summary.stage_transitions_unretained,
            "unresolved_flow_owners": summary.unresolved_flow_owners,
            "terminal_state": evidence.terminal_state, "stop_reason": evidence.stop_reason,
            "completeness": summary.completeness, "finalization": summary.finalization
        }),
        sequence,
    );
    ProcessTrace {
        jsonl: output,
        summary,
    }
}

/// Read and validate the final trailer without trusting a manifest claim.
pub fn read_process_trace(value: &str) -> Option<ProcessTraceSummary> {
    let lines = value.lines().collect::<Vec<_>>();
    let records = lines
        .iter()
        .map(|line| serde_json::from_str::<Value>(line).ok())
        .collect::<Option<Vec<_>>>()?;
    if records.first()?["type"] != "process-trace.header" {
        return None;
    }
    let last = records.last()?;
    if last["type"] != "process-trace.trailer" || last["finalization"] != "complete" {
        return None;
    }
    let trailer_count = records
        .iter()
        .filter(|record| record["type"] == "process-trace.trailer")
        .count();
    if trailer_count != 1 {
        return None;
    }
    Some(ProcessTraceSummary {
        finalization: "complete",
        completeness: match last["completeness"].as_str()? {
            "complete" => "complete",
            "partial" => "partial",
            "unavailable" => "unavailable",
            "failed" => "failed",
            _ => return None,
        },
        records: last["records"].as_u64()?,
        process_instances: last["process_instances"].as_u64()?,
        flow_owner_intervals: last["flow_owner_intervals"].as_u64()?,
        limitations: last["limitations"].as_u64()?,
        events_lost: last["events_lost"].as_u64()?,
        unparseable_events: last["unparseable_events"].as_u64()?,
        buffers_lost: last["buffers_lost"].as_u64()?,
        rundown_ignored: last["rundown_ignored"].as_u64()?,
        events_unretained: last["events_unretained"].as_u64()?,
        stage_transitions_unretained: last["stage_transitions_unretained"].as_u64()?,
        unresolved_flow_owners: last["unresolved_flow_owners"].as_u64()?,
    })
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use fragcap_core::{Attribution, Fidelity, FlowId, FlowObservation, ProcessRecord};

    #[test]
    fn pid_reuse_produces_distinct_instances_and_a_trailer() {
        let mut evidence = CaptureProcessEvidence::default();
        evidence.launch_pid = Some(7);
        evidence.events = vec![
            ProcessEvent::started(7, 1, "client.exe", "client", Timestamp::from_nanos(10)),
            ProcessEvent::Exited {
                pid: 7,
                at: Timestamp::from_nanos(20),
            },
            ProcessEvent::started(7, 1, "client.exe", "client", Timestamp::from_nanos(30)),
            ProcessEvent::Exited {
                pid: 7,
                at: Timestamp::from_nanos(40),
            },
        ];
        evidence.terminal_state = "complete".to_string();
        let trace = build_process_trace(ProcessTraceInput {
            session_id: "s",
            target_id: Some(1),
            target_handle: "t",
            launch_case: "direct",
            evidence: &evidence,
            flows: &[],
            globally_unretained_flow_observations: 0,
        });
        assert!(trace.jsonl.contains("process-7-10"));
        assert!(trace.jsonl.contains("process-7-30"));
        assert!(read_process_trace(&trace.jsonl).is_some());
    }

    #[test]
    fn startup_snapshot_lifetime_survives_later_pid_reuse() {
        let evidence = CaptureProcessEvidence {
            startup_snapshot: vec![ProcessRecord::new(7, 1, "original.exe")],
            snapshot_at: Some(Timestamp::from_nanos(10)),
            launch_pid: Some(7),
            launch_at: Some(Timestamp::from_nanos(35)),
            events: vec![
                ProcessEvent::Exited {
                    pid: 7,
                    at: Timestamp::from_nanos(20),
                },
                ProcessEvent::started(7, 1, "reused.exe", "reused", Timestamp::from_nanos(30)),
                ProcessEvent::Exited {
                    pid: 7,
                    at: Timestamp::from_nanos(40),
                },
            ],
            terminal_state: "complete".to_string(),
            ..CaptureProcessEvidence::default()
        };
        let trace = build_process_trace(ProcessTraceInput {
            session_id: "s",
            target_id: Some(1),
            target_handle: "t",
            launch_case: "direct",
            evidence: &evidence,
            flows: &[],
            globally_unretained_flow_observations: 0,
        });
        let records = trace
            .jsonl
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert!(records.iter().any(|record| {
            record["type"] == "process.exited"
                && record["at_nanos"] == 20
                && record["process_instance_id"] == "snapshot-7-10"
        }));
        assert!(records.iter().any(|record| {
            record["type"] == "process.exited"
                && record["at_nanos"] == 40
                && record["process_instance_id"] == "process-7-30"
        }));
        assert!(records.iter().any(|record| {
            record["type"] == "launch.receipt" && record["process_instance_id"] == "process-7-30"
        }));
    }

    #[test]
    fn overlapping_snapshot_and_start_are_one_observed_instance() {
        let evidence = CaptureProcessEvidence {
            startup_snapshot: vec![ProcessRecord::new(7, 1, "client.exe")],
            snapshot_at: Some(Timestamp::from_nanos(10)),
            launch_pid: Some(7),
            launch_at: Some(Timestamp::from_nanos(12)),
            events: vec![ProcessEvent::started(
                7,
                1,
                "C:\\game\\client.exe",
                "client",
                Timestamp::from_nanos(5),
            )],
            terminal_state: "complete".to_string(),
            ..CaptureProcessEvidence::default()
        };
        let trace = build_process_trace(ProcessTraceInput {
            session_id: "s",
            target_id: Some(1),
            target_handle: "t",
            launch_case: "direct",
            evidence: &evidence,
            flows: &[],
            globally_unretained_flow_observations: 0,
        });
        assert_eq!(trace.summary.process_instances, 1);
        assert!(trace.jsonl.contains("process-7-5"));
        assert!(!trace.jsonl.contains("snapshot-7-10"));
    }

    #[test]
    fn ancestry_stops_at_the_managed_launch_root() {
        let evidence = CaptureProcessEvidence {
            startup_snapshot: vec![ProcessRecord::new(99, 0, "fragcap.exe")],
            snapshot_at: Some(Timestamp::from_nanos(1)),
            launch_pid: Some(7),
            launch_at: Some(Timestamp::from_nanos(12)),
            events: vec![ProcessEvent::started(
                7,
                99,
                "client.exe",
                "client",
                Timestamp::from_nanos(10),
            )],
            terminal_state: "complete".to_string(),
            ..CaptureProcessEvidence::default()
        };
        let trace = build_process_trace(ProcessTraceInput {
            session_id: "s",
            target_id: Some(1),
            target_handle: "t",
            launch_case: "direct",
            evidence: &evidence,
            flows: &[],
            globally_unretained_flow_observations: 0,
        });
        assert_eq!(trace.summary.process_instances, 1);
        assert!(!trace.jsonl.contains("fragcap.exe"));
        assert!(!trace.jsonl.contains("parent-instance-unavailable"));
    }

    #[test]
    fn a_crash_prefix_never_claims_finalization() {
        assert!(read_process_trace("{\"type\":\"process-trace.header\"}\n").is_none());
    }

    #[test]
    fn event_retention_is_bounded_and_counted() {
        let mut evidence = CaptureProcessEvidence {
            event_limit: 1,
            ..CaptureProcessEvidence::default()
        };
        evidence.observe(&ProcessEvent::started(
            1,
            0,
            "a.exe",
            "a",
            Timestamp::from_nanos(1),
        ));
        evidence.observe(&ProcessEvent::started(
            2,
            0,
            "b.exe",
            "b",
            Timestamp::from_nanos(2),
        ));
        assert_eq!(evidence.events.len(), 1);
        assert_eq!(evidence.events_unretained, 1);
    }

    #[test]
    fn stage_transition_retention_is_bounded_and_counted() {
        let mut evidence = CaptureProcessEvidence {
            event_limit: 1,
            ..CaptureProcessEvidence::default()
        };
        let transition = |pid| StageTransition {
            kind: StageTransitionKind::Matched,
            pid,
            role: "client".to_string(),
            stage: Some("client".to_string()),
            at: Timestamp::from_nanos(i64::from(pid)),
        };
        evidence.observe_stage(transition(1), true);
        evidence.observe_stage(transition(2), true);
        evidence.observe_stage(transition(3), false);

        assert_eq!(evidence.stage_transitions.len(), 1);
        assert_eq!(evidence.stage_transitions_unretained, 2);
        let trace = build_process_trace(ProcessTraceInput {
            session_id: "s",
            target_id: Some(1),
            target_handle: "t",
            launch_case: "direct",
            evidence: &evidence,
            flows: &[],
            globally_unretained_flow_observations: 0,
        });
        assert_eq!(trace.summary.stage_transitions_unretained, 2);
        assert!(trace.jsonl.contains("stage-transition-retention-overflow"));
    }

    #[test]
    fn adjacent_packet_observations_collapse_to_one_owner_interval() {
        let mut evidence = CaptureProcessEvidence::default();
        evidence.launch_pid = Some(7);
        evidence.events = vec![
            ProcessEvent::started(7, 0, "client.exe", "client", Timestamp::from_nanos(10)),
            ProcessEvent::Exited {
                pid: 7,
                at: Timestamp::from_nanos(40),
            },
        ];
        evidence.terminal_state = "complete".to_string();
        let owner = Attribution::new(7, "client.exe", Fidelity::Live);
        let flow = FlowSummary {
            id: FlowId::new(1).unwrap(),
            observations: vec![
                FlowObservation {
                    timestamp: Timestamp::from_nanos(20),
                    attribution: Some(owner.clone()),
                },
                FlowObservation {
                    timestamp: Timestamp::from_nanos(30),
                    attribution: Some(owner),
                },
            ],
            unretained_observations: 0,
            global_unretained_observations: 0,
        };
        let trace = build_process_trace(ProcessTraceInput {
            session_id: "s",
            target_id: Some(1),
            target_handle: "t",
            launch_case: "direct",
            evidence: &evidence,
            flows: &[flow],
            globally_unretained_flow_observations: 0,
        });
        assert_eq!(trace.summary.flow_owner_intervals, 1);
        let interval = trace
            .jsonl
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .find(|record| record["type"] == "socket-owner.interval")
            .unwrap();
        assert_eq!(interval["from_nanos"], 20);
        assert_eq!(interval["to_nanos"], 30);
    }

    #[test]
    fn duplicate_trailers_are_not_orderly_finalization() {
        let mut evidence = CaptureProcessEvidence::default();
        evidence.launch_pid = Some(1);
        evidence.events.push(ProcessEvent::started(
            1,
            0,
            "a.exe",
            "a",
            Timestamp::from_nanos(1),
        ));
        let trace = build_process_trace(ProcessTraceInput {
            session_id: "s",
            target_id: Some(1),
            target_handle: "t",
            launch_case: "direct",
            evidence: &evidence,
            flows: &[],
            globally_unretained_flow_observations: 0,
        });
        let trailer = trace.jsonl.lines().last().unwrap();
        assert!(read_process_trace(&format!("{}{trailer}\n", trace.jsonl)).is_none());
    }

    #[test]
    fn event_delivery_permutations_reconcile_identically() {
        let events = vec![
            ProcessEvent::started(2, 1, "client.exe", "client", Timestamp::from_nanos(20)),
            ProcessEvent::started(1, 0, "launcher.exe", "launcher", Timestamp::from_nanos(10)),
            ProcessEvent::Exited {
                pid: 1,
                at: Timestamp::from_nanos(30),
            },
            ProcessEvent::Exited {
                pid: 2,
                at: Timestamp::from_nanos(40),
            },
        ];
        let mut forward = CaptureProcessEvidence {
            launch_pid: Some(1),
            events: events.clone(),
            terminal_state: "complete".to_string(),
            ..CaptureProcessEvidence::default()
        };
        forward.stage_transitions.push(StageTransition {
            kind: StageTransitionKind::Matched,
            pid: 2,
            role: "client".to_string(),
            stage: Some("client".to_string()),
            at: Timestamp::from_nanos(20),
        });
        forward.stage_transitions.push(StageTransition {
            kind: StageTransitionKind::Exited,
            pid: 2,
            role: "client".to_string(),
            stage: Some("client".to_string()),
            at: Timestamp::from_nanos(40),
        });
        let mut reverse = forward.clone();
        reverse.events.reverse();
        reverse.stage_transitions.reverse();
        let build = |evidence| {
            build_process_trace(ProcessTraceInput {
                session_id: "s",
                target_id: Some(1),
                target_handle: "t",
                launch_case: "publisher",
                evidence,
                flows: &[],
                globally_unretained_flow_observations: 0,
            })
            .jsonl
        };
        assert_eq!(build(&forward), build(&reverse));
    }

    #[test]
    fn lifecycle_sequence_is_globally_ordered_by_event_time() {
        let evidence = CaptureProcessEvidence {
            launch_pid: Some(1),
            events: vec![
                ProcessEvent::started(2, 1, "client.exe", "client", Timestamp::from_nanos(100)),
                ProcessEvent::started(1, 0, "launcher.exe", "launcher", Timestamp::from_nanos(10)),
                ProcessEvent::Exited {
                    pid: 1,
                    at: Timestamp::from_nanos(30),
                },
                ProcessEvent::Exited {
                    pid: 2,
                    at: Timestamp::from_nanos(110),
                },
            ],
            stage_transitions: vec![
                StageTransition {
                    kind: StageTransitionKind::Matched,
                    pid: 1,
                    role: "launcher".to_string(),
                    stage: Some("launcher".to_string()),
                    at: Timestamp::from_nanos(20),
                },
                StageTransition {
                    kind: StageTransitionKind::Matched,
                    pid: 2,
                    role: "client".to_string(),
                    stage: Some("client".to_string()),
                    at: Timestamp::from_nanos(105),
                },
            ],
            terminal_state: "complete".to_string(),
            ..CaptureProcessEvidence::default()
        };
        let trace = build_process_trace(ProcessTraceInput {
            session_id: "s",
            target_id: Some(1),
            target_handle: "t",
            launch_case: "publisher",
            evidence: &evidence,
            flows: &[],
            globally_unretained_flow_observations: 0,
        });
        let instants = trace
            .jsonl
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .filter_map(|record| record["at_nanos"].as_i64())
            .collect::<Vec<_>>();
        assert!(instants.windows(2).all(|pair| pair[0] <= pair[1]));
    }

    #[test]
    fn malformed_interior_record_invalidates_the_stream() {
        let mut evidence = CaptureProcessEvidence::default();
        evidence.launch_pid = Some(1);
        evidence.events.push(ProcessEvent::started(
            1,
            0,
            "a.exe",
            "a",
            Timestamp::from_nanos(1),
        ));
        let trace = build_process_trace(ProcessTraceInput {
            session_id: "s",
            target_id: Some(1),
            target_handle: "t",
            launch_case: "direct",
            evidence: &evidence,
            flows: &[],
            globally_unretained_flow_observations: 0,
        });
        let damaged = trace.jsonl.replacen('\n', "\nnot-json\n", 1);
        assert!(read_process_trace(&damaged).is_none());
    }

    #[test]
    fn watcher_loss_classes_remain_separate_and_partial() {
        let evidence = CaptureProcessEvidence {
            launch_pid: Some(1),
            launch_at: Some(Timestamp::from_nanos(2)),
            events: vec![ProcessEvent::started(
                1,
                0,
                "a.exe",
                "a",
                Timestamp::from_nanos(1),
            )],
            watcher_report: Some(WatcherReport {
                events_lost: 5,
                buffers_lost: 2,
                running: false,
            }),
            unparseable_events: 3,
            terminal_state: "partial".to_string(),
            ..CaptureProcessEvidence::default()
        };
        let trace = build_process_trace(ProcessTraceInput {
            session_id: "s",
            target_id: Some(1),
            target_handle: "t",
            launch_case: "platform",
            evidence: &evidence,
            flows: &[],
            globally_unretained_flow_observations: 0,
        });
        assert_eq!(trace.summary.events_lost, 2);
        assert_eq!(trace.summary.unparseable_events, 3);
        assert_eq!(trace.summary.buffers_lost, 2);
        assert_eq!(trace.summary.completeness, "partial");
    }

    #[test]
    fn capture_wide_packet_loss_survives_an_empty_flow_snapshot() {
        let evidence = CaptureProcessEvidence {
            launch_pid: Some(1),
            launch_at: Some(Timestamp::from_nanos(2)),
            events: vec![ProcessEvent::started(
                1,
                0,
                "a.exe",
                "a",
                Timestamp::from_nanos(1),
            )],
            terminal_state: "complete".to_string(),
            ..CaptureProcessEvidence::default()
        };
        let trace = build_process_trace(ProcessTraceInput {
            session_id: "s",
            target_id: Some(1),
            target_handle: "t",
            launch_case: "direct",
            evidence: &evidence,
            flows: &[],
            globally_unretained_flow_observations: 4,
        });
        assert_eq!(trace.summary.limitations, 5);
        assert_eq!(trace.summary.completeness, "partial");
        assert!(trace.jsonl.contains("packet-evidence-unretained"));
    }
}
