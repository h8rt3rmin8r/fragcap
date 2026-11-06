// SPDX-License-Identifier: Apache-2.0

//! Bounded exact-owned route release for one authorized Deep Capture launch.

use std::time::{Duration, Instant};

use fragcap::deep_capture::{Budget, CaptureProcessEvidence, CleanupResult, CleanupStatus};
use fragcap::managed_shutdown::ProcessIdentity;
use fragcap::ProcessEvent;
use serde_json::{json, Value};

pub(super) struct ShutdownOutcome {
    pub result: CleanupResult,
    pub steps: Vec<Value>,
}

#[derive(Debug)]
struct Owner {
    identity: ProcessIdentity,
    depth: usize,
    force_attempted: bool,
    control_refused: bool,
}

#[derive(Debug)]
struct OwnerSelection {
    owners: Vec<Owner>,
    unresolved: Vec<String>,
}

fn failed(reason: impl Into<String>, steps: Vec<Value>) -> ShutdownOutcome {
    ShutdownOutcome {
        result: CleanupResult {
            resource: "route-owner".to_string(),
            status: CleanupStatus::Failed,
            reason: format!(
                "{}; close the launched application normally and relaunch it outside this session before using its networked features",
                reason.into()
            ),
        },
        steps,
    }
}

pub(super) fn missing_evidence() -> ShutdownOutcome {
    failed(
        "managed launch has no retained process-object evidence",
        vec![json!({"action":"identity-refusal","reason":"process evidence unavailable"})],
    )
}

