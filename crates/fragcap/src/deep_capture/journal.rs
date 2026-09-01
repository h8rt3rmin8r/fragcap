// SPDX-License-Identifier: Apache-2.0

//! Durable, bounded ownership records for Deep Capture external effects.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use super::artifacts::open_sensitive_file;

pub const RESOURCE_JOURNAL: &str = "resource-journal.jsonl";
const SCHEMA_VERSION: u64 = 1;
const MAX_BYTES: u64 = 4 * 1024 * 1024;
const MAX_RECORDS: usize = 16_384;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceKind {
    Artifact,
    Capture,
    Launch,
    Proxy,
    Route,
    Trust,
}

impl ResourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Artifact => "artifact",
            Self::Capture => "capture",
            Self::Launch => "launch",
            Self::Proxy => "proxy",
            Self::Route => "route",
            Self::Trust => "trust",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "artifact" => Self::Artifact,
            "capture" => Self::Capture,
            "launch" => Self::Launch,
            "proxy" => Self::Proxy,
            "route" => Self::Route,
            "trust" => Self::Trust,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceState {
    Pending,
    Applied,
    CleanupPending,
    Released,
    Retained,
    Failed,
    TimedOut,
    NotApplied,
}

impl ResourceState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Applied => "applied",
            Self::CleanupPending => "cleanup-pending",
            Self::Released => "released",
            Self::Retained => "retained",
            Self::Failed => "failed",
            Self::TimedOut => "timed-out",
            Self::NotApplied => "not-applied",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "pending" => Self::Pending,
            "applied" => Self::Applied,
            "cleanup-pending" => Self::CleanupPending,
            "released" => Self::Released,
            "retained" => Self::Retained,
            "failed" => Self::Failed,
            "timed-out" => Self::TimedOut,
            "not-applied" => Self::NotApplied,
            _ => return None,
        })
    }

    pub fn terminal(self) -> bool {
        matches!(self, Self::Released | Self::Retained | Self::NotApplied)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceTransition {
    pub sequence: u64,
    pub resource_id: String,
    pub kind: ResourceKind,
    pub target: String,
    pub ownership: String,
    pub action: String,
    pub state: ResourceState,
    pub detail: String,
}

impl ResourceTransition {
    pub fn new(
        resource_id: impl Into<String>,
        kind: ResourceKind,
        target: impl Into<String>,
        ownership: impl Into<String>,
        action: impl Into<String>,
        state: ResourceState,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            sequence: 0,
            resource_id: resource_id.into(),
            kind,
            target: target.into(),
            ownership: ownership.into(),
            action: action.into(),
            state,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JournalStatus {
    Complete,
    CrashPrefix,
    UnknownVersion,
}

#[derive(Clone, Debug)]
pub struct JournalPrefix {
    pub session_id: String,
    pub plan_id: String,
    pub transitions: Vec<ResourceTransition>,
    pub status: JournalStatus,
}

impl JournalPrefix {
    pub fn latest(&self) -> BTreeMap<&str, &ResourceTransition> {
        let mut latest = BTreeMap::new();
        for transition in &self.transitions {
            latest.insert(transition.resource_id.as_str(), transition);
        }
        latest
    }

    pub fn recovery_plan(&self) -> RecoveryPlan {
        let mut actions = Vec::new();
        let mut refusals = Vec::new();
        for transition in self.latest().into_values() {
            if transition.state.terminal() {
                continue;
            }
            if transition.ownership.trim().is_empty() || transition.target.trim().is_empty() {
                refusals.push(RecoveryRefusal {
                    resource_id: transition.resource_id.clone(),
                    reason: "exact ownership evidence is unavailable".into(),
                });
                continue;
            }
            if transition.state == ResourceState::Pending {
                refusals.push(RecoveryRefusal {
                    resource_id: transition.resource_id.clone(),
                    reason: "the journal does not prove that the pending effect occurred".into(),
                });
                continue;
            }
            let exact = match transition.kind {
                ResourceKind::Trust => {
                    transition
                        .target
                        .strip_prefix("sha1:")
                        .is_some_and(|value| {
                            value.len() == 40 && value.chars().all(|ch| ch.is_ascii_hexdigit())
                        })
                }
                ResourceKind::Route | ResourceKind::Proxy | ResourceKind::Capture => true,
                ResourceKind::Launch => {
                    transition.target.contains("pid:") && transition.target.contains("created:")
                }
                ResourceKind::Artifact => transition.target.contains(transition.ownership.as_str()),
            };
            if !exact {
                refusals.push(RecoveryRefusal {
                    resource_id: transition.resource_id.clone(),
                    reason:
                        "resource identity is insufficient for unrelated-resource-safe recovery"
                            .into(),
                });
                continue;
            }
            actions.push(RecoveryAction {
                resource_id: transition.resource_id.clone(),
                kind: transition.kind,
                target: transition.target.clone(),
                ownership: transition.ownership.clone(),
                action: transition.action.clone(),
            });
        }
        RecoveryPlan { actions, refusals }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryAction {
    pub resource_id: String,
    pub kind: ResourceKind,
    pub target: String,
    pub ownership: String,
    pub action: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryRefusal {
    pub resource_id: String,
    pub reason: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecoveryPlan {
    pub actions: Vec<RecoveryAction>,
    pub refusals: Vec<RecoveryRefusal>,
}

pub struct ResourceJournal {
    path: PathBuf,
    file: File,
    session_id: String,
    plan_id: String,
    sequence: u64,
    finished: bool,
}

impl ResourceJournal {
    pub fn create(
        bundle: &Path,
        session_id: impl Into<String>,
        plan_id: impl Into<String>,
    ) -> io::Result<Self> {
        fs::create_dir_all(bundle)?;
        let path = bundle.join(RESOURCE_JOURNAL);
        let file = open_sensitive_file(&path)?;
        let mut journal = Self {
            path,
            file,
            session_id: session_id.into(),
            plan_id: plan_id.into(),
            sequence: 0,
            finished: false,
        };
        let session_id = journal.session_id.clone();
        let plan_id = journal.plan_id.clone();
        journal.write_sync(&json!({
            "type": "resource-journal.header",
            "schema_version": SCHEMA_VERSION,
            "session_id": session_id,
            "plan_id": plan_id,
        }))?;
        Ok(journal)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn resume(path: &Path) -> io::Result<Self> {
        let metadata = path.symlink_metadata()?;
        if !metadata.file_type().is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "resource journal is not a regular file",
            ));
        }
        let prefix = read_resource_journal(path)?;
        if prefix.status != JournalStatus::CrashPrefix {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "only a crash-prefix journal can resume",
            ));
        }
        let file = OpenOptions::new().append(true).read(true).open(path)?;
        Ok(Self {
            path: path.to_path_buf(),
            file,
            session_id: prefix.session_id,
            plan_id: prefix.plan_id,
            sequence: prefix.transitions.len() as u64,
            finished: false,
        })
    }

    /// Persist one transition and synchronize it before returning.
    pub fn append(&mut self, mut transition: ResourceTransition) -> io::Result<u64> {
        if self.finished {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "journal is finished",
            ));
        }
        self.sequence = self.sequence.checked_add(1).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "journal sequence overflow")
        })?;
        transition.sequence = self.sequence;
        self.write_sync(&json!({
            "type": "resource.transition",
            "schema_version": SCHEMA_VERSION,
            "sequence": transition.sequence,
            "resource_id": transition.resource_id,
            "kind": transition.kind.as_str(),
            "target": transition.target,
            "ownership": transition.ownership,
            "action": transition.action,
            "state": transition.state.as_str(),
            "detail": transition.detail,
        }))?;
        Ok(self.sequence)
    }

    pub fn finish(&mut self) -> io::Result<()> {
        if self.finished {
            return Ok(());
        }
        let terminal_resources = read_resource_journal(&self.path)?
            .latest()
            .into_values()
            .filter(|transition| transition.state.terminal())
            .count();
        self.write_sync(&json!({
            "type": "resource-journal.trailer",
            "schema_version": SCHEMA_VERSION,
            "session_id": self.session_id,
            "records": self.sequence,
            "terminal_resources": terminal_resources,
        }))?;
        self.finished = true;
        Ok(())
    }

    /// Replace a completed journal with a compact audit snapshot atomically.
    pub fn compact(&mut self) -> io::Result<()> {
        self.finish()?;
        let prefix = read_resource_journal(&self.path)?;
        if prefix.status != JournalStatus::Complete {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "only complete journals compact",
            ));
        }
        let temporary = self.path.with_extension("jsonl.tmp");
        let mut output = open_sensitive_file(&temporary)?;
        write_line(
            &mut output,
            &json!({
                "type": "resource-journal.header",
                "schema_version": SCHEMA_VERSION,
                "session_id": prefix.session_id,
                "plan_id": prefix.plan_id,
                "compacted": true,
            }),
        )?;
        for (index, transition) in prefix.latest().into_values().enumerate() {
            let mut transition = transition.clone();
            transition.sequence = index as u64 + 1;
            write_line(&mut output, &transition_json(&transition))?;
        }
        write_line(
            &mut output,
            &json!({
                "type": "resource-journal.trailer",
                "schema_version": SCHEMA_VERSION,
                "session_id": self.session_id,
                "records": prefix.latest().len(),
                "compacted": true,
            }),
        )?;
        output.sync_all()?;
        drop(output);
        atomic_replace(&temporary, &self.path)?;
        #[cfg(unix)]
        if let Some(parent) = self.path.parent() {
            File::open(parent)?.sync_all()?;
        }
        Ok(())
    }

    fn write_sync(&mut self, value: &Value) -> io::Result<()> {
        write_line(&mut self.file, value)?;
        self.file.sync_all()
    }
}

/// Replay only recovery decisions whose exact mutation is supplied by the
/// caller. Every attempt and outcome is synchronized into the same journal.
pub fn recover_resource_journal(
    path: &Path,
    mut execute: impl FnMut(&RecoveryAction) -> Result<String, String>,
) -> io::Result<RecoveryPlan> {
    let prefix = read_resource_journal(path)?;
    let plan = prefix.recovery_plan();
    if plan.actions.is_empty() {
        return Ok(plan);
    }
    let latest = prefix.latest();
    let mut journal = ResourceJournal::resume(path)?;
    let mut failed = false;
    for action in &plan.actions {
        let Some(previous) = latest.get(action.resource_id.as_str()) else {
            continue;
        };
        journal.append(ResourceTransition::new(
            &action.resource_id,
            action.kind,
            &action.target,
            &action.ownership,
            &action.action,
            ResourceState::CleanupPending,
            "recovery attempt",
        ))?;
        let (state, detail) = match execute(action) {
            Ok(detail) => (ResourceState::Released, detail),
            Err(detail) => {
                failed = true;
                (ResourceState::Failed, detail)
            }
        };
        journal.append(ResourceTransition::new(
            &previous.resource_id,
            previous.kind,
            &previous.target,
            &previous.ownership,
            &previous.action,
            state,
            detail,
        ))?;
    }
    if !failed && plan.refusals.is_empty() {
        journal.finish()?;
    }
    Ok(plan)
}