fn same_path(left: &std::path::Path, right: &std::path::Path) -> bool {
    let normalized = |path: &std::path::Path| {
        let value = path.to_string_lossy();
        let ordinary = value.strip_prefix(r"\\?\").unwrap_or(&value);
        ordinary.replace('/', "\\").to_lowercase()
    };
    normalized(left) == normalized(right)
}

fn event_path_matches(image: &str, identity: &ProcessIdentity) -> bool {
    if image.contains(['\\', '/']) {
        same_path(std::path::Path::new(image), &identity.path)
    } else {
        identity
            .path
            .file_name()
            .is_some_and(|file| file.to_string_lossy().eq_ignore_ascii_case(image))
    }
}

fn exited_after(evidence: &CaptureProcessEvidence, pid: u32, started: fragcap::Timestamp) -> bool {
    evidence.events.iter().any(|event| {
        matches!(event, ProcessEvent::Exited { pid: exited, at } if *exited == pid && *at >= started)
    })
}

/// Descendants without an exact shutdown receipt are observation obligations,
/// not controllable owners. Their exit may follow a proven parent's close.
fn live_helpers(evidence: &CaptureProcessEvidence) -> Vec<(u32, String)> {
    let Some(root) = evidence.launch_pid else {
        return Vec::new();
    };
    evidence
        .events
        .iter()
        .filter_map(|event| {
            let ProcessEvent::Started { pid, image, at, .. } = event else {
                return None;
            };
            if *pid == root
                || !evidence.possible_descendant(*pid)
                || exited_after(evidence, *pid, *at)
            {
                return None;
            }
            let name = image.rsplit(['\\', '/']).next().unwrap_or(image);
            (!evidence.shutdown_scope.iter().any(|path| {
                path.file_name()
                    .is_some_and(|file| file.to_string_lossy().eq_ignore_ascii_case(name))
            }))
            .then(|| (*pid, name.to_string()))
        })
        .collect()
}

fn select_owners(evidence: &CaptureProcessEvidence) -> Result<OwnerSelection, String> {
    if evidence.shutdown_scope.is_empty() {
        return Err("authorized shutdown scope is absent".to_string());
    }
    if evidence.events_unretained > 0
        || evidence.unparseable_events > 0
        || evidence.managed_instance_limit_reached
        || evidence.watcher_ended
        || evidence
            .watcher_report
            .is_none_or(|report| report.lost_anything())
    {
        return Err("process event authority is incomplete".to_string());
    }
    let root = evidence
        .launch_pid
        .ok_or("managed launch receipt has no process ID")?;
    let earliest = evidence
        .launch_earliest_at
        .ok_or("managed launch receipt has no start bound")?;
    let latest = evidence
        .launch_at
        .ok_or("managed launch receipt has no end bound")?;
    let starts: Vec<_> = evidence
        .events
        .iter()
        .filter_map(|event| match event {
            ProcessEvent::Started {
                pid, parent, at, ..
            } => Some((*pid, *parent, *at)),
            _ => None,
        })
        .collect();
    let roots: Vec<_> = starts.iter().filter(|(pid, _, _)| *pid == root).collect();
    if roots.len() != 1
        || roots[0].2.nanos_since(earliest) < -1_000_000_000
        || roots[0].2.nanos_since(latest) > 1_000_000_000
    {
        return Err(
            "exact created root event is missing or outside its launch receipt".to_string(),
        );
    }
    let current = ProcessIdentity::current()
        .map_err(|error| format!("current account/session unavailable: {error}"))?;
    let mut owners = Vec::new();
    let mut unresolved = Vec::new();
    for event in &evidence.events {
        let ProcessEvent::Started { pid, image, at, .. } = event else {
            continue;
        };
        let name = image.rsplit(['\\', '/']).next().unwrap_or(image);
        let declared = evidence.shutdown_scope.iter().any(|path| {
            path.file_name()
                .is_some_and(|file| file.to_string_lossy().eq_ignore_ascii_case(name))
        });
        let depth = evidence.causal_depth(*pid, *at);
        if !declared && depth.is_none() {
            continue;
        }
        let Some(depth) = depth else {
            if *at >= earliest && !exited_after(evidence, *pid, *at) {
                unresolved.push(format!(
                    "candidate pid {pid} has no proven ancestry to the created root"
                ));
            }
            continue;
        };
        let receipts: Vec<_> = evidence
            .managed_instances
            .iter()
            .filter(|receipt| receipt.identity.pid == *pid && receipt.event_at == *at)
            .collect();
        let root_receipt = (*pid == root)
            .then_some(evidence.launch_identity.as_ref())
            .flatten();
        if root_receipt.is_none() && receipts.len() != 1 {
            if exited_after(evidence, *pid, *at) {
                continue;
            }
            unresolved.push(format!(
                "exact process-object receipt unavailable for causal owner pid {pid}"
            ));
            continue;
        }
        let identity = root_receipt.unwrap_or_else(|| &receipts[0].identity);
        let event_nanos = i128::from(at.as_nanos());
        if (identity.created_unix_nanos() - event_nanos).abs() > 5_000_000_000 {
            let reason = format!("creation-time mismatch for owner pid {pid}");
            if *pid == root {
                return Err(reason);
            }
            unresolved.push(reason);
            continue;
        }
        let path_proven = if declared {
            evidence
                .shutdown_scope
                .iter()
                .any(|path| same_path(path, &identity.path))
                && event_path_matches(image, identity)
        } else {
            event_path_matches(image, identity)
        };
        if !identity.same_user_session(&current) || !path_proven {
            let reason =
                format!("account, session, or canonical path mismatch for owner pid {pid}");
            if *pid == root {
                return Err(reason);
            }
            unresolved.push(reason);
            continue;
        }
        owners.push(Owner {
            identity: identity.clone(),
            depth,
            force_attempted: false,
            control_refused: false,
        });
    }
    if !owners.iter().any(|owner| owner.identity.pid == root)
        && !exited_after(evidence, root, roots[0].2)
    {
        return Err("created root has no exact process-object receipt".to_string());
    }
    owners.sort_by(|left, right| {
        right
            .depth
            .cmp(&left.depth)
            .then(left.identity.pid.cmp(&right.identity.pid))
    });
    Ok(OwnerSelection { owners, unresolved })
}

/// Retain every observed process that may still hold the child route. The
/// returned start identity is conservative: a missing or reused PID can later
/// prove exit, while a live matching PID or incomplete event stream blocks
/// Doctor recovery. No process control follows from this list.
pub(super) fn owner_exit_prerequisite(
    evidence: &CaptureProcessEvidence,
) -> Option<Vec<(u32, i64)>> {
    select_owners(evidence).ok()?;
    evidence.launch_pid?;
    let earliest = evidence.launch_earliest_at?;
    let mut pending = Vec::new();
    for event in &evidence.events {
        let ProcessEvent::Started { pid, image, at, .. } = event else {
            continue;
        };
        if *at < earliest || exited_after(evidence, *pid, *at) {
            continue;
        }
        let name = image.rsplit(['\\', '/']).next().unwrap_or(image);
        let declared = evidence.shutdown_scope.iter().any(|path| {
            path.file_name()
                .is_some_and(|file| file.to_string_lossy().eq_ignore_ascii_case(name))
        });
        if declared || evidence.possible_descendant(*pid) {
            pending.push((*pid, at.as_nanos()));
        }
    }
    pending.sort_unstable();
    pending.dedup();
    (pending.len() <= 256).then_some(pending)
}

pub(super) fn release(
    evidence: &mut CaptureProcessEvidence,
    budget: Budget,
    watch: &mut crate::orchestrator::ProcessReleaseWatch,
) -> ShutdownOutcome {
    release_monitored(evidence, budget, Duration::from_secs(60), Some(watch))
}

#[cfg(all(test, windows))]
fn release_with_grace(
    evidence: &CaptureProcessEvidence,
    budget: Budget,
    requested_grace: Duration,
) -> ShutdownOutcome {
    release_monitored(&mut evidence.clone(), budget, requested_grace, None)
}

fn release_monitored(
    evidence: &mut CaptureProcessEvidence,
    budget: Budget,
    requested_grace: Duration,
    mut watch: Option<&mut crate::orchestrator::ProcessReleaseWatch>,
) -> ShutdownOutcome {
    let mut steps = Vec::new();
    let mut owners: Vec<Owner> = Vec::new();
    let started = Instant::now();
    let ceiling = budget.remaining();
    let grace = requested_grace.min(ceiling);
    loop {
        if let Some(watch) = watch.as_deref_mut() {
            watch.drain_into(evidence);
        }
        let fresh = match select_owners(evidence) {
            Ok(selection) => selection,
            Err(reason) => {
                steps.push(json!({"action":"identity-refusal","reason":reason}));
                return failed(reason, steps);
            }
        };
        for mut owner in fresh.owners {
            if owners.iter().any(|known| {
                known.identity.pid == owner.identity.pid
                    && known.identity.created == owner.identity.created
            }) {
                continue;
            }
            match owner.identity.exited() {
                Ok(true) => steps.push(json!({"action":"exit-verified","pid":owner.identity.pid,"phase":"before-grace"})),
                Ok(false) => match owner.identity.request_graceful_close() {
                    Ok(windows) => steps.push(json!({"action":"graceful-request","pid":owner.identity.pid,"windows":windows})),
                    Err(error) => steps.push(json!({"action":"graceful-failed","pid":owner.identity.pid,"reason":error.to_string()})),
                },
                Err(error) => {
                    owner.control_refused = true;
                    steps.push(json!({"action":"identity-refusal","pid":owner.identity.pid,"reason":error.to_string()}));
                }
            }
            owners.push(owner);
        }
        let mut remaining = Vec::new();
        for owner in &owners {
            match owner.identity.exited() {
                Ok(true) => {}
                Ok(false) => remaining.push(owner.identity.pid),
                Err(error) => {
                    steps.push(json!({"action":"exit-check-failed","pid":owner.identity.pid,"reason":error.to_string()}));
                    remaining.push(owner.identity.pid);
                }
            }
        }
        let helpers = live_helpers(evidence);
        if remaining.is_empty() && helpers.is_empty() && fresh.unresolved.is_empty() {
            // A graceful-close handler can launch a child after Capture has
            // stopped. Keep ETW running through this decision, close it only
            // after known owners exit, then reconcile every queued start/exit.
            if let Some(watch) = watch.take() {
                watch.finish_into(evidence);
                let final_selection = match select_owners(evidence) {
                    Ok(selection) => selection,
                    Err(reason) => {
                        steps.push(json!({"action":"identity-refusal","reason":reason}));
                        return failed(reason, steps);
                    }
                };
                for owner in final_selection.owners {
                    if !owner.identity.exited().unwrap_or(false) {
                        steps.push(
                            json!({"action":"late-owner-unresolved","pid":owner.identity.pid}),
                        );
                        return failed(
                            format!("route owner pid {} started before watcher closure and remains live", owner.identity.pid),
                            steps,
                        );
                    }
                }
                let late_helpers = live_helpers(evidence);
                if !late_helpers.is_empty() || !final_selection.unresolved.is_empty() {
                    steps.push(json!({"action":"unresolved-route-events","helpers":late_helpers,"declared_images":final_selection.unresolved}));
                    return failed(
                        format!(
                            "late route events have no verified exit: helpers={late_helpers:?}, declared_images={:?}",
                            final_selection.unresolved
                        ),
                        steps,
                    );
                }
            }
            for owner in &owners {
                steps.push(json!({"action":"exit-verified","pid":owner.identity.pid,"phase":"route-release"}));
            }
            return ShutdownOutcome {
                result: CleanupResult {
                    resource: "route-owner".to_string(),
                    status: CleanupStatus::Released,
                    reason: "all exact session-owned route owners exited before proxy stop"
                        .to_string(),
                },
                steps,
            };
        }
        let elapsed = started.elapsed();
        if elapsed >= grace {
            for owner in &mut owners {
                if owner.force_attempted
                    || owner.control_refused
                    || owner.identity.exited().unwrap_or(false)
                {
                    continue;
                }
                owner.force_attempted = true;
                steps.push(json!({"action":"grace-expired","pid":owner.identity.pid}));
                match owner.identity.force_close() {
                    Ok(()) => steps.push(json!({"action":"force-request","pid":owner.identity.pid,"result":"issued-awaiting-exit"})),
                    Err(error) => steps.push(json!({"action":"force-failed","pid":owner.identity.pid,"reason":error.to_string()})),
                }
            }
        }
        if elapsed >= ceiling {
            steps.push(json!({"action":"release-deadline","unresolved_pids":remaining,"unresolved_helpers":helpers,"unresolved_declared_images":fresh.unresolved}));
            return failed(
                format!("exact route owners or observed events remain or cannot be verified: owners={remaining:?}, helpers={helpers:?}, declared_images={:?}", fresh.unresolved),
                steps,
            );
        }
        std::thread::sleep((ceiling - elapsed).min(Duration::from_millis(250)));
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use fragcap::deep_capture::ManagedProcessInstance;
    use std::os::windows::process::CommandExt;
    use std::process::{Child, Command, Stdio};

    struct ChildGuard(Child);

    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    fn child() -> ChildGuard {
        let mut command = Command::new(std::env::current_exe().expect("test executable"));
        command
            .arg("--ignored")
            .arg("managed_shutdown_child_waits")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        command.creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW);
        let child = command.spawn().expect("hidden controlled child");
        ChildGuard(child)
    }

    fn child_evidence(child: &ChildGuard) -> CaptureProcessEvidence {
        let identity = ProcessIdentity::observe(child.0.id()).expect("child identity");
        let created = i64::try_from(identity.created_unix_nanos()).expect("creation timestamp");
        let at = fragcap::Timestamp::from_nanos(created);
        let mut evidence = CaptureProcessEvidence {
            shutdown_scope: vec![identity.path.clone()],
            launch_pid: Some(identity.pid),
            launch_earliest_at: Some(fragcap::Timestamp::from_nanos(created - 1_000_000_000)),
            launch_at: Some(fragcap::Timestamp::from_nanos(created + 1_000_000_000)),
            watcher_report: Some(Default::default()),
            ..Default::default()
        };
        evidence.events.push(ProcessEvent::started(
            identity.pid,
            std::process::id(),
            identity.path.display().to_string(),
            "",
            at,
        ));
        evidence.managed_instances.push(ManagedProcessInstance {
            event_at: at,
            identity,
        });
        evidence
    }

    #[test]
    #[ignore = "spawned only by exact owned shutdown tests"]
    fn managed_shutdown_child_waits() {
        std::thread::sleep(Duration::from_secs(30));
    }

    #[test]
    fn exact_owned_child_exits_after_bounded_force() {
        let owned = child();
        let evidence = child_evidence(&owned);
        let outcome = release_with_grace(
            &evidence,
            Budget::new(Duration::from_secs(3)),
            Duration::from_millis(50),
        );
        assert_eq!(
            outcome.result.status,
            CleanupStatus::Released,
            "{:?}",
            outcome.steps
        );
        assert!(outcome
            .steps
            .iter()
            .any(|step| step["action"] == "force-request"));
        assert!(outcome
            .steps
            .iter()
            .any(|step| step["action"] == "graceful-request" && step["windows"] == 0));
        assert!(evidence.managed_instances[0].identity.exited().unwrap());
    }

    #[test]
    fn receipt_backed_undeclared_helper_is_closed_after_client_observation() {
        let root = child();
        let helper = child();
        let mut unrelated = child();
        let mut evidence = child_evidence(&root);
        let identity = ProcessIdentity::observe(helper.0.id()).unwrap();
        let at = fragcap::Timestamp::from_nanos(identity.created_unix_nanos() as i64);
        evidence.events.push(ProcessEvent::started(
            identity.pid,
            root.0.id(),
            identity.path.display().to_string(),
            "",
            at,
        ));
        evidence.managed_instances.push(ManagedProcessInstance {
            event_at: at,
            identity,
        });
        let outcome = release_with_grace(
            &evidence,
            Budget::new(Duration::from_secs(3)),
            Duration::from_millis(50),
        );
        assert_eq!(
            outcome.result.status,
            CleanupStatus::Released,
            "{:?}",
            outcome.steps
        );
        assert!(outcome
            .steps
            .iter()
            .any(|step| { step["action"] == "force-request" && step["pid"] == helper.0.id() }));
        assert!(unrelated.0.try_wait().unwrap().is_none());
    }

    #[test]
    fn same_name_unrelated_process_receives_no_control_request() {
        let owned = child();
        let mut unrelated = child();
        let mut evidence = child_evidence(&owned);
        evidence.events.push(ProcessEvent::started(
            unrelated.0.id(),
            std::process::id(),
            evidence.shutdown_scope[0].display().to_string(),
            "",
            evidence.launch_at.unwrap(),
        ));
        let outcome = release_with_grace(
            &evidence,
            Budget::new(Duration::from_millis(100)),
            Duration::ZERO,
        );
        assert_eq!(outcome.result.status, CleanupStatus::Failed);
        assert!(unrelated.0.try_wait().unwrap().is_none());
        let prerequisite = owner_exit_prerequisite(&evidence).expect("complete event authority");
        assert!(prerequisite.iter().any(|(pid, _)| *pid == unrelated.0.id()));
        assert!(!outcome.steps.iter().any(|step| {
            matches!(
                step["action"].as_str(),
                Some("graceful-request" | "force-request")
            ) && step["pid"] == unrelated.0.id()
        }));
    }

    #[test]
    fn full_event_path_must_match_the_canonical_owned_path() {
        let owned = child();
        let mut evidence = child_evidence(&owned);
        if let ProcessEvent::Started { image, .. } = &mut evidence.events[0] {
            *image = r"C:\different\same-name.exe".into();
        }
        assert!(select_owners(&evidence).is_err());
        assert!(!evidence.managed_instances[0].identity.exited().unwrap());
    }

    #[test]
    fn proven_root_and_client_exit_despite_unproven_declared_image() {
        let root = child();
        let client = child();
        let mut unrelated = child();
        let mut evidence = child_evidence(&root);
        let client_identity = ProcessIdentity::observe(client.0.id()).unwrap();
        let client_at = fragcap::Timestamp::from_nanos(
            i64::try_from(client_identity.created_unix_nanos()).unwrap(),
        );
        evidence.events.push(ProcessEvent::started(
            client_identity.pid,
            root.0.id(),
            client_identity.path.display().to_string(),
            "",
            client_at,
        ));
        evidence.managed_instances.push(ManagedProcessInstance {
            event_at: client_at,
            identity: client_identity,
        });
        evidence.events.push(ProcessEvent::started(
            unrelated.0.id(),
            std::process::id(),
            evidence.shutdown_scope[0].display().to_string(),
            "",
            evidence.launch_at.unwrap(),
        ));
        let outcome = release_with_grace(
            &evidence,
            Budget::new(Duration::from_secs(3)),
            Duration::from_millis(50),
        );
        assert_eq!(outcome.result.status, CleanupStatus::Failed);
        assert!(
            outcome.result.reason.contains("no proven ancestry"),
            "{:?}",
            outcome.steps
        );
        for pid in [root.0.id(), client.0.id()] {
            assert!(outcome
                .steps
                .iter()
                .any(|step| step["action"] == "force-request" && step["pid"] == pid));
        }
        assert!(evidence
            .managed_instances
            .iter()
            .all(|receipt| receipt.identity.exited().unwrap()));
        assert!(unrelated.0.try_wait().unwrap().is_none());
        assert!(!outcome.steps.iter().any(|step| {
            matches!(
                step["action"].as_str(),
                Some("graceful-request" | "force-request")
            ) && step["pid"] == unrelated.0.id()
        }));
    }

    #[test]
    fn exited_unproven_declared_image_does_not_block_complete_release() {
        let root = child();
        let mut unrelated = child();
        let mut evidence = child_evidence(&root);
        let at = evidence.launch_at.unwrap();
        evidence.events.push(ProcessEvent::started(
            unrelated.0.id(),
            std::process::id(),
            evidence.shutdown_scope[0].display().to_string(),
            "",
            at,
        ));
        unrelated.0.kill().unwrap();
        unrelated.0.wait().unwrap();
        evidence.events.push(ProcessEvent::Exited {
            pid: unrelated.0.id(),
            at: fragcap::Timestamp::from_nanos(at.as_nanos() + 1),
        });
        let outcome = release_with_grace(
            &evidence,
            Budget::new(Duration::from_secs(3)),
            Duration::from_millis(50),
        );
        assert_eq!(
            outcome.result.status,
            CleanupStatus::Released,
            "{:?}",
            outcome.steps
        );
        assert!(evidence.managed_instances[0].identity.exited().unwrap());
        assert!(!outcome.steps.iter().any(|step| {
            matches!(
                step["action"].as_str(),
                Some("graceful-request" | "force-request")
            ) && step["pid"] == unrelated.0.id()
        }));
    }

    #[test]
    fn mismatched_receipt_refuses_control() {
        let mut owned = child();
        let mut evidence = child_evidence(&owned);
        evidence.managed_instances[0].event_at = fragcap::Timestamp::from_nanos(0);
        let outcome = release_with_grace(&evidence, Budget::new(Duration::ZERO), Duration::ZERO);
        assert_eq!(outcome.result.status, CleanupStatus::Failed);
        assert!(owned.0.try_wait().unwrap().is_none());
        assert_eq!(outcome.steps[0]["action"], "identity-refusal");
    }

    #[test]
    fn creation_path_and_ancestry_mismatches_refuse_control() {
        let mut owned = child();
        let mut evidence = child_evidence(&owned);
        evidence.managed_instances[0].identity.created += 100_000_000;
        assert!(select_owners(&evidence)
            .unwrap_err()
            .contains("creation-time mismatch"));
        evidence = child_evidence(&owned);
        evidence.shutdown_scope = vec![std::path::PathBuf::from("X:\\Different")
            .join(evidence.shutdown_scope[0].file_name().unwrap())];
        assert!(select_owners(&evidence)
            .unwrap_err()
            .contains("canonical path mismatch"));
        evidence = child_evidence(&owned);
        evidence.managed_instances[0].identity.session = u32::MAX;
        assert!(select_owners(&evidence)
            .unwrap_err()
            .contains("account, session, or canonical path mismatch"));
        evidence = child_evidence(&owned);
        evidence.events.push(ProcessEvent::started(
            333_333,
            222_222,
            evidence.shutdown_scope[0].display().to_string(),
            "",
            evidence.launch_at.unwrap(),
        ));
        assert!(select_owners(&evidence)
            .unwrap()
            .unresolved
            .iter()
            .any(|reason| reason.contains("no proven ancestry")));
        assert!(owned.0.try_wait().unwrap().is_none());
    }

    #[test]
    fn etw_loss_refuses_control_and_root_launch_receipt_survives_event_query_loss() {
        let mut owned = child();
        let mut evidence = child_evidence(&owned);
        evidence.launch_identity = Some(evidence.managed_instances.remove(0).identity);
        assert_eq!(select_owners(&evidence).unwrap().owners.len(), 1);
        evidence.watcher_report.as_mut().unwrap().events_lost = 1;
        assert!(select_owners(&evidence)
            .unwrap_err()
            .contains("process event authority is incomplete"));
        assert!(owner_exit_prerequisite(&evidence).is_none());
        assert!(owned.0.try_wait().unwrap().is_none());
    }

    #[test]
    fn synthetic_publisher_chain_releases_descendant_before_root() {
        let root = child();
        let descendant = child();
        let mut evidence = child_evidence(&root);
        let identity = ProcessIdentity::observe(descendant.0.id()).unwrap();
        let at =
            fragcap::Timestamp::from_nanos(i64::try_from(identity.created_unix_nanos()).unwrap());
        evidence.events.push(ProcessEvent::started(
            identity.pid,
            root.0.id(),
            identity.path.display().to_string(),
            "",
            at,
        ));
        evidence.managed_instances.push(ManagedProcessInstance {
            event_at: at,
            identity,
        });
        let owners = select_owners(&evidence).unwrap().owners;
        assert_eq!(
            owners.iter().map(|owner| owner.depth).collect::<Vec<_>>(),
            [1, 0]
        );
    }

    #[test]
    fn exited_game_does_not_release_live_launcher_route_owner() {
        let mut launcher = child();
        let mut game = child();
        let mut evidence = child_evidence(&launcher);
        let identity = ProcessIdentity::observe(game.0.id()).unwrap();
        let at =
            fragcap::Timestamp::from_nanos(i64::try_from(identity.created_unix_nanos()).unwrap());
        evidence.events.push(ProcessEvent::started(
            identity.pid,
            launcher.0.id(),
            identity.path.display().to_string(),
            "",
            at,
        ));
        evidence.managed_instances.push(ManagedProcessInstance {
            event_at: at,
            identity,
        });
        game.0.kill().unwrap();
        game.0.wait().unwrap();
        evidence.events.push(ProcessEvent::Exited {
            pid: game.0.id(),
            at: fragcap::Timestamp::from_nanos(at.as_nanos() + 1),
        });
        let pending = release_with_grace(&evidence, Budget::new(Duration::ZERO), Duration::ZERO);
        assert_ne!(pending.result.status, CleanupStatus::Released);
        assert!(launcher.0.try_wait().unwrap().is_none());
        launcher.0.kill().unwrap();
        launcher.0.wait().unwrap();
        let released = release_with_grace(&evidence, Budget::new(Duration::ZERO), Duration::ZERO);
        assert_eq!(released.result.status, CleanupStatus::Released);
    }

    #[test]
    fn force_revalidation_rejects_changed_pid_without_killing_either_child() {
        let mut owned = child();
        let mut unrelated = child();
        let mut forged = ProcessIdentity::observe(owned.0.id()).unwrap();
        forged.pid = unrelated.0.id();
        assert!(forged.force_close().is_err());
        assert!(owned.0.try_wait().unwrap().is_none());
        assert!(unrelated.0.try_wait().unwrap().is_none());
    }

    #[test]
    fn publisher_depth_requires_unambiguous_chain() {
        let at = fragcap::Timestamp::from_nanos(1);
        let mut evidence = CaptureProcessEvidence {
            launch_pid: Some(10),
            launch_earliest_at: Some(at),
            launch_at: Some(at),
            ..CaptureProcessEvidence::default()
        };
        evidence.events.extend([
            ProcessEvent::started(10, 0, "root.exe", "", at),
            ProcessEvent::started(20, 10, "publisher.exe", "", at),
            ProcessEvent::started(30, 20, "client.exe", "", at),
        ]);
        assert_eq!(evidence.causal_depth(30, at), Some(2));
        evidence
            .events
            .push(ProcessEvent::started(20, 99, "other.exe", "", at));
        assert_eq!(evidence.causal_depth(30, at), None);
    }

    #[test]
    fn unrelated_descendant_blocks_release_after_exact_owner_close_request() {
        let owned = child();
        let mut evidence = child_evidence(&owned);
        evidence
            .shutdown_scope
            .insert(0, std::path::PathBuf::from("steam.exe"));
        evidence.events.push(ProcessEvent::started(
            999_999,
            owned.0.id(),
            "unrelated-game.exe",
            "",
            evidence.launch_at.unwrap(),
        ));
        let outcome = release_with_grace(&evidence, Budget::new(Duration::ZERO), Duration::ZERO);
        assert_eq!(outcome.result.status, CleanupStatus::Failed);
        assert!(outcome
            .steps
            .iter()
            .any(|step| step["action"] == "graceful-request"));
        assert!(outcome.result.reason.contains("unrelated-game.exe"));
    }

    #[test]
    fn undeclared_direct_descendant_cannot_be_silently_released() {
        let owned = child();
        let mut evidence = child_evidence(&owned);
        evidence.events.push(ProcessEvent::started(
            999_998,
            owned.0.id(),
            "helper.exe",
            "",
            evidence.launch_at.unwrap(),
        ));
        evidence.managed_instances[0]
            .identity
            .force_close()
            .unwrap();
        let started = Instant::now();
        while !evidence.managed_instances[0].identity.exited().unwrap()
            && started.elapsed() < Duration::from_secs(2)
        {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(evidence.managed_instances[0].identity.exited().unwrap());
        evidence.events.push(ProcessEvent::Exited {
            pid: owned.0.id(),
            at: evidence.launch_at.unwrap(),
        });
        let outcome = release_with_grace(&evidence, Budget::new(Duration::ZERO), Duration::ZERO);
        assert_eq!(outcome.result.status, CleanupStatus::Failed);
        assert!(outcome.result.reason.contains("helper.exe"));
    }

    #[test]
    fn observed_helper_exit_allows_release_after_owner_exit() {
        let owned = child();
        let mut evidence = child_evidence(&owned);
        let at = evidence.launch_at.unwrap();
        evidence.events.push(ProcessEvent::started(
            999_998,
            owned.0.id(),
            "helper.exe",
            "",
            at,
        ));
        evidence
            .events
            .push(ProcessEvent::Exited { pid: 999_998, at });
        evidence.managed_instances[0]
            .identity
            .force_close()
            .unwrap();
        let started = Instant::now();
        while !evidence.managed_instances[0].identity.exited().unwrap()
            && started.elapsed() < Duration::from_secs(2)
        {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(evidence.managed_instances[0].identity.exited().unwrap());
        evidence.events.push(ProcessEvent::Exited {
            pid: owned.0.id(),
            at,
        });

        let outcome = release_with_grace(&evidence, Budget::new(Duration::ZERO), Duration::ZERO);
        assert_eq!(outcome.result.status, CleanupStatus::Released);
    }

    #[test]
    fn absent_client_releases_but_late_client_after_root_exit_remains_an_obligation() {
        let root = child();
        let mut evidence = child_evidence(&root);
        let at = evidence.launch_at.unwrap();
        let exited_at = fragcap::Timestamp::from_nanos(at.as_nanos() + 1);
        evidence.managed_instances[0]
            .identity
            .force_close()
            .unwrap();
        let started = Instant::now();
        while !evidence.managed_instances[0].identity.exited().unwrap()
            && started.elapsed() < Duration::from_secs(2)
        {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(evidence.managed_instances[0].identity.exited().unwrap());
        evidence.events.push(ProcessEvent::Exited {
            pid: root.0.id(),
            at: exited_at,
        });
        let absent = release_with_grace(&evidence, Budget::new(Duration::ZERO), Duration::ZERO);
        assert_eq!(absent.result.status, CleanupStatus::Released);

        let late_at = fragcap::Timestamp::from_nanos(at.as_nanos() + 2);
        evidence.events.push(ProcessEvent::started(
            999_996,
            root.0.id(),
            "game.exe",
            "",
            late_at,
        ));
        let late = release_with_grace(&evidence, Budget::new(Duration::ZERO), Duration::ZERO);
        assert_eq!(late.result.status, CleanupStatus::Failed);
        assert!(late.result.reason.contains("game.exe"));
        assert!(owner_exit_prerequisite(&evidence)
            .unwrap()
            .iter()
            .any(|(pid, _)| *pid == 999_996));

        evidence.events.push(ProcessEvent::Exited {
            pid: 999_996,
            at: fragcap::Timestamp::from_nanos(late_at.as_nanos() + 1),
        });
        let recovered = release_with_grace(&evidence, Budget::new(Duration::ZERO), Duration::ZERO);
        assert_eq!(recovered.result.status, CleanupStatus::Released);
        assert_eq!(late.result.status, CleanupStatus::Failed);
    }

    #[cfg(feature = "etw")]
    #[test]
    fn late_helper_during_grace_blocks_route_release() {
        let owned = child();
        let mut evidence = child_evidence(&owned);
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watch = crate::orchestrator::ProcessReleaseWatch::test_receiver(rx);
        let root = owned.0.id();
        let at = evidence.launch_at.unwrap();
        let late_tx = tx.clone();
        let sender = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(30));
            late_tx
                .send(ProcessEvent::started(999_997, root, "helper.exe", "", at))
                .expect("watch remains alive through release");
        });

        let outcome = release_monitored(
            &mut evidence,
            Budget::new(Duration::from_secs(2)),
            Duration::from_millis(250),
            Some(&mut watch),
        );
        sender.join().unwrap();
        drop(tx);
        assert_eq!(outcome.result.status, CleanupStatus::Failed);
        assert!(
            outcome.result.reason.contains("helper.exe"),
            "{:?}",
            outcome.steps
        );
        assert!(outcome
            .steps
            .iter()
            .any(|step| step["action"] == "graceful-request"));
    }
}