pub fn read_resource_journal(path: &Path) -> io::Result<JournalPrefix> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "resource journal exceeds byte limit",
        ));
    }
    let mut lines = BufReader::new(File::open(path)?).lines();
    let header = lines.next().ok_or_else(|| {
        io::Error::new(io::ErrorKind::UnexpectedEof, "resource journal is empty")
    })??;
    let header: Value = serde_json::from_str(&header).map_err(invalid_json)?;
    if header.get("type").and_then(Value::as_str) != Some("resource-journal.header") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "resource journal header is missing",
        ));
    }
    if header.get("schema_version").and_then(Value::as_u64) != Some(SCHEMA_VERSION) {
        return Ok(JournalPrefix {
            session_id: String::new(),
            plan_id: String::new(),
            transitions: Vec::new(),
            status: JournalStatus::UnknownVersion,
        });
    }
    let session_id = required_string(&header, "session_id")?;
    let plan_id = required_string(&header, "plan_id")?;
    let mut transitions = Vec::new();
    let mut latest = BTreeMap::<String, ResourceTransition>::new();
    let mut trailer = false;
    for (index, line) in lines.enumerate() {
        if index >= MAX_RECORDS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "resource journal exceeds record limit",
            ));
        }
        let line = line?;
        let value: Value = serde_json::from_str(&line).map_err(invalid_json)?;
        match value.get("type").and_then(Value::as_str) {
            Some("resource.transition") => {
                if trailer {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "record follows journal trailer",
                    ));
                }
                let transition = parse_transition(&value)?;
                let expected = transitions.len() as u64 + 1;
                if transition.sequence != expected {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "journal sequence is not contiguous",
                    ));
                }
                validate_transition(latest.get(&transition.resource_id), &transition)?;
                latest.insert(transition.resource_id.clone(), transition.clone());
                transitions.push(transition);
            }
            Some("resource-journal.trailer") => trailer = true,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unknown resource journal record",
                ))
            }
        }
    }
    Ok(JournalPrefix {
        session_id,
        plan_id,
        transitions,
        status: if trailer {
            JournalStatus::Complete
        } else {
            JournalStatus::CrashPrefix
        },
    })
}

fn validate_transition(
    previous: Option<&ResourceTransition>,
    next: &ResourceTransition,
) -> io::Result<()> {
    if next.resource_id.trim().is_empty() || next.action.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "resource transition identity is empty",
        ));
    }
    if let Some(previous) = previous.filter(|value| value.resource_id == next.resource_id) {
        let allowed = match previous.state {
            ResourceState::Pending => matches!(
                next.state,
                ResourceState::Applied | ResourceState::Failed | ResourceState::NotApplied
            ),
            ResourceState::Applied | ResourceState::Failed | ResourceState::TimedOut => matches!(
                next.state,
                ResourceState::CleanupPending | ResourceState::Retained
            ),
            ResourceState::CleanupPending => matches!(
                next.state,
                ResourceState::Released
                    | ResourceState::Failed
                    | ResourceState::TimedOut
                    | ResourceState::Retained
            ),
            ResourceState::Released | ResourceState::Retained | ResourceState::NotApplied => false,
        };
        if !allowed {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid resource state transition",
            ));
        }
    }
    Ok(())
}

fn parse_transition(value: &Value) -> io::Result<ResourceTransition> {
    let kind = ResourceKind::parse(&required_string(value, "kind")?)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "unknown resource kind"))?;
    let state = ResourceState::parse(&required_string(value, "state")?)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "unknown resource state"))?;
    Ok(ResourceTransition {
        sequence: value
            .get("sequence")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "transition sequence is missing")
            })?,
        resource_id: required_string(value, "resource_id")?,
        kind,
        target: required_string(value, "target")?,
        ownership: required_string(value, "ownership")?,
        action: required_string(value, "action")?,
        state,
        detail: required_string(value, "detail")?,
    })
}

fn transition_json(value: &ResourceTransition) -> Value {
    json!({
        "type": "resource.transition",
        "schema_version": SCHEMA_VERSION,
        "sequence": value.sequence,
        "resource_id": value.resource_id,
        "kind": value.kind.as_str(),
        "target": value.target,
        "ownership": value.ownership,
        "action": value.action,
        "state": value.state.as_str(),
        "detail": value.detail,
    })
}

fn required_string(value: &Value, field: &str) -> io::Result<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("journal field `{field}` is missing"),
            )
        })
}

fn write_line(file: &mut File, value: &Value) -> io::Result<()> {
    serde_json::to_writer(&mut *file, value).map_err(io::Error::other)?;
    file.write_all(b"\n")
}

fn invalid_json(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

#[cfg(not(windows))]
fn atomic_replace(source: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(source, destination)
}

#[cfg(windows)]
fn atomic_replace(source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{ReplaceFileW, REPLACEFILE_WRITE_THROUGH};

    let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    // SAFETY: both paths are NUL-terminated buffers kept alive for the call;
    // no backup path or reserved pointers are supplied.
    let result = unsafe {
        ReplaceFileW(
            destination.as_ptr(),
            source.as_ptr(),
            std::ptr::null(),
            REPLACEFILE_WRITE_THROUGH,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
